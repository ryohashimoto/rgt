use git2::Repository;

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
  println!("{}", &onbranch(name));
}

fn onbranch(branch_name: String) -> String {
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
