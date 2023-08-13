use crate::responses::{
    error_message::ErrorMessage, generic_response::Responses, success_messages::SuccessMessage,
};
use base45::decode;
use cbor::Decoder;
use cose::{
    algs,
    keys::{self, CoseKey},
    message::CoseMessage,
};
use flate2::read::ZlibDecoder;
use log::{debug, info};
use nom::AsBytes;
use rocket::serde::json::Json;
use std::io::Read;
use uuid::Uuid;

/// Todo: Get from public key base directory instead
pub async fn get_keys_by_country_code(_country_code: &str) -> anyhow::Result<Vec<CoseKey>> {
    let mut cose_keys = Vec::new();

    let mut v = Vec::new();
    let n = base64_url::decode_to_vec(
        "zxp1DiBxSi2wzZoVbuH9cZIc-td-LjQ3DxKOvRt32tUw-EnLAgiQPANhp_M0Rrsxty9mq9KXg59NrUrbe3BbyHxK-imQJXD6vs21wt_HPc2KgmJ9n9jz4DcqK-FZuvoucmsv_7oRZQO0Xhevd1FxzjiXhKyrb-Wf-dQsLwrdEug-xQG9D8ye6cvUHDMj0FgdkVFY8Jtf25i5t99i5u1LG_h2xzK3QDrs1lACzyVxEktY2Sss5_aLES-gIo1o8EXMi9FkFsi7OXJX8vmvfE4YR-gcpbnjE7vT8saDbv2SNFNotLW5P3gEvLVds02AD0dz8c8Hl5ny23K4C_xQzmGnQQ",
        &mut v
    ).unwrap().to_vec();
    let lac_exp = "010001";
    let e = hex::decode(lac_exp).unwrap();
    let mut key = keys::CoseKey::new();
    key.kty(keys::RSA);
    key.n(n);
    key.e(e);
    key.alg(algs::PS256); // TODO: update from header
    key.key_ops(vec![keys::KEY_OPS_VERIFY]);
    cose_keys.push(key);

    let create_pub_key = "ebaecb5ae07b0c75a5e564eef9228f936d90a6056fb36489dc19eec7b1b93340b38ee96bb193b5e81930ddc04eb1e4963e2697578bd0320eece7cbce7b3dba1f6c2fa0b323b008465e30a0c003bdcef4e430b9a08e69f478d5b93d3b0b47d78797190f36aefd316188ef39cfb66855732cee81c9157db7130ad9ebbcfcc0a9e5a2a369a49117cd2daea3abb8261a2f01a1e65fba0459022f82d49e5c0a7a8a2b9c8ba6c07de99ca6d56fba34b0de2d467f09f751182b0849d7a191460b860d676f44b181aa4be0b43a2bbcde05066d40ce067875429c1ff5a0d52de392a3abf87a9ca97565e2a6a1848f54e9ac0ce947c540acb2b8a6baca0a6d477cfc316469d48740f4a5fe20ec4749ab9180e62f4874fbaeec7a32c83eaded539d9f86b7bd3cf3205d73d251d383f7fb5476fc2673b44b98cfee94bfed247d7c20264b2b418cfe170d98b50c55c937dafa487c7e02f464762cd49d7809734716a33eaf89f33a7e6824a79b452515bcc6813cbf46e099836458c747fabbc03671789e7c8ff8134f6f2f0bc0505978e08e33dd7343fda2a547b953267858e623e0d8780991fb72ae59baa65d843525be73c5f0e4438bb641becbefe9745f77603236a8a2fd3f01210ebef7966b578e1b686f06754a0f2c64683cdeb51a23a46b2999f60cdbcec1c12722801b22883984c0f4dff73a0186f7c9947adebf9ea6d5538540e9e2c1";
    let create_exp = "010001";
    let n = hex::decode(create_pub_key).unwrap();
    let e = hex::decode(create_exp).unwrap();
    let mut key = keys::CoseKey::new();
    key.kty(keys::RSA);
    key.n(n);
    key.e(e);
    key.alg(algs::PS256); // TODO: update from header
    key.key_ops(vec![keys::KEY_OPS_VERIFY]);
    cose_keys.push(key);

    Ok(cose_keys)
}

/// Returns a country code according to urn:iso:std:iso:3166
/// TODO: check against a list of valid countriy codes
/// TODO: Doesn' work with qrs found on ddcc validator
pub fn get_country_from_hc1_payload(payload: Vec<u8>) -> Option<String> {
    let mut issuer_country = Default::default();
    let mut d = Decoder::from_bytes(payload.clone());
    let _ = d
        .items()
        .into_iter()
        .map(|v| match v {
            Ok(c) => match c {
                cbor::Cbor::Map(el) => {
                    if el.contains_key("vaccination") {
                        match el.get("vaccination").unwrap() {
                            cbor::Cbor::Map(vaccine_fields) => {
                                if vaccine_fields.contains_key("country") {
                                    match vaccine_fields.get("country").unwrap() {
                                        cbor::Cbor::Map(ic) => {
                                            if ic.contains_key("code") {
                                                match ic.get("code").unwrap() {
                                                    cbor::Cbor::Unicode(found_code) => {
                                                        issuer_country = found_code.to_owned();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Err(_) => {}
        })
        .collect::<Vec<_>>();
    info!("issuer_country: {}", issuer_country);
    if issuer_country.is_empty() {
        return None;
    }
    Some(issuer_country)
}

pub async fn is_valid_message(
    message: &mut CoseMessage,
    country_code: String,
    trace_id: Uuid,
) -> Responses<Json<SuccessMessage<bool>>, Json<ErrorMessage<'static>>> {
    match get_keys_by_country_code(&country_code).await {
        Ok(cose_keys) => {
            let result = cose_keys.into_iter().find(|key| {
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
                        debug!("TRACE_ID: {}: Successful verification", trace_id);
                        return true;
                    }
                    Err(e) => {
                        debug!("TRACE_ID: {}, DESCRIPTION (validation): {:?}", trace_id, &e);
                        return false;
                    }
                }
            });
            match result {
                Some(_) => {
                    return Responses::Sucess(Json::from(SuccessMessage {
                        data: true,
                        trace_id: trace_id.to_string(),
                    }));
                }
                None => {
                    debug!("TRACE_ID: {}, DESCRIPTION: No key matched", trace_id);
                    return Responses::Sucess(Json::from(SuccessMessage {
                        data: false,
                        trace_id: trace_id.to_string(),
                    }));
                }
            }
        }
        Err(e) => {
            debug!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, &e);
            return Responses::BadRequest(Json::from(ErrorMessage {
                message: "Internal Error while getting keys",
                trace_id: trace_id.to_string(),
            }));
        }
    };
}

pub async fn verify_base45(
    data: String,
) -> Responses<Json<SuccessMessage<bool>>, Json<ErrorMessage<'static>>> {
    let data = data.trim();
    let data = data.replace("HC1:", "");
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
                    match get_country_from_hc1_payload(payload) {
                        Some(country_code) => {
                            return is_valid_message(&mut cose_message, country_code, trace_id)
                                .await
                        }
                        None => {
                            let message = "Country Code not found";
                            debug!("TRACE_ID: {}, DESCRIPTION: ({})", trace_id, message);
                            return Responses::BadRequest(Json::from(ErrorMessage {
                                message,
                                trace_id: trace_id.to_string(),
                            }));
                        }
                    }
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
