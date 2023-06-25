use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::Select;
use uuid::Uuid;

use crate::entities::entities::PublicKeyEntity;

use super::model;

impl PublicKeyEntity {
    pub fn find_by_hash_and_did_id(content_hash: &str, did_id: &Uuid) -> Select<Self> {
        Self::find().filter(
            model::Column::ContentHash
                .contains(content_hash)
                .and(model::Column::DidId.eq(*did_id)),
        )
    }

    pub fn find_by_id(id: &Uuid) -> Select<Self> {
        Self::find().filter(model::Column::Id.eq(*id))
    }
}
