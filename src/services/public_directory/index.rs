use std::collections::HashMap;

use log::info;
use sea_orm::DatabaseConnection;

use crate::services::{
    trusted_registry::trusted_registry::Contract,
    web3::utils::{get_string_from_string_in_log, get_u64_from_log},
};

use super::{contract_interface::ContractInterface, data_interface::DataInterfaceService};

pub struct PublicDirectoryService {
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
            contract_interface,
            data_interface: DataInterfaceService::new(params),
        })
    }
    pub async fn sweep(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        let contract_last_block: u64;
        match self.contract_interface.get_last_block().await {
            Ok(result) => {
                if result == 0 {
                    info!("No need to sweep public directory ... skipping sweep");
                    return Ok(());
                }

                contract_last_block = result;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
        let last_block_saved;
        match self.data_interface.get_last_block(&db).await {
            Ok(result) => {
                last_block_saved = result;
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        // TODO: find vector of intermediate blocks to sweep since the last processed block
        self.process_events_in_block_range(contract_last_block, last_block_saved)
            .await
    }

    pub async fn process_events_in_block_range(
        &self,
        contract_last_block: u64,
        last_block_saved: u64,
    ) -> anyhow::Result<()> {
        if contract_last_block > last_block_saved {
            info!(
                "Starting sweep; last block saved {}, last block on Public directory  {}",
                &last_block_saved, &contract_last_block
            );
            match self
                .process_events_in_block(contract_last_block.to_string())
                .await
            {
                Ok(_prev_block) => {
                    // TODO: update last block processed in database
                    // TODO: Get next block (moving backward)
                    // if prev_block <= last_block_saved {}
                    // TODO: repeat the flow
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
            return Ok(());
        }
        if contract_last_block == last_block_saved {
            return Ok(());
        }
        panic!("Unexpected values, last block saved on database:{}, is greater than contract_last_block: {}", &last_block_saved, &contract_last_block);
    }

    pub async fn get_did_associated_map(
        &self,
        block: &str,
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
    pub async fn process_events_in_block(&self, block: String) -> anyhow::Result<u64> {
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
            .get_events_in_block_by_method("DidAssociated", &block)
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
