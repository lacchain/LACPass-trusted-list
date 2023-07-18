use rocket::{fairing, Build, Rocket};
pub use sea_orm_migration::prelude::*;
use sea_orm_rocket::Database;

use crate::{
    databases::pool::Db,
    migration::{
        m20230617_195505_public_directory, m20230622_011005_did, m20230622_035815_pd_member,
        m20230622_044839_pd_did_member, m20230623_215702_public_key,
    },
};
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230617_195505_public_directory::Migration),
            Box::new(m20230622_011005_did::Migration),
            Box::new(m20230622_035815_pd_member::Migration),
            Box::new(m20230622_044839_pd_did_member::Migration),
            Box::new(m20230623_215702_public_key::Migration),
        ]
    }
}

pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = Migrator::up(conn, None).await;
    Ok(rocket)
}
