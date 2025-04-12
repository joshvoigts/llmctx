use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();

  let mut max_tokens = 100_000_000; // Default to a large value
  let mut paths: Vec<PathBuf> = Vec::new();

  // Parse command-line arguments
  let mut i = 1;
  while i < args.len() {
    if args[i] == "--max-tokens" {
      if i + 1 >= args.len() {
        eprintln!("Error: --max-tokens requires a value");
        return Err("Invalid arguments".into());
      }
      max_tokens = args[i + 1].parse::<u64>().unwrap();
      i += 2;
    } else {
      // Add all other arguments as paths
      let path = Path::new(&args[i]).to_path_buf();
      paths.push(path);
      i += 1;
    }
  }

  let max_length = max_tokens * 5;

  // If no paths are specified, default to the current Git repository (if it's a Git repo)
  if paths.is_empty() {
    let current_dir = PathBuf::from(".");
    if is_git_repo(&current_dir) {
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
        let path = PathBuf::from(file);
        paths.push(path);
      }
    } else {
      eprintln!("Warning: Not a Git repository, defaulting to current directory");
      paths.push(PathBuf::from("."));
    }
  }

  let mut total_chars: u64 = 0;

  for path in paths {
    process_path(&path, max_length, &mut total_chars)?;
  }

  Ok(())
}

fn process_path(
  path: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
) -> Result<(), Box<dyn std::error::Error>> {
  if path.is_dir() {
    process_directory(path, max_length, total_chars)?;
  } else if path.is_file() {
    process_file(path, max_length, total_chars)?;
  } else {
    eprintln!(
      "Warning: {} is neither a file nor a directory, skipping.",
      path.display()
    );
  }

  Ok(())
}

fn process_directory(
  directory: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
) -> Result<(), Box<dyn std::error::Error>> {
  let entries = fs::read_dir(directory)?;

  for entry in entries {
    let entry = entry?;
    let path = entry.path();

    if path.is_dir() {
      // Skip subdirectories
      continue;
    }

    if path.starts_with(".git") || path.ends_with(".lock") {
      continue;
    }

    process_file(&path, max_length, total_chars)?;
  }

  Ok(())
}

fn process_file(
  file: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
) -> Result<(), Box<dyn std::error::Error>> {
  let content = fs::read_to_string(file)?;
  let trimmed_content = content.trim();
  let content_len = trimmed_content.len() as u64;

  let path_str = file.display().to_string();
  let file_output_len = (path_str.len() as u64) + content_len + 11;

  if *total_chars + file_output_len > max_length {
    return Ok(());
  }

  println!("\n{}", file.display());
  println!("```");
  println!("{}", trimmed_content);
  println!("```");

  *total_chars += file_output_len;
  Ok(())
}

fn is_git_repo(path: &PathBuf) -> bool {
  let output = Command::new("git")
    .args(&["rev-parse", "--is-inside-work-tree"])
    .current_dir(path)
    .output()
    .ok()
    .map(|out| out.status.success());

  output.unwrap_or(false)
}
