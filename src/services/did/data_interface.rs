use crate::entities::entities::DidEntity;
use crate::entities::models::DidActiveModel;
use crate::entities::models::DidModel;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Set;
use uuid::Uuid;

pub struct DidDataInterfaceService {}

impl DidDataInterfaceService {
    pub fn new() -> DidDataInterfaceService {
        DidDataInterfaceService {}
    }
    pub async fn get_did_from_database(
        &self,
        db: &DatabaseConnection,
        did: &str,
    ) -> Result<Option<DidModel>, sea_orm::DbErr> {
        DidEntity::find_by_did(did).one(db).await
    }

    pub async fn insert_did_to_database(
        &self,
        db: &DatabaseConnection,
        did: &str,
    ) -> anyhow::Result<DidModel> {
        let db_registry = DidActiveModel {
            id: Set(Uuid::new_v4()),
            did: Set(did.to_owned()),
        };
        match db_registry.insert(db).await {
            Ok(res) => return Ok(res),
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn find_all(
        &self,
        db: &DatabaseConnection,
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Result<Vec<DidModel>, sea_orm::DbErr> {
        DidEntity::find_all(public_directory_contract_address, chain_id)
            .all(db)
            .await
    }
}
