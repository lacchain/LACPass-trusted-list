use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PublicDirectory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PublicDirectory::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PublicDirectory::ContractAddress)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PublicDirectory::UpperBlock)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PublicDirectory::LastProcessedBlock)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PublicDirectory::LastBlockSaved)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PublicDirectory::ChainId).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PublicDirectory::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PublicDirectory {
    Table,
    Id,
    ContractAddress,
    UpperBlock,
    LastProcessedBlock,
    LastBlockSaved,
    ChainId,
}
