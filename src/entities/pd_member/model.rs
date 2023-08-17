use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "pd_member")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub member_id: i64,
    pub exp: i64,
    pub public_directory_id: Uuid,
    pub block_number: i64,
    pub country_code: String,
    pub url: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::entities::entities::PdDidMemberEntity")]
    PdDidMember,
}

impl Related<crate::entities::pd_did_member::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PdDidMember.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
