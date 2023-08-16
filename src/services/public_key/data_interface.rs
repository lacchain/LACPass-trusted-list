use crate::databases::pool::Db;
use crate::dto::response::public_key_response_dto::{PublicKeyCoreResponse, PublicKeyResponseDto};
use crate::entities::entities::PublicKeyEntity;
use crate::entities::models::{PublicKeyActiveModel, PublicKeyModel};
use crate::responses::error_message::ErrorMessage;
use crate::responses::generic_response::Responses;
use crate::responses::success_messages::SuccessMessage;
use log::info;
use rocket::serde::json::Json;
use sea_orm::{ActiveModelTrait, DatabaseConnection, PaginatorTrait, Set};
use sea_orm_rocket::Connection;
use uuid::Uuid;
pub struct PublicKeyService {}

const DEFAULT_RESULTS_PER_PAGE: u64 = 10;

impl PublicKeyService {
    pub fn new() -> PublicKeyService {
        PublicKeyService {}
    }

    pub async fn find_public_key_by_content_hash(
        &self,
        db: &DatabaseConnection,
        content_hash: &str,
        did_id: &Uuid,
    ) -> Result<Option<PublicKeyModel>, sea_orm::DbErr> {
        PublicKeyEntity::find_by_hash_and_did_id(content_hash, did_id)
            .one(db)
            .await
    }

    pub async fn find_public_key_by_content_hash_and_country_code(
        &self,
        db: &DatabaseConnection,
        content_hash: &str,
        country_code: &str,
    ) -> Result<Option<PublicKeyModel>, sea_orm::DbErr> {
        PublicKeyEntity::find_by_hash_and_country_code(content_hash, country_code)
            .one(db)
            .await
    }

    pub async fn find_by_id(
        &self,
        db: &DatabaseConnection,
        id: &Uuid,
    ) -> Result<Option<PublicKeyModel>, sea_orm::DbErr> {
        PublicKeyEntity::find_by_id(id).one(db).await
    }

    pub async fn insert_public_key(
        &self,
        db: &DatabaseConnection,
        did_id: Option<Uuid>,
        block_number: Option<i64>,
        jwk: Vec<u8>,
        content_hash: &str,
        exp: &u64,
        is_compromised: Option<bool>,
        country_code: &str,
    ) -> anyhow::Result<PublicKeyModel> {
        let db_registry = PublicKeyActiveModel {
            id: Set(Uuid::new_v4()),
            did_id: Set(did_id),
            block_number: Set(block_number),
            jwk: Set(jwk),
            content_hash: Set(content_hash.to_owned()),
            exp: Set(Some(*exp as i64)),
            is_compromised: Set(is_compromised),
            country_code: Set(country_code.to_owned()),
        };
        match db_registry.insert(db).await {
            Ok(res) => return Ok(res),
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn update_public_key(
        &self,
        db: &DatabaseConnection,
        public_key_id: &Uuid,
        block_number: Option<u64>,
        exp: Option<u64>,
        is_compromised: Option<bool>,
    ) -> anyhow::Result<PublicKeyModel> {
        match self.find_by_id(db, public_key_id).await {
            Ok(v) => match v {
                Some(v) => {
                    let mut s: PublicKeyActiveModel = v.into();
                    match block_number {
                        Some(v) => {
                            s.block_number = Set(Some(v as i64));
                        }
                        None => {}
                    }
                    match exp {
                        Some(v) => {
                            s.exp = Set(Some(v as i64));
                        }
                        None => {}
                    }
                    match is_compromised {
                        Some(v) => {
                            s.is_compromised = Set(Some(v));
                        }
                        None => {}
                    }
                    match s.update(db).await {
                        Ok(res) => return Ok(res),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                None => {
                    return Err(anyhow::anyhow!(format!(
                        "Pd Did member with id {:?} does not exist",
                        public_key_id
                    )))
                }
            },
            Err(e) => return Err(e.into()),
        }
    }

    pub async fn get_all(
        connection: Connection<'_, Db>,
        page: Option<u64>,
        page_size: Option<u64>,
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Responses<Json<SuccessMessage<PublicKeyResponseDto>>, Json<ErrorMessage<'static>>> {
        let db = connection.into_inner();
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(DEFAULT_RESULTS_PER_PAGE);
        let trace_id: Uuid = Uuid::new_v4();
        match page {
            0 => {
                let message = "'page' param cannot be zero";
                error!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, message);
                return Responses::BadRequest(Json::from(ErrorMessage {
                    message,
                    trace_id: trace_id.to_string(),
                }));
            }
            _ => {}
        }
        let paginator =
            PublicKeyEntity::find_by_public_directory(public_directory_contract_address, chain_id)
                .paginate(db, page_size);
        let num_pages;
        match paginator.num_pages().await {
            Ok(r) => {
                num_pages = r;
            }
            Err(e) => {
                let message = "Internal error when retrieving public keys";
                error!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, &e);
                return Responses::BadRequest(Json::from(ErrorMessage {
                    message,
                    trace_id: trace_id.to_string(),
                }));
            }
        }

        let result = paginator
            .fetch_page((page - 1).try_into().unwrap())
            .await
            .map(|r| {
                r.into_iter()
                    .map(|el| {
                        return (String::from_utf8(el.jwk), el.country_code);
                    })
                    .filter_map(|el| match el.0 {
                        Ok(key) => Some((key, el.1)),
                        _ => None,
                    })
                    .map(|(jwk_str, country_code)| {
                        let parsed = serde_json::from_str(&jwk_str);
                        (parsed, country_code)
                    })
                    .filter_map(|el| match el.0 {
                        Ok(jwk) => Some(PublicKeyCoreResponse { country: el.1, jwk }),
                        Err(e) => {
                            info!("Error parsing {:?}", &e);
                            return None;
                        }
                    })
                    .collect::<Vec<_>>()
            });

        match result {
            Ok(keys) => {
                return Responses::Sucess(Json::from(SuccessMessage {
                    data: PublicKeyResponseDto {
                        page,
                        results_per_page: page_size,
                        num_pages,
                        keys,
                    },
                    trace_id: trace_id.to_string(),
                }));
            }
            Err(e) => {
                error!("TRACE_ID: {}, DESCRIPTION: {}", trace_id, &e);
                return Responses::BadRequest(Json::from(ErrorMessage {
                    message: "Internal error when retrieving public keys",
                    trace_id: trace_id.to_string(),
                }));
            }
        };
    }
}
