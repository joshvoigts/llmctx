use anyhow::Result;
use optz::{Opt, Optz};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

const DEFAULT_MAX_TOKENS: u64 = 100_000_000;

fn main() -> Result<()> {
  let true_value = Some("true".to_string());

  let opts = Optz::new("my_program")
    .description("Process files and directories")
    .usage("Usage: llmctx [options] [paths]")
    .option(
      Opt::new("max-tokens")
        .description("Maximum number of tokens")
        .arg("NUM"),
    )
    .option(
      Opt::new("copy")
        .description("Copy output to clipboard")
        .short("-c"),
    )
    .option(
      Opt::new("debug")
        .description("Enable debug mode")
        .short("-d"),
    )
    .parse();

  let max_tokens_str = opts
    .get("max-tokens")
    .unwrap_or(DEFAULT_MAX_TOKENS.to_string());
  let max_tokens = max_tokens_str
    .parse::<u64>()
    .expect("Invalid max-tokens value");
  let should_copy_to_clipboard = opts.get("copy") == true_value;
  let should_debug = opts.get("debug") == true_value;

  let max_length = max_tokens * 5;
  let mut total_chars: u64 = 0;
  let mut output = String::new();

  let mut paths: Vec<PathBuf> =
    opts.rest.iter().map(PathBuf::from).collect();

  if paths.is_empty() {
    match get_git_root_path() {
      Ok(path) => paths.push(PathBuf::from(path)),
      Err(_) => paths.push(PathBuf::from(".")),
    }
  }

  for path in paths {
    process_path(&path, max_length, &mut total_chars, &mut output)?;
  }

  if should_copy_to_clipboard {
    copy_to_clipboard(&output)?;
  } else {
    println!("{}", output);
  }

  if should_debug {
    output.push_str("\n\n```\n");
    run_debug_command(&mut output)?;
    output.push_str("\n```");
  }

  if should_copy_to_clipboard {
    copy_to_clipboard(&output)?;
  } else {
    println!("{}", output);
  }

  Ok(())
}

fn process_path(
  path: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
  output: &mut String,
) -> Result<()> {
  if path.is_dir() {
    process_directory(path, max_length, total_chars, output)?;
  } else if !skip_path(path) {
    process_file(path, max_length, total_chars, output)?;
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
  output: &mut String,
) -> Result<()> {
  let entries =
    fs::read_dir(directory).expect("Failed to read directory");

  for entry in entries {
    let entry = entry?;
    let path = entry.path();

    if skip_path(&path) {
      continue;
    }

    process_path(&path, max_length, total_chars, output)?;
  }

  Ok(())
}

fn process_file(
  file: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
  output: &mut String,
) -> Result<()> {
  let content = fs::read_to_string(file).expect(
    format!("Failed to read file: {}", file.display()).as_str(),
  );
  let trimmed_content = content.trim();
  let content_len = trimmed_content.len() as u64;

  let path_str = file.display().to_string();
  let file_output_len = (path_str.len() as u64) + content_len + 11;

  if *total_chars + file_output_len > max_length {
    return Ok(());
  }

  output.push_str(&format!("\n\n{}:\n", file.display()));
  output.push_str("```\n");
  output.push_str(&trimmed_content);
  output.push_str("\n```");

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

fn copy_to_clipboard(output: &str) -> Result<()> {
  Command::new("pbcopy")
    .stdin(Stdio::piped())
    .spawn()?
    .stdin
    .unwrap()
    .write_all(output.as_bytes())?;
  Ok(())
}

fn run_debug_command(output: &mut String) -> Result<()> {
  let mut child = Command::new("cargo")
    .arg("build")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  let mut stdout = child.stdout.take().unwrap();
  let mut stderr = child.stderr.take().unwrap();

  let mut stdout_bytes = Vec::new();
  let mut stderr_bytes = Vec::new();

  stdout.read_to_end(&mut stdout_bytes)?;
  stderr.read_to_end(&mut stderr_bytes)?;

  let output_str = String::from_utf8_lossy(&stdout_bytes);
  let error_str = String::from_utf8_lossy(&stderr_bytes);

  output.push_str(&format!("Build Output:\n{}\n", output_str));
  output.push_str(&format!("Build Errors:\n{}\n", error_str));

  child.wait()?;
  Ok(())
}
