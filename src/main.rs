use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();

  let mut max_tokens = 100_000_000; // Default to a large value
  let mut repo_arg = None;

  // Parse command-line arguments
  let mut i = 1;
  while i < args.len() {
    if args[i] == "--max-tokens" {
      if i + 1 >= args.len() {
        eprintln!("Error: --max-tokens requires a value");
        return Err("Invalid arguments".into());
      }
      max_tokens = args[i + 1].parse::<u64>().unwrap();
      i += 2; // Move past both "--max-tokens" and its value
    } else if repo_arg.is_none() && i < args.len() {
      repo_arg = Some(Path::new(&args[i]));
      i += 1; // Move to the next argument
    } else {
      i += 1; // Move to the next argument if it's neither an option nor the repo path
    }
  }

  let max_length = max_tokens * 5;

  let repo_path = repo_arg.unwrap_or(Path::new("."));

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

  let mut total_chars: u64 = 0;

  for file in files {
    // Skip files that start with '.' or end with .lock
    if file.starts_with(".") || file.ends_with(".lock") {
      continue;
    }

    let path = Path::new(file);
    let path_str = path.display().to_string();
    let content =
      fs::read_to_string(path).expect("Failed to read file");
    let trimmed_content = content.trim();
    let content_len = trimmed_content.len() as u64;

    // Calculate the total output length this file would contribute
    let file_output_len =
      (path_str.len() as u64) + content_len + 11 as u64;

    if total_chars + file_output_len > max_length {
      break;
    }

    println!("\n{}", path.display());
    println!("```");
    println!("{}", trimmed_content);
    println!("```");

    total_chars += file_output_len;
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
