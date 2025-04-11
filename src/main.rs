use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
  let output = Command::new("git")
    .args(&["ls-tree", "-r", "--name-only", "HEAD"])
    .output()
    .expect("Failed to run git command");

  dbg!(&output);

  // First convert the output to a String and store it
  let file_list = String::from_utf8(output.stdout)
    .expect("Invalid UTF-8 output from git");

  // Then collect the lines into a Vec
  let files = file_list
    .lines()
    .filter(|line| !line.starts_with(".git"))
    .collect::<Vec<_>>();

  for file in files {
    let path = Path::new(file);
    let content =
      fs::read_to_string(path).expect("Failed to read file");

    println!("{}\n```", path.display());
    println!("{}", content);
    println!("```");
  }
}
