use base64::{engine::general_purpose, Engine};
use cose::keys::{self, CoseKey};
use log::debug;
use x509_certificate::{rfc5280, X509Certificate};

use crate::{
    dto::response::public_key_response_dto::Jwk,
    services::validator::certificate_error::CertificateError,
};

pub struct X509Utils {}

impl X509Utils {
    const BEGIN: &'static str = "-----BEGIN CERTIFICATE-----";
    const END: &'static str = "-----END CERTIFICATE-----";
    /// returns a string pem certificate that contains "-----BEGIN CERTIFICATE-----" and "-----END CERTIFICATE-----"
    /// Trims whitespaces and break lines
    pub fn format_pem(pem_candidate: String) -> String {
        let begin = "BEGIN CERTIFICATE";
        let end = "END CERTIFICATE";
        let mut pem_cert = pem_candidate.replace("\n", "");
        pem_cert.retain(|c| !c.is_whitespace());
        if pem_cert.contains("BEGINCERTIFICATE") {
            let pem_cert = pem_cert
                .replace("BEGINCERTIFICATE", begin)
                .replace("ENDCERTIFICATE", end);
            return pem_cert;
        }
        let mut cert = Vec::new();
        cert.push(Self::BEGIN);
        cert.push(&pem_cert);
        cert.push(Self::END);
        let pem_cert = cert.join("");
        pem_cert
    }

    pub fn get_expiration_from_pem(pem_cert: String) -> anyhow::Result<u64> {
        let pem_cert = Self::format_pem(pem_cert.clone());
        match X509Certificate::from_pem(pem_cert) {
            Ok(x509_key) => {
                let cert = rfc5280::Certificate::from(x509_key);
                let validity = cert.tbs_certificate.validity;
                match validity.not_after {
                    x509_certificate::asn1time::Time::UtcTime(t1) => {
                        return Ok(t1.timestamp() as u64);
                    }
                    x509_certificate::asn1time::Time::GeneralTime(t2) => {
                        let s = chrono::DateTime::from(t2);
                        return Ok(s.timestamp() as u64);
                    }
                }
            }
            Err(e) => {
                let message = format!("Get expiration: failed to parse certificate: {:?}", e);
                debug!("{}", message);
                return Err(anyhow::anyhow!(message));
            }
        }
    }

