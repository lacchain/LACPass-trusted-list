use super::*;
use crate::entities::entities::PdMemberEntity;
use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::public_directory::model::Column as Pd;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{entity::*, query::*};
use uuid::Uuid;

impl PdMemberEntity {
    pub fn find_by_public_directory_id(public_directory_id: &str) -> Select<Self> {
        Self::find().filter(model::Column::PublicDirectoryId.contains(public_directory_id))
    }

    pub fn find_by_pd_member_id(pd_member_id: &Uuid) -> Select<Self> {
        Self::find().filter(model::Column::Id.eq(*pd_member_id))
    }

    pub fn find_pd_member(
        member_id: &i64,
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Select<Self> {
        let public_directory_contract_address = public_directory_contract_address.to_owned();
        let chain_id = chain_id.to_owned();
        Self::find()
            .join(
                JoinType::InnerJoin,
                PdMemberEntity::belongs_to(PublicDirectoryEntity)
                    .from(crate::entities::pd_member::model::Column::PublicDirectoryId)
                    .to(Pd::Id)
                    .on_condition(move |_left, right| {
                        Expr::col((right, Pd::ContractAddress))
                            .eq(public_directory_contract_address.clone())
                            .and(Pd::ChainId.contains(&chain_id))
                            .into_condition()
                    })
                    .into(),
            )
            .filter(model::Column::MemberId.eq(*member_id))
    }
}
