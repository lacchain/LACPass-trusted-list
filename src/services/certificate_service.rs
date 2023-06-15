use rocket::serde::json::Json;
use uuid::Uuid;

use crate::responses::{
    error_message::ErrorMessage, generic_response::Responses, success_messages::SuccessMessage,
};

pub async fn verify_certificate(
    _data: String,
) -> Responses<Json<SuccessMessage<bool>>, Json<ErrorMessage<'static>>> {
    let trace_id: Uuid = Uuid::new_v4();
    Responses::Sucess(Json::from(SuccessMessage {
        data: true,
        trace_id: trace_id.to_string(),
    }))
}
