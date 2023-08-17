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
    fn get_trusted_registries() -> String {
        match Utils::get_env_or_err("TRUSTED_REGISTRIES") {
            Ok(s) => s,
            Err(_) => panic!("Please set TRUSTED_REGISTRIES environment variable"),
        }
    }

    /// index must start from "1"
    fn get_external_source_1_at_index(index: &str) -> Option<String> {
        match index.parse::<usize>() {
            Ok(v) => {
                if v == 0 as usize {
                    panic!("Invalid index passed to get_external_source_1");
                }
            }
            Err(e) => panic!("Error while pasing index: {}", e),
        };
        match Utils::get_env_or_err("EXTERNAL_SOURCE_1") {
            Ok(s) => {
                let found = s.clone();
                found
                    .split("--")
                    .collect::<Vec<_>>()
                    .into_iter()
                    .find_map(|el| {
                        if let [candidate_index, url] = el.split(",").collect::<Vec<_>>().as_slice()
                        {
                            if **candidate_index == *index {
                                return Some((**url).to_owned());
                            }
                            None
                        } else {
                            return None;
                        }
                    })
            }
            Err(_) => None, // since it is not striclty required just returning NONE
        }
    }

    pub fn process_env_trusted_registries() -> Vec<TrustedRegistry> {
        let binding = TrustedRegistries::get_trusted_registries();
        let raw_trusted_registries = binding
            .split("--")
            .collect::<Vec<_>>()
            .into_iter()
            .map(|tr_str| {
                if let [index, pd, pd_cid, cot, cot_cid] =
                    tr_str.split(",").collect::<Vec<_>>().as_slice()
                {
                    let pd = Utils::trim_0x_from_hex_string(pd);
                    let cot = Utils::trim_0x_from_hex_string(cot);
                    let public_directory_address = <[u8; 20]>::from_hex(pd)
                        .expect("Invalid public directory contract address");
                    let cot_address =
                        <[u8; 20]>::from_hex(cot).expect("Invalid chain of trust contract address");
                    let external_source_1_url =
                        TrustedRegistries::get_external_source_1_at_index(index);
                    let t1 = TrustedRegistry {
                        index: index.to_string(),
                        period_seconds: 400,
                        start_up: 5,
                        public_directory: Contract {
                            chain_id: pd_cid.to_string(),
                            contract_address: H160(public_directory_address),
                        },
                        chain_of_trust: Contract {
                            chain_id: cot_cid.to_string(),
                            contract_address: H160(cot_address),
                        },
                        retry_period: 0,
                        external_source_1_url,
                    };
                    t1
                } else {
                    panic!("Error decoding trusted registry params from environment variables");
                }
            })
            .collect::<Vec<_>>();
        raw_trusted_registries
    }

    fn set_trusted_registries(&mut self) -> () {
        self.registries = TrustedRegistries::process_env_trusted_registries();
    }

    fn set_start_up_and_period(&mut self) -> () {
        // TODO: improve start up
        let _p = (0..self.registries.len())
            .into_iter()
            .map(|i| {
                self.registries[i].start_up = (i as u64) * 100;
                self.registries[i].period_seconds = 2000;
                self.registries[i].retry_period = 10;
            })
            .collect::<Vec<_>>();
    }

    pub fn get_trusted_registry_by_index() -> TrustedRegistry {
        let trusted_registries = TrustedRegistries::process_env_trusted_registries();
        let index: String;
        match Utils::get_env_or_err("TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE") {
            Ok(s) => index = s,
            Err(e) => {
                error!("{}", e);
                panic!(
                    "Please set TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE environment variable"
                );
            }
        }
        let tr = trusted_registries
            .into_iter()
            .filter(|e| e.index == index)
            .collect::<Vec<_>>();
        if tr.len() != 1 {
            let message = format!("TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE '{:?}' was (not found/or more than one) with the pointed index in TRUSTED_REGISTRIES", index);
            panic!("{}", message);
        };
        tr[0].clone()
    }
}