    /// Given a pem certificate this method removes "------BEGIN CERTIFICATE-----" as well as "------ENDCERTIFICATE-----"
    /// Also removes whitespaces and break lines.
    pub fn trim_pem(pem_cert: String) -> String {
        let mut t1 = pem_cert
            .replace(&Self::BEGIN, "")
            .replace(&Self::END, "")
            .replace("\n", "");
        t1.retain(|c| !c.is_whitespace());
        t1
    }
    pub fn get_jwk_from_pem(pem_cert: String) -> anyhow::Result<Jwk> {
        let pem_cert = Self::format_pem(pem_cert);
        let trimmed_for_x5c = Self::trim_pem(pem_cert.clone());
        let mut x5c = Vec::new();
        x5c.push(trimmed_for_x5c);
        let x5c = Some(x5c);
        match x509_certificate::X509Certificate::from_pem(pem_cert.clone()) {
            Ok(x509_key) => match x509_key.key_algorithm() {
                Some(alg) => match alg {
                    x509_certificate::KeyAlgorithm::Rsa => {
                        let s = x509_key.rsa_public_key_data();
                        let rsa_public_key = s.unwrap();
                        let n: Vec<u8> = rsa_public_key.clone().modulus.into_bytes().to_vec();
                        let e = rsa_public_key.clone().public_exponent.into_bytes().to_vec();
                        let jwk = Ok(Jwk {
                            kty: Some("RSA".to_owned()),
                            e: Some(base64_url::encode(&e)),
                            alg: None,
                            r#use: None,
                            kid: None,
                            x5c,
                            n: Some(base64_url::encode(&n)),
                            x: None,
                            y: None,
                            crv: None,
                            x5t: None,
                        });
                        jwk
                    }
                    x509_certificate::KeyAlgorithm::Ecdsa(curve) => {
                        let pub_key = x509_key.public_key_data().to_owned();
                        let pub_key = pub_key.to_vec();

                        match curve {
                            x509_certificate::EcdsaCurve::Secp256r1 => {
                                match Self::get_x_y(pub_key) {
                                    Err(e) => {
                                        let message = format!(
                                            "DESCRIPTION (x/y coordinate extraction): {:?}",
                                            &e
                                        );
                                        debug!("{}", message);
                                        return Err(anyhow::anyhow!(message));
                                    }
                                    Ok(xy) => {
                                        let jwk = Ok(Jwk {
                                            alg: None,
                                            r#use: None,
                                            kty: Some("EC".to_owned()),
                                            kid: None,
                                            x5c,
                                            x5t: None,
                                            n: None,
                                            e: None,
                                            x: Some(base64_url::encode(&xy.get(0).unwrap())),
                                            y: Some(base64_url::encode(&xy.get(1).unwrap())),
                                            crv: Some("P-256".to_owned()),
                                        });
                                        jwk
                                    }
                                }
                            }
                            x509_certificate::EcdsaCurve::Secp384r1 => {
                                let message =
                                    format!("Ecdsa jwk for Secp384r1 is not implemented yet");
                                debug!("{}", message);
                                return Err(anyhow::anyhow!(message));
                            }
                        }
                    }
                    x509_certificate::KeyAlgorithm::Ed25519 => {
                        let message = format!("Ed25519 jwk encoding Not implemented yet");
                        debug!("{}", message);
                        return Err(anyhow::anyhow!(message));
                    }
                },
                None => {
                    let message = format!("Invalid key algorithm, got 'None'");
                    debug!("{}", message);
                    return Err(anyhow::anyhow!(message));
                }
            },
            Err(e) => {
                let message = format!("failed to parse certificate: {:?}", e);
                debug!("{}", message);
                return Err(anyhow::anyhow!(message));
            }
        }
    }

    /// Returns cose keys according to cose-rust library format.
    /// If some key in the incoming "pem_keys" argument is not valid then it is just ommited.
    pub fn pem_to_cose_keys(pem_keys: Vec<String>, signing_alg: &i32) -> Option<Vec<CoseKey>> {
        let cose_keys = pem_keys
        .into_iter()
        .filter_map(
            |pem_key| match x509_certificate::X509Certificate::from_pem(pem_key.clone()) {
                Err(e) => {
                    debug!(
                        "DESCRIPTION (Public Key Pem Decoding): Parsing to X509 Object failed: {:?} {:?}", pem_key, &e
                    );
                    None
                }
                Ok(v) => {
                    match v.key_algorithm() {
                        Some(alg) => {
                            debug!("key_algorithm {:?}", alg);
                            match alg {
                                x509_certificate::KeyAlgorithm::Rsa => {
                                    match v.rsa_public_key_data() {
                                        Ok(rsa_public_key) => {
                                            let n: Vec<u8> = rsa_public_key.clone().modulus.into_bytes().to_vec();
                                            let e = rsa_public_key.clone().public_exponent.into_bytes().to_vec();
                                            let mut key = keys::CoseKey::new();
                                            key.kty(keys::RSA);
                                            key.n(n);
                                            key.e(e);
                                            key.alg(*signing_alg);
                                            key.key_ops(vec![keys::KEY_OPS_VERIFY]);
                                            return Some(key);
                                        },
                                        Err(e) => {
                                            debug!("DESCRIPTION (x509 rsa public key extraction): {:?}", &e); 
                                            return None;
                                        },
                                    }
                                },
                                x509_certificate::KeyAlgorithm::Ecdsa(ecdsa_curve) => { // p-256 (according to https://github.com/WorldHealthOrganization/tng-participants-dev)
                                    match ecdsa_curve {
                                        x509_certificate::EcdsaCurve::Secp256r1 => {
                                            let pub_key = v.public_key_data().to_owned();
                                            let pub_key = pub_key.to_vec();
                                            match Self::get_x_y(pub_key) {
                                                Err(e) => {
                                                    debug!(
                                                        "DESCRIPTION (x/y coordinate extraction): {:?}", &e
                                                    );
                                                    return None;
                                                },
                                                Ok(xy) => {
                                                    let mut key = keys::CoseKey::new();
                                                    key.kty(keys::P_256);
                                                    key.x(xy.get(0).unwrap().to_owned());
                                                    key.y(xy.get(1).unwrap().to_owned());
                                                    key.alg(*signing_alg);
                                                    key.key_ops(vec![keys::KEY_OPS_VERIFY]);
                                                    return Some(key);
                                                },
                                            }
                                        },
                                        x509_certificate::EcdsaCurve::Secp384r1 => None,
                                    }
                                },
                                x509_certificate::KeyAlgorithm::Ed25519 => None,
                            }
                        },
                        None => {
                            debug!(
                                "DESCRIPTION (Public Key Pem decoding): No key algorithm found"
                            );
                            return None;
                        }
                    }
                }
            },
        )
        .collect::<Vec<_>>();
        Some(cose_keys)
    }

