use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();

  let repo_path = if args.len() > 1 {
    Path::new(&args[1])
  } else {
    Path::new(".")
  };

  if !is_git_repo(repo_path) {
    eprintln!(
      "Error: {} is not a Git repository",
      repo_path.display()
    );
    return Err("Invalid repository".into());
  }

  let path = repo_path.canonicalize()?;
  env::set_current_dir(&path)?;

  let output = Command::new("git")
    .args(&["ls-tree", "-r", "--name-only", "HEAD"])
    .output()
    .expect("Failed to run git command");

  let file_list = String::from_utf8(output.stdout)
    .expect("Invalid UTF-8 output from git");

  let files = file_list
    .lines()
    .filter(|line| !line.starts_with(".git"))
    .collect::<Vec<_>>();

  for file in files {
    let path = Path::new(file);
    let content =
      fs::read_to_string(path).expect("Failed to read file");
    let trimmed_content = content.trim();

    println!("\n{}", path.display());
    println!("```");
    println!("{}", trimmed_content);
    println!("```");
  }

  Ok(())
}

fn is_git_repo(path: &Path) -> bool {
  let output = Command::new("git")
    .args(&["rev-parse", "--is-inside-work-tree"])
    .current_dir(path)
    .output()
    .ok()
    .map(|out| out.status.success());

  output.unwrap_or(false)
}
