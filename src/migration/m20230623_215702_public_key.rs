use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PublicKey::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PublicKey::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PublicKey::DidId).uuid().not_null())
                    .col(
                        ColumnDef::new(PublicKey::BlockNumber)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PublicKey::PemKey).binary().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PublicKey::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum PublicKey {
    Table,
    Id,
    DidId,
    BlockNumber,
    PemKey,
}
