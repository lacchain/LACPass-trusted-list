use crate::{
    dto::response::hc1_response_dto::{
        Certificate, CodeSystem, DdccCoreDataSet, HC1ValidationResponseDto, Identifier,
        Vaccination, Value,
    },
    responses::{
        error_message::ErrorMessage, generic_response::Responses, success_messages::SuccessMessage,
    },
    services::{public_key::data_interface::PublicKeyService, x509::x509_utils::X509Utils},
};
use base45::decode;
use cbor::{Cbor, Decoder};
use cose::{keys::CoseKey, message::CoseMessage};
use flate2::read::ZlibDecoder;
use log::{debug, info};
use nom::AsBytes;
use rocket::serde::json::Json;
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, io::Read};
use uuid::Uuid;

pub async fn get_pem_keys_by_country(
    db: &DatabaseConnection,
    country_code: &str,
) -> anyhow::Result<Vec<String>> {
    match PublicKeyService::find_public_key_by_country(db, country_code).await {
        Ok(registries) => {
            let s = registries
                .into_iter()
                .filter_map(|registry| {
                    match String::from_utf8(registry.jwk) {
                        Ok(jwk_str) => {
                            match X509Utils::get_pem_from_string_jwk(&jwk_str) {
                                Ok(pem) => {
                                    return Some(X509Utils::format_pem(pem));
                                },
                                Err(e) => {
                                    let message = format!("Error while getting pem from string jwk for country: {}. Error was {:?}", country_code, &e);
                                    debug!("{}", message);
                                    return None;
                                },
                            }
                        },
                        Err(e) => {
                            let message = format!("Error while decoding jwk bytes to string for country: {}. Error was {:?}", country_code, &e);
                            debug!("{}", message);
                            return None;
                        },
                    }
                })
                .collect::<Vec<_>>();
            Ok(s)
        }
        Err(e) => {
            let message = format!(
                "Error while getting keys from database for country: {}. Error was {:?}",
                country_code, &e
            );
            debug!("{}", message);
            return Err(anyhow::anyhow!(message));
        }
    }
}

/// Returns cose keys according to cose-rust library format.
pub async fn get_cose_keys_by_country_code(
    db: &DatabaseConnection,
    country_code: &str,
    track_id: Option<Uuid>,
    signing_alg: &i32,
) -> anyhow::Result<Vec<CoseKey>> {
    let trace_id;
    if let Some(t_id) = track_id {
        trace_id = t_id;
    } else {
        trace_id = Uuid::new_v4();
    }
    match get_pem_keys_by_country(db, country_code).await {
        Err(e) => {
            debug!(
                "TRACE_ID: {}, DESCRIPTION: (pem keys retrieval), {:?}",
                trace_id, &e
            );
            return Err(e);
        }
        Ok(pem_keys) => match X509Utils::pem_to_cose_keys(pem_keys, signing_alg) {
            Some(cose_keys) => Ok(cose_keys),
            None => {
                let message = format!("No keys found for country code: {}", country_code);
                debug!("DESCRIPTION: ({:?})", message);
                return Err(anyhow::anyhow!(message));
            }
        },
    }
}

pub fn get_child_string_from_cbor_map(
    cbor_map: &HashMap<String, Cbor>,
    child: &str,
) -> Option<String> {
    if !cbor_map.contains_key(child) {
        return None;
    }
    match cbor_map.get(child) {
        Some(ic) => match ic {
            cbor::Cbor::Unicode(el) => {
                return Some(el.clone());
            }
            _ => {
                return None;
            }
        },
        None => {
            return None;
        }
    }
}

pub fn get_child_u8_from_cbor_map(cbor_map: &HashMap<String, Cbor>, child: &str) -> Option<u8> {
    if !cbor_map.contains_key(child) {
        return None;
    }
    match cbor_map.get(child) {
        Some(ic) => match ic {
            cbor::Cbor::Unsigned(el) => match el {
                cbor::CborUnsigned::UInt8(el) => Some(el.clone()),
                cbor::CborUnsigned::UInt16(_) => None,
                cbor::CborUnsigned::UInt32(_) => None,
                cbor::CborUnsigned::UInt64(_) => None,
            },
            _ => {
                return None;
            }
        },
        None => {
            return None;
        }
    }
}

