use log::info;
use sea_orm::Database;
use serde::{Deserialize, Serialize};

use crate::config::env_config::Config;

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
        info!("Connection is {}", Config::get_config().database.url);
        match Database::connect(Config::get_config().database.url).await {
            Ok(_c) => {
                info!("Successfully connected a database connection");
                Ok(())
            }
            Err(e) => {
                let message = format!("There was an error connecting to the database: {:?}", e);
                error!("{}", &message);
                Err(e.into())
                // panic!("{}", message)
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
