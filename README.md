# llmctx

`llmctx` is a command-line tool designed to collect and process
file content for use as context in large language models (LLMs).
It provides a simple way to gather files, with options to control
output size, copy results to the clipboard, and/or include build
errors.

---

## Installation

```bash
cargo install llmctx
```

---

## Usage

```bash
llmctx [OPTIONS] [PATHS]
```

### Options

| Flag              | Description                      | Short Flag |
|-------------------|----------------------------------|------------|
| `--copy`          | Copy output to clipboard         | `-c`       |
| `--debug`         | Run build and include in output  | `-d`       |
| `--test`          | Run tests and include in output  | `-t`       |
| `--num-tokens`    | Get an estimate of tokens        | `-n`       |
| `--max-tokens`    | Limit output by max tokens       |            |
| `--exclude`       | Exclude files matching a pattern | `-e`       |

### Examples

1. **Basic usage** (process current directory):
   ```bash
   llmctx .
   ```

2. **Copy output to clipboard**:
   ```bash
   llmctx . -c
   ```

3. **Build and copy to clipboard (only works with rust currently)**:
   ```bash
   llmctx --debug -c
   ```

### Example Output

When running `llmctx` without any special flags, the tool will
process files in the current directory (or the Git root if in a
repo) and output their contents in a structured format:

````text
src/main.rs:
```
fn main() {
    println!("Hello, world!");
}
```

README.md:
```
# My Project

This is a sample project for demonstration purposes.
```
````

---

## Clipboard Integration

On macOS, `llmctx` uses the `pbcopy` command to copy output to the
clipboard. For other systems, this feature may require alternative
tools or configurations.
