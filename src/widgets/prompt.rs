use crate::formatting::{zw, FG_YELLOW, SGR_RESET};

pub fn generate() -> String {
    format!("{}%#{}", zw(FG_YELLOW), zw(SGR_RESET))
}
