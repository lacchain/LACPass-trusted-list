use log::info;
use sea_orm::DatabaseConnection;

use crate::services::trusted_registry::trusted_registry::Contract;

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
        let contract_last_block: i64;
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

        if contract_last_block > last_block_saved {
            info!(
                "Starting sweep; last block saved {}, last block on Public directory  {}",
                &last_block_saved, &contract_last_block
            );
            // TODO: sweep from contract_last_block to last_block_saved (reverse order)
            return Ok(());
        }

        panic!("Unexpected values, last block saved on database:{}, is greater than contract_last_block: {}", &last_block_saved, &contract_last_block);
    }
}
