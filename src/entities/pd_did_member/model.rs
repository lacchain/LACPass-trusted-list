use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "pd_did_member")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub did_id: Uuid,
    pub pd_member_id: Uuid,
    pub block_number: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::entities::entities::DidEntity",
        from = "Column::DidId",
        to = "crate::entities::did::model::Column::Id"
    )]
    Did,
    #[sea_orm(
        belongs_to = "crate::entities::entities::PdMemberEntity",
        from = "Column::PdMemberId",
        to = "crate::entities::pd_member::model::Column::Id"
    )]
    PdMember,
}

impl Related<crate::entities::pd_member::model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PdMember.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
