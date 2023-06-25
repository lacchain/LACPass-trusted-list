use web3::{ethabi::Log, types::H160};

pub fn get_u64_from_log(log: &Log, param_name: &str) -> u64 {
    match log.params.iter().find(|&param| param.name == param_name) {
        Some(param) => match param.value.clone().into_uint() {
            Some(value) => u64::from(value.as_u64()), // 10 digit timestamp
            None => panic!("Error decoding param {}", param_name),
        },
        None => panic!("No value found for `{}`", param_name),
    }
}

pub fn get_string_from_string_in_log(log: &Log, param_name: &str) -> String {
    match log.params.iter().find(|&param| param.name == param_name) {
        Some(param) => match param.value.clone() {
            web3::ethabi::Token::String(v) => v,
            _ => panic!("Passed value is not a fixed bytes"),
        },
        None => panic!("No value found for param {}", param_name),
    }
}

pub fn get_address_from_log(log: &Log, param_name: &str) -> H160 {
    match log.params.iter().find(|&param| param.name == param_name) {
        Some(param) => match param.value.clone() {
            web3::ethabi::Token::Address(v) => v,
            _ => panic!("Error extracting address from log"),
        },
        None => panic!("No value found for param {}", param_name),
    }
}

pub fn get_bytes_from_log(log: &Log, param_name: &str) -> Vec<u8> {
    match log.params.iter().find(|&param| param.name == param_name) {
        Some(param) => match param.value.clone() {
            web3::ethabi::Token::Bytes(v) => v,
            _ => panic!("Passed value are not bytes"),
        },
        None => panic!("No value found for param {}", param_name),
    }
}

pub fn get_bool_from_log(log: &Log, param_name: &str) -> bool {
    match log.params.iter().find(|&param| param.name == param_name) {
        Some(param) => match param.value.clone() {
            web3::ethabi::Token::Bool(v) => v,
            _ => panic!("Passed value is not a bool"),
        },
        None => panic!("No value found for param {}", param_name),
    }
}
