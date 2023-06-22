use log::info;
use sea_orm::Database;
use serde::{Deserialize, Serialize};
use web3::types::H160;

use crate::{
    config::env_config::Config,
    services::{
        public_directory::index::PublicDirectoryService,
        trusted_registry::core_worker::CoreWorkerService,
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
            Ok(c) => {
                info!("Successfully connected a database connection");
                let core_worker_service: CoreWorkerService;
                match PublicDirectoryService::new(self.public_directory.clone()).await {
                    Ok(result) => {
                        core_worker_service = CoreWorkerService::new(result);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
                match core_worker_service.sweep(&c).await {
                    Ok(_) => {
                        // sweep chain of trust
                        // read did registry changes
                    }
                    Err(e) => {
                        error!("There was an error while trying to retrieve public directory last block saved ---> {:?}", e);
                        return Err(e.into());
                    }
                }
                // sweep to get members in the public directory and for each save it to the database
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
