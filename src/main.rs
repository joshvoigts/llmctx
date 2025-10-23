use anyhow::{anyhow, Result};
use ignore::WalkBuilder;
use optz::{Opt, Optz};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

const DEFAULT_MAX_TOKENS: &str = "100000000";

fn main() -> Result<()> {
  let optz = Optz::new("llmctx")
    .description("Process files and directories for LLM context")
    .usage("Usage: llmctx [options] [paths]")
    .option(
      Opt::flag("copy")
        .description("Copy output to clipboard")
        .short("-c"),
    )
    .option(
      Opt::flag("debug")
        .description("Enable debug mode")
        .short("-d"),
    )
    .option(
      Opt::flag("test")
        .description("Run tests and include output")
        .short("-t"),
    )
    .option(
      Opt::flag("num-tokens")
        .description("Get an estimate of tokens")
        .short("-n"),
    )
    .option(
      Opt::arg("max-tokens")
        .default_value(DEFAULT_MAX_TOKENS)
        .description("Limit output by max tokens"),
    )
    .option(
      Opt::arg("exclude")
        .description("Exclude files matching a pattern")
        .short("-e")
        .multiple(true),
    )
    .parse()?;

  let max_tokens: u64 =
    optz.get("max-tokens")?.expect("Invalid max_tokens");
  let should_copy_to_clipboard: bool = optz.has("copy")?;
  let should_debug: bool = optz.has("debug")?;
  let should_test: bool = optz.has("test")?;
  let should_count_tokens: bool = optz.has("num-tokens")?;

  let excludes: Vec<String> = optz.get_values("exclude")?;

  let max_length = max_tokens * 4;
  let mut total_chars: u64 = 0;
  let mut output = String::new();

  let mut paths: Vec<PathBuf> =
    optz.rest.iter().map(PathBuf::from).collect();

  if paths.is_empty() && !should_test && !should_debug {
    match get_git_root_path() {
      Ok(path) => paths.push(PathBuf::from(path)),
      Err(_) => paths.push(PathBuf::from(".")),
    }
  }

  // Handle the new count option
  if should_count_tokens {
    count_files(&paths, &excludes, &mut total_chars)?;
    println!("Total tokens: {}", total_chars / 4);
    return Ok(());
  }

  // Process all paths using WalkBuilder for unified traversal
  process_paths(
    &paths,
    max_length,
    &mut total_chars,
    &mut output,
    &excludes,
  )?;

  // Display or copy output
  if should_copy_to_clipboard {
    copy_to_clipboard(&output)?;
  } else {
    println!("{}", output);
  }

  // Process debug information if requested
  if should_debug {
    run_debug_command(&mut output)?;

    // Display or copy the updated output
    if should_copy_to_clipboard {
      copy_to_clipboard(&output)?;
    } else {
      println!("{}", output);
    }
  }

  if should_test {
    run_test_command(&mut output)?;

    // Display or copy the updated output
    if should_copy_to_clipboard {
      copy_to_clipboard(&output)?;
    } else {
      println!("{}", output);
    }
  }

  if !should_copy_to_clipboard {
    println!();
  }

  Ok(())
}

fn count_files(
  paths: &[PathBuf],
  excludes: &[String],
  total_chars: &mut u64,
) -> Result<()> {
  for path in paths {
    let walker = WalkBuilder::new(path).ignore(true).build();

    for result in walker {
      match result {
        Ok(entry) => {
          let file_path = entry.into_path();

          if file_path.is_file() && !should_skip(&file_path, excludes)
          {
            let content = fs::read_to_string(&file_path)?;
            let trimmed_content = content.trim();
            *total_chars += trimmed_content.len() as u64;
          }
        }
        Err(e) => {
          return Err(anyhow!("Error accessing path: {}", e));
        }
      }
    }
  }

  Ok(())
}

