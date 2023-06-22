use super::*;
use crate::entities::entities::PdMemberEntity;
use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::public_directory::model::Column as Pd;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{entity::*, query::*};

impl PdMemberEntity {
    pub fn find_by_public_directory_id(public_directory_id: &str) -> Select<Self> {
        Self::find().filter(model::Column::PubicDirectoryId.contains(public_directory_id))
    }

    pub fn find_pd_member(
        member_id: &i64,
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Select<Self> {
        let public_directory_contract_address = public_directory_contract_address.to_owned();
        let chain_id = chain_id.to_owned();
        Self::find()
            .join_rev(
                JoinType::InnerJoin,
                PdMemberEntity::belongs_to(PublicDirectoryEntity)
                    .from(crate::entities::pd_member::model::Column::PubicDirectoryId)
                    .to(Pd::Id)
                    .on_condition(move |_left, right| {
                        Expr::col((right, Pd::ContractAddress))
                            .contains(public_directory_contract_address.clone())
                            .and(Pd::ChainId.contains(&chain_id))
                            .into_condition()
                    })
                    .into(),
            )
            .filter(model::Column::MemberId.eq(*member_id))
    }
}
