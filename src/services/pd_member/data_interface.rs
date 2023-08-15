use crate::entities::entities::PdMemberEntity;
use crate::entities::models::{PdMemberActiveModel, PdMemberModel};
use crate::services::public_directory::index::PublicDirectoryService;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

pub struct PdMemberDataInterfaceService {
    pub public_directory_service: PublicDirectoryService,
}

impl PdMemberDataInterfaceService {
    pub fn new(public_directory_service: PublicDirectoryService) -> PdMemberDataInterfaceService {
        PdMemberDataInterfaceService {
            public_directory_service,
        }
    }
    pub async fn get_pd_member_from_database(
        &self,
        db: &DatabaseConnection,
        member_id: &i64,
    ) -> Result<Option<PdMemberModel>, sea_orm::DbErr> {
        PdMemberEntity::find_pd_member(
            member_id,
            &self
                .public_directory_service
                .params
                .contract_address
                .to_string(),
            &self.public_directory_service.params.chain_id,
        )
        .one(db)
        .await
    }

    pub async fn get_pd_member_by_id(
        &self,
        db: &DatabaseConnection,
        pd_member_id: &Uuid,
    ) -> Result<Option<PdMemberModel>, sea_orm::DbErr> {
        PdMemberEntity::find_by_pd_member_id(pd_member_id)
            .one(db)
            .await
    }

    /// insert public directory member to database
    pub async fn insert_pd_member(
        &self,
        db: &DatabaseConnection,
        member_id: &i64,
        exp: &i64,
        block_number: &i64,
        country_code: String,
    ) -> anyhow::Result<PdMemberModel> {
        match self
            .public_directory_service
            .data_interface
            .get_public_directory_from_database(db)
            .await
        {
            Ok(wrapped) => match wrapped {
                Some(pd) => {
                    let db_registry = PdMemberActiveModel {
                        id: Set(Uuid::new_v4()),
                        exp: Set(*exp),
                        member_id: Set(*member_id),
                        public_directory_id: Set(pd.id),
                        block_number: Set(*block_number),
                        country_code: Set(country_code),
                    };
                    match db_registry.insert(db).await {
                        Ok(res) => return Ok(res),
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                None => {
                    panic!("Public directory with contract address {} and chainId {} not found in database", self.public_directory_service.params.contract_address, self.public_directory_service.params.chain_id);
                }
            },
            Err(e) => return Err(e.into()),
        }
    }

    /// insert or update public directory member to database
    pub async fn update_pd_member(
        &self,
        db: &DatabaseConnection,
        pd_member_id: Uuid,
        exp: &i64,
        block_number: &i64,
    ) -> anyhow::Result<PdMemberModel> {
        match self.get_pd_member_by_id(db, &pd_member_id).await {
            Ok(v) => match v {
                Some(v) => {
                    let mut s: PdMemberActiveModel = v.into();
                    s.exp = Set(*exp);
                    s.block_number = Set(*block_number);
                    match s.update(db).await {
                        Ok(res) => return Ok(res),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                None => panic!("Pd member with id {:?} does not exist", pd_member_id),
            },
            Err(e) => return Err(e.into()),
        }
    }

    pub async fn find_one_by_did(
        db: &DatabaseConnection,
        did_id: Uuid,
    ) -> Result<Option<PdMemberModel>, sea_orm::DbErr> {
        PdMemberEntity::find_by_did(did_id).one(db).await
    }
}
