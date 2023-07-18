use super::*;
use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::entities::{PdDidMemberEntity, PdMemberEntity};
use crate::entities::public_directory::model::Column as Pd;
use sea_orm::sea_query::{Expr, IntoCondition};
use sea_orm::{entity::*, query::*};
use uuid::Uuid;

impl PdDidMemberEntity {
    pub fn find_pd_did_member(did_id: Uuid, pd_member_id: Uuid) -> Select<Self> {
        Self::find().filter(
            model::Column::DidId
                .eq(did_id)
                .and(model::Column::PdMemberId.eq(pd_member_id)),
        )
    }

    pub fn find_by_pd_did_member_id(pd_did_member_id: &Uuid) -> Select<Self> {
        Self::find().filter(model::Column::Id.eq(*pd_did_member_id))
    }

    pub fn find_all(public_directory_contract_address: &str, chain_id: &str) -> Select<Self> {
        let public_directory_contract_address = public_directory_contract_address.to_owned();
        let chain_id = chain_id.to_owned();
        Self::find()
            .join(
                JoinType::InnerJoin,
                PdDidMemberEntity::belongs_to(PdMemberEntity)
                    .from(crate::entities::pd_did_member::model::Column::PdMemberId)
                    .to(crate::entities::pd_member::model::Column::Id)
                    .into(),
            )
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
    }
}
