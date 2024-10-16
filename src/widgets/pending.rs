use crate::context::Context;
use crate::formatting::{zw, FG_RED};

pub fn generate(context: &Context) -> String {
    match context.git_root() {
        Some(git_root) => {
            let mut pending = Vec::new();
            if git_root.join(".git/CHERRY_PICK_HEAD").exists() {
                pending.push("ᴾ");
            }
            if git_root.join(".git/MERGE_HEAD").exists() {
                pending.push("ᴹ");
            }
            if git_root.join(".git/BISECT_LOG").exists() {
                pending.push("ᴮ");
            }
            if git_root.join(".git/rebase-apply").exists() {
                pending.push("ᴿ");
            }
            if git_root.join(".git/rebase-merge").exists() {
                pending.push("ʳ");
            }
            match pending.len() {
                0 => "".to_string(),
                _ => format!("{}{}", zw(FG_RED), pending.join("")),
            }
        }
        None => "".to_string(),
    }
}
