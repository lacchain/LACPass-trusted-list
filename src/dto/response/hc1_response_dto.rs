use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::okapi::schemars::{self, JsonSchema};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct HC1ValidationResponseDto {
    pub is_valid: bool,
    pub ddcc_core_data_set: DdccCoreDataSet,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct DdccCoreDataSet {
    pub vaccination: Vaccination,
    pub resource_type: String,
    pub birth_date: String,
    pub name: String,
    pub identifier: String,
    pub sex: String,
    pub certificate: Certificate,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Vaccination {
    pub date: String,
    pub dose: u8,
    pub vaccine: CodeSystem,
    pub country: CodeSystem,
    pub maholder: CodeSystem,
    pub lot: String,
    pub centre: String,
    pub brand: CodeSystem,
    pub manufacturer: CodeSystem,
    pub valid_from: String,
    pub total_doses: u8,
    pub practitioner: Value,
    pub disease: CodeSystem,
    pub next_dose: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub hcid: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    pub issuer: Identifier,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CodeSystem {
    pub code: String,
    pub system: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Identifier {
    pub identifier: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Value {
    pub value: String,
}
