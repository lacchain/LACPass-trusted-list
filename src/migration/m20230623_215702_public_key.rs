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
                    .col(ColumnDef::new(PublicKey::CountryCode).string().not_null())
                    .col(ColumnDef::new(PublicKey::ContentHash).string().not_null()) // TODO: Add fhir url
                    .col(ColumnDef::new(PublicKey::DidId).uuid().null())
                    .col(ColumnDef::new(PublicKey::Jwk).binary().not_null())
                    .col(ColumnDef::new(PublicKey::Exp).big_integer().not_null())
                    .col(
                        ColumnDef::new(PublicKey::IsCompromised)
                            .boolean()
                            .default(false),
                    )
                    .col(ColumnDef::new(PublicKey::BlockNumber).big_integer().null())
                    .index(
                        Index::create()
                            .name("content_hash_did_id")
                            .col(PublicKey::ContentHash)
                            .col(PublicKey::DidId)
                            .unique(),
                    )
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

#[derive(Iden)]
enum PublicKey {
    Table,
    CountryCode,
    Id,
    DidId,
    BlockNumber,
    Jwk,
    ContentHash,
    Exp,
    IsCompromised,
}