pub fn get_string_by_name_from_vec(payload: &Vec<u8>, child_name: &str) -> Option<String> {
    let mut d = Decoder::from_bytes(payload.clone());
    let found = d.items().into_iter().find_map(|v| match v {
        Ok(c) => match c {
            cbor::Cbor::Map(el) => {
                if el.contains_key(child_name) {
                    match el.get(child_name).unwrap() {
                        cbor::Cbor::Unicode(el) => {
                            return Some(el.clone());
                        }
                        _ => None,
                    }
                } else {
                    return None;
                }
            }
            _ => None,
        },
        Err(_) => None,
    });
    return found;
}

pub fn get_map_by_name_from_vec(
    payload: &Vec<u8>,
    child_name: &str,
) -> Option<HashMap<String, Cbor>> {
    let mut d = Decoder::from_bytes(payload.clone());
    let found = d.items().into_iter().find_map(|v| match v {
        Ok(c) => match c {
            cbor::Cbor::Map(el) => {
                if el.contains_key(child_name) {
                    match el.get(child_name).unwrap() {
                        cbor::Cbor::Map(el) => {
                            return Some(el.clone());
                        }
                        _ => None,
                    }
                } else {
                    return None;
                }
            }
            _ => None,
        },
        Err(_) => None,
    });
    return found;
}

pub fn get_child_map_from_cbor_map(
    cbor_map: &HashMap<String, Cbor>,
    child: &str,
) -> Option<HashMap<String, Cbor>> {
    if !cbor_map.contains_key(child) {
        return None;
    }
    match cbor_map.get(child) {
        Some(ic) => match ic {
            cbor::Cbor::Map(el) => {
                return Some(el.clone());
            }
            _ => {
                return None;
            }
        },
        None => {
            return None;
        }
    }
}

