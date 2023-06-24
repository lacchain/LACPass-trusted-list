use std::collections::HashMap;

use log::{debug, info};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use web3::ethabi::Log;

use crate::services::{
    did::did_service::DidService,
    pd_did_member::data_interface::PdDidMemberDataInterfaceService,
    pd_member::data_interface::PdMemberDataInterfaceService,
    public_directory::index::PublicDirectoryService,
    web3::utils::{get_string_from_string_in_log, get_u64_from_log},
};

pub struct PublicDirectoryWorkerService {
    pub pd_did_member_data_interface_service: PdDidMemberDataInterfaceService,
}

impl PublicDirectoryWorkerService {
    pub fn new(public_directory_service: PublicDirectoryService) -> PublicDirectoryWorkerService {
        let pd_member_data_service = PdMemberDataInterfaceService::new(public_directory_service);
        PublicDirectoryWorkerService {
            pd_did_member_data_interface_service: PdDidMemberDataInterfaceService::new(
                pd_member_data_service,
                DidService::new(),
            ),
        }
    }

    pub async fn exec_or_resume_scheduled_sweep(
        &self,
        db: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        info!("excuting scheduled worker thread");
        match self
            .pd_did_member_data_interface_service
            .pd_member_data_service
            .public_directory_service
            .data_interface
            .get_public_directory_from_database(&db)
            .await
        {
            Ok(v) => match v {
                Some(v) => {
                    if v.upper_block == v.last_block_saved {
                        info!("All up to date in this scheduled task");
                        return Ok(());
                    }

                    if v.upper_block >= v.last_processed_block
                        && v.last_processed_block > v.last_block_saved
                    {
                        info!("Found not finished job, resuming");
                        match self
                            .pd_did_member_data_interface_service
                            .pd_member_data_service
                            .public_directory_service
                            .contract_interface
                            .find_previous_block(&(v.last_processed_block as u64))
                            .await
                        {
                            Ok(u) => {
                                if let Some(prev_to_last_processed_block) = u {
                                    return self
                                        .process_events_in_block_range(
                                            db,
                                            prev_to_last_processed_block,
                                            v.last_block_saved as u64,
                                        )
                                        .await;
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    info!("Starting scheduled job");
                    return self
                        .process_events_in_block_range(
                            db,
                            v.upper_block as u64,
                            v.last_block_saved as u64,
                        )
                        .await;
                }
                None => {
                    debug!("None was found in the database");
                    return Ok(());
                }
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn sweep(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        match self.exec_or_resume_scheduled_sweep(db).await {
            Ok(_v) => {
                let contract_last_block: u64;
                match self
                    .pd_did_member_data_interface_service
                    .pd_member_data_service
                    .public_directory_service
                    .contract_interface
                    .get_last_block()
                    .await
                {
                    Ok(result) => {
                        if result == 0 {
                            info!("No events found in contract... skipping sweep");
                            return Ok(());
                        }

                        contract_last_block = result;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
                // set upper block to contract last saved block
                // set last processed block to zero value
                match self
                    .pd_did_member_data_interface_service
                    .pd_member_data_service
                    .public_directory_service
                    .data_interface
                    .get_public_directory_from_database(&db)
                    .await
                {
                    Ok(v) => {
                        match v {
                            Some(m) => {
                                // verify whether insert is needed
                                if contract_last_block > 0
                                    && contract_last_block == m.upper_block as u64
                                    && contract_last_block == m.last_block_saved as u64
                                {
                                    info!("There are no changes in the contract. Last block saved is {}", contract_last_block);
                                    return Ok(());
                                }
                                match self
                                    .pd_did_member_data_interface_service
                                    .pd_member_data_service
                                    .public_directory_service
                                    .data_interface
                                    .update(db, Some(contract_last_block), Some(0), None)
                                    .await
                                {
                                    Ok(_) => self.exec_or_resume_scheduled_sweep(db).await,
                                    Err(err) => {
                                        return Err(err.into());
                                    }
                                }
                            }
                            None => {
                                info!("Initializing metadata for contract in database");
                                match self
                                    .pd_did_member_data_interface_service
                                    .pd_member_data_service
                                    .public_directory_service
                                    .data_interface
                                    .save_contract_last_block(db, &contract_last_block)
                                    .await
                                {
                                    Ok(_) => self.exec_or_resume_scheduled_sweep(db).await,
                                    Err(e) => {
                                        return Err(e.into());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn process_events_in_block_range(
        &self,
        db: &DatabaseConnection,
        contract_block: u64,
        target_block: u64,
    ) -> anyhow::Result<()> {
        let mut block_to_process = contract_block;
        while block_to_process > target_block {
            info!(
                "Starting sweep; from block {}, to target block  {}",
                &block_to_process, &target_block
            );
            match self
                .public_directory_process_events_in_block(db, &block_to_process)
                .await
            {
                Ok(prev_block) => {
                    match self
                        .pd_did_member_data_interface_service
                        .pd_member_data_service
                        .public_directory_service
                        .data_interface
                        .update(db, None, Some(block_to_process), None)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                    block_to_process = prev_block;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        if block_to_process == target_block {
            match self
                .pd_did_member_data_interface_service
                .pd_member_data_service
                .public_directory_service
                .data_interface
                .update(db, None, Some(0), Some(contract_block))
                .await
            {
                Ok(_) => {
                    info!("Reached target block {}", block_to_process);
                    Ok(())
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        } else {
            panic!("Unexpected values, last block saved on database:{}, is greater than passed contract block: {}", &target_block, &block_to_process);
        }
    }

    pub async fn get_did_associated_map(
        &self,
        block: &u64,
    ) -> anyhow::Result<HashMap<u64, Vec<String>>> {
        let did_associated_logs = self
            .pd_did_member_data_interface_service
            .pd_member_data_service
            .public_directory_service
            .contract_interface
            .get_events_in_block_by_method("DidAssociated", &block)
            .await
            .unwrap();
        let mut did_associated_map: HashMap<u64, Vec<String>> = HashMap::new();
        let _l = did_associated_logs
            .into_iter()
            .map(|did_associated_log| {
                let did = get_string_from_string_in_log(&did_associated_log, "did");
                let member_id = get_u64_from_log(&did_associated_log, "memberId");
                match did_associated_map.get(&member_id) {
                    Some(f) => {
                        let mut _f = f.clone();
                        _f.push(did);
                        did_associated_map.insert(member_id, _f);
                    }
                    None => {
                        let mut v = Vec::new();
                        v.push(did);
                        did_associated_map.insert(member_id, v);
                    }
                };
            })
            .collect::<Vec<_>>();
        Ok(did_associated_map)
    }

    //// Returns previous block
    pub async fn public_directory_process_events_in_block(
        &self,
        db: &DatabaseConnection,
        block: &u64,
    ) -> anyhow::Result<u64> {
        // TODO: implement did associated cases
        // let did_associated_map = match self.get_did_associated_map(&block).await {
        //     Ok(v) => v,
        //     Err(e) => {
        //         return Err(e);
        //     }
        // };
        let prev_block: u64;

        match self
            .pd_did_member_data_interface_service
            .pd_member_data_service
            .public_directory_service
            .contract_interface
            .get_events_in_block_by_method("MemberChanged", &block)
            .await
        {
            Ok(member_changed_logs) => {
                match self
                    .process_member_changed_event(db, member_changed_logs, block)
                    .await
                {
                    Err(e) => return Err(e.into()),
                    Ok(v) => {
                        prev_block = v;
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
        // TODO: implement did disassociated cases
        // // DidAssociated && !Memberchanged -> just new Did associated to an existing member
        // let _ = did_associated_map
        //     .into_iter()
        //     .map(|_el| {
        //         // TODO: find that member in database
        //         // TODO: associate the new did with the db found member
        //     })
        //     .collect::<Vec<_>>();

        // // DidDisassociated -> remove just that did from database that did
        // match self
        //     .public_directory_service
        //     .contract_interface
        //     .get_events_in_block_by_method("DidDisassociated", &block)
        //     .await
        // {
        //     Ok(_did_disassociated_logs) => {
        //         // TODO: remove these dids from database
        //     }
        //     Err(e) => {
        //         return Err(e);
        //     }
        // }
        Ok(prev_block)
    }

    pub async fn process_member_changed_event(
        &self,
        db: &DatabaseConnection,
        member_changed_logs: Vec<Log>,
        block: &u64,
    ) -> anyhow::Result<u64> {
        let mut prev_block: u64 = 0;
        for member_changed_log in member_changed_logs {
            let exp = get_u64_from_log(&member_changed_log, "exp");
            let iat = get_u64_from_log(&member_changed_log, "iat");
            let member_id = get_u64_from_log(&member_changed_log, "memberId");
            let did = get_string_from_string_in_log(&member_changed_log, "did");
            prev_block = get_u64_from_log(&member_changed_log, "prevBlock");
            let transaction_timestamp = get_u64_from_log(&member_changed_log, "currentTimestap");
            if transaction_timestamp == iat {
                // issuance case scenario
                info!("new member was added/updated {} {}", did, member_id);
                let pd_member_id: Uuid;
                match self
                    .pd_did_member_data_interface_service
                    .pd_member_data_service
                    .get_pd_member_from_database(db, &(member_id as i64))
                    .await
                {
                    // "SAVE" to PdMember table
                    Ok(wrapped) => match wrapped {
                        Some(found_pd_member) => {
                            pd_member_id = found_pd_member.id;
                            if (found_pd_member.block_number as u64) < *block {
                                match self
                                    .pd_did_member_data_interface_service
                                    .pd_member_data_service
                                    .update_pd_member(
                                        db,
                                        found_pd_member.id,
                                        &(exp as i64),
                                        &(*block as i64),
                                    )
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(e) => return Err(e.into()),
                                }
                            }
                        }
                        None => {
                            match self
                                .pd_did_member_data_interface_service
                                .pd_member_data_service
                                .insert_pd_member(
                                    db,
                                    &(member_id as i64),
                                    &(exp as i64),
                                    &(*block as i64),
                                )
                                .await
                            {
                                Ok(saved_pd_member) => pd_member_id = saved_pd_member.id,
                                Err(e) => return Err(e.into()),
                            }
                        }
                    },
                    Err(e) => return Err(e.into()),
                }

                // "SAVE" to Did table
                let did_id: Uuid;
                match self
                    .pd_did_member_data_interface_service
                    .did_service
                    .did_data_interface_service
                    .get_did_from_database(db, &did)
                    .await
                {
                    Ok(v) => match v {
                        Some(existing_did) => {
                            did_id = existing_did.id;
                        }
                        None => {
                            match self
                                .pd_did_member_data_interface_service
                                .did_service
                                .did_data_interface_service
                                .insert_did_to_database(db, &did)
                                .await
                            {
                                Ok(v) => did_id = v.id,
                                Err(e) => return Err(e.into()),
                            }
                        }
                    },
                    Err(e) => return Err(e.into()),
                }

                // "SAVE" to pd_did_member table
                match self
                    .pd_did_member_data_interface_service
                    .get_pd_did_member_by_ids(db, &did_id, &pd_member_id)
                    .await
                {
                    Ok(v) => match v {
                        Some(pd_did_member_found) => {
                            if pd_did_member_found.block_number as u64 >= *block {
                                continue;
                            }
                            match self
                                .pd_did_member_data_interface_service
                                .update_pd_did_member(db, pd_member_id, &(*block as i64))
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => return Err(e.into()),
                            }
                        }
                        None => {
                            match self
                                .pd_did_member_data_interface_service
                                .insert_did_pd_member(db, &pd_member_id, &did_id, &(*block as i64))
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => return Err(e.into()),
                            }
                        }
                    },
                    Err(e) => return Err(e.into()),
                }
            } else if transaction_timestamp == exp {
                // revocation case scenario
                // TODO: MemberChanged with currentTimestamp==exp -> remove the entity did and all its dids from the database
                info!("a member was removed");
            }
        }
        Ok(prev_block)
    }
}