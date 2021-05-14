extern crate termion;

use std::process::Command;
use termion::clear;

struct RGTLog {
    path: String,
}

impl Default for RGTLog {
    fn default() -> Self {
        Self {
            path: "".to_string(),
        }
    }
}

impl RGTLog {
    fn open(&mut self, path: String) {
        self.path = path.clone();
        self.git_log_command();
    }
    fn git_log_command(&mut self) {
        Command::new("git")
            .arg(&"log")
            .status()
            .expect("Could not execute git log command");
    }
}

pub fn main(path: String) {
    let mut state = RGTLog::default();
    println!("{}", clear::All);
    state.open(path);
}