pub fn get_code_system_from_map(cbor_map: &HashMap<String, Cbor>) -> anyhow::Result<CodeSystem> {
    let vaccine_code_option = get_child_string_from_cbor_map(&cbor_map, "code");
    if let None = vaccine_code_option {
        let message = format!("No 'vaccine code' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let vaccine_system_option = get_child_string_from_cbor_map(&cbor_map, "system");
    if let None = vaccine_system_option {
        let message = format!("No 'vaccine system' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    Ok(CodeSystem {
        code: vaccine_code_option.unwrap(),
        system: vaccine_system_option.unwrap(),
    })
}

pub fn get_certificate_struct(payload: &Vec<u8>) -> anyhow::Result<Certificate> {
    let map_option = get_map_by_name_from_vec(payload, "certificate");
    if let None = map_option {
        let message = format!("No 'certificate' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let map = map_option.unwrap();
    //// hcid
    let hcid_map_option = get_child_map_from_cbor_map(&map, "hcid");
    if let None = hcid_map_option {
        let message = format!("No 'hcid' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let hcid_map = hcid_map_option.unwrap();
    let hcid_value_option = get_child_string_from_cbor_map(&hcid_map, "value");
    if let None = hcid_value_option {
        let message = format!("No 'hcid value' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let hcid_value = hcid_value_option.unwrap();
    let hcid_value_struct = Value { value: hcid_value };
    // issuer
    let issuer_map_option = get_child_map_from_cbor_map(&map, "issuer");
    if let None = issuer_map_option {
        let message = format!("No 'issuer map' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let issuer_map = issuer_map_option.unwrap();

    let issuer_identifier_map_option = get_child_map_from_cbor_map(&issuer_map, "identifier");
    if let None = issuer_identifier_map_option {
        let message = format!("No 'issuer identifier' field found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let issuer_identifier_map = issuer_identifier_map_option.unwrap();
    let issuer_identifier_value_option =
        get_child_string_from_cbor_map(&issuer_identifier_map, "value");
    if let None = issuer_identifier_value_option {
        let message = format!("No 'issuer identifier value' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let issuer_identifier_value = issuer_identifier_value_option.unwrap();
    let issuer_identifier_struct = Identifier {
        identifier: Value {
            value: issuer_identifier_value,
        },
    };
    // version
    let version_option = get_child_string_from_cbor_map(&map, "version");
    if let None = version_option {
        let message = format!("No 'version' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let version = version_option.unwrap();
    Ok(Certificate {
        hcid: hcid_value_struct,
        period: None,
        issuer: issuer_identifier_struct,
        version,
    })
}
pub fn get_vaccination_struct(payload: &Vec<u8>) -> anyhow::Result<Vaccination> {
    let vaccination_map_option = get_map_by_name_from_vec(payload, "vaccination");
    if let None = vaccination_map_option {
        let message = format!("No 'vaccination' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let vaccination_map = vaccination_map_option.unwrap();

    /////////// vaccine
    let vaccine_map_option = get_child_map_from_cbor_map(&vaccination_map, "vaccine");
    if let None = vaccine_map_option {
        let message = format!("No 'vaccine' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    };
    let vaccine_map = vaccine_map_option.unwrap();
    let vaccine_code_system_result = get_code_system_from_map(&vaccine_map);
    if let Err(e) = vaccine_code_system_result {
        let message = format!("vaccine code system error: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let vaccine_code_system = vaccine_code_system_result.unwrap();

    /////////// manufacturer
    let manufacturer_map_option = get_child_map_from_cbor_map(&vaccination_map, "manufacturer");
    if let None = manufacturer_map_option {
        let message = format!("No 'manufacturer' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    };
    let manufacturer_map = manufacturer_map_option.unwrap();
    let manufacturer_code_system_result = get_code_system_from_map(&manufacturer_map);
    if let Err(e) = manufacturer_code_system_result {
        let message = format!("manufacturer code system error: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let manufacturer_code_system = manufacturer_code_system_result.unwrap();

    /////////// country
    let country_map_option = get_child_map_from_cbor_map(&vaccination_map, "country");
    if let None = country_map_option {
        let message = format!("No 'country' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    };
    let country_map = country_map_option.unwrap();
    let country_code_system_result = get_code_system_from_map(&country_map);
    if let Err(e) = country_code_system_result {
        let message = format!("country code system error: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let country_code_system = country_code_system_result.unwrap();

    /////////// maholder
    let maholder_map_option = get_child_map_from_cbor_map(&vaccination_map, "maholder");
    if let None = maholder_map_option {
        let message = format!("No 'maholder' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    };
    let maholder_map = maholder_map_option.unwrap();
    let maholder_code_system_result = get_code_system_from_map(&maholder_map);
    if let Err(e) = maholder_code_system_result {
        let message = format!("maholder code system error: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let maholder_code_system = maholder_code_system_result.unwrap();

    /////////// brand
    let brand_map_option = get_child_map_from_cbor_map(&vaccination_map, "brand");
    if let None = brand_map_option {
        let message = format!("No 'brand' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    };
    let brand_map = brand_map_option.unwrap();
    let brand_code_system_result = get_code_system_from_map(&brand_map);
    if let Err(e) = brand_code_system_result {
        let message = format!("brand code system error: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let brand_code_system = brand_code_system_result.unwrap();

    ////////// practitioner map
    let practitioner_map_option = get_child_map_from_cbor_map(&vaccination_map, "practitioner");
    if let None = practitioner_map_option {
        let message = format!("No 'practitioner' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let practitioner_map = practitioner_map_option.unwrap();
    let practitioner_value_option = get_child_string_from_cbor_map(&practitioner_map, "value");
    if let None = practitioner_value_option {
        let message = format!("No 'practitioner value' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let practitioner_value = practitioner_value_option.unwrap();
    let practitioner_value_struct = Value {
        value: practitioner_value,
    };

    //////// Disease
    let disease_map_option = get_child_map_from_cbor_map(&vaccination_map, "disease");
    if let None = disease_map_option {
        let message = format!("No 'disease' map found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    };
    let disease_map = disease_map_option.unwrap();
    let disease_code_system_result = get_code_system_from_map(&disease_map);
    if let Err(e) = disease_code_system_result {
        let message = format!("disease code system error: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let disease_code_system = disease_code_system_result.unwrap();

    ///////////
    let dose_option = get_child_u8_from_cbor_map(&vaccination_map, "dose");
    if let None = dose_option {
        let message = format!("vaccination error: No 'brand' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let dose = dose_option.unwrap();

    let total_doses_option = get_child_u8_from_cbor_map(&vaccination_map, "totalDoses");
    if let None = total_doses_option {
        let message = format!("vaccination error: No 'brand' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let total_doses = total_doses_option.unwrap();

    let date_option = get_child_string_from_cbor_map(&vaccination_map, "date");
    if let None = date_option {
        let message = format!("vaccination error: No 'date' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let date = date_option.unwrap();

    let valid_from_option = get_child_string_from_cbor_map(&vaccination_map, "validFrom");
    if let None = valid_from_option {
        let message = format!("vaccination error: No 'valid_from' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let valid_from = valid_from_option.unwrap();

    let lot_option = get_child_string_from_cbor_map(&vaccination_map, "lot");
    if let None = lot_option {
        let message = format!("vaccination error: No 'lot' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let lot = lot_option.unwrap();

    let centre_option = get_child_string_from_cbor_map(&vaccination_map, "centre");
    if let None = centre_option {
        let message = format!("vaccination error: No 'centre' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let centre = centre_option.unwrap();

    let next_dose_option = get_child_string_from_cbor_map(&vaccination_map, "nextDose");
    if let None = next_dose_option {
        let message = format!("vaccination error: No 'next_dose' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let next_dose = next_dose_option.unwrap();

    Ok(Vaccination {
        date,
        dose,
        vaccine: vaccine_code_system,
        country: country_code_system,
        maholder: maholder_code_system,
        lot,
        centre,
        brand: brand_code_system,
        manufacturer: manufacturer_code_system,
        valid_from,
        total_doses,
        practitioner: practitioner_value_struct,
        disease: disease_code_system,
        next_dose,
    })
}

pub fn get_hc1_struct(payload: &Vec<u8>) -> anyhow::Result<DdccCoreDataSet> {
    let vaccination_result = get_vaccination_struct(&payload);
    if let Err(e) = vaccination_result {
        let message = format!("Error getting vaccine data: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let vaccination = vaccination_result.unwrap();

    // identification fields
    let resource_type_option = get_string_by_name_from_vec(&payload, "resourceType");
    if let None = resource_type_option {
        let message = format!("No 'resourceType' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let resource_type = resource_type_option.unwrap();

    let birth_date_option = get_string_by_name_from_vec(&payload, "birthDate");
    if let None = birth_date_option {
        let message = format!("No 'birthDate' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let birth_date = birth_date_option.unwrap();

    let name_option = get_string_by_name_from_vec(&payload, "name");
    if let None = name_option {
        let message = format!("No 'name' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let name = name_option.unwrap();

    let identifier_option = get_string_by_name_from_vec(&payload, "identifier");
    if let None = identifier_option {
        let message = format!("No 'identifier' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let identifier = identifier_option.unwrap();

    let sex_option = get_string_by_name_from_vec(&payload, "sex");
    if let None = sex_option {
        let message = format!("No 'sex' found");
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let sex = sex_option.unwrap();

    // certificate
    let certificate_result = get_certificate_struct(&payload);
    if let Err(e) = certificate_result {
        let message = format!("Error getting 'certificate' data: {}", e);
        debug!("{}", message);
        return Err(anyhow::anyhow!(message));
    }
    let certificate = certificate_result.unwrap();

    Ok(DdccCoreDataSet {
        vaccination,
        resource_type,
        birth_date,
        name,
        identifier,
        sex,
        certificate,
    })
}

pub async fn is_valid_message(
    db: &DatabaseConnection,
    message: &mut CoseMessage,
    country_code: String,
    trace_id: Uuid,
) -> anyhow::Result<bool> {
    match message.header.alg {
        Some(alg) => {
            match get_cose_keys_by_country_code(db, &country_code, Some(trace_id), &alg).await {
                Ok(cose_keys) => {
                    let result = cose_keys.into_iter().enumerate().find(|(idx, key)| {
                        match message.key(&key) {
                            Ok(_) => {}
                            Err(e) => {
                                debug!(
                                    "TRACE_ID: {}, DESCRIPTION (key attachment): {:?}",
                                    trace_id, &e
                                );
                                return false;
                            }
                        };
                        match message.decode(None, None) {
                            Ok(_) => {
                                debug!(
                                    "TRACE_ID: {}: Successful verification in iteration #{}",
                                    trace_id,
                                    idx + 1
                                );
                                return true;
                            }
                            Err(e) => {
                                debug!(
                                    "TRACE_ID: {}, DESCRIPTION (validation failed in iteration #{}): {:?}",
                                    trace_id,
                                    idx + 1,
                                    &e
                                );
                                return false;
                            }
                        }
                    });
                    match result {
                        Some(_) => {
                            return Ok(true);
                        }
                        None => {
                            let message = format!("No key matched");
                            debug!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, message);
                            return Ok(false);
                        }
                    }
                }
                Err(e) => {
                    let message = "Internal Error while getting keys";
                    debug!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, &e);
                    return Err(anyhow::anyhow!(message));
                }
            };
        }
        None => {
            let message = "No algoritm found for incoming message";
            debug!("TRACE_ID: {}, DESCRIPTION ({})", trace_id, message);
            return Err(anyhow::anyhow!(message));
        }
    }
}

pub async fn verify_base45(
    db: &DatabaseConnection,
    data: String,
) -> Responses<Json<SuccessMessage<HC1ValidationResponseDto>>, Json<ErrorMessage<'static>>> {
    let data = data.trim();
    let data: String = data.replace("HC1:", "");
    let trace_id: Uuid = Uuid::new_v4();
    match decode(&data) {
        Ok(zlib_encoded) => {
            info!("New Verification request: {:?}", trace_id);
            let mut zlib_data = ZlibDecoder::new(zlib_encoded.as_bytes());
            let mut cose_full_message = Vec::new();
            let _ = zlib_data.read_to_end(&mut cose_full_message).unwrap();

            let mut cose_message = CoseMessage::new_sign();
            cose_message.bytes = cose_full_message;

            match cose_message.init_decoder(None) {
                Ok(_) => {
                    let payload = cose_message.payload.clone();
                    let hc1_result = get_hc1_struct(&payload);
                    if let Err(e) = hc1_result {
                        let message = "message decoding failed";
                        debug!(
                            "TRACE_ID: {}, DESCRIPTION ({}), error was: {}",
                            trace_id, message, e
                        );
                        return Responses::BadRequest(Json::from(ErrorMessage {
                            message,
                            trace_id: trace_id.to_string(),
                        }));
                    }
                    let ddcc_core_data_set = hc1_result.unwrap();
                    info!("hc1 struct: {:?}", ddcc_core_data_set);

                    let is_valid_result = is_valid_message(
                        db,
                        &mut cose_message,
                        ddcc_core_data_set.vaccination.country.code.clone(),
                        trace_id,
                    )
                    .await;
                    if let Err(e) = is_valid_result {
                        let message = "message validation failed";
                        debug!(
                            "TRACE_ID: {}, DESCRIPTION ({}), error was: {}",
                            trace_id, message, e
                        );
                        return Responses::BadRequest(Json::from(ErrorMessage {
                            message,
                            trace_id: trace_id.to_string(),
                        }));
                    }

                    let is_valid = is_valid_result.unwrap();

                    return Responses::Sucess(Json::from(SuccessMessage {
                        data: HC1ValidationResponseDto {
                            is_valid,
                            ddcc_core_data_set,
                        },
                        trace_id: trace_id.to_string(),
                    }));
                }
                Err(e) => {
                    debug!(
                        "TRACE_ID: {}, DESCRIPTION (init decoder): {:?}",
                        trace_id, &e
                    );
                    Responses::BadRequest(Json::from(ErrorMessage {
                        message: "Failed while trying to decode COSE message",
                        trace_id: trace_id.to_string(),
                    }))
                }
            }
        }
        Err(e) => {
            debug!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, &e);
            Responses::BadRequest(Json::from(ErrorMessage {
                message: "Invalid Base45 encoded message",
                trace_id: trace_id.to_string(),
            }))
        }
    }
}

async fn _get_pem_test_keys() -> anyhow::Result<Vec<String>> {
    Ok(get_rsa_pem_test_keys().unwrap()) // secure since keys are pre-established- just for testing purposes
}

#[allow(dead_code)]
fn get_rsa_pem_test_keys() -> Option<Vec<String>> {
    // call redis and obtain stream of keys in pem format
    let mut pem_keys = Vec::new();
    let lacchain_cert = "-----BEGIN CERTIFICATE-----
    MIIEfDCCAmSgAwIBAgIUKfVsK6TJIMYWxATipARQVKOgN5gwDQYJKoZIhvcNAQEN
    BQAwSDELMAkGA1UEBhMCVVMxCzAJBgNVBAgMAkNBMRswGQYDVQQKDBJNaW5pc3Ry
    eSBPZiBIZWFsdGgxDzANBgNVBAMMBkNBLU1vSDAeFw0yMzA4MDkxNDU1NDRaFw0y
    NDEyMjExNDU1NDRaME8xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEhMB8GA1UE
    CgwYRFNDIC0gTWluaXN0cnkgb2YgSGVhbHRoMRAwDgYDVQQDDAdEU0MtTW9IMIIB
    IjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAzxp1DiBxSi2wzZoVbuH9cZIc
    +td+LjQ3DxKOvRt32tUw+EnLAgiQPANhp/M0Rrsxty9mq9KXg59NrUrbe3BbyHxK
    +imQJXD6vs21wt/HPc2KgmJ9n9jz4DcqK+FZuvoucmsv/7oRZQO0Xhevd1FxzjiX
    hKyrb+Wf+dQsLwrdEug+xQG9D8ye6cvUHDMj0FgdkVFY8Jtf25i5t99i5u1LG/h2
    xzK3QDrs1lACzyVxEktY2Sss5/aLES+gIo1o8EXMi9FkFsi7OXJX8vmvfE4YR+gc
    pbnjE7vT8saDbv2SNFNotLW5P3gEvLVds02AD0dz8c8Hl5ny23K4C/xQzmGnQQID
    AQABo1cwVTAJBgNVHRMEAjAAMB0GA1UdDgQWBBT+ZC6bTgAvVFzkplz/LvdH0oyf
    wjALBgNVHQ8EBAMCB4AwHAYDVR0RBBUwE4cEAQIDBIILbXkuZG5zLm5hbWUwDQYJ
    KoZIhvcNAQENBQADggIBAJctk6hY+/NPQ3V8WGNhnXOjqjLNrM+EBEe1NFETiyvX
    oXe5bESF0GjrQxI5bpiBI3/GfTdI4CyDLLxi6YBTeegHwhPaY51H5AF3MMF7uSuQ
    gzSyPMoXGbxhzsMbPw71Ecr2ZrhvFaLH3xB+3g4aUUeFDn8pr7eeS1MQoFpiFkYk
    +cU44lvNt34DuASR3dEuqUvCDLt0z29ysfjNs5hxU12rYH8uj5vPRJMS1LdmdEsV
    TlofKRUYeGfPzxw4vagVwEV+Ht/J8quSufwBD3aljHQhWFBGCYBSoKJrOes5jpT2
    +NBtIGBK9Vq8rWG9myLSy3dpBQFRMUKlQn6ZsDrKspv0Wd2/2EF/DOD4mTl4e+bH
    8E4gXA98gxTn/Eo47A/FUnLh1DDE9odVys/iJgXKakjDxXCPDBhBCso1OrC2d/uI
    dCs88yZaMqn6ASj+JtHXnJMFedHLSMj9aIOTYn2SWznNUX+COu5uGYkepQhl6DU6
    g9DbauiaVbZ+v5bH7OUr3SYOfr5GnnSD0b9MiqKC2iQEVt2yVZXsK31jMojKB1/0
    siOe0PV+zx2e+Ke4efrqBEIrfH+m5Yv/ePuuFLC7WqrtPh3Kh38bCBR9JXmY6r6H
    M3OBAZWKWffO2Pdjl2guuhqgwofeNALrfJeZEeGpNc2hPGZK6UNhpKB7F1QOiR2s
    -----END CERTIFICATE-----";
    let begin = "BEGIN CERTIFICATE";
    let end = "END CERTIFICATE";
    let mut lacchain_cert = lacchain_cert.replace("\n", "");
    lacchain_cert.retain(|c| !c.is_whitespace());
    let lacchain_cert = lacchain_cert
        .replace("BEGINCERTIFICATE", begin)
        .replace("ENDCERTIFICATE", end);
    let create_cert = "-----BEGIN CERTIFICATE-----
    MIIFXTCCA0WgAwIBAgIUXttAp46FGR4WOjpyWb8a2HeTwgMwDQYJKoZIhvcNAQELBQAwPjELMAkGA1UEBhMCQ0wxCzAJBgNVBAgMAk1UMREwDwYDVQQHDAhTYW50aWFnbzEPMA0GA1UECgwGQ3JlYXRlMB4XDTIzMDgwOTE0Mzc1OFoXDTI0MDgwODE0Mzc1OFowPjELMAkGA1UEBhMCQ0wxCzAJBgNVBAgMAk1UMREwDwYDVQQHDAhTYW50aWFnbzEPMA0GA1UECgwGQ3JlYXRlMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA667LWuB7DHWl5WTu+SKPk22QpgVvs2SJ3Bnux7G5M0CzjulrsZO16Bkw3cBOseSWPiaXV4vQMg7s58vOez26H2wvoLMjsAhGXjCgwAO9zvTkMLmgjmn0eNW5PTsLR9eHlxkPNq79MWGI7znPtmhVcyzugckVfbcTCtnrvPzAqeWio2mkkRfNLa6jq7gmGi8BoeZfugRZAi+C1J5cCnqKK5yLpsB96Zym1W+6NLDeLUZ/CfdRGCsISdehkUYLhg1nb0SxgapL4LQ6K7zeBQZtQM4GeHVCnB/1oNUt45Kjq/h6nKl1ZeKmoYSPVOmsDOlHxUCssrimusoKbUd8/DFkadSHQPSl/iDsR0mrkYDmL0h0+67sejLIPq3tU52fhre9PPMgXXPSUdOD9/tUdvwmc7RLmM/ulL/tJH18ICZLK0GM/hcNmLUMVck32vpIfH4C9GR2LNSdeAlzRxajPq+J8zp+aCSnm0UlFbzGgTy/RuCZg2RYx0f6u8A2cXiefI/4E09vLwvAUFl44I4z3XND/aKlR7lTJnhY5iPg2HgJkftyrlm6pl2ENSW+c8Xw5EOLtkG+y+/pdF93YDI2qKL9PwEhDr73lmtXjhtobwZ1Sg8sZGg83rUaI6RrKZn2DNvOwcEnIoAbIog5hMD03/c6AYb3yZR63r+eptVThUDp4sECAwEAAaNTMFEwHQYDVR0OBBYEFCigJrDkS+j0cVzk6/j3xvYeNyMHMB8GA1UdIwQYMBaAFCigJrDkS+j0cVzk6/j3xvYeNyMHMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADggIBAHvRztwPbdidBNc4zg9K5bbU/8coVwTb4qMMYSzFHcFLAqa1AShI5jvFoFpp98ILdVbg2R3e02DPtrw7SQn3Gb9xSEGO45/dymTDHW6Pez/Q5Q7QrPLIe5i2f1gIjsMGhtW4/tMvmT7qYCma85s3pY+Ea4TSS/jlcoJ6HW/KY74WeOxsSshWoeT6weogBtnLxTsHZOWuJuLpiQcNWh0SqExihwfEjN+CZQQzHFjHj/BcGXS0ckbjlUVPuRokIkfO4oOyQwfgbM/Gk+tQA9XnowANcP1i/CLEC/GwOggs2r9blnb94zqvy5BEMYhUQjNRnBudSrsBdkSxrjIyHVMBer3XuWaxjqsaaVOZkI8mtcKlIYj2F4SP78iFSHdRLWv/QF1pnjqtpQkl21rIQvdiOWDLiSloRwT94F3hGRSSBVSlw7E4eqv+YIaJ/49JOja2Ezr/XpYWfWUAZl8kL6cj7SDqtldG1T4Z29ukcRZ74aWh88MIBc0hswJCr5MPTn0jPaf+w3TRQJyJcPeB05pKmBrz9DN5baZgjAJLUlSHM5WJzS7vQj+7b4x98D1C31AEgB5+PU5dRdUdSPfqm6zetAeG1kEyjJDv0/0sDQERcmNjZolH+5pHnbJKF1elM0VjRSe6J8ZIxK2sYp9d9twCjr9XlC0l8lL5pQGYqKd6l2/F
    -----END CERTIFICATE-----";
    pem_keys.push(lacchain_cert.to_owned());
    pem_keys.push(create_cert.to_owned());
    Some(pem_keys)
}

#[allow(dead_code)]
pub(crate) fn get_p256_pem_test_keys() -> Option<Vec<String>> {
    let mut pem_keys = Vec::new();
    let pem_cert = "-----BEGIN CERTIFICATE-----
    MIIB8TCCAZagAwIBAgIUJKdl9T2GbSYmHns/gbZWFDFdLXQwCgYIKoZIzj0EAwQw
    SDELMAkGA1UEBhMCVVMxCzAJBgNVBAgMAkNBMRswGQYDVQQKDBJNaW5pc3RyeSBP
    ZiBIZWFsdGgxDzANBgNVBAMMBkNBLU1vSDAeFw0yMzA4MTMxNTU5NDRaFw0yNDEy
    MjUxNTU5NDRaME8xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEhMB8GA1UECgwY
    RFNDIC0gTWluaXN0cnkgb2YgSGVhbHRoMRAwDgYDVQQDDAdEU0MtTW9IMFkwEwYH
    KoZIzj0CAQYIKoZIzj0DAQcDQgAELItVqrJgxpDlM2a7+XzsYZI/iDdsBOXlQw8v
    ISHyMgmpCV6W449m76YeyobYQrlxTznalLZAi7dmnML1D9fkF6NXMFUwCQYDVR0T
    BAIwADAdBgNVHQ4EFgQU3v2TKjW/tALEPuSquRYCMkwRMqIwCwYDVR0PBAQDAgeA
    MBwGA1UdEQQVMBOHBAECAwSCC215LmRucy5uYW1lMAoGCCqGSM49BAMEA0kAMEYC
    IQCpVLj/D8Ai+6Z77118Q1mYDaf28FnjdEfzle+yflguPQIhAPio4utr6irjvxlS
    mLPoZq8IqTcacI4Dqsuyu0xk8xH+
    -----END CERTIFICATE-----";
    let begin = "BEGIN CERTIFICATE";
    let end = "END CERTIFICATE";
    let mut pem_cert = pem_cert.replace("\n", "");
    pem_cert.retain(|c| !c.is_whitespace());
    let pem_cert = pem_cert
        .replace("BEGINCERTIFICATE", begin)
        .replace("ENDCERTIFICATE", end);
    pem_keys.push(pem_cert.to_owned());
    Some(pem_keys)
}
#[cfg(test)]
mod tests {
    use cose::algs;

    use crate::services::x509::x509_utils::X509Utils;

    use super::*;
    // use std::println;
    #[test]
    fn get_cose_keys_containing_rsa_key_test() {
        let pem_keys = get_rsa_pem_test_keys().unwrap();
        let cose_keys = X509Utils::pem_to_cose_keys(pem_keys, &algs::PS256).unwrap();
        assert_eq!(cose_keys.len(), 2)
    }

    #[test]
    fn get_cose_keys_containing_p256_key_test() {
        let pem_keys = get_p256_pem_test_keys().unwrap();
        let cose_keys = X509Utils::pem_to_cose_keys(pem_keys, &algs::ES256).unwrap();
        assert_eq!(cose_keys.len(), 1);
    }
}
