use std::path::PathBuf;
use std::cell::RefCell;
use std::process::Command;

pub struct Context {
    git_root: RefCell<Option<Option<PathBuf>>>,
    git_head: RefCell<Option<Option<String>>>,
    pub no_worldpath: bool,
}

impl Context {
    pub fn new(no_worldpath: bool) -> Self {
        Context {
            git_root: RefCell::new(None),
            git_head: RefCell::new(None),
            no_worldpath,
        }
    }

    pub fn git_root(&self) -> Option<PathBuf> {
        if self.git_root.borrow().is_none() {
            let root = Self::find_git_root();
            *self.git_root.borrow_mut() = Some(root);
        }
        self.git_root.borrow().as_ref().unwrap().clone()
    }

    pub fn git_head(&self) -> Option<String> {
        if self.git_head.borrow().is_none() {
            let head = self.find_git_head();
            *self.git_head.borrow_mut() = Some(head);
        }
        self.git_head.borrow().as_ref().unwrap().clone()
    }

    fn find_git_root() -> Option<PathBuf> {
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

    fn find_git_head(&self) -> Option<String> {
        self.git_root().as_ref().and_then(|root| {
            let output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(root)
                .output()
                .ok()?;

            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
    }
}
