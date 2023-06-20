use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::models::PublicDirectoryModel;
use crate::services::trusted_registry::trusted_registry::Contract;
use sea_orm::DatabaseConnection;

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

    pub async fn get_last_block(&self, db: &DatabaseConnection) -> anyhow::Result<i64> {
        match self.get_public_directory_from_database(&db).await {
            Ok(result) => match result {
                Some(v) => Ok(v.last_block_saved),
                None => Ok(0),
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
