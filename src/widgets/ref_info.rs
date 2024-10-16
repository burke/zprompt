use crate::context::Context;

pub fn generate(context: &Context) -> String {
    match context.git_head() {
        Some(head) => {
            if head.starts_with("ref: ") {
                let head = head.trim_start_matches("ref: ").trim_start_matches("refs/heads/");
                if head == "master" || head == "main" {
                    "𝒎".to_string()
                } else {
                    head.to_string()
                }
            } else {
                if head.len() >= 8 {
                    head[0..8].to_string()
                } else {
                    "???".to_string()
                }
            }
        }
        None => "".to_string(),
    }
}
