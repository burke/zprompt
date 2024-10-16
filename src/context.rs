use std::path::PathBuf;
use std::cell::RefCell;

pub struct Context {
    git_root: RefCell<Option<Option<PathBuf>>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            git_root: RefCell::new(None),
        }
    }

    pub fn git_root(&self) -> Option<PathBuf> {
        if self.git_root.borrow().is_none() {
            let root = Self::find_git_root();
            *self.git_root.borrow_mut() = Some(root);
        }
        self.git_root.borrow().as_ref().unwrap().clone()
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
}
