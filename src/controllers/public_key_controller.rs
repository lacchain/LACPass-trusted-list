use crate::databases::pool::Db;
use crate::dto::response::public_key_response_dto::PublicKeyResponseDto;
use crate::responses::error_message::ErrorMessage;
use crate::responses::generic_response::Responses;
use crate::responses::success_messages::SuccessMessage;
use crate::services::public_key::data_interface::PublicKeyService;
use crate::utils::utils::Utils;
use crate::CONTROLLER_TRUSTED_REGISTRY;
use log::error;
use rocket::get;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use sea_orm_rocket::Connection;
use uuid::Uuid;

/// # Return public keys
#[openapi(tag = "Public keys")]
#[get("/get-all?<page>&<results_per_page>")]
pub async fn get_all(
    connection: Connection<'_, Db>,
    page: Option<u64>,
    results_per_page: Option<u64>,
) -> Responses<Json<SuccessMessage<PublicKeyResponseDto>>, Json<ErrorMessage<'static>>> {
    match CONTROLLER_TRUSTED_REGISTRY.get() {
        Some(tr) => {
            let public_directory_contract_address = Utils::vec_u8_to_hex_string(
                tr.public_directory.contract_address.as_bytes().to_vec(),
            )
            .unwrap();
            let public_directory_chain_id = &tr.public_directory.chain_id;
            PublicKeyService::get_all_from_lacchain(
                connection,
                page,
                results_per_page,
                &public_directory_contract_address,
                &public_directory_chain_id,
            )
            .await
        }
        None => {
            let trace_id = Uuid::new_v4();
            let message = "Unable to get Trusted Registiries";
            error!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, message);
            return Responses::BadRequest(Json::from(ErrorMessage {
                message,
                trace_id: trace_id.to_string(),
            }));
        }
    }
}