    /// expect 65 bytes containing 1 byte to indicate full compression, 32 bytes for x coordinate and 64 bytes for y coordinate
    pub fn get_x(pub_key: Vec<u8>) -> Result<Vec<u8>, CertificateError> {
        if pub_key.is_empty() || pub_key.len() != 65 {
            return Err(CertificateError::INVALID);
        }
        Ok((1..33)
            .into_iter()
            .map(|i| pub_key.get(i).unwrap().to_owned())
            .collect::<Vec<_>>())
    }

    pub fn get_y(pub_key: Vec<u8>) -> Result<Vec<u8>, CertificateError> {
        if pub_key.is_empty() || pub_key.len() != 65 {
            return Err(CertificateError::INVALID);
        }
        Ok((33..65)
            .into_iter()
            .map(|i| pub_key.get(i).unwrap().to_owned())
            .collect::<Vec<_>>())
    }

    /// Returns 64 bytes, the first 32 corresponding to "x" coordinate and the second 32 bytes corresponding to "y" coordinate
    /// of the EC curve
    fn get_x_y(pub_key: Vec<u8>) -> Result<Vec<Vec<u8>>, CertificateError> {
        match Self::get_x(pub_key.clone()) {
            Ok(x) => match Self::get_y(pub_key) {
                Ok(y) => {
                    let mut r = Vec::new();
                    r.push(x);
                    r.push(y);
                    Ok(r)
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    /// given a pem certificate it removes all whitespaces, break lines, header and footer and then decodes this according to base64
    pub fn get_decoded_pem_bytes(pem_cert: String) -> anyhow::Result<Vec<u8>> {
        let formated_pem = X509Utils::trim_pem(pem_cert.to_string());
        let decoded_base64 = general_purpose::STANDARD.decode(formated_pem);
        match decoded_base64 {
            Ok(decoded) => Ok(decoded),
            Err(e) => {
                let message = format!("Unable to decode pem cert, error was: {:?}", e);
                debug!("{}", message);
                return Err(anyhow::anyhow!(message));
            }
        }
    }

    pub fn get_pem_from_string_jwk(jwk_str: &str) -> anyhow::Result<String> {
        match serde_json::from_str::<Jwk>(jwk_str) {
            Ok(jwk) => match jwk.x5c {
                Some(x5c) => match x5c.get(0) {
                    Some(pem_candidate) => return Ok(pem_candidate.to_string()),
                    None => {
                        let message = format!("No fields were found in x5c");
                        debug!("{}", message);
                        return Err(anyhow::anyhow!(message));
                    }
                },
                None => {
                    let message = format!("Unable to extract x5c from jwk");
                    debug!("{}", message);
                    return Err(anyhow::anyhow!(message));
                }
            },
            Err(e) => {
                let message = format!("Unable to parse string to jwk, error was: {}", &e);
                debug!("{}", message);
                return Err(anyhow::anyhow!(message));
            }
        }
    }
}

#[allow(dead_code)]
fn get_rsa_pem_test_keys() -> Option<Vec<String>> {
    // call redis and obtain stream of keys in pem format
    let mut pem_keys = Vec::new();
    let lacchain_cert = "-----BEGIN CERTIFICATE-----
    MIIErTCCApWgAwIBAgIUVchUxtzzkaaCN7uGIY+YP24A8iIwDQYJKoZIhvcNAQEN
    BQAwZDELMAkGA1UEBhMCQ0wxETAPBgNVBAgMCFNhbnRpYWdvMSwwKgYDVQQKDCNN
    aW5pc3RyeSBvZiBIZWFsdGggLSBMQUNQYXNzIC0gRGVtbzEUMBIGA1UEAwwLTW9I
    X0xQX0RlbW8wHhcNMjMwOTI1MTk0MjQ0WhcNMjUwMjA2MTk0MjQ0WjBkMQswCQYD
    VQQGEwJDTDERMA8GA1UECAwIU2FudGlhZ28xLDAqBgNVBAoMI01pbmlzdHJ5IG9m
    IEhlYWx0aCAtIExBQ1Bhc3MgLSBEZW1vMRQwEgYDVQQDDAtNb0hfTFBfRGVtbzCC
    ASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAK8cXWc+j6PkqEwZJyEuGlAs
    OeHoq0CeSFCQ92ZWtX+VmRcYaaOeTcR2ZQQaVUKVMxbUHm+1DLD2XerE9Amg6S75
    ILgwUGI10xIWrEt43ZwI4d3kOvyItNxhOrMAsM6sF3vdVSfbouhXPU13wwbOGpKk
    W/S0YjxzM/HVt7hP82ImvJ6TOmyA0QLIGSbamxWuB+YJnl646AD2lqeJcZajzUYs
    +hes4ShbjBRp4AspDFPyY8IHqBidDmwKcRrWCmtK4rGK8Gv7ryOacdY8YxvOUqml
    mnQGlUTXV8Y9OCriIGYmoNG/U2VX5IHiHsXN7rxIycaezQBkXAzqyJ2AVVqkXucC
    AwEAAaNXMFUwCQYDVR0TBAIwADAdBgNVHQ4EFgQUS+aOmyNJE8QDmDJclQ+NWeLV
    /1AwCwYDVR0PBAQDAgeAMBwGA1UdEQQVMBOHBAECAwSCC215LmRucy5uYW1lMA0G
    CSqGSIb3DQEBDQUAA4ICAQBnIlGge+PczNYSZIzQgGrtKCL2uKp7eR6MLSuVOoKg
    0ewI9bMt6093/lcyNKO2XttvkjWa+pIZ6jfh8psnv9JQpXCqSH35vfFP6pc9/dFJ
    FDRG9Nw+e3vx567wE50YhW1TQQcsKycXz8HZjPNryZH1drtLsLkORqRzH+jkWp4d
    SQ8YYvoC6N4u6zTDI8FCyfcoQL7+mTmwAjYAl5fvwlgmkvNZeZ31JrWXcNx1PvJv
    9OuaRdbLUHwPwWBUKwDBdO06XctXxGT221lUIiymkU/gAr8QJP25HM4wtMhCk9i9
    jeRxzGIM71Uq7Q+EjSasfXEfQbbsTOa2NSOw/EuZARve2qspHQCYNAq1SWAeBQyU
    lPuEcFZgdTGyKPVGtoqIHvJlt0Gobm0m0A46Om1UTG7b82fsA44gduX1YU+Xp19q
    b+hGU9u07aWLEbH0sXiuitZzrmI7R6koyhR7ZZ9/X+apVWg+ICcC3uhIrk/BCTE+
    PkwD2Iw79hIFuO58PMw6F4+HusQz8XfT5Z+KiZGoazQ0HOk2NVnHGjXmZ8OMZcmu
    Rm3OlEhGS9C60U/r7vishcAC42AupfysH60nRLCc3l5iCGVSjUyiZ+PYXpxrbqgL
    0/5iKtbVWjCyT7vUckwgmKcOki/gNmSpqyhyhMFiu7MwkxwXcydZCuB8y4P1F+z4
    WA==
    -----END CERTIFICATE-----";
    let begin = "BEGIN CERTIFICATE";
    let end = "END CERTIFICATE";
    let mut lacchain_cert = lacchain_cert.replace("\n", "");
    lacchain_cert.retain(|c| !c.is_whitespace());
    let lacchain_cert = lacchain_cert
        .replace("BEGINCERTIFICATE", begin)
        .replace("ENDCERTIFICATE", end);
    let create_cert = "-----BEGIN CERTIFICATE-----
    MIIErTCCApWgAwIBAgIUVchUxtzzkaaCN7uGIY+YP24A8iIwDQYJKoZIhvcNAQENBQAwZDELMAkGA1UEBhMCQ0wxETAPBgNVBAgMCFNhbnRpYWdvMSwwKgYDVQQKDCNNaW5pc3RyeSBvZiBIZWFsdGggLSBMQUNQYXNzIC0gRGVtbzEUMBIGA1UEAwwLTW9IX0xQX0RlbW8wHhcNMjMwOTI1MTk0MjQ0WhcNMjUwMjA2MTk0MjQ0WjBkMQswCQYDVQQGEwJDTDERMA8GA1UECAwIU2FudGlhZ28xLDAqBgNVBAoMI01pbmlzdHJ5IG9mIEhlYWx0aCAtIExBQ1Bhc3MgLSBEZW1vMRQwEgYDVQQDDAtNb0hfTFBfRGVtbzCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAK8cXWc+j6PkqEwZJyEuGlAsOeHoq0CeSFCQ92ZWtX+VmRcYaaOeTcR2ZQQaVUKVMxbUHm+1DLD2XerE9Amg6S75ILgwUGI10xIWrEt43ZwI4d3kOvyItNxhOrMAsM6sF3vdVSfbouhXPU13wwbOGpKkW/S0YjxzM/HVt7hP82ImvJ6TOmyA0QLIGSbamxWuB+YJnl646AD2lqeJcZajzUYs+hes4ShbjBRp4AspDFPyY8IHqBidDmwKcRrWCmtK4rGK8Gv7ryOacdY8YxvOUqmlmnQGlUTXV8Y9OCriIGYmoNG/U2VX5IHiHsXN7rxIycaezQBkXAzqyJ2AVVqkXucCAwEAAaNXMFUwCQYDVR0TBAIwADAdBgNVHQ4EFgQUS+aOmyNJE8QDmDJclQ+NWeLV/1AwCwYDVR0PBAQDAgeAMBwGA1UdEQQVMBOHBAECAwSCC215LmRucy5uYW1lMA0GCSqGSIb3DQEBDQUAA4ICAQBnIlGge+PczNYSZIzQgGrtKCL2uKp7eR6MLSuVOoKg0ewI9bMt6093/lcyNKO2XttvkjWa+pIZ6jfh8psnv9JQpXCqSH35vfFP6pc9/dFJFDRG9Nw+e3vx567wE50YhW1TQQcsKycXz8HZjPNryZH1drtLsLkORqRzH+jkWp4dSQ8YYvoC6N4u6zTDI8FCyfcoQL7+mTmwAjYAl5fvwlgmkvNZeZ31JrWXcNx1PvJv9OuaRdbLUHwPwWBUKwDBdO06XctXxGT221lUIiymkU/gAr8QJP25HM4wtMhCk9i9jeRxzGIM71Uq7Q+EjSasfXEfQbbsTOa2NSOw/EuZARve2qspHQCYNAq1SWAeBQyUlPuEcFZgdTGyKPVGtoqIHvJlt0Gobm0m0A46Om1UTG7b82fsA44gduX1YU+Xp19qb+hGU9u07aWLEbH0sXiuitZzrmI7R6koyhR7ZZ9/X+apVWg+ICcC3uhIrk/BCTE+PkwD2Iw79hIFuO58PMw6F4+HusQz8XfT5Z+KiZGoazQ0HOk2NVnHGjXmZ8OMZcmuRm3OlEhGS9C60U/r7vishcAC42AupfysH60nRLCc3l5iCGVSjUyiZ+PYXpxrbqgL0/5iKtbVWjCyT7vUckwgmKcOki/gNmSpqyhyhMFiu7MwkxwXcydZCuB8y4P1F+z4WA==
    -----END CERTIFICATE-----";
    pem_keys.push(lacchain_cert.to_owned());
    pem_keys.push(create_cert.to_owned());
    Some(pem_keys)
}

#[allow(dead_code)]
fn get_p256_pem_test_keys() -> Option<Vec<String>> {
    // call redis and obtain stream of keys in pem format
    let mut pem_keys = Vec::new();
    let lacchain_cert = "-----BEGIN CERTIFICATE-----
    MIIB8TCCAZagAwIBAgIUVMPmb9VzhvWhfBQLcjG7yS6+Py4wCgYIKoZIzj0EAwQw
    SDELMAkGA1UEBhMCVVMxCzAJBgNVBAgMAkNBMRswGQYDVQQKDBJNaW5pc3RyeSBP
    ZiBIZWFsdGgxDzANBgNVBAMMBkNBLU1vSDAeFw0yMzA5MjYwNDMwMjFaFw0yNTAy
    MDcwNDMwMjFaME8xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEhMB8GA1UECgwY
    RFNDIC0gTWluaXN0cnkgb2YgSGVhbHRoMRAwDgYDVQQDDAdEU0MtTW9IMFkwEwYH
    KoZIzj0CAQYIKoZIzj0DAQcDQgAEWY9cYJMCATULyyMS8WRtZao09HnBotms6ynA
    eF1dJ471FiGPWp5AjpRmd2pnHnkLHAxbdTEUYhFRwVsowsY4SaNXMFUwCQYDVR0T
    BAIwADAdBgNVHQ4EFgQU+SB2R0Cff1Vf6Gf9M5k25Nu6JqMwCwYDVR0PBAQDAgeA
    MBwGA1UdEQQVMBOHBAECAwSCC215LmRucy5uYW1lMAoGCCqGSM49BAMEA0kAMEYC
    IQD8JKiU8LB+saxWpbjvAwkGghYjKwSL3B9X/VKeZin3EQIhAPDiuOvM9G9W5ger
    Yz/thKgQfKOtQS9JbgASgQSCeW4i
    -----END CERTIFICATE-----";
    let begin = "BEGIN CERTIFICATE";
    let end = "END CERTIFICATE";
    let mut lacchain_cert = lacchain_cert.replace("\n", "");
    lacchain_cert.retain(|c| !c.is_whitespace());
    let lacchain_cert = lacchain_cert
        .replace("BEGINCERTIFICATE", begin)
        .replace("ENDCERTIFICATE", end);
    pem_keys.push(lacchain_cert.to_owned());
    Some(pem_keys)
}

#[cfg(test)]
mod tests {
    // use std::println;

    use super::*;
    #[test]
    fn get_expiration_from_pem_test() {
        let pem_keys = get_rsa_pem_test_keys().unwrap();
        let pem_key = pem_keys.get(0).unwrap();
        match X509Utils::get_expiration_from_pem(pem_key.to_string()) {
            Ok(v) => {
                println!("Expiration {}", v);
                assert_eq!(v, 1738870964);
            }
            Err(_) => {
                assert_eq!(true, false);
            }
        }
    }

    #[test]
    fn get_decoded_pem_bytes_rsa_test() {
        let pem_keys = get_rsa_pem_test_keys().unwrap();
        let pem_key = pem_keys.get(1).unwrap();
        match X509Utils::get_decoded_pem_bytes(pem_key.to_string()) {
            Ok(_v) => {
                assert_eq!(true, true);
            }
            Err(_e) => {
                assert_eq!(true, false);
            }
        }
    }
    #[test]
    fn get_decoded_pem_bytes_p256_test() {
        let pem_keys = get_p256_pem_test_keys().unwrap();
        let pem_key = pem_keys.get(0).unwrap();
        match X509Utils::get_decoded_pem_bytes(pem_key.to_string()) {
            Ok(_v) => {
                assert_eq!(true, true);
            }
            Err(_e) => {
                assert_eq!(true, false);
            }
        }
    }
}
