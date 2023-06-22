use super::*;
use crate::entities::entities::DidEntity;
use sea_orm::{entity::*, query::*};

impl DidEntity {
    pub fn find_by_did(did: &str) -> Select<Self> {
        Self::find().filter(model::Column::Did.contains(did))
    }
}
