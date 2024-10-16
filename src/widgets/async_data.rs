use fork::Fork;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::formatting::{zw, FG_GREEN, FG_MAGENTA};

pub fn generate() -> String {
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
