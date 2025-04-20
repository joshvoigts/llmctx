use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

fn main() -> Result<()> {
  let args: Vec<String> = env::args().collect();

  let mut max_tokens = 100_000_000; // Default to a large value
  let mut paths: Vec<PathBuf> = Vec::new();

  // Parse command-line arguments
  let mut i = 1;
  while i < args.len() {
    if args[i] == "--max-tokens" {
      if i + 1 >= args.len() {
        eprintln!("Error: --max-tokens requires a value");
        return Err(anyhow::anyhow!("Invalid arguments"));
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

  // If no paths are specified, default to the current Git repository
  // (if it's a Git repo).
  if paths.is_empty() {
    match get_git_root_path() {
      Ok(path) => {
        paths.push(PathBuf::from(path));
      }
      Err(_) => {
        paths.push(PathBuf::from("."));
      }
    }
  }

  let max_length = max_tokens * 5;
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
) -> Result<()> {
  if path.is_dir() {
    process_directory(path, max_length, total_chars)?;
  } else if !skip_path(path) {
    process_file(path, max_length, total_chars)?;
  } else {
    eprintln!(
      "Warning: {} is skipped due to naming conventions.",
      path.display()
    );
  }

  Ok(())
}

fn process_directory(
  directory: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
) -> Result<()> {
  let entries = fs::read_dir(directory)?;

  for entry in entries {
    let entry = entry?;
    let path = entry.path();

    if skip_path(&path) {
      continue;
    }

    process_path(&path, max_length, total_chars)?;
  }

  Ok(())
}

fn process_file(
  file: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
) -> Result<()> {
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

fn skip_path(path: &PathBuf) -> bool {
  let name = path
    .file_name()
    .map(|name| name.to_str().unwrap_or(""))
    .unwrap_or("");
  name.starts_with(".") || name.ends_with(".lock")
}

fn get_git_root_path() -> Result<String> {
  // Run the 'git rev-parse --show-toplevel' command
  let output = Command::new("git")
    .arg("rev-parse")
    .arg("--show-toplevel")
    .stdout(Stdio::piped())
    .spawn()?;

  // Read the output of the command
  let output = output.wait_with_output()?;

  if !output.status.success() {
    return Err(anyhow::anyhow!("Git command failed"));
  }

  // Convert the output to a string and return the path
  let git_root_path =
    std::str::from_utf8(&output.stdout)?.trim().to_string();

  Ok(git_root_path)
}
