use crate::{
    responses::{
        error_message::ErrorMessage, generic_response::Responses, success_messages::SuccessMessage,
    },
    services::x509::x509_utils::X509Utils,
};
use base45::decode;
use cbor::Decoder;
use cose::{keys::CoseKey, message::CoseMessage};
use flate2::read::ZlibDecoder;
use log::{debug, info};
use nom::AsBytes;
use rocket::serde::json::Json;
use std::io::Read;
use uuid::Uuid;

pub async fn get_pem_keys_by_country(_country_code: &str) -> anyhow::Result<Vec<String>> {
    // TODO: call redis and obtain stream of keys in pem format
    get_pem_test_keys().await
}

/// Returns cose keys according to cose-rust library format.
pub async fn get_cose_keys_by_country_code(
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
    match get_pem_keys_by_country(country_code).await {
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
    match message.header.alg {
        Some(alg) => {
            match get_cose_keys_by_country_code(&country_code, Some(trace_id), &alg).await {
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
        None => {
            let message = "No algoritm found for incoming message";
            debug!("TRACE_ID: {}, DESCRIPTION ({})", trace_id, message);
            return Responses::BadRequest(Json::from(ErrorMessage {
                message: "Internal Error while getting keys",
                trace_id: trace_id.to_string(),
            }));
        }
    }
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

async fn get_pem_test_keys() -> anyhow::Result<Vec<String>> {
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
