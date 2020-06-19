use git2::Repository;
use std::process::Command;

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
  let file_results: Vec<&str> = result.split("\n").collect();
  let mut file_indexes: Vec<FileIndex> = Vec::new();
  for file_result in file_results.iter() {
    let status_and_file: Vec<&str> = file_result.split("\t").collect();
    let status = status_and_file.first().unwrap();
    let file_name = status_and_file.last().unwrap();
    let file_index = FileIndex {
      status: status.to_string(),
      name: file_name.to_string(),
    };
    file_indexes.push(file_index)
  }
  return file_indexes;
}
