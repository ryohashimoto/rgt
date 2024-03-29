use git2::Repository;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIndex {
    pub status: String,
    pub name: String,
    pub staged: bool,
    pub untracked: bool,
}

pub fn default_file_index() -> FileIndex {
    return FileIndex {
        status: "".to_string(),
        name: "".to_string(),
        staged: false,
        untracked: true,
    };
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
    return file_indexes_for_output(output, true, false);
}

pub fn modified_file_indexes() -> Vec<FileIndex> {
    let output = modified_files_command_output();
    return file_indexes_for_output(output, false, false);
}

pub fn untracked_file_indexes() -> Vec<FileIndex> {
    let output = untracked_files_command_output();
    return file_indexes_for_output(output, false, true);
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

fn untracked_files_command_output() -> std::process::Output {
    let output = Command::new("git")
        .args(&["ls-files", "--others", "--exclude-standard"])
        .output()
        .expect("failed to execute the command: git ls-files --others --exclude-standard");
    return output;
}

fn file_indexes_for_output(
    output: std::process::Output,
    staged: bool,
    untracked: bool,
) -> Vec<FileIndex> {
    let result = String::from(std::str::from_utf8(&(output.stdout)).unwrap());
    let mut file_results: Vec<&str> = result.split("\n").collect();
    file_results.pop();
    let mut file_indexes: Vec<FileIndex> = Vec::new();
    for file_result in file_results.iter() {
        let status: String;
        let file_name: String;
        if untracked {
            status = "?".to_string();
            file_name = file_result.to_string();
        } else {
            let status_and_file: Vec<&str> = file_result.split("\t").collect();
            status = status_and_file.first().unwrap().to_string();
            file_name = status_and_file.last().unwrap().to_string();
        }
        let file_index = FileIndex {
            status: status,
            name: file_name,
            staged: staged,
            untracked: untracked,
        };
        file_indexes.push(file_index)
    }
    return file_indexes;
}
