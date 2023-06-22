use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Did::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Did::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Did::UpperBlock).big_integer().not_null())
                    .col(
                        ColumnDef::new(Did::LastProcessedBlock)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Did::LastBlockSaved).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Did::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub(crate) enum Did {
    Table,
    Id,
    UpperBlock,
    LastProcessedBlock,
    LastBlockSaved,
}
