use std::process::Command;
use crate::context::Context;
use crate::formatting::{zw, FG_BLUE, FG_GREEN, SGR_RESET};

pub fn generate(context: &Context) -> String {
    if context.no_worldpath {
        generate_truncated_path()
    } else {
        Command::new("worldpath")
            .arg("-z")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
            .unwrap_or_else(|_| generate_truncated_path())
    }
}

fn generate_truncated_path() -> String {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return String::from("?"),
    };
    let home = std::env::var("HOME").unwrap_or_default();

    // Green for SSH, blue otherwise
    let fg_color = if std::env::var("SSH_CONNECTION").is_ok() {
        FG_GREEN
    } else {
        FG_BLUE
    };

    let path_str = cwd.to_string_lossy();
    let display_path = if !home.is_empty() && path_str.starts_with(&home) {
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str.into_owned()
    };

    let components: Vec<&str> = display_path.split('/').filter(|s| !s.is_empty()).collect();

    if components.is_empty() {
        return format!("{}/{}", zw(fg_color), zw(SGR_RESET));
    }

    let truncated: Vec<String> = components.iter().enumerate().map(|(i, comp)| {
        if i == components.len() - 1 {
            // Last component: show in full
            comp.to_string()
        } else {
            // Intermediate: show first char (handle unicode)
            comp.chars().next().map(|c| c.to_string()).unwrap_or_default()
        }
    }).collect();

    let result = if display_path.starts_with('~') {
        format!("~/{}", truncated[1..].join("/"))
    } else {
        format!("/{}", truncated.join("/"))
    };
    format!("{}{}{}", zw(fg_color), result, zw(SGR_RESET))
}
