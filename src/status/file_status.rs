use git2::Repository;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIndex {
  pub status: String,
  pub name: String,
}

pub fn branch_name(path: String) -> String {
  let ref_name = ref_name(path);
  let pattern = "refs/heads/";
  if ref_name.starts_with(pattern) {
    return ref_name.trim_start_matches(pattern).to_string();
  } else {
    return ref_name;
  }
}

pub fn staged_file_indexes() -> Vec<FileIndex> {
  let output = staged_files_command_output();
  return file_indexes_for_output(output);
}

pub fn modified_file_indexes() -> Vec<FileIndex> {
  let output = modified_files_command_output();
  return file_indexes_for_output(output);
}

pub fn stage_file(path: String) {
  stage_file_command_output(path);
}

pub fn unstage_file(path: String) {
  unstage_file_command_output(path);
}

fn stage_file_command_output(path: String) -> std::process::Output {
  let output = Command::new("git")
    .args(&["add", &path])
    .output()
    .expect("failed to stage the file");
  return output;
}

fn unstage_file_command_output(path: String) -> std::process::Output {
  let output = Command::new("git")
    .args(&["reset", "HEAD", &path])
    .output()
    .expect("failed to unstage the file");
  return output;
}

fn ref_name(path: String) -> String {
  let repo = match Repository::open(path) {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };
  let head = match repo.head() {
    Ok(head) => head,
    Err(e) => panic!("failed to open: {}", e),
  };
  let ref_name = head.name().unwrap();
  return ref_name.to_string();
}

fn staged_files_command_output() -> std::process::Output {
  let output = Command::new("git")
    .args(&["diff", "--cached", "--name-status"])
    .output()
    .expect("failed to execute the command: git diff --cached --name-status");
  return output;
}

fn modified_files_command_output() -> std::process::Output {
  let output = Command::new("git")
    .args(&["diff", "--name-status"])
    .output()
    .expect("failed to execute the command: git diff --cached --name-status");
  return output;
}

fn file_indexes_for_output(output: std::process::Output) -> Vec<FileIndex> {
  let result = String::from(std::str::from_utf8(&(output.stdout)).unwrap());
  let mut file_results: Vec<&str> = result.split("\n").collect();
  file_results.pop();
  let mut file_indexes: Vec<FileIndex> = Vec::new();
  for file_result in file_results.iter() {
    let status_and_file: Vec<&str> = file_result.split("\t").collect();
    let status = status_and_file.first().unwrap().to_string();
    let file_name = status_and_file.last().unwrap().to_string();
    let file_index = FileIndex {
      status: status,
      name: file_name,
    };
    file_indexes.push(file_index)
  }
  return file_indexes;
}
