use web3::ethabi::Log;

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
