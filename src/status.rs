extern crate termion;
mod file_status;

use std::collections::LinkedList;
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
struct Cursor {
  row: usize,
  column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileRow {
  line_index: usize,
  file_index: file_status::FileIndex,
}

// internal state of the rgt status screen
#[derive(Debug, Clone, PartialEq, Eq)]
struct RGTStatus {
  path: String,
  cursor: Cursor,
  branch_name: String,
  staged_file_indexes: Vec<file_status::FileIndex>,
  modified_file_indexes: Vec<file_status::FileIndex>,
  file_list: LinkedList<FileRow>,
  file_row: FileRow,
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
      file_list: LinkedList::new(),
      file_row: FileRow {
        line_index: 0,
        file_index: file_status::FileIndex {
          status: "".to_string(),
          name: "".to_string(),
        },
      },
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
    self.file_list = LinkedList::new();
    let mut line_index = 2;
    for file_index in &self.staged_file_indexes {
      let file_row = FileRow {
        line_index: line_index,
        file_index: file_index.clone(),
      };
      if self.file_list.is_empty() {
        self.file_row = file_row.clone();
      }
      self.file_list.push_back(file_row);
      line_index += 1;
    }
    line_index += 1;
    for file_index in &self.modified_file_indexes {
      let file_row = FileRow {
        line_index: line_index,
        file_index: file_index.clone(),
      };
      if self.file_list.is_empty() {
        self.file_row = file_row.clone();
      }
      self.file_list.push_back(file_row);
      line_index += 1;
    }
    self.max_line_index = line_index;
  }

  fn reopen(&mut self) {
    println!("{:?}", self);
    self.open(self.path.clone());
    println!("{:?}", self);
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
    write!(
      out,
      "{}Changes not staged for commit:\r\n{}",
      color::Fg(color::Blue),
      color::Fg(color::Reset)
    )
    .unwrap();
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
    file_status::stage_file(self.find_file_row().file_index.clone().name);
  }
  fn unstage_file(&mut self) {
    file_status::unstage_file(self.find_file_row().file_index.clone().name);
  }
  fn edit_file(&mut self) {
    let editor = var("EDITOR").unwrap();
    let file_name = self.find_file_row().file_index.clone().name;
    Command::new(editor)
      .arg(&file_name)
      .status()
      .expect("Could not open file by editor");
  }

  fn find_file_row(&mut self) -> &FileRow {
    return self
      .file_list
      .iter()
      .find(|&file_row| file_row.line_index == self.cursor.row)
      .unwrap();
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
      Event::Key(Key::Char('a')) => {
        state.stage_file();
        state.reopen();
      }
      Event::Key(Key::Char('u')) => {
        state.unstage_file();
        state.reopen();
      }
      Event::Key(Key::Char('e')) => {
        state.edit_file();
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
