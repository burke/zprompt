use fork::Fork;
use getopts::Options;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const ZERO_WIDTH_BEGIN: &str = "%{";
const ZERO_WIDTH_END: &str = "%}";
const SGR_RESET: &str = "\x1b[0m";
const FG_RED: &str = "\x1b[31m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_BLUE: &str = "\x1b[34m";
const FG_MAGENTA: &str = "\x1b[35m";
const FG_WHITE: &str = "\x1b[37m";

const BG_SHADOWENV: &str = "\x1b[48;5;238m";

fn zw(s: &str) -> String {
    format!("{}{}{}", ZERO_WIDTH_BEGIN, s, ZERO_WIDTH_END)
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = Options::new();
    opts.optflag("p", "", "print path info");
    opts.optflag("s", "", "print stash info");
    opts.optflag("a", "", "print async data");
    opts.optflag("r", "", "print ref info");
    opts.optflag("n", "", "print git pending");
    opts.optflag("y", "", "print git sync status");
    opts.optflag("e", "", "print exit status");
    opts.optflag("P", "", "print prompt char");
    opts.optflag("j", "", "print jobs");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => panic!("invalid options"),
    };

    let names = &["p", "s", "a", "r", "n", "y", "e", "P", "j"].map(|s| s.to_string());

    let git_root = git_root();

    if !matches.opts_present(names) {
        print_all(&git_root);
        return;
    }

    // print each option present
    for name in names {
        if matches.opt_present(name) {
            match name.as_ref() {
                "p" => print!("{}", gen_path()),
                "s" => print!("{}", gen_stash(&git_root)),
                "a" => print!("{}", supervise_job(&git_root)),
                "r" => print!("{}", gen_ref(&git_root)),
                "n" => print!("{}", gen_pending(&git_root)),
                "y" => print!("{}", gen_sync(&git_root)),
                "e" => print!("{}", gen_exit()),
                "P" => print!("{}", gen_prompt()),
                "j" => print!("{}", gen_jobs()),
                _ => panic!("invalid option"),
            }
        }
    }
}

fn supervise_job(_git_root: &Option<std::path::PathBuf>) -> String {
    let data = load_async_data();
    if data.is_none() {
        start_job(shell_pid().unwrap());
    }
    let data = load_async_data();
    let dc = match data {
        Some(data) => match data.content {
            Some(content) => content,
            None => tickdata(),
        },
        None => tickdata(),
    };
    zw(dc.as_ref())
}

fn start_job(shell_pid: i32) {
    let (r_fd, w_fd) = nix::unistd::pipe().unwrap();

    // nochdir=true,noclose=true => don't chdir, don't close stdin/stdout/stderr
    if let Ok(Fork::Child) = fork::daemon(true, true) {
        nix::unistd::close(r_fd).unwrap();
        // nochdir=true,noclose=false => don't chdir, DO close stdin/stdout/stderr
        if let Ok(Fork::Child) = fork::daemon(true, false) {
            nix::unistd::setsid().unwrap();
            write_initial_async_data(shell_pid);
            nix::unistd::close(w_fd).unwrap();
            let pid = Pid::from_raw(shell_pid);
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                nix::sys::signal::kill(pid, Signal::SIGALRM).unwrap();
            });
            run_job(shell_pid);
            nix::sys::signal::kill(pid, Signal::SIGALRM).unwrap();
            std::process::exit(0);
        }
        std::process::exit(0);
    }
    nix::unistd::close(w_fd).unwrap();
    nix::unistd::read(r_fd, &mut [0; 1]).unwrap();
}

fn run_job(shell_pid: i32) {
    // run git status --porcelain
    let mut cmd = std::process::Command::new("git");
    cmd.arg("status");
    cmd.arg("--porcelain");
    let output = cmd.output().unwrap();
    let color = if output.stdout.is_empty() {
        FG_GREEN
    } else {
        FG_MAGENTA
    };
    let data = AsyncData {
        pid: None,
        exec_no: exec_no().unwrap(),
        content: Some(color.to_string()),
    };
    write_async_data(data, shell_pid);
}

fn write_initial_async_data(shell_pid: i32) {
    let data = AsyncData {
        pid: Some(Pid::this().into()),
        exec_no: exec_no().unwrap(),
        content: None,
    };
    write_async_data(data, shell_pid);
}

