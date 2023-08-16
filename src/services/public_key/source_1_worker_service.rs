use crypto::{digest::Digest, sha3::Sha3};
use log::debug;
use reqwest::Client;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::services::{public_directory::country_code, x509::x509_utils::X509Utils};

use super::data_interface::PublicKeyService;

pub struct ExternalSource1WorkerService {
    url_connection: String,
    client: Client,
    public_key_service: PublicKeyService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSource1FormatData {
    pub url: String,
    pub country: String,
    pub public_key: String,
    pub valid_until: String,
}

impl ExternalSource1WorkerService {
    pub fn new(url_connection: String) -> Self {
        ExternalSource1WorkerService {
            url_connection,
            client: reqwest::Client::new(),
            public_key_service: PublicKeyService::new(),
        }
    }

    pub async fn update_or_insert_public_key(
        &self,
        db: &DatabaseConnection,
        content_hash: String,
        jwk_bytes: Vec<u8>,
        valid_to: u64,
        country_code: String,
    ) -> anyhow::Result<()> {
        match self
            .public_key_service
            .find_public_key_by_content_hash_and_country_code(db, &content_hash, &country_code)
            .await
        {
            Ok(wrapped) => match wrapped {
                Some(_found_public_key) => {
                    // save in memory to use later
                    let message = "Public key already exists in database, skipping";
                    debug!("{}", message);
                    Ok(())
                }
                None => {
                    match self
                        .public_key_service
                        .insert_public_key(
                            db,
                            None,
                            None,
                            jwk_bytes,
                            &content_hash,
                            &valid_to,
                            None,
                            &country_code,
                        )
                        .await
                    {
                        Ok(_) => {
                            info!("Inserted new public key for country: {}", country_code);
                            Ok(())
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            },
            Err(e) => return Err(e.into()),
        }
    }

    pub async fn sweep(&self, db: &DatabaseConnection) -> anyhow::Result<()> {
        match self.client.get(self.url_connection.clone()).send().await {
            Ok(v) => match v.json::<Vec<ExternalSource1FormatData>>().await {
                Ok(keys) => {
                    let s = keys
                        .iter()
                        .filter_map(|key| {
                            match X509Utils::get_jwk_from_pem(key.public_key.clone()) {
                                Ok(jwk) => Some((jwk, key)),
                                Err(e) => {
                                    let message = format!(
                                        "Error while decoding jwk for country {:?}; error was: {:?}",
                                        key.country, &e
                                    );
                                    debug!("{}", message);
                                    return None;
                                }
                            }
                        })
                        .filter_map(|(jwk, key)|  {
                            match country_code::ALPHA2_TO_ALPHA3.get(&key.country) {
                                Some(_) => {},
                                None => {
                                    let message = format!("Got error when validating country code {}", 
                                    key.country.clone());
                                    debug!("{}", message);
                                    return None;
                                },
                            }
                            // extract pem hash
                            let mut h = Sha3::keccak256();
                            match jwk.x5c.clone() {
                                Some(x5c) => match x5c.get(0) {
                                    Some(pem_candidate) => {
                                        match X509Utils::get_decoded_pem_bytes(pem_candidate.to_string()) {
                                            Ok(decoded) => {
                                                h.input(&decoded);
                                                let content_hash = h.result_str();
                                                let jwk_string: String;
                                                match serde_json::to_string(&jwk) {
                                                    Ok(jwk_str) => {
                                                        jwk_string = jwk_str;
                                                    },
                                                    Err(e) => {
                                                        let message = format!("Got error when encoding jwk to string for country {}, error was: {:?}", 
                                                        key.country.clone(), &e);
                                                        debug!("{}", message);
                                                        return None;
                                                    },
                                                }
                                                let jwk_bytes = jwk_string.as_bytes();

                                                match X509Utils::get_expiration_from_pem(pem_candidate.to_string()) {
                                                    Ok(expiration) => Some((content_hash, jwk_bytes.to_owned(), expiration, key.clone().country )),
                                                    Err(e) => {
                                                        let message = format!(
                                                            "Error while getting 'Expiration' from pem - for country {:?}; error was: {:?}",
                                                            key.country, &e
                                                        );
                                                        debug!("{}", message);
                                                        return None;
                                                    },
                                                }

                                                // return Some((jwk_bytes.to_owned(), key, content_hash));
                                            }
                                            Err(e) => {
                                                debug!(
                                                    "Unable to decode pem certificate coming in jwk exposed from entity in country: {}, error was: {:?}",
                                                    key.country, &e
                                                );
                                                return None;
                                            }
                                        }
                                    }
                                    None => {
                                        debug!(
                                                "Unable to extract x5c from jwk comimg from an exposed jwk from country: {}", key.country
                                            );
                                        return None;
                                    }
                                },
                                None => {
                                    debug!(
                                            "Unable to extract x5c since it is not present in jwk exposed from country: {}", key.country
                                        );
                                    return None;
                                }
                            }
                            // if not found extract otherwise skip
                        })
                        .collect::<Vec<_>>();
                    // TODO: add just the ones that are not added yet
                    for candidate in s {
                        let content_hash = candidate.0;
                        let jwk_bytes = candidate.1;
                        let valid_to = candidate.2;
                        let country_code = candidate.3;
                        match self
                            .update_or_insert_public_key(
                                db,
                                content_hash,
                                jwk_bytes,
                                valid_to,
                                country_code.clone(),
                            )
                            .await
                        {
                            Ok(_) => {
                                let message = format!(
                                    "Successfully performed certificate update for country {}",
                                    country_code
                                );
                                info!("{}", message);
                            }
                            Err(e) => {
                                let message = format!(
                                    "There was an error while trying to add certificate to country {}, error was: {:?}", country_code ,&e);
                                debug!("{}", message);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    let message = format!(
                        "Error while getting from external source 1 ({}), error was: {:?} ",
                        self.url_connection, e
                    );
                    debug!("{}", message);
                    return Err(anyhow::anyhow!(message));
                }
            },
            Err(e) => {
                let message = format!(
                    "Error parsing url before hitting external source 1 ({}), error was: {:?} ",
                    self.url_connection, e
                );
                debug!("{}", message);
                return Err(anyhow::anyhow!(message));
            }
        }
    }
}
