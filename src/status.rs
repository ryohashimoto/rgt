extern crate termion;
mod file_status;

use std::io::{stdin, stdout, Write};
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
struct RGTStatus {
  cursor: Cursor,
  lines: usize,
  branch_name: String,
  staged_file_indexes: Vec<file_status::FileIndex>,
  modified_file_indexes: Vec<file_status::FileIndex>,
}

impl Default for RGTStatus {
  fn default() -> Self {
    Self {
      cursor: Cursor { row: 0, column: 0 },
      lines: 10, // TODO: Get from actual lines
      branch_name: "".to_string(),
      staged_file_indexes: Vec::new(),
      modified_file_indexes: Vec::new(),
    }
  }
}

impl RGTStatus {
  fn open(&mut self, path: String) {
    self.branch_name = file_status::branch_name(path);
    self.staged_file_indexes = file_status::staged_file_indexes();
    self.modified_file_indexes = file_status::modified_file_indexes();
  }

  fn draw<T: Write>(&self, out: &mut T) {
    write!(out, "{}", clear::All).unwrap();
    write!(out, "{}", cursor::Goto(1, 1)).unwrap();

    write!(
      out,
      "{}On branch {}",
      color::Fg(color::Green),
      self.branch_name
    )
    .unwrap();
    write!(out, "\r\n{}", color::Fg(color::Reset)).unwrap();
    write!(out, "{}Changes to be commited:", color::Fg(color::Blue)).unwrap();
    write!(out, "\r\n{}", color::Fg(color::Reset)).unwrap();

    for file_index in &self.staged_file_indexes {
      write!(
        out,
        "{}{}{} ",
        color::Fg(color::Magenta),
        file_index.status,
        color::Fg(color::Reset)
      )
      .unwrap();
      write!(out, "{}", file_index.name).unwrap();
      write!(out, "\r\n").unwrap();
    }
    write!(
      out,
      "{}Changes not staged for commit:",
      color::Fg(color::Blue)
    )
    .unwrap();
    write!(out, "\r\n{}", color::Fg(color::Reset)).unwrap();
    for file_index in &self.modified_file_indexes {
      write!(
        out,
        "{}{}{} ",
        color::Fg(color::Magenta),
        file_index.status,
        color::Fg(color::Reset)
      )
      .unwrap();
      write!(out, "{}", file_index.name).unwrap();
      write!(out, "\r\n").unwrap();
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
    if self.cursor.row + 1 < self.lines {
      self.cursor.row += 1;
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
      Event::Key(Key::Ctrl('c')) => {
        return;
      }
      Event::Key(Key::Up) => state.cursor_up(),
      Event::Key(Key::Down) => state.cursor_down(),
      _ => {}
    }
    state.draw(&mut stdout)
  }

  stdout.flush().unwrap();
}
