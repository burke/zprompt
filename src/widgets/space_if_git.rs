use crate::context::Context;

pub fn generate(context: &Context) -> String {
    if context.git_root().is_some() {
        " ".to_string()
    } else {
        "".to_string()
    }
}
