mod config;
pub mod controllers;
pub mod databases;
pub mod dto;
pub mod entities;
pub mod jobs;
mod logger_config;
pub mod migration;
pub mod responses;
pub mod services;
pub mod utils;

#[macro_use]
extern crate rocket;

use controllers::index::stage;
use jobs::index::JobManager;

use crate::config::log_config::get_envs;
use crate::logger_config::setup_logger;

#[launch]
async fn rocket() -> _ {
    let envs = get_envs().await.unwrap();
    setup_logger(true, envs.value_of("log-conf"));
    tokio::spawn(async move {
        JobManager::sweep_trusted_registries();
    });
    rocket::build().attach(stage())
}
