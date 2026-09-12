use clap::Parser;
use std::str::FromStr;

use crate::app::gen_name;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = false)]
    tls: bool,
    #[arg(default_value_t = String::from_str("").unwrap())]
    suffix: String,
}

pub fn init() -> (String, String, u16, bool, String) {
    dronoid_common::init_logger();
    (
        gen_name(),
        "127.0.0.1".to_string(),
        8080,
        false,
        String::from_str("").unwrap(),
    )
}
