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

use clap::__macro_refs::once_cell::sync::OnceCell;
use controllers::index::stage;
use jobs::index::JobManager;
use jobs::trusted_registries::TrustedRegistries;
use log::info;
use services::trusted_registry::trusted_registry::TrustedRegistry;

use crate::config::log_config::get_envs;
use crate::logger_config::setup_logger;

static CONTROLLER_TRUSTED_REGISTRY: OnceCell<TrustedRegistry> = OnceCell::new();

#[launch]
async fn rocket() -> _ {
    let envs = get_envs().await.unwrap();
    setup_logger(true, envs.value_of("log-conf"));
    match CONTROLLER_TRUSTED_REGISTRY.set(TrustedRegistries::get_trusted_registry_by_index()) {
        Ok(_) => {}
        Err(err) => {
            info!(
                "Error while setting trusted registry for controllers {:?}",
                err
            );
        }
    }
    tokio::spawn(async move {
        JobManager::sweep_trusted_registries();
    });
    rocket::build().attach(stage())
}