fn write_async_data(data: AsyncData, shell_pid: i32) {
    let json = serde_json::to_string(&data).unwrap();
    let mut rng = rand::thread_rng();
    let rnd = rng.gen::<u32>();
    let path = json_path(shell_pid);
    let tmp_path = path.with_extension(format!("{}.tmp", rnd));

    // write json to tmp_path
    let mut file = File::create(&tmp_path).unwrap();
    file.write_all(json.as_bytes()).unwrap();
    drop(file);

    std::fs::rename(tmp_path, path).unwrap();
}

fn tickdata() -> String {
    // milliseconds since the epoch
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let tenths: u64 = (millis / 100) as u64;
    TICKS[(tenths % (TICKS.len() as u64)) as usize].to_string()
}

// TODO: something in 256 color for terminals without true color support.
const TICKS: [&str; 11] = [
    "\x1b[38;2;69;132;135m",
    "\x1b[38;2;58;144;127m",
    "\x1b[38;2;78;154;106m",
    "\x1b[38;2;115;159;76m",
    "\x1b[38;2;162;159;46m",
    "\x1b[38;2;214;152;33m",
    "\x1b[38;2;162;159;46m",
    "\x1b[38;2;115;159;76m",
    "\x1b[38;2;78;154;106m",
    "\x1b[38;2;58;144;127m",
    "\x1b[38;2;69;132;135m",
];

#[derive(Serialize, Deserialize)]
struct AsyncData {
    pid: Option<i32>,
    exec_no: u32,
    content: Option<String>,
}

fn shell_pid() -> Option<i32> {
    match std::env::var("SHELL_PID") {
        Ok(pid) => Some(pid.parse::<i32>().unwrap()),
        Err(_) => None,
    }
}

fn exec_no() -> Option<u32> {
    match std::env::var("PS1_EXEC_NO") {
        Ok(pid) => Some(pid.parse::<u32>().unwrap()),
        Err(_) => None,
    }
}

fn load_async_data() -> Option<AsyncData> {
    let shell_pid: Option<i32> = shell_pid();
    match shell_pid {
        Some(shell_pid) => {
            let exec_no: u32 = exec_no().unwrap();
            let json_path = json_path(shell_pid);
            let file = match File::open(&json_path) {
                Ok(file) => file,
                Err(_) => return None,
            };
            let reader = BufReader::new(file);
            let data: AsyncData = serde_json::from_reader(reader).unwrap();
            if data.exec_no == exec_no {
                return Some(data);
            }
            if let Some(pid) = data.pid {
                // If we use SIGKILL, it leaves .git/index.lock around.
                signal::kill(Pid::from_raw(-pid), Signal::SIGTERM).ok();
            }
            std::fs::remove_file(json_path).ok();
            None
        }
        None => None,
    }
}

fn json_path(shell_pid: i32) -> std::path::PathBuf {
    runtime_dir().join(format!("shell-prompt-{}.json", shell_pid))
}

fn runtime_dir() -> std::path::PathBuf {
    // $XDG_RUNTIME_DIR, or /tmp
    std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or("/tmp".to_string())
        .into()
}

