use sea_orm::sea_query::Expr;
use sea_orm::sea_query::Query;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::DbBackend;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::QueryTrait;
use sea_orm::RelationTrait;
use sea_orm::Select;
use sea_orm::SelectModel;
use sea_orm::SelectorRaw;
use uuid::Uuid;

use crate::entities::entities::PdDidMemberEntity;
use crate::entities::entities::PublicKeyEntity;

use crate::entities::entities::PdMemberEntity;
use crate::entities::entities::PublicDirectoryEntity;
use crate::entities::pd_member;
use crate::entities::public_directory::model::Column as Pd;

use super::model;

#[derive(Debug, FromQueryResult)]
pub struct PublicKeyMemberView {
    pub country_code: String,
    pub id: Uuid,
    pub did_id: Uuid,
    pub block_number: i64,
    pub jwk: Vec<u8>,
    pub content_hash: String,
    pub exp: i64,
    pub is_compromised: bool,
}

impl PublicKeyEntity {
    pub fn find_by_hash_and_did_id(content_hash: &str, did_id: &Uuid) -> Select<Self> {
        Self::find().filter(
            model::Column::ContentHash
                .contains(content_hash)
                .and(model::Column::DidId.eq(*did_id)),
        )
    }

    pub fn find_by_hash_and_country_code(content_hash: &str, country_code: &str) -> Select<Self> {
        Self::find().filter(
            model::Column::ContentHash
                .contains(content_hash)
                .and(model::Column::CountryCode.eq(country_code)),
        )
    }

    pub fn find_by_id(id: &Uuid) -> Select<Self> {
        Self::find().filter(model::Column::Id.eq(*id))
    }
    pub fn find_by_public_directory(
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Select<Self> {
        let public_directory_contract_address = public_directory_contract_address.to_owned();
        let chain_id = chain_id.to_owned();
        let select = Self::find()
            .column(pd_member::model::Column::CountryCode)
            .join_rev(
                JoinType::InnerJoin,
                crate::entities::did::model::Relation::PublicKey.def(),
            )
            .join_rev(
                JoinType::InnerJoin,
                crate::entities::pd_did_member::model::Relation::Did.def(),
            )
            .join_rev(
                JoinType::InnerJoin,
                crate::entities::pd_member::model::Relation::PdDidMember.def(),
            )
            .filter(
                Condition::any().add(
                    model::Column::DidId.in_subquery(
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
            );
        select
    }
    pub fn find_with_country(
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> SelectorRaw<SelectModel<PublicKeyMemberView>> {
        let pd_select = Self::find_by_public_directory(public_directory_contract_address, chain_id);
        PublicKeyMemberView::find_by_statement(pd_select.build(DbBackend::Postgres))
    }
}
