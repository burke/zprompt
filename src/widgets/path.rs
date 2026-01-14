use std::path::Path;
use crate::formatting::{zw, FG_BLUE, FG_GREEN, SGR_RESET};

pub fn generate() -> String {
    // if SSH_CONNECTION is set, green; otherwise blue
    let fg_color = if std::env::var("SSH_CONNECTION").is_ok() {
        FG_GREEN
    } else {
        FG_BLUE
    };
    let cwd = std::env::current_dir().unwrap();
    let home = std::env::var("HOME").unwrap();
    let world_path = Path::new(&home).join("world");

    let basename = cwd.file_name().unwrap().to_str().unwrap();
    let world_prefix = if cwd.starts_with(&world_path) {
        format!("{}⊕", zw(FG_GREEN)).to_string()
    } else {
        "".to_string()
    };

    format!(
        "{}{}{}{}",
        world_prefix,
        zw(fg_color),
        basename.to_string(),
        zw(SGR_RESET)
    )
}
