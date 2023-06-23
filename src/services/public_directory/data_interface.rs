use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::models::PublicDirectoryActiveModel;
use crate::entities::models::PublicDirectoryModel;
use crate::services::trusted_registry::trusted_registry::Contract;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Set;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DataInterfaceService {
    params: Contract,
}

impl DataInterfaceService {
    pub fn new(params: Contract) -> DataInterfaceService {
        DataInterfaceService { params }
    }
    pub async fn get_public_directory_from_database(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Option<PublicDirectoryModel>, sea_orm::DbErr> {
        PublicDirectoryEntity::find_by_contract_address(
            &self.params.contract_address.to_string(),
            &self.params.chain_id,
        )
        .one(db)
        .await
    }

    pub async fn get_last_block(&self, db: &DatabaseConnection) -> anyhow::Result<u64> {
        match self.get_public_directory_from_database(&db).await {
            Ok(result) => match result {
                Some(v) => Ok(v.last_block_saved as u64),
                None => Ok(0),
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    /// updates
    pub async fn update(
        &self,
        db: &DatabaseConnection,
        upper_block: Option<u64>,
        last_processed_block: Option<u64>,
        last_block_saved: Option<u64>,
    ) -> anyhow::Result<PublicDirectoryModel> {
        match self.get_public_directory_from_database(&db).await {
            Ok(v) => match v {
                Some(m) => {
                    let mut s: PublicDirectoryActiveModel = m.into();
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

    /// updates or inserts
    pub async fn save_contract_last_block(
        &self,
        db: &DatabaseConnection,
        contract_last_block: &u64,
    ) -> anyhow::Result<PublicDirectoryModel> {
        match self.get_public_directory_from_database(&db).await {
            Ok(v) => match v {
                Some(m) => {
                    let mut s: PublicDirectoryActiveModel = m.into();
                    s.upper_block = Set(*contract_last_block as i64);
                    s.last_processed_block = Set(0);
                    match s.update(db).await {
                        Ok(v) => Ok(v),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                None => {
                    let db_registry = PublicDirectoryActiveModel {
                        id: Set(Uuid::new_v4()),
                        contract_address: Set(self.params.contract_address.to_string()),
                        upper_block: Set(*contract_last_block as i64),
                        last_processed_block: Set(0),
                        last_block_saved: Set(0),
                        chain_id: Set(self.params.chain_id.clone()),
                    };
                    match db_registry.insert(db).await {
                        Ok(v) => Ok(v),
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
