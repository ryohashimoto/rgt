extern crate termion;
mod file_status;

use std::env::var;
use std::io::{stdin, stdout, Write};
use std::process::Command;
use termion::cursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{clear, color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
  pub width: usize,
  pub height: usize,
}

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
  terminal_size: Size,
  branch_name: String,
  staged_file_indexes: Vec<file_status::FileIndex>,
  modified_file_indexes: Vec<file_status::FileIndex>,
  untracked_file_indexes: Vec<file_status::FileIndex>,
  file_list: Vec<file_status::FileIndex>,
  max_line_index: usize,
  status_message: String,
}

impl Default for RGTStatus {
  fn default() -> Self {
    Self {
      path: "".to_string(),
      cursor: Cursor { row: 0, column: 0 },
      terminal_size: Size {
        width: 0,
        height: 0,
      },
      branch_name: "".to_string(),
      staged_file_indexes: Vec::new(),
      modified_file_indexes: Vec::new(),
      untracked_file_indexes: Vec::new(),
      file_list: Vec::new(),
      max_line_index: 0,
      status_message: "[status] Nothing to update".to_string(),
    }
  }
}

impl RGTStatus {
  fn open(&mut self, path: String) {
    self.path = path.clone();
    self.branch_name = file_status::branch_name(path);
    self.staged_file_indexes = file_status::staged_file_indexes();
    self.modified_file_indexes = file_status::modified_file_indexes();
    self.untracked_file_indexes = file_status::untracked_file_indexes();
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
    self.forward_file_list(1);
    if self.untracked_file_indexes.is_empty() {
      self.forward_file_list(1);
    } else {
      self.push_file_indexes(self.untracked_file_indexes.to_vec());
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
      "{}Untracked files:\r\n{}",
      color::Fg(color::Blue),
      color::Fg(color::Reset)
    )
    .unwrap();
    if self.untracked_file_indexes.is_empty() {
      write!(out, "  (no files)\r\n").unwrap();
    } else {
      for file_index in &self.untracked_file_indexes {
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

    for _ in self.max_line_index..self.terminal_size.height - 2 {
      write!(out, "\r\n").unwrap();
    }
    let mut status_message = self.status_message.clone();
    let pad_size = self.terminal_size.width - status_message.len();
    if pad_size > 0 {
      status_message.push_str(&" ".repeat(pad_size));
    }
    write!(
      out,
      "{}{}{}\r\n",
      color::Bg(color::Blue),
      status_message,
      color::Bg(color::Reset)
    )
    .unwrap();

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
    self.update_status_message();
  }
  fn cursor_down(&mut self) {
    if self.cursor.row + 1 < self.max_line_index {
      self.cursor.row += 1;
    }
    self.update_status_message();
  }
  fn commit_files(&mut self) {
    Command::new("git")
      .arg(&"commit")
      .status()
      .expect("Could not commit files");
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
  fn diff_file(&mut self) {
    if let None = self.find_file_index() {
      return;
    }
    let file_index = self.find_file_index().unwrap();
    let file_name = &file_index.clone().name;
    let mut args = vec!["diff"];
    if file_index.staged {
      args.push("--cached");
    }
    args.push(&file_name);
    Command::new("git")
      .args(&args)
      .status()
      .expect("Could not execute gif diff command");
  }
  fn pager_file(&mut self) {
    if let None = self.find_file_index() {
      return;
    }
    let file_index = self.find_file_index().unwrap();
    let file_name = &file_index.clone().name;
    let pager = var("PAGER").unwrap();
    if pager.len() == 0 {
      return;
    }
    Command::new(pager)
      .arg(&file_name)
      .status()
      .expect("Could not open file by pager");
  }
  fn diff_or_pager_file(&mut self) {
    if let None = self.find_file_index() {
      return;
    }
    let file_index = self.find_file_index().unwrap();
    if file_index.untracked {
      self.pager_file();
    } else {
      self.diff_file();
    }
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
  fn update_status_message(&mut self) {
    match self.find_file_index() {
      Some(file_index) => {
        if file_index.untracked && file_index.name.len() > 0 {
          self.status_message = format!(
            "[status] Press u to stage '{}' for addition",
            file_index.name
          );
        } else if file_index.staged {
          self.status_message = format!(
            "[status] Press u to unstage '{}' for commit",
            file_index.name
          );
        } else if file_index.name.len() > 0 {
          self.status_message =
            format!("[status] Press u to stage '{}' for commit", file_index.name);
        } else {
          self.status_message = "[status] Nothing to update".to_string();
        }
      }
      None => self.status_message = "[status] Nothing to update".to_string(),
    }
  }
}

pub fn main(path: String) {
  let mut state = RGTStatus::default();
  state.open(path);
  let terminal_size = termion::terminal_size().unwrap();
  state.terminal_size.width = terminal_size.0 as usize;
  state.terminal_size.height = terminal_size.1 as usize;

  let stdin = stdin();
  let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

  state.draw(&mut stdout);

  for evt in stdin.events() {
    match evt.unwrap() {
      Event::Key(Key::Char('c')) => {
        state.commit_files();
      }
      Event::Key(Key::Char('u')) => {
        state.stage_or_unstage_file();
        state.reopen();
      }
      Event::Key(Key::Char('e')) => {
        state.edit_file();
      }
      Event::Key(Key::Char('\n')) => {
        state.diff_or_pager_file();
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
