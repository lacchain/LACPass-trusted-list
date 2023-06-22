use sea_orm_migration::prelude::*;

use super::{m20230622_011005_did::Did, m20230622_035815_pd_member::PdMember};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PdDidMember::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PdDidMember::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PdDidMember::DidId).uuid().not_null())
                    .col(ColumnDef::new(PdDidMember::PdMemberId).uuid().not_null())
                    .col(
                        ColumnDef::new(PdDidMember::BlockNumber)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("did_id")
                            .from(PdDidMember::Table, PdDidMember::DidId)
                            .to(Did::Table, Did::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("pd_member_id")
                            .from(PdDidMember::Table, PdDidMember::PdMemberId)
                            .to(PdMember::Table, PdMember::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PdDidMember::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PdDidMember {
    Table,
    Id,
    DidId,
    PdMemberId,
    BlockNumber,
}
