use serde::{Deserialize, Serialize};

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
}

impl TrustedRegistry {
    pub fn sweep(&self) {
        info!(
            "Sweeping trusted registry ... {:?} {:?}",
            self.public_directory, self.chain_of_trust
        );
        // read public directory last changes
        // read chain of trust last changes
        // read did registry changes
    }
}
