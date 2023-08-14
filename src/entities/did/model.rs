use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
// use

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "did")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub did: String,
    pub upper_block: i64,
    pub last_processed_block: i64,
    pub last_block_saved: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::entities::entities::PublicKeyEntity")]
    PublicKey,
}

impl Related<crate::entities::public_key::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PublicKey.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
