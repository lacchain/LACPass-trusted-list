use crate::utils::utils::Utils;
use serde::{Deserialize, Serialize};

use super::trusted_registry::{Contract, TrustedRegistry};

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
        let t1 = TrustedRegistry {
            period_seconds: 400,
            start_up: 5,
            public_directory: Contract {
                chain_id: 0x9e55c,
                contract_address: "0x4A1bD1198890af301AF9b6F3a3a11952a86C1c8e".to_owned(),
            },
            chain_of_trust: Contract {
                chain_id: 0x9e55c,
                contract_address: "0xEBB6854aa875867f684dd1d2338eC20908039c67".to_owned(),
            },
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
            })
            .collect::<Vec<_>>();
    }
}
