use std::process::Command;

pub fn generate() -> String {
    Command::new("worldpath")
        .arg("-z")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default()
}
