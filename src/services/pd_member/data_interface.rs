use crate::entities::entities::PdMemberEntity;
use crate::entities::models::{PdMemberActiveModel, PdMemberModel};
use crate::services::public_directory::index::PublicDirectoryService;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

pub struct DataInterfaceService {
    public_directory: PublicDirectoryService,
}

impl DataInterfaceService {
    pub fn new(public_directory: PublicDirectoryService) -> DataInterfaceService {
        DataInterfaceService { public_directory }
    }
    pub async fn get_pd_member_from_database(
        &self,
        db: &DatabaseConnection,
        member_id: &i64,
    ) -> Result<Option<PdMemberModel>, sea_orm::DbErr> {
        PdMemberEntity::find_pd_member(
            member_id,
            &self.public_directory.params.contract_address.to_string(),
            &self.public_directory.params.chain_id,
        )
        .one(db)
        .await
    }

    /// insert or update public directory member to database
    pub async fn save_pd_member_to_database(
        &self,
        db: &DatabaseConnection,
        member_id: &i64,
        exp: &i64,
        block_number: &i64,
    ) -> anyhow::Result<()> {
        match self.get_pd_member_from_database(db, member_id).await {
            Ok(u) => match u {
                Some(v) => {
                    let mut s: PdMemberActiveModel = v.into();
                    s.exp = Set(*exp);
                    s.block_number = Set(*block_number);
                    match s.update(db).await {
                        Ok(_) => return Ok(()),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                None => {
                    match self
                        .public_directory
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
                                    pubic_directory_id: Set(pd.id),
                                    block_number: Set(*block_number),
                                };
                                match db_registry.insert(db).await {
                                    Ok(_) => return Ok(()),
                                    Err(e) => {
                                        return Err(e.into());
                                    }
                                }
                            }
                            None => {
                                panic!("Public directory with contract address {} and chainId {} not found in database", self.public_directory.params.contract_address, self.public_directory.params.chain_id);
                            }
                        },
                        Err(e) => return Err(e.into()),
                    }
                }
            },
            Err(_) => {}
        }
        Ok(())
    }
}
