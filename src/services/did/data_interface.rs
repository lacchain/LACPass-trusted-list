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
        db: &DatabaseConnection,
        did: &str,
    ) -> Result<Option<DidModel>, sea_orm::DbErr> {
        DidEntity::find_by_did(did).one(db).await
    }

    pub async fn insert_did_to_database(
        db: &DatabaseConnection,
        did: &str,
        upper_block: Option<u64>,
        last_processed_block: Option<u64>,
        last_block_saved: Option<u64>,
    ) -> anyhow::Result<DidModel> {
        let mut ub = 0_i64;
        match upper_block {
            Some(v) => {
                ub = v as i64;
            }
            None => {}
        }

        let mut lpb = 0_i64;
        match last_processed_block {
            Some(v) => {
                lpb = v as i64;
            }
            None => {}
        }

        let mut lbs = 0_i64;
        match last_block_saved {
            Some(v) => {
                lbs = v as i64;
            }
            None => {}
        }

        let db_registry = DidActiveModel {
            id: Set(Uuid::new_v4()),
            did: Set(did.to_owned()),
            upper_block: Set(ub),
            last_processed_block: Set(lpb),
            last_block_saved: Set(lbs),
        };
        match db_registry.insert(db).await {
            Ok(res) => return Ok(res),
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn find_all(
        db: &DatabaseConnection,
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Result<Vec<DidModel>, sea_orm::DbErr> {
        DidEntity::find_all(public_directory_contract_address, chain_id)
            .all(db)
            .await
    }

    pub async fn update(
        db: &DatabaseConnection,
        upper_block: Option<u64>,
        last_processed_block: Option<u64>,
        last_block_saved: Option<u64>,
        did: &str,
    ) -> anyhow::Result<DidModel> {
        match DidDataInterfaceService::get_did_from_database(&db, did).await {
            Ok(v) => match v {
                Some(m) => {
                    let mut s: DidActiveModel = m.into();
                    match upper_block {
                        Some(v) => {
                            s.upper_block = Set(v as i64);
                        }
                        _ => {}
                    }

                    match last_processed_block {
                        Some(v) => {
                            s.last_processed_block = Set(v as i64);
                        }
                        _ => {}
                    }

                    match last_block_saved {
                        Some(v) => {
                            s.last_block_saved = Set(v as i64);
                        }
                        _ => {}
                    }
                    match s.update(db).await {
                        Ok(v) => Ok(v),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                None => panic!("Error, registry doesn't exist"),
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
