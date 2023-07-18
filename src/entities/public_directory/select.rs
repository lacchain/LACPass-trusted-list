use super::*;
use crate::entities::entities::PublicDirectoryEntity;
use sea_orm::{entity::*, query::*};

impl PublicDirectoryEntity {
    pub fn find_by_contract_address(contract_address: &str, chain_id: &str) -> Select<Self> {
        Self::find().filter(
            model::Column::ContractAddress
                .contains(contract_address)
                .and(model::Column::ChainId.contains(chain_id)),
        )
    }
}
