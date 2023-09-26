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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<Certificate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Vaccination {
    pub date: String,
    pub dose: u8,
    pub vaccine: CodeSystem,
    pub country: CodeSystem,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maholder: Option<CodeSystem>,
    pub lot: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub centre: Option<String>,
    pub brand: CodeSystem,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<CodeSystem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_doses: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub practitioner: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease: Option<CodeSystem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_dose: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub hcid: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    pub issuer: Identifier,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CodeSystem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Identifier {
    pub identifier: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Value {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Period {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}
