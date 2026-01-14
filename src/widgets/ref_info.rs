use crate::context::Context;

pub fn generate(context: &Context) -> String {
    match context.git_head() {
        Some(head) => {
            if head == "HEAD" {
                "???".to_string()
            } else if head == "master" || head == "main" {
                "𝒎".to_string()
            } else {
                head
            }
        }
        None => "".to_string(),
    }
}
