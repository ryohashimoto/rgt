extern crate termion;
mod file_status;

use std::io::{stdin, stdout, Write};
use termion::cursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{clear, color};

pub fn main(path: String) {
  let stdin = stdin();
  let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
  write!(stdout, "{}", clear::All).unwrap();
  write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();

  let name = file_status::branch_name(path);
  write!(stdout, "{}On branch {}", color::Fg(color::Green), name).unwrap();
  write!(stdout, "\r\n{}", color::Fg(color::Reset)).unwrap();
  write!(stdout, "{}Changes to be commited:", color::Fg(color::Blue)).unwrap();
  write!(stdout, "\r\n{}", color::Fg(color::Reset)).unwrap();
  for file_index in file_status::staged_file_indexes() {
    write!(
      stdout,
      "{}{}{} ",
      color::Fg(color::Magenta),
      file_index.status,
      color::Fg(color::Reset)
    )
    .unwrap();
    write!(stdout, "{}", file_index.name).unwrap();
    write!(stdout, "\r\n").unwrap();
  }
  write!(
    stdout,
    "{}Changes not staged for commit:",
    color::Fg(color::Blue)
  )
  .unwrap();
  write!(stdout, "\r\n{}", color::Fg(color::Reset)).unwrap();
  for file_index in file_status::modified_file_indexes() {
    write!(
      stdout,
      "{}{}{} ",
      color::Fg(color::Magenta),
      file_index.status,
      color::Fg(color::Reset)
    )
    .unwrap();
    write!(stdout, "{}", file_index.name).unwrap();
    write!(stdout, "\r\n").unwrap();
  }
  stdout.flush().unwrap();
  for evt in stdin.events() {
    if evt.unwrap() == Event::Key(Key::Ctrl('c')) {
      return;
    }
  }
}
