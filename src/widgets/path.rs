use std::env;
use std::path::Path;
use std::process::Command;

use crate::formatting::{zw, FG_BLUE, FG_GREEN, BG_SHADOWENV, SGR_RESET};

pub fn gen_path() -> String {
    if env::var("USE_PWWD").is_ok() {
        return gen_path2();
    } else {
        // if SSH_CONNECTION is set, green; otherwise blue
        let fg_color = if std::env::var("SSH_CONNECTION").is_ok() {
            FG_GREEN
        } else {
            FG_BLUE
        };
        // if shadowenv_active, grey, otherwise blank
        let color = if shadowenv_active() {
            format!("{}{}", fg_color, BG_SHADOWENV)
        } else {
            fg_color.to_string()
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
            zw(color.as_ref()),
            basename.to_string(),
            zw(SGR_RESET)
        )
    }
}

fn gen_path2() -> String {
    let mut cmd = Command::new("/Users/burke/world/trees/root/src/.meta/substrate/bin/pwwd");
    cmd.arg("-cz");
    let output = cmd.output().unwrap();

    if output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        panic!("wups")
    }
}

fn shadowenv_active() -> bool {
    // $__shadowenv_data is present and doesn't start with "0000"
    let shadowenv_data = std::env::var("__shadowenv_data").unwrap_or_default();
    !shadowenv_data.is_empty() && !shadowenv_data.starts_with("0000")
}
