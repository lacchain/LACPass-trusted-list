use crate::services::trusted_registry::trusted_registry::Contract;

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
}
