use super::*;
use crate::entities::entities::DidEntity;
use crate::entities::entities::PdDidMemberEntity;
use crate::entities::entities::PdMemberEntity;
use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::public_directory::model::Column as Pd;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::Query;
use sea_orm::{entity::*, query::*};

impl DidEntity {
    pub fn find_by_did(did: &str) -> Select<Self> {
        Self::find().filter(model::Column::Did.contains(did))
    }

    pub fn find_all(public_directory_contract_address: &str, chain_id: &str) -> Select<Self> {
        let public_directory_contract_address = public_directory_contract_address.to_owned();
        let chain_id = chain_id.to_owned();
        Self::find().filter(
            Condition::any().add(
                model::Column::Id.in_subquery(
                    Query::select()
                        .column(crate::entities::pd_did_member::model::Column::DidId)
                        .from(PdDidMemberEntity)
                        .join(
                            JoinType::InnerJoin,
                            PdMemberEntity,
                            Expr::col((
                                PdDidMemberEntity,
                                crate::entities::pd_did_member::model::Column::PdMemberId,
                            ))
                            .equals((
                                PdMemberEntity,
                                crate::entities::pd_member::model::Column::Id,
                            )),
                        )
                        .join(
                            JoinType::InnerJoin,
                            PublicDirectoryEntity,
                            Expr::col((
                                PdMemberEntity,
                                crate::entities::pd_member::model::Column::PublicDirectoryId,
                            ))
                            .equals((PublicDirectoryEntity, Pd::Id))
                            .and(
                                Expr::col((PublicDirectoryEntity, Pd::ContractAddress))
                                    .eq(public_directory_contract_address.clone())
                                    .and(Pd::ChainId.contains(&chain_id)),
                            ),
                        )
                        .to_owned(),
                ),
            ),
        )
    }
}
