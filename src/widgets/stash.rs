use crate::context::Context;
use crate::formatting::{zw, FG_WHITE, SGR_RESET, SUPERSCRIPT_CHARS};

pub fn gen_stash(context: &Context) -> String {
    match context.git_root() {
        Some(git_root) => {
            let stash_file = git_root.join(".git/logs/refs/stash");
            let num_lines = std::fs::read_to_string(&stash_file)
                .unwrap_or_else(|_| "".to_string())
                .lines()
                .count();
            let num_lines = num_lines.min(9);
            let superchar = SUPERSCRIPT_CHARS.chars().nth(num_lines).unwrap();
            match num_lines {
                0 => "".to_string(),
                _ => format!("{}{}{}", zw(FG_WHITE), superchar, zw(SGR_RESET)),
            }
        }
        None => "".to_string(),
    }
}
