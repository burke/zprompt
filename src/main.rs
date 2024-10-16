use fork::Fork;
use getopts::Options;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// Near the top of the file, with the other `use` statements
mod widgets;
use widgets::{stash, path, prompt, ref_info, pending, exit, jobs, sync};

// Add this near the top with other use statements
mod formatting;
use formatting::{zw, FG_GREEN, FG_MAGENTA};

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
                "p" => print!("{}", path::generate()),
                "s" => print!("{}", stash::generate(&context)),
                "a" => print!("{}", supervise_job()),
                "r" => print!("{}", ref_info::generate(&context)),
                "n" => print!("{}", pending::generate(&context)),
                "y" => print!("{}", sync::generate(&context)),
                "e" => print!("{}", exit::generate()),
                "P" => print!("{}", prompt::generate()),
                "j" => print!("{}", jobs::generate()),
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
    out.push_str(&path::generate());
    if context.git_root().is_some() {
        out.push_str(" ");
    }
    out.push_str(&stash::generate(context));
    out.push_str(async_data.as_str());
    out.push_str(&ref_info::generate(context));
    out.push_str(&pending::generate(context));
    out.push_str(&sync::generate(context));
    out.push_str(" ");
    out.push_str(&exit::generate());
    out.push_str(&prompt::generate());
    out.push_str(&jobs::generate());
    out.push_str(" ");
    print!("{}", out);
}
