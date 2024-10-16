use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::fs;

pub struct Context {
    git_root: RefCell<Option<Option<PathBuf>>>,
    git_head: RefCell<Option<Option<String>>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            git_root: RefCell::new(None),
            git_head: RefCell::new(None),
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
            let git_path = root.join(".git");

            if git_path.is_file() {
                fs::read_to_string(&git_path).ok().and_then(|content| {
                    let gitdir = content.strip_prefix("gitdir: ")?.trim_end();
                    let actual_git_dir = if Path::new(gitdir).is_absolute() {
                        PathBuf::from(gitdir)
                    } else {
                        root.join(gitdir)
                    };
                    Self::read_head_file(&actual_git_dir.join("HEAD"))
                })
            } else {
                Self::read_head_file(&git_path.join("HEAD"))
            }
        })
    }

    fn read_head_file(head_file: &Path) -> Option<String> {
        fs::read_to_string(head_file)
            .ok()
            .map(|content| content.trim_end().to_string())
    }
}
