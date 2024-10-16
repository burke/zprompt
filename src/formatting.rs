pub const ZERO_WIDTH_BEGIN: &str = "%{";
pub const ZERO_WIDTH_END: &str = "%}";
pub const SGR_RESET: &str = "\x1b[0m";
pub const FG_RED: &str = "\x1b[31m";
pub const FG_GREEN: &str = "\x1b[32m";
pub const FG_YELLOW: &str = "\x1b[33m";
pub const FG_BLUE: &str = "\x1b[34m";
pub const FG_MAGENTA: &str = "\x1b[35m";
pub const FG_WHITE: &str = "\x1b[37m";

pub const BG_SHADOWENV: &str = "\x1b[48;5;238m";

pub const SUPERSCRIPT_CHARS: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹";

pub fn zw(s: &str) -> String {
    format!("{}{}{}", ZERO_WIDTH_BEGIN, s, ZERO_WIDTH_END)
}
