use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "public_key")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub country_code: String,
    #[sea_orm(column_type = "Text")]
    pub content_hash: String,
    pub jwk: Vec<u8>,
    pub exp: Option<i64>,
    pub is_compromised: Option<bool>,
    pub did_id: Option<Uuid>,
    pub block_number: Option<i64>,
    pub url: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::entities::did::model::Entity",
        from = "Column::DidId",
        to = "crate::entities::did::model::Column::Id"
    )]
    Did,
}

impl Related<crate::entities::did::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Did.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
