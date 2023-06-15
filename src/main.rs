mod config;
pub mod controllers;
pub mod dto;
mod logger_config;
pub mod responses;
pub mod services;
pub mod utils;

#[macro_use]
extern crate rocket;

use controllers::index::stage;

use crate::config::get_envs;
use crate::logger_config::setup_logger;

#[launch]
async fn rocket() -> _ {
    let envs = get_envs().await.unwrap();
    setup_logger(true, envs.value_of("log-conf"));
    rocket::build().attach(stage())
}
