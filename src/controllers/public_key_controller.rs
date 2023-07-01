use crate::databases::pool::Db;
use crate::dto::response::public_key_response_dto::PublicKeyResponseDto;
use crate::jobs::trusted_registries::TrustedRegistries;
use crate::responses::error_message::ErrorMessage;
use crate::responses::generic_response::Responses;
use crate::responses::success_messages::SuccessMessage;
use crate::services::public_key::data_interface::PublicKeyService;
use crate::services::trusted_registry::trusted_registry::TrustedRegistry;
use crate::utils::utils::Utils;
use log::error;
use rocket::get;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use sea_orm_rocket::Connection;

fn get_trusted_registry_by_index() -> TrustedRegistry {
    let trusted_registries = TrustedRegistries::process_env_trusted_registries();
    let index: String;
    match Utils::get_env_or_err("TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE") {
        Ok(s) => index = s,
        Err(e) => {
            error!("{}", e);
            panic!(
                "Please set TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE environment variable"
            );
        }
    }
    let tr = trusted_registries
        .into_iter()
        .filter(|e| e.index == index)
        .collect::<Vec<_>>();
    if tr.len() != 1 {
        let message = format!("TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE '{:?}' was (not found/or more than one) with the pointed index was found in TRUSTED_REGISTRIES", index);
        panic!("{}", message);
    };
    tr[0].clone()
}

/// # Return public keys
#[openapi(tag = "Public keys")]
#[get("/get-all?<page>&<results_per_page>")]
pub async fn get_all(
    connection: Connection<'_, Db>,
    page: Option<u64>,
    results_per_page: Option<u64>,
) -> Responses<Json<SuccessMessage<PublicKeyResponseDto>>, Json<ErrorMessage<'static>>> {
    let tr = get_trusted_registry_by_index();
    let public_directory_contract_address =
        Utils::vec_u8_to_hex_string(tr.public_directory.contract_address.as_bytes().to_vec())
            .unwrap();
    let public_directory_chain_id = tr.public_directory.chain_id;
    PublicKeyService::get_all(
        connection,
        page,
        results_per_page,
        &public_directory_contract_address,
        &public_directory_chain_id,
    )
    .await
}
