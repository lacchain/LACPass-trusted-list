use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct MemberData {
    #[serde(rename = "identificationData")]
    pub identification_data: Option<IdentificationData>,
    #[serde(rename = "certificateAuthority")]
    pub certificate_authority: String,
    pub version: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct IdentificationData {
    pub id: String,
    #[serde(rename = "legalName")]
    pub legal_name: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "url")]
    pub url: String,
}
