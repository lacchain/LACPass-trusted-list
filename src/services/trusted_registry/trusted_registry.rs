use log::info;
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use web3::types::H160;

use crate::{
    config::env_config::Config,
    services::{
        did::did_registry_worker_service::DidRegistryWorkerService, did::did_service::DidService,
        public_directory::index::PublicDirectoryService,
        public_directory::public_directory_worker_service::PublicDirectoryWorkerService,
    },
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Contract {
    pub chain_id: String,
    pub contract_address: H160,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TrustedRegistry {
    pub public_directory: Contract,
    pub chain_of_trust: Contract,
    pub period_seconds: u64,
    pub start_up: u64,
    pub retry_period: u64,
}

impl TrustedRegistry {
    pub async fn sweep(&self) -> anyhow::Result<()> {
        info!(
            "Sweeping trusted registry ... {:?} {:?}",
            self.public_directory, self.chain_of_trust
        );
        match Database::connect(Config::get_config().databases.dbconnection.url).await {
            Ok(db) => {
                info!("Established a database connection");
                let public_directory_worker_service: PublicDirectoryWorkerService;
                match PublicDirectoryService::new(self.public_directory.clone()).await {
                    Ok(result) => {
                        public_directory_worker_service = PublicDirectoryWorkerService::new(result);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
                match public_directory_worker_service.sweep(&db).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("There was an error while trying to retrieve public directory last block saved ---> {:?}", e);
                        return Err(e.into());
                    }
                }
                // seep cot
                // sweep dids that matches with pulbic directory
                // TODO: match them with Chain Of Trust also
                let did_service = DidService::new();
                let did_registry_worker_service = DidRegistryWorkerService::new();
                match did_service
                    .did_data_interface_service
                    .find_all(
                        &db,
                        &self.public_directory.contract_address.to_string(),
                        &self.public_directory.chain_id,
                    )
                    .await
                {
                    Ok(dids) => {
                        for did in dids {
                            match did_registry_worker_service.sweep(&db, &did.id).await {
                                Ok(_) => {}
                                Err(e) => panic!("{}", e),
                            }
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
                Ok(())
            }
            Err(e) => {
                let message = format!("There was an error connecting to the database: {:?}", e);
                error!("{}", &message);
                Err(e.into())
            }
        }
    }
}
