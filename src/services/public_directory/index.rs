use std::collections::HashMap;

use log::{debug, info};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

use crate::services::{
    trusted_registry::trusted_registry::Contract,
    web3::utils::{get_string_from_string_in_log, get_u64_from_log},
};

use crate::entities::models::PublicDirectoryActiveModel;

use super::{contract_interface::ContractInterface, data_interface::DataInterfaceService};

pub struct PublicDirectoryService {
    pub params: Contract,
    pub contract_interface: ContractInterface,
    pub data_interface: DataInterfaceService,
}

impl PublicDirectoryService {
    pub async fn new(params: Contract) -> anyhow::Result<PublicDirectoryService> {
        let contract_interface: ContractInterface;
        match ContractInterface::new(params.clone()).await {
            Ok(v) => {
                contract_interface = v;
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(PublicDirectoryService {
            params: params.clone(),
            contract_interface,
            data_interface: DataInterfaceService::new(params),
        })
    }

    pub async fn exec_or_resume_scheduled_sweep(
        &self,
        db: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        info!("excuting scheduled worker thread");
        match self
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
                            .contract_interface
                            .find_previous_block(&(v.last_processed_block as u64))
                            .await
                        {
                            Ok(u) => {
                                if let Some(prev_to_last_processed_block) = u {
                                    return self
                                        .process_events_in_block_range(
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
                match self.contract_interface.get_last_block().await {
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
                // set last processed block to upper block
                match self
                    .data_interface
                    .get_public_directory_from_database(&db)
                    .await
                {
                    Ok(v) => match v {
                        Some(m) => {
                            // verify whether insert is needed
                            if contract_last_block > 0
                                && contract_last_block == m.upper_block as u64
                                && contract_last_block == m.last_block_saved as u64
                            {
                                info!("There are no changes in the contract .. skipping sweep");
                                return Ok(());
                            }
                            let mut s: PublicDirectoryActiveModel = m.into();
                            s.upper_block = Set(contract_last_block as i64);
                            s.last_processed_block = Set(0);
                            match s.update(db).await {
                                Ok(_) => self.exec_or_resume_scheduled_sweep(db).await,
                                Err(err) => {
                                    return Err(err.into());
                                }
                            }
                        }
                        None => {
                            info!("Initializing metadata for contract in database");
                            let db_registry = PublicDirectoryActiveModel {
                                id: Set(Uuid::new_v4()),
                                contract_address: Set(self.params.contract_address.to_string()),
                                upper_block: Set(contract_last_block as i64),
                                last_processed_block: Set(0),
                                last_block_saved: Set(0),
                                chain_id: Set(self.params.chain_id.clone()),
                            };
                            match db_registry.insert(db).await {
                                Ok(_) => self.exec_or_resume_scheduled_sweep(db).await,
                                Err(e) => {
                                    return Err(e.into());
                                }
                            }
                        }
                    },
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
        contract_block: u64,
        target_block: u64,
    ) -> anyhow::Result<()> {
        if contract_block > target_block {
            info!(
                "Starting sweep; from block {}, to target block  {}",
                &contract_block, &target_block
            );
            match self.process_events_in_block(&contract_block).await {
                Ok(_prev_block) => {
                    // TODO: update last block processed in database
                    // TODO: Get next block (moving backward)
                    // if prev_block <= target_block {}
                    // TODO: repeat the flow
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
            return Ok(());
        }
        if contract_block == target_block {
            return Ok(());
        }
        panic!("Unexpected values, last block saved on database:{}, is greater than passed contract block: {}", &target_block, &contract_block);
    }

    pub async fn get_did_associated_map(
        &self,
        block: &u64,
    ) -> anyhow::Result<HashMap<u64, Vec<String>>> {
        let did_associated_logs = self
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
    pub async fn process_events_in_block(&self, block: &u64) -> anyhow::Result<u64> {
        let mut did_associated_map = match self.get_did_associated_map(&block).await {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            }
        };
        let mut prev_block: u64 = 0;
        match self
            .contract_interface
            .get_events_in_block_by_method("MemberChanged", &block)
            .await
        {
            Ok(member_changed_logs) => {
                let _ = member_changed_logs
                    .into_iter()
                    .map(|member_changed_log| {
                        let exp = get_u64_from_log(&member_changed_log, "exp");
                        let iat = get_u64_from_log(&member_changed_log, "iat");
                        let member_id = get_u64_from_log(&member_changed_log, "memberId");
                        let did = get_string_from_string_in_log(&member_changed_log, "did");
                        prev_block = get_u64_from_log(&member_changed_log, "prevBlock");
                        let transaction_timestamp =
                            get_u64_from_log(&member_changed_log, "currentTimestap");
                        if transaction_timestamp == iat {
                            info!("new member was added/updated {} {}", did, member_id);
                            // 1. remove existing did in did associated mapping
                            match did_associated_map.get_mut(&member_id) {
                                Some(v) => {
                                    let pos =
                                        v.into_iter().rposition(|e| e.to_string() == did).unwrap();
                                    v.remove(pos);
                                }
                                None => panic!(
                                    "Expected value in didAssociated event but was not found {}",
                                    did
                                ),
                            };
                            // TODO: 2. save to database -> if there exist a registry in which the block_number is greater then skip otherwise update
                        } else if transaction_timestamp == exp {
                            // MemberChanged with currentTimestamp==exp -> remove the entity did and all its dids from the database
                            info!("a member was removed");
                        } else {
                            panic!(
                                "Unexpected values for iat: {}, exp: {}, transaction timestamp: {}",
                                iat, exp, transaction_timestamp
                            );
                        }
                    })
                    .collect::<Vec<_>>();
            }
            Err(e) => {
                return Err(e);
            }
        }
        // DidAssociated && !Memberchanged -> just new Did associated to an existing member
        let _ = did_associated_map
            .into_iter()
            .map(|_el| {
                // TODO: find that member in database
                // TODO: associate the new did with the db found member
            })
            .collect::<Vec<_>>();

        // DidDisassociated -> remove just that did from database that did
        match self
            .contract_interface
            .get_events_in_block_by_method("DidDisassociated", &block)
            .await
        {
            Ok(_did_disassociated_logs) => {
                // TODO: remove these dids from database
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(prev_block)
    }
}
