# cli -- Command-Line Argument Handling

```brainfuck
@import "std/cli.bf"
```

The `cli` module provides functions for parsing command-line arguments passed to brainfuck programs via ogre's `--` syntax (e.g., `ogre run program.bf -- arg1 arg2`). It includes utilities for skipping prefixes, reading arguments and flags, and printing standard error/usage messages.

## Function Reference

### Argument Skipping

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `skip_dashes` | Read and discard 2 characters (a `--` prefix). c0 cleared. | c0 | c0 |
| `skip_to_space` | Read and discard characters until a space (32) is found. Space consumed. c0 cleared. | c0, c1 as scratch | c0 |
| `skip_to_newline` | Read and discard characters until a newline (10) is found. Newline consumed. c0 cleared. | c0, c1 as scratch | c0 |

### Argument Reading

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `read_arg` | Read characters until space (32) or EOF (0), storing starting at c0 with one char per cell. The terminating space is not stored. | c0 onward (one cell per char) | Zero-terminator cell (one past last char) |
| `read_flag_char` | Skip `--` prefix, read one flag identifier character into c0, then skip to next space. c0 contains the flag character. | c0, c1--c2 as scratch | c0 |

### Message Printing

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `print_error_prefix` | Print the string `"Error: "` to output. | c0 | c0 |
| `print_usage_prefix` | Print the string `"Usage: "` to output. | c0 | c0 |

### Character Matching

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `match_char` | Compare c0 against c1. Set c2 to 1 if equal, 0 if not. c0 preserved, c1 zeroed. | c0 (preserved), c1 (consumed), c2 (result), c3 as scratch | c0 |

## Usage Example

```brainfuck
@import "std/cli.bf"
@import "std/io.bf"

Parse a simple flag from CLI input (e.g., "--v"):
@call read_flag_char
Cell 0 now contains the flag character (e.g., 'v' = 118)

Read an argument word:
@call read_arg
Characters stored in successive cells, pointer at terminator

Print an error message:
@call print_error_prefix
Outputs: Error:
Then print specific error text after...
@call print_newline
```