fn process_paths(
  paths: &[PathBuf],
  max_length: u64,
  total_chars: &mut u64,
  output: &mut String,
  excludes: &[String],
) -> Result<()> {
  for path in paths {
    // For each path, create a walker that respects .gitignore
    let walker = WalkBuilder::new(path).ignore(true).build();

    for result in walker {
      match result {
        Ok(entry) => {
          let file_path = entry.into_path();

          // Only process files that aren't skipped
          if file_path.is_file() && !should_skip(&file_path, excludes)
          {
            if let Err(e) = process_file(
              &file_path,
              max_length,
              total_chars,
              output,
            ) {
              return Err(anyhow!(
                "Error processing {}: {}",
                file_path.display(),
                e
              ));
            }

            // Stop if we've reached the token limit
            if *total_chars >= max_length {
              return Ok(());
            }
          }
        }
        Err(e) => {
          return Err(anyhow!("Error accessing path: {}", e));
        }
      }
    }
  }

  Ok(())
}

fn process_file(
  file: &PathBuf,
  max_length: u64,
  total_chars: &mut u64,
  output: &mut String,
) -> Result<()> {
  let content = fs::read_to_string(file)?;
  let trimmed_content = content.trim();
  let content_len = trimmed_content.len() as u64;

  let path_str = file.display().to_string();
  let file_output_len = (path_str.len() as u64) + content_len + 11; // Format overhead

  if *total_chars + file_output_len > max_length {
    return Ok(());
  }

  output.push_str(&format!("{}:\n", file.display()));
  output.push_str("```\n");
  output.push_str(trimmed_content);
  output.push_str("\n```\n\n");

  *total_chars += file_output_len;
  Ok(())
}

fn should_skip(path: &PathBuf, excludes: &[String]) -> bool {
  let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

  // Check if the file name matches any exclude pattern
  for pattern in excludes {
    if name.contains(pattern) {
      return true;
    }
  }

  // Existing conditions
  name.starts_with(".")
    || name.ends_with(".lock")
    || name == "LICENSE"
    || path.to_string_lossy().contains("node_modules")
    || name == "package-lock.json"
}

fn get_git_root_path() -> Result<String> {
  let output = Command::new("git")
    .arg("rev-parse")
    .arg("--show-toplevel")
    .stdout(Stdio::piped())
    .spawn()?
    .wait_with_output()?;

  if !output.status.success() {
    return Err(anyhow!("Git command failed"));
  }

  let git_root_path =
    String::from_utf8(output.stdout)?.trim().to_string();
  Ok(git_root_path)
}

fn copy_to_clipboard(output: &str) -> Result<()> {
  let mut child =
    Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

  if let Some(mut stdin) = child.stdin.take() {
    stdin.write_all(output.as_bytes())?;
  }

  child.wait()?;
  Ok(())
}

fn run_debug_command(output: &mut String) -> Result<()> {
  let mut child = Command::new("cargo")
    .arg("build")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  let mut stdout_bytes = Vec::new();
  let mut stderr_bytes = Vec::new();

  if let Some(mut stdout) = child.stdout.take() {
    stdout.read_to_end(&mut stdout_bytes)?;
  }

  if let Some(mut stderr) = child.stderr.take() {
    stderr.read_to_end(&mut stderr_bytes)?;
  }

  let output_str = String::from_utf8_lossy(&stdout_bytes);
  let error_str = String::from_utf8_lossy(&stderr_bytes);

  output.push_str("Build Output:\n");
  output.push_str("```\n");
  output.push_str(&format!("{error_str}{output_str}"));
  output.push_str("```\n\n");

  child.wait()?;
  Ok(())
}

fn run_test_command(output: &mut String) -> Result<()> {
  let mut child = Command::new("cargo")
    .arg("test")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  let mut stdout_bytes = Vec::new();
  let mut stderr_bytes = Vec::new();

  if let Some(mut stdout) = child.stdout.take() {
    stdout.read_to_end(&mut stdout_bytes)?;
  }

  if let Some(mut stderr) = child.stderr.take() {
    stderr.read_to_end(&mut stderr_bytes)?;
  }

  let output_str = String::from_utf8_lossy(&stdout_bytes);
  let error_str = String::from_utf8_lossy(&stderr_bytes);

  output.push_str("Test Output:\n");
  output.push_str("```\n");
  output.push_str(&format!("{error_str}{output_str}"));
  output.push_str("```\n\n");

  child.wait()?;
  Ok(())
}
