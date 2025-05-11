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

| Flag              | Description                              | Short Flag |
|-------------------|------------------------------------------|------------|
| `--max-tokens`    | Maximum number of tokens for output      |            |
| `--copy`          | Copy output to clipboard                 | `-c`       |
| `--debug`         | Enable debug mode (shows build info)     | `-d`       |
| `--test`          | Run tests and include output             | `-t`       |

### Examples

1. **Basic usage** (process current directory):
   ```bash
   llmctx
   ```

2. **Specify a directory**:
   ```bash
   llmctx ./src
   ```

3. **Limit tokens and copy to clipboard**:
   ```bash
   llmctx --max-tokens 1000 -c
   ```

4. **Debug mode (only works with rust currently)**:
   ```bash
   llmctx --debug
   ```

---

## Clipboard Integration

On macOS, `llmctx` uses the `pbcopy` command to copy output to the
clipboard. For other systems, this feature may require alternative
tools or configurations.
