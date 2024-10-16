use std::fs;
use crate::context::Context;
use crate::formatting::{zw, FG_RED, FG_YELLOW};

pub fn generate(context: &Context) -> String {
    match context.git_head() {
        None => "".to_string(),
        Some(head) => {
            if let Some(git_root) = context.git_root() {
                if head.starts_with("ref: ") {
                    let head = head.trim_start_matches("ref: ");
                    // Remove a leading "refs/heads/"
                    let head = head.trim_start_matches("refs/heads/");
                    // read <git_root>/.git/refs/heads/<head>
                    let local_sha =
                        fs::read_to_string(&git_root.join(".git/refs/heads/").join(head))
                            .unwrap_or_else(|_| "".to_string())
                            .trim_end()
                            .to_string();
                    let remote_sha =
                        fs::read_to_string(&git_root.join(".git/refs/remotes/origin/").join(head));
                    match remote_sha {
                        Ok(remote_sha) => {
                            let remote_sha = remote_sha.trim_end().to_string();
                            if local_sha == remote_sha {
                                "".to_string()
                            } else {
                                format!("{} ≠", zw(FG_RED))
                            }
                        }
                        Err(_) => format!("{} ≟", zw(FG_YELLOW)),
                    }
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        }
    }
}
