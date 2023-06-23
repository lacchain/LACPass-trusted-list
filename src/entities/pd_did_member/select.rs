use super::*;
use crate::entities::entities::PdDidMemberEntity;
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
}
