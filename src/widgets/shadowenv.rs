use std::env;
use crate::formatting::{zw, FG_GRAY, SGR_RESET};

pub fn generate() -> String {
    if shadowenv_active() {
        format!("{}░{}", zw(FG_GRAY), zw(SGR_RESET))
    } else {
        " ".to_string()
    }
}

fn shadowenv_active() -> bool {
    match env::var("__shadowenv_data") {
        Ok(data) => !data.is_empty() && !data.starts_with("0000"),
        Err(_) => false,
    }
}
