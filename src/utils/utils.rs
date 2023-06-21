use log::error;
use std::{env, num::ParseIntError};

pub struct Utils {}

impl Utils {
    pub fn get_env(env_name: &'static str) -> String {
        match env::var(env_name) {
            Ok(s) => s,
            Err(e) => {
                error!("Environment Variable with name {} not found", env_name);
                panic!("{}", e)
            }
        }
    }

    pub fn get_env_or_err(env_name: &str) -> Result<String, &'static str> {
        match env::var(env_name) {
            Ok(s) => Ok(s),
            Err(_) => Err("not found"),
        }
    }

    pub fn i32_from_string(s: String) -> i32 {
        let i: i32 = s.parse().unwrap();
        match i {
            i => i,
        }
    }
    pub fn integer_part(value: &str) -> Result<u64, ParseIntError> {
        let dot_pos = value.find(".").unwrap_or(value.len());
        value[..dot_pos].parse()
    }
}
