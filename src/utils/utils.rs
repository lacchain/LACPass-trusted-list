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

    pub fn vec_u8_to_hex_string(value: Vec<u8>) -> Option<String> {
        let value = value
            .into_iter()
            .map(|el| format!("{:02x?}", el))
            .collect::<Vec<_>>()
            .concat();
        Some(format!("0x{}", value))
    }

    pub fn vec_u8_to_u64(value: Vec<u8>) -> Option<u64> {
        match (0..value.len())
            .into_iter()
            .map(|i| 16_u64.pow((i as u64).try_into().unwrap()) * value[i] as u64)
            .reduce(|acc, e| acc + e)
        {
            Some(i) => Some(i),
            None => None,
        }
    }

    pub fn trim_0x_from_hex_string(value: &str) -> String {
        if value.starts_with("0x") {
            value[2..].to_string()
        } else {
            value.to_owned()
        }
    }
}
