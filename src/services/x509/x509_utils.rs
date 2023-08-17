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
        let decoded_base64 = general_purpose::STANDARD_NO_PAD.decode(formated_pem);
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
                assert_eq!(v, 1734792944);
            }
            Err(_) => {
                assert_eq!(true, false);
            }
        }
    }
}
