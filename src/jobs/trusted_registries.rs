use crate::{
    services::trusted_registry::trusted_registry::{Contract, TrustedRegistry},
    utils::utils::Utils,
};
use hex::FromHex;
use serde::{Deserialize, Serialize};
use web3::types::H160;

#[derive(Deserialize, Serialize)]
pub struct TrustedRegistries {
    pub registries: Vec<TrustedRegistry>,
}

impl TrustedRegistries {
    pub fn new() -> TrustedRegistries {
        let mut s = TrustedRegistries {
            registries: Vec::new(),
        };
        s.set_trusted_registries();
        s.set_start_up_and_period();
        s
    }
    fn get_trusted_registries(&self) -> String {
        match Utils::get_env_or_err("TRUSTED_REGISTRIES") {
            Ok(s) => s,
            Err(_) => panic!("Please set TRUSTED_REGISTRIES environment variable"),
        }
    }

    fn set_trusted_registries(&mut self) -> () {
        let _raw_trusted_registries = self.get_trusted_registries();
        let pd_str = "fee5C6939309a9906e292753B1947c8De1FD4423"; // "e647e8e076cffA10425c0C49aAaC1036a3b2ddB5"; // TODO: factor better error
        let public_directory_address =
            <[u8; 20]>::from_hex(pd_str).expect("Invalid public directory contract address");
        let cot_str = "EBB6854aa875867f684dd1d2338eC20908039c67";
        let cot_address =
            <[u8; 20]>::from_hex(cot_str).expect("Invalid chain of trust contract address");
        let t1 = TrustedRegistry {
            period_seconds: 400,
            start_up: 5,
            public_directory: Contract {
                chain_id: "648540".to_owned(),
                contract_address: H160(public_directory_address),
            },
            chain_of_trust: Contract {
                chain_id: "648540".to_owned(),
                contract_address: H160(cot_address),
            },
            retry_period: 0,
        };
        let mut trs: Vec<TrustedRegistry> = Vec::new();
        trs.push(t1);
        self.registries = trs;
    }

    fn set_start_up_and_period(&mut self) -> () {
        // TODO: improve start up
        let _p = (0..self.registries.len())
            .into_iter()
            .map(|i| {
                self.registries[i].start_up = (i as u64) * 100;
                self.registries[i].period_seconds = 2000;
                self.registries[i].retry_period = 30;
            })
            .collect::<Vec<_>>();
    }
}