fn print_all(git_root: &Option<std::path::PathBuf>) {
    let async_data = supervise_job(&git_root);

    let mut out = String::new();
    out.push_str(&gen_path());
    if !git_root.is_none() {
        out.push_str(" ");
    }
    out.push_str(&gen_stash(&git_root));
    out.push_str(async_data.as_str());
    out.push_str(&gen_ref(&git_root));
    out.push_str(&gen_pending(&git_root));
    out.push_str(&gen_sync(&git_root));
    out.push_str(" ");
    out.push_str(&gen_exit());
    out.push_str(&gen_prompt());
    out.push_str(&gen_jobs());
    out.push_str(" ");
    print!("{}", out);
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

fn gen_path() -> String {
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
        let world_path = std::path::Path::new(&home).join("world");

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

fn shadowenv_active() -> bool {
    // $__shadowenv_data is present and doesn't start with "0000"
    let shadowenv_data = std::env::var("__shadowenv_data").unwrap_or("".to_string());
    shadowenv_data.len() > 0 && !shadowenv_data.starts_with("0000")
}

// root directory of the git repo found by traversing up from the current working directory
fn git_root() -> Option<std::path::PathBuf> {
    let mut cwd = std::env::current_dir().unwrap();
    for _ in 0.. {
        if cwd.join(".git").exists() {
            return Some(cwd);
        }
        if cwd.parent().is_none() {
            return None;
        }
        cwd = cwd.parent().unwrap().to_path_buf();
    }
    None
}

const SUPERSCRIPT_CHARS: &'static str = "⁰¹²³⁴⁵⁶⁷⁸⁹";

fn gen_stash(git_root: &Option<std::path::PathBuf>) -> String {
    match git_root {
        Some(git_root) => {
            let stash_file = git_root.join(".git/logs/refs/stash");
            // number of lines in file (or zero if it doesn't exist)
            let num_lines = std::fs::read_to_string(&stash_file)
                .unwrap_or_else(|_| "".to_string())
                .lines()
                .count();
            // clamp num_lines to 9
            let num_lines = if num_lines > 9 { 9 } else { num_lines };
            let superchar = SUPERSCRIPT_CHARS.chars().nth(num_lines).unwrap();
            match num_lines {
                0 => "".to_string(),
                _ => format!("{}{}{}", zw(FG_WHITE), superchar, zw(SGR_RESET)),
            }
        }
        None => "".to_string(),
    }
}

fn gen_ref(git_root: &Option<std::path::PathBuf>) -> String {
    let head = git_head(&git_root);
    match head {
        Some(head) => {
            // if HEAD starts with "ref:", extract the ref name; otherwise, take the first 8 bytes
            if head.starts_with("ref: ") {
                // Remove a leading "ref:"
                let head = head.trim_start_matches("ref: ");
                // Remove a leading "refs/heads/"
                let head = head.trim_start_matches("refs/heads/");
                // if head is "master" or "main", use "➜"
                if head == "master" || head == "main" {
                    "𝒎".to_string()
                } else {
                    head.to_string()
                }
            } else {
                // if >= 8 bytes...
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

fn git_head(git_root: &Option<PathBuf>) -> Option<String> {
    git_root.as_ref().and_then(|root| {
        let git_path = root.join(".git");

        if git_path.is_file() {
            // This is a worktree, read the gitdir from the .git file
            fs::read_to_string(&git_path).ok().and_then(|content| {
                let gitdir = content.strip_prefix("gitdir: ")?.trim_end();
                let actual_git_dir = if Path::new(gitdir).is_absolute() {
                    PathBuf::from(gitdir)
                } else {
                    root.join(gitdir)
                };
                read_head_file(&actual_git_dir.join("HEAD"))
            })
        } else {
            // This is a standard repository
            read_head_file(&git_path.join("HEAD"))
        }
    })
}

fn read_head_file(head_file: &Path) -> Option<String> {
    fs::read_to_string(head_file)
        .ok()
        .map(|content| content.trim_end().to_string())
}

fn gen_pending(git_root: &Option<std::path::PathBuf>) -> String {
    match git_root {
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

fn gen_sync(git_root: &Option<std::path::PathBuf>) -> String {
    match git_head(&git_root) {
        None => "".to_string(),
        Some(head) => {
            let git_root = git_root.as_ref().unwrap();
            if head.starts_with("ref: ") {
                let head = head.trim_start_matches("ref: ");
                // Remove a leading "refs/heads/"
                let head = head.trim_start_matches("refs/heads/");
                // read <git_root>/.git/refs/heads/<head>
                let local_sha =
                    std::fs::read_to_string(&git_root.join(".git/refs/heads/").join(head))
                        .unwrap_or_else(|_| "".to_string())
                        .trim_end()
                        .to_string();
                let remote_sha =
                    std::fs::read_to_string(&git_root.join(".git/refs/remotes/origin/").join(head));
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
        }
    }
}

fn gen_exit() -> String {
    // if EXIT_STATUS is set and nonzero, return it; if it's zero, return blank; if it's not set,
    // return "?"
    let exit_status = std::env::var("EXIT_STATUS").unwrap_or("?".to_string());
    if exit_status.len() > 0 && exit_status != "0" {
        format!("{}{}{}", zw(FG_RED), exit_status, zw(SGR_RESET))
    } else {
        "".to_string()
    }
}

fn gen_prompt() -> String {
    format!("{}%#{}", zw(FG_YELLOW), zw(SGR_RESET))
}

fn gen_jobs() -> String {
    "%(1j.%j.)".to_string()
}
