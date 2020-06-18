use git2::Repository;
use std::process::Command;

struct FileIndex {
  status: String,
  name: String,
}

pub fn main(path: String) {
  let repo = match Repository::open(path) {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };
  let head = match repo.head() {
    Ok(head) => head,
    Err(e) => panic!("failed to open: {}", e),
  };
  let ref_name = head.name().unwrap();
  let name = branch_name(ref_name.to_string());
  println!("{}", &on_branch(name));
  let files: Vec<String> = modified_file_indexes()
    .into_iter()
    .map(|file| format!("{} {}", &file.status, &file.name))
    .rev()
    .collect();
  for file in files {
    println!("{}", &file);
  }
}

fn on_branch(branch_name: String) -> String {
  let message = format!("On branch {}", branch_name);
  return message.to_string();
}

fn branch_name(ref_name: String) -> String {
  let pattern = "refs/heads/";
  if ref_name.starts_with(pattern) {
    return ref_name.trim_start_matches(pattern).to_string();
  } else {
    return ref_name;
  }
}

fn modified_file_indexes() -> Vec<FileIndex> {
  let command = Command::new("git")
    .args(&["diff", "--name-status"])
    .output()
    .expect("failed to execute the command");
  let result = String::from(std::str::from_utf8(&(command.stdout)).unwrap());
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
