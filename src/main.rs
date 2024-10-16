use fork::Fork;
use getopts::Options;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// Near the top of the file, with the other `use` statements
mod widgets;
use widgets::{stash::gen_stash, path::gen_path};

// Add this near the top with other use statements
mod formatting;
use formatting::{zw, FG_GREEN, FG_MAGENTA, FG_RED, FG_YELLOW, SGR_RESET};

// Add this function after the imports and before the main function

// Add this near the top with other use statements
mod context;
use context::Context;

pub fn main() {
    let context = Context::new();

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

    if !matches.opts_present(names) {
        print_all(&context);
        return;
    }

    // print each option present
    for name in names {
        if matches.opt_present(name) {
            match name.as_ref() {
                "p" => print!("{}", gen_path()),
                "s" => print!("{}", gen_stash(&context)),
                "a" => print!("{}", supervise_job()),
                "r" => print!("{}", gen_ref(&context)),
                "n" => print!("{}", gen_pending(&context)),
                "y" => print!("{}", gen_sync(&context)),
                "e" => print!("{}", gen_exit()),
                "P" => print!("{}", gen_prompt()),
                "j" => print!("{}", gen_jobs()),
                _ => panic!("invalid option"),
            }
        }
    }
}

fn supervise_job() -> String {
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

fn print_all(context: &Context) {
    let async_data = supervise_job();
    let mut out = String::new();
    out.push_str(&gen_path());
    if context.git_root().is_some() {
        out.push_str(" ");
    }
    out.push_str(&gen_stash(context));
    out.push_str(async_data.as_str());
    out.push_str(&gen_ref(context));
    out.push_str(&gen_pending(context));
    out.push_str(&gen_sync(context));
    out.push_str(" ");
    out.push_str(&gen_exit());
    out.push_str(&gen_prompt());
    out.push_str(&gen_jobs());
    out.push_str(" ");
    print!("{}", out);
}

fn gen_ref(context: &Context) -> String {
    let head = git_head(&context);
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

fn git_head(context: &Context) -> Option<String> {
    context.git_root().as_ref().and_then(|root| {
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

fn gen_pending(context: &Context) -> String {
    match context.git_root() {
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

fn gen_sync(context: &Context) -> String {
    match git_head(&context) {
        None => "".to_string(),
        Some(head) => {
            if let Some(git_root) = context.git_root() {
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
