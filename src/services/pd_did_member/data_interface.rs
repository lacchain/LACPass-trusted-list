use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::Set;
use uuid::Uuid;

use crate::services::did::data_interface::DidDataInterfaceService;
use crate::services::pd_member::data_interface::PdMemberDataInterfaceService;

use crate::entities::entities::PdDidMemberEntity;
use crate::entities::models::PdDidMemberActiveModel;
use crate::entities::models::PdDidMemberModel;

pub struct PdDidMemberDataInterfaceService {
    pub pd_member_data_service: PdMemberDataInterfaceService,
}

impl PdDidMemberDataInterfaceService {
    pub fn new(pd_member_data_service: PdMemberDataInterfaceService) -> Self {
        Self {
            pd_member_data_service,
        }
    }

    pub async fn find_all(
        &self,
        db: &DatabaseConnection,
        public_directory_contract_address: &str,
        chain_id: &str,
    ) -> Result<Vec<PdDidMemberModel>, sea_orm::DbErr> {
        PdDidMemberEntity::find_all(public_directory_contract_address, chain_id)
            .all(db)
            .await
    }

    pub async fn get_pd_did_member(
        &self,
        db: &DatabaseConnection,
        did: &str,
        member_id: &i64,
    ) -> Result<Option<PdDidMemberModel>, sea_orm::DbErr> {
        match DidDataInterfaceService::get_did_from_database(db, did).await {
            Ok(u) => match u {
                Some(found_did) => match self
                    .pd_member_data_service
                    .get_pd_member_from_database(db, member_id)
                    .await
                {
                    Ok(u) => match u {
                        Some(found_pd_member) => {
                            PdDidMemberEntity::find_pd_did_member(found_did.id, found_pd_member.id)
                                .one(db)
                                .await
                        }
                        None => return Ok(None),
                    },
                    Err(e) => return Err(e.into()),
                },
                None => return Ok(None),
            },
            Err(e) => return Err(e.into()),
        }
    }

    pub async fn get_pd_did_member_by_ids(
        &self,
        db: &DatabaseConnection,
        did_id: &Uuid,
        pd_member_id: &Uuid,
    ) -> Result<Option<PdDidMemberModel>, sea_orm::DbErr> {
        PdDidMemberEntity::find_pd_did_member(*did_id, *pd_member_id)
            .one(db)
            .await
    }

    pub async fn insert_did_pd_member(
        &self,
        db: &DatabaseConnection,
        pd_member_id: &Uuid,
        did_id: &Uuid,
        block_number: &i64,
    ) -> anyhow::Result<PdDidMemberModel> {
        let db_registry = PdDidMemberActiveModel {
            id: Set(Uuid::new_v4()),
            did_id: Set(*did_id),
            pd_member_id: Set(*pd_member_id),
            block_number: Set(*block_number),
        };
        match db_registry.insert(db).await {
            Ok(res) => return Ok(res),
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    pub async fn get_pd_did_member_by_id(
        &self,
        db: &DatabaseConnection,
        pd_did_member_id: &Uuid,
    ) -> Result<Option<PdDidMemberModel>, sea_orm::DbErr> {
        PdDidMemberEntity::find_by_pd_did_member_id(pd_did_member_id)
            .one(db)
            .await
    }

    pub async fn update_pd_did_member(
        &self,
        db: &DatabaseConnection,
        pd_did_member_id: Uuid,
        block_number: &i64,
    ) -> anyhow::Result<PdDidMemberModel> {
        match self.get_pd_did_member_by_id(db, &pd_did_member_id).await {
            Ok(v) => match v {
                Some(v) => {
                    let mut s: PdDidMemberActiveModel = v.into();
                    s.block_number = Set(*block_number);
                    match s.update(db).await {
                        Ok(res) => return Ok(res),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                None => panic!(
                    "Pd Did member with id {:?} does not exist",
                    pd_did_member_id
                ),
            },
            Err(e) => return Err(e.into()),
        }
    }
}
