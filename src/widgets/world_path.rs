use std::env;
use std::path::PathBuf;
use std::convert::TryFrom;
use regex::Regex;
use crate::formatting::{zw, FG_MAGENTA, FG_GREEN, FG_BLUE, FG_BOLD_BLUE, BOLD_OFF, SGR_RESET};

#[derive(Debug)]
pub struct WorldInfo {
    pub worldlet: Option<String>,
    pub tree: String,
    pub substrate_path: Option<String>,
    pub project_path: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PrintMode {
    Full,
    Ellipsis,
    Compact,
}

impl WorldInfo {
    pub fn pretty_print(&self, mode: PrintMode) -> String {
        let mut output = String::new();

        // Worldlet name in magenta
        if let Some(worldlet) = &self.worldlet {
            output.push_str(&format!("{}{}", zw(FG_MAGENTA), worldlet));
        }

        // Tree name in green if not "root"
        if self.tree != "root" {
            if output.is_empty() {
                output.push_str(&format!("{}+{}", zw(FG_GREEN), self.tree));
            } else {
                output.push_str(&format!("{}{}+{}", zw(SGR_RESET), zw(FG_GREEN), self.tree));
            }
        }

        // Always show '//' in bold blue
        if output.is_empty() {
            output.push_str(&format!("{}//", zw(FG_BOLD_BLUE)));
        } else {
            output.push_str(&format!("{}{}//", zw(SGR_RESET), zw(FG_BOLD_BLUE)));
        }

        // Substrate path in bold blue
        if let Some(substrate) = &self.substrate_path {
            let formatted_substrate = format_path(substrate, mode);
            output.push_str(&formatted_substrate);
        }
        output.push_str(&zw(BOLD_OFF));

        // Project path in (non-bold) blue
        if let Some(project) = &self.project_path {
            if self.substrate_path.is_some() {
                output.push('/');
            }
            let formatted_project = format_path(project, mode);
            output.push_str(&format!("{}{}", zw(FG_BLUE), formatted_project));
        }

        // Final reset
        output.push_str(&zw(SGR_RESET));

        output
    }
}

fn format_path(path: &str, mode: PrintMode) -> String {
    let had_leading_slash = path.starts_with('/');
    let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let formatted = match mode {
        PrintMode::Full => components.join("/"),
        PrintMode::Ellipsis => {
            if components.len() > 1 {
                format!("…/{}", components.last().unwrap())
            } else {
                components.join("/")
            }
        },
        PrintMode::Compact => {
            if components.len() > 1 {
                let truncated: String = components[..components.len() - 1]
                    .iter()
                    .map(|c| c.chars().next().unwrap().to_string())
                    .collect::<Vec<String>>()
                    .join("/");
                format!("{}/{}", truncated, components.last().unwrap())
            } else {
                components.join("/")
            }
        },
    };
    
    // Add a leading slash only if the original path had one
    if had_leading_slash {
        format!("/{}", formatted)
    } else {
        formatted
    }
}

impl TryFrom<PathBuf> for WorldInfo {
    type Error = String;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let home = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
        let path_str = path.to_str().ok_or("Invalid path")?;
        let relative_path = path_str.strip_prefix(&home).ok_or("Path is not in HOME directory")?;
        
        let re = Regex::new(r"^/(?:world|worldlets/([^/]+))/trees/([^/]+)/src(?:/(.+))?$").unwrap();
        
        if let Some(captures) = re.captures(relative_path) {
            let worldlet = captures.get(1).map(|m| m.as_str().to_string());
            let tree = captures.get(2).unwrap().as_str().to_string();
            let substrate_path = captures.get(3).map(|m| m.as_str().to_string());
            
            let (substrate_path, project_path) = if let Some(substrate_path) = substrate_path {
                let project_re = Regex::new(r"^((?:areas|libraries)/[^/]+/[^/]+)(?:/(.*))?$").unwrap();
                if let Some(project_captures) = project_re.captures(&substrate_path) {
                    (
                        Some(project_captures.get(1).unwrap().as_str().to_string()),
                        project_captures.get(2).map(|m| m.as_str().to_string()),
                    )
                } else {
                    (Some(substrate_path), None)
                }
            } else {
                (None, None)
            };

            Ok(WorldInfo {
                worldlet,
                tree,
                substrate_path,
                project_path,
            })
        } else {
            Err("Not a valid world path".to_string())
        }
    }
}

pub fn generate() -> String {
    let pwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = env::var("HOME").unwrap_or_else(|_| String::new());

    let relative_path = if !home.is_empty() {
        pwd.strip_prefix(&home)
            .map(|p| format!("~/{}", p.display()))
            .unwrap_or_else(|_| pwd.display().to_string())
    } else {
        pwd.display().to_string()
    };

    match WorldInfo::try_from(pwd.clone()) {
        Ok(world_info) => world_info.pretty_print(PrintMode::Compact),
        Err(_) => {
            // Not in a world path, format the regular path
            let formatted_path = format_path(&relative_path, PrintMode::Compact);
            format!("{}{}{}",
                zw(FG_BLUE),
                formatted_path,
                zw(SGR_RESET)
            )
        }
    }
}
