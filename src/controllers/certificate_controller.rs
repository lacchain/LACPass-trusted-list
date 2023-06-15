use rocket::data::{Limits, ToByteUnit};
use rocket::post;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::dto::raw::RawData;
use crate::responses::error_message::ErrorMessage;
use crate::responses::generic_response::Responses;
use crate::responses::success_messages::SuccessMessage;
use crate::services::certificate_service::verify_certificate as verify_certificate_service;

/// # Create contract registry config using params
#[openapi(tag = "Verify")]
#[post("/verify", data = "<data>")]
pub async fn verify_certificate(
    data: RawData<'_>,
    limits: &Limits,
) -> Responses<Json<SuccessMessage<bool>>, Json<ErrorMessage<'static>>> {
    limits.get("data").unwrap_or(1.megabytes());
    let data: &str = data.0;
    verify_certificate_service(data.to_string()).await
}
