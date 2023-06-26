use crate::entities::entities::PublicKeyEntity;
use crate::entities::models::{PublicKeyActiveModel, PublicKeyModel};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;
pub struct PublicKeyService {}

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
        did_id: &Uuid,
        block_number: &u64,
        jwk: Vec<u8>,
        content_hash: &str,
        exp: &u64,
        is_compromised: bool,
    ) -> anyhow::Result<PublicKeyModel> {
        let db_registry = PublicKeyActiveModel {
            id: Set(Uuid::new_v4()),
            did_id: Set(*did_id),
            block_number: Set(*block_number as i64),
            jwk: Set(jwk),
            content_hash: Set(content_hash.to_owned()),
            exp: Set(*exp as i64),
            is_compromised: Set(is_compromised),
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
                            s.block_number = Set(v as i64);
                        }
                        None => {}
                    }
                    match exp {
                        Some(v) => {
                            s.exp = Set(v as i64);
                        }
                        None => {}
                    }
                    match is_compromised {
                        Some(v) => {
                            s.is_compromised = Set(v);
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
}
