use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::models::PublicDirectoryModel;
use sea_orm::DatabaseConnection;

pub struct PublicDirectoryService {}

impl PublicDirectoryService {
    pub async fn get_public_directory(
        db: &DatabaseConnection,
        public_directory_address: &str,
        chain_id: &str,
    ) -> Result<Option<PublicDirectoryModel>, sea_orm::DbErr> {
        PublicDirectoryEntity::find_by_contract_address(public_directory_address, chain_id)
            .one(db)
            .await
    }
}
