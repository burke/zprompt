use std::env;
use crate::formatting::{zw, FG_RED, SGR_RESET};

pub fn generate() -> String {
    // if EXIT_STATUS is set and nonzero, return it; if it's zero, return blank; if it's not set,
    // return "?"
    let exit_status = env::var("EXIT_STATUS").unwrap_or_else(|_| "?".to_string());
    if !exit_status.is_empty() && exit_status != "0" {
        format!("{}{}{}", zw(FG_RED), exit_status, zw(SGR_RESET))
    } else {
        "".to_string()
    }
}
