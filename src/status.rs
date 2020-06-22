extern crate termion;
mod file_status;

use std::env::var;
use std::io::{stdin, stdout, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::{Child, Command, Stdio};
use termion::cursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{clear, color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cursor {
  row: usize,
  column: usize,
}

// internal state of the rgt status screen
#[derive(Debug, Clone, PartialEq, Eq)]
struct RGTStatus {
  path: String,
  cursor: Cursor,
  branch_name: String,
  staged_file_indexes: Vec<file_status::FileIndex>,
  modified_file_indexes: Vec<file_status::FileIndex>,
  file_list: Vec<file_status::FileIndex>,
  max_line_index: usize,
}

impl Default for RGTStatus {
  fn default() -> Self {
    Self {
      path: "".to_string(),
      cursor: Cursor { row: 0, column: 0 },
      branch_name: "".to_string(),
      staged_file_indexes: Vec::new(),
      modified_file_indexes: Vec::new(),
      file_list: Vec::new(),
      max_line_index: 0,
    }
  }
}

impl RGTStatus {
  fn open(&mut self, path: String) {
    self.path = path.clone();
    self.branch_name = file_status::branch_name(path);
    self.staged_file_indexes = file_status::staged_file_indexes();
    self.modified_file_indexes = file_status::modified_file_indexes();
    self.file_list = Vec::new();
    self.forward_file_list(2);
    if self.staged_file_indexes.is_empty() {
      self.forward_file_list(1);
    } else {
      self.push_file_indexes(self.staged_file_indexes.to_vec());
    }
    self.forward_file_list(1);
    if self.modified_file_indexes.is_empty() {
      self.forward_file_list(1);
    } else {
      self.push_file_indexes(self.modified_file_indexes.to_vec());
    }
  }

  fn push_file_indexes(&mut self, file_indexes: Vec<file_status::FileIndex>) {
    for file_index in file_indexes {
      self.push_file_index(file_index.clone());
    }
  }

  fn push_file_index(&mut self, file_index: file_status::FileIndex) {
    self.file_list.push(file_index.clone());
    self.max_line_index += 1;
  }

  fn forward_file_list(&mut self, n: usize) {
    for _ in 1..=n {
      self.file_list.push(file_status::default_file_index());
    }
    self.max_line_index += n;
  }

  fn reopen(&mut self) {
    self.open(self.path.clone());
  }

  fn draw<T: Write>(&self, out: &mut T) {
    write!(out, "{}", clear::All).unwrap();
    write!(out, "{}", cursor::Goto(1, 1)).unwrap();

    write!(
      out,
      "{}On branch {}\r\n{}",
      color::Fg(color::Green),
      self.branch_name,
      color::Fg(color::Reset)
    )
    .unwrap();
    write!(
      out,
      "{}Changes to be commited:\r\n{}",
      color::Fg(color::Blue),
      color::Fg(color::Reset)
    )
    .unwrap();

    if self.staged_file_indexes.is_empty() {
      write!(out, "  (no files)\r\n").unwrap();
    } else {
      for file_index in &self.staged_file_indexes {
        write!(
          out,
          "{}{}{} {}\r\n",
          color::Fg(color::Magenta),
          file_index.status,
          color::Fg(color::Reset),
          file_index.name
        )
        .unwrap();
      }
    }
    write!(
      out,
      "{}Changes not staged for commit:\r\n{}",
      color::Fg(color::Blue),
      color::Fg(color::Reset)
    )
    .unwrap();
    if self.modified_file_indexes.is_empty() {
      write!(out, "  (no files)\r\n").unwrap();
    } else {
      for file_index in &self.modified_file_indexes {
        write!(
          out,
          "{}{}{} {}\r\n",
          color::Fg(color::Magenta),
          file_index.status,
          color::Fg(color::Reset),
          file_index.name
        )
        .unwrap();
      }
    }

    write!(
      out,
      "{}",
      cursor::Goto(self.cursor.column as u16 + 1, self.cursor.row as u16 + 1)
    )
    .unwrap();
    out.flush().unwrap();
  }
  fn cursor_up(&mut self) {
    if self.cursor.row > 0 {
      self.cursor.row -= 1;
    }
  }
  fn cursor_down(&mut self) {
    if self.cursor.row + 1 < self.max_line_index {
      self.cursor.row += 1;
    }
  }
  fn stage_file(&mut self) {
    match self.find_file_name() {
      Some(file_name) => file_status::stage_file(file_name),
      None => return,
    }
  }
  fn unstage_file(&mut self) {
    match self.find_file_name() {
      Some(file_name) => file_status::unstage_file(file_name),
      None => return,
    }
  }
  fn stage_or_unstage_file(&mut self) {
    match self.find_file_index() {
      Some(file_index) => {
        if file_index.staged {
          self.unstage_file()
        } else {
          self.stage_file()
        }
      }
      None => return,
    }
  }
  fn edit_file(&mut self) {
    match self.find_file_name() {
      Some(file_name) => {
        let editor = var("EDITOR").unwrap();
        Command::new(editor)
          .arg(&file_name)
          .status()
          .expect("Could not open file by editor");
      }
      None => return,
    }
  }
  fn stdout_to_stdin(&mut self, process: &Child) -> Option<Stdio> {
    if let Some(ref stdout) = process.stdout {
      return Some(unsafe { Stdio::from_raw_fd(stdout.as_raw_fd()) });
    }
    None
  }
  fn diff_file(&mut self) {
    if let None = self.find_file_name() {
      return;
    }
    let file_name = self.find_file_name().unwrap();
    let mut git_diff_command = Command::new("git")
      .args(&["diff", &file_name])
      .stdout(Stdio::piped())
      .spawn()
      .expect("Could not execute gif diff command");
    let mut delta_command = Command::new("delta")
      .stdin(
        self
          .stdout_to_stdin(&git_diff_command)
          .expect("broken pipe"),
      )
      .spawn()
      .expect("Could not execute delta command");
    git_diff_command.wait().unwrap();
    delta_command.wait().unwrap();
  }
  fn find_file_index(&mut self) -> Option<&file_status::FileIndex> {
    return self.file_list.get(self.cursor.row);
  }
  fn find_file_name(&mut self) -> Option<String> {
    match self.find_file_index() {
      Some(file_index) => {
        let file_name = file_index.clone().name;
        if file_name == "" {
          None
        } else {
          Some(file_name)
        }
      }
      None => None,
    }
  }
}

pub fn main(path: String) {
  let mut state = RGTStatus::default();
  state.open(path);

  let stdin = stdin();
  let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

  state.draw(&mut stdout);

  for evt in stdin.events() {
    match evt.unwrap() {
      Event::Key(Key::Char('u')) => {
        state.stage_or_unstage_file();
        state.reopen();
      }
      Event::Key(Key::Char('e')) => {
        state.edit_file();
      }
      Event::Key(Key::Char('\n')) => {
        state.diff_file();
      }
      Event::Key(Key::Char('q')) => {
        return;
      }
      Event::Key(Key::Char('k')) => state.cursor_up(),
      Event::Key(Key::Char('j')) => state.cursor_down(),
      _ => {}
    }
    state.draw(&mut stdout)
  }

  stdout.flush().unwrap();
}
