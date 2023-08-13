use rocket::data::{Limits, ToByteUnit};
use rocket::post;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::dto::raw::RawData;
use crate::responses::error_message::ErrorMessage;
use crate::responses::generic_response::Responses;
use crate::responses::success_messages::SuccessMessage;
use crate::services::validator::certificate_service::verify_base45;

/// # Verify base45 HC1 health certificates
#[openapi(tag = "Verify From Base45")]
#[post("/verify-b45", format = "text/plain", data = "<data>")]
pub async fn verify_base45_certificate(
    data: RawData<'_>,
    limits: &Limits,
) -> Responses<Json<SuccessMessage<bool>>, Json<ErrorMessage<'static>>> {
    limits.get("data").unwrap_or(1.megabytes());
    let data: &str = data.0;
    verify_base45(data.to_string()).await
}
