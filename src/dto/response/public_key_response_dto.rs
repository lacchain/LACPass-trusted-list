use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::okapi::schemars::{self, JsonSchema};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct PublicKeyResponseDto {
    pub page: u64,
    pub results_per_page: u64,
    pub num_pages: u64,
    pub keys: Vec<PublicKeyCoreResponse>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct PublicKeyCoreResponse {
    pub country: String,
    pub jwk: Jwk,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Jwk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#use: Option<String>,
    pub kty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    pub x5c: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5t: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<String>,
}
