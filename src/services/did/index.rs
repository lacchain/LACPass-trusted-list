use crate::{services::trusted_registry::trusted_registry::Contract, utils::utils::Utils};
use crypto::{digest::Digest, sha3::Sha3};
use web3::types::H160;

use super::{contract_interface::ContractInterface, data_interface::DidDataInterfaceService};

pub struct DidLac1 {
    pub address: H160,
    pub did_registry_address: H160,
    pub chain_id: i64,
}

pub struct DidService {
    pub did_data_interface_service: DidDataInterfaceService,
    pub contract_interface_service: ContractInterface,
}

impl DidService {
    pub async fn new(params: Contract) -> anyhow::Result<DidService> {
        let contract_interface: ContractInterface;
        match ContractInterface::new(params.clone()).await {
            Ok(v) => {
                contract_interface = v;
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(DidService {
            did_data_interface_service: DidDataInterfaceService {},
            contract_interface_service: contract_interface,
        })
    }

    /// did_type_ lac1, version 1.
    pub fn decode_did(did: &str) -> anyhow::Result<DidLac1> {
        let did = did.trim();
        let core = did.replace("did:lac1:", "");
        match bs58::decode(core).into_vec() {
            Ok(decoded) => {
                let size = decoded.len();
                let (encoded_payload, checksum) = decoded.split_at(size - 4);
                // checksum
                let mut h = Sha3::keccak256();
                h.input(encoded_payload);
                let mut out: [u8; 32] = [0; 32];
                h.result(&mut out);
                let (computed_checksum, _) = out.split_at(4);
                if computed_checksum != checksum.to_vec().as_slice() {
                    return Err(anyhow::anyhow!("checksum error"));
                }
                let (_version, right) = encoded_payload.split_at(2);
                let (_did_type, right) = right.split_at(2);
                // TODO: verify did version and type
                let (address, right) = right.split_at(20);
                let (did_registry_address, chain_id) = right.split_at(20);
                // let address = Utils::vec_u8_to_hex_string(address.to_vec()).unwrap();
                let address = H160::from_slice(address);
                // let did_registry_address =
                //     Utils::vec_u8_to_hex_string(did_registry_address.to_vec()).unwrap();
                let did_registry_address = H160::from_slice(did_registry_address);
                let chain_id = Utils::vec_u8_to_hex_string(chain_id.to_vec()).unwrap();
                let chain_id_int;
                match i64::from_str_radix(chain_id.trim_start_matches("0x"), 16) {
                    Ok(v) => chain_id_int = v,
                    Err(_) => panic!("Invalid chain id: {}", chain_id),
                }
                Ok(DidLac1 {
                    address,
                    did_registry_address,
                    chain_id: chain_id_int,
                })
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::println;
    #[test]
    fn decode_did_sucess_test() {
        let did =
            "did:lac1:1iT5D8E51oULzpTrFePKe9KibQh4sEnqLCRDdFSLRNPfmf89seFJvjcfgKrtZ5YdGBX1".trim();
        let decoded = DidService::decode_did(did).unwrap();
        assert_eq!(
            Utils::vec_u8_to_hex_string(decoded.address.as_bytes().to_vec()).unwrap(),
            "0x560ff31E783570097c18bd342e524Ef4c36fE7AE"
                .to_owned()
                .to_lowercase()
        );

        assert_eq!(
            Utils::vec_u8_to_hex_string(decoded.did_registry_address.as_bytes().to_vec()).unwrap(),
            "0x54358D929CCf45C7cCEE8Ca60FCD0C0402489F54"
                .to_owned()
                .to_lowercase(),
        );

        assert_eq!(decoded.chain_id, 648540)
    }

    #[test]
    fn decode_did_failure_test() {
        let did = "some value";
        match DidService::decode_did(did) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }
}
