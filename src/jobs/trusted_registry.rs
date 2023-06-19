use log::info;
use sea_orm::Database;
use serde::{Deserialize, Serialize};

use crate::{config::env_config::Config, services::public_directory::PublicDirectoryService};

#[derive(Deserialize, Serialize, Debug)]
pub struct Contract {
    pub chain_id: i32,
    pub contract_address: String,
}

#[derive(Deserialize, Serialize, Debug)]
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
                match PublicDirectoryService::get_public_directory(
                    &c,
                    &self.public_directory.contract_address,
                    &self.public_directory.chain_id.to_string(),
                )
                .await
                {
                    Ok(result) => match result {
                        Some(v) => {
                            info!("Last block saved is {:?}", v);
                            Ok(())
                        }
                        None => {
                            info!("Nothing was found");
                            Ok(())
                        }
                    },
                    Err(e) => {
                        error!("There was an error while trying to retrieve public directory last block saved {:?}", e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                let message = format!("There was an error connecting to the database: {:?}", e);
                error!("{}", &message);
                Err(e.into())
            }
        }
        // read from database the last block saved for public directory configured smart contract
        // let s = Config::get_config();
        // let _p = Database::connect(s.);
        // read public directory last changes
        // read chain of trust last changes
        // read did registry changes
    }
}
