use rocket::{fairing, Build, Rocket};
pub use sea_orm_migration::prelude::*;
use sea_orm_rocket::Database;

use crate::{databases::pool::Db, migration::m20230617_195505_public_directory};
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20230617_195505_public_directory::Migration)]
    }
}

pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = Migrator::up(conn, None).await;
    Ok(rocket)
}
