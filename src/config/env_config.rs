use figment::providers::Serialized;
use figment::value::{Dict, Map};
use figment::{Error, Figment, Metadata, Profile, Provider};
use log::info;
use rocket::data::Limits;
use rocket::data::ToByteUnit;
use serde::{Deserialize, Serialize};
use std::env;
use yansi::Paint;

use crate::utils::constants;
use crate::utils::utils::Utils;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
enum Profiles {
    DEV,
    PROD,
    DEFAULT,
}

#[derive(Deserialize, Serialize)]
pub struct D {
    pub url: String,
}
#[derive(Deserialize, Serialize)]
pub struct Databases {
    pub dbconnection: D,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    port: i32,
    profile: Profiles,
    address: String,
    pub databases: Databases,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            port: Config::get_port(constants::PORT.to_owned()),
            profile: Profiles::DEFAULT,
            address: "0.0.0.0".to_string(),
            databases: Databases {
                dbconnection: D {
                    url: Config::get_database_url(constants::URL_POSTGRES_CONNECTION_NAME),
                },
            },
        }
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("LACPass-trusted-list")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}

impl Config {
    fn development() -> Config {
        Config {
            port: Config::get_port(constants::DEV_PORT.to_owned()),
            profile: Profiles::DEV,
            address: "0.0.0.0".to_string(),
            databases: Databases {
                dbconnection: D {
                    url: Config::get_database_url(constants::DEV_URL_POSTGRES_CONNECTION_NAME),
                },
            },
        }
    }

    fn production() -> Config {
        Config {
            port: Config::get_port(constants::PROD_PORT.to_owned()),
            profile: Profiles::PROD,
            address: "0.0.0.0".to_string(),
            databases: Databases {
                dbconnection: D {
                    url: Config::get_database_url(constants::PROD_URL_POSTGRES_CONNECTION_NAME),
                },
            },
        }
    }

    pub fn get_config() -> Config {
        match Config::get_profile_env_var() {
            Profiles::DEV => Config::development(),
            Profiles::PROD => Config::production(),
            _ => Config::default(),
        }
    }

    pub fn figment() -> Figment {
        let config = Config::get_config();

        Config::print_profile(&config);
        Figment::from(rocket::Config::default())
            .merge(Serialized::defaults(config))
            .merge((
                "limits",
                Limits::new()
                    .limit("json", 5.mebibytes())
                    .limit("data-form", 5.mebibytes()), // set data for data forms
            ))
    }

    fn get_profile_env_var() -> Profiles {
        let profile_variable = env::var("PROFILE");
        match profile_variable {
            Ok(s) => match &*s {
                "DEV" => Profiles::DEV,
                "PROD" => Profiles::PROD,
                _ => Profiles::DEFAULT,
            },
            Err(_) => Profiles::DEFAULT,
        }
    }

    fn print_profile(&self) {
        info!("Using profile {:?}", Paint::blue(self.profile).bold());
    }

    fn get_port(name: String) -> i32 {
        match Utils::get_env_or_err(&name) {
            Ok(s) => Utils::i32_from_string(s),
            Err(_) => 3025,
        }
    }
    fn get_database_url(name: &'static str) -> String {
        Utils::get_env(name)
    }
    pub fn get_provider(chain_id: String) -> String {
        let mut key = "RPC_CONNECTION_".to_owned();
        key.push_str(&chain_id);
        match Utils::get_env_or_err(&key) {
            Ok(s) => s, // TODO: Validate is a valid http connection string
            Err(e) => panic!("{}", e),
        }
    }
}
