use sea_orm_migration::prelude::*;

use super::m20230617_195505_public_directory::PublicDirectory;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PdMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PdMember::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(PdMember::MemberId)
                            .big_integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(PdMember::Exp).big_integer().not_null())
                    .col(
                        ColumnDef::new(PdMember::BlockNumber)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PdMember::PublicDirectoryId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("public_directory_id")
                            .from(PdMember::Table, PdMember::PublicDirectoryId)
                            .to(PublicDirectory::Table, PublicDirectory::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PdMember::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub(crate) enum PdMember {
    Table,
    Id,
    MemberId,
    Exp,
    PublicDirectoryId,
    BlockNumber,
}
