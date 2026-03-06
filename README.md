# ogre

A Cargo-like all-in-one brainfuck tool. One binary covering the full development lifecycle for brainfuck programs: running, compiling to native binaries, formatting, static analysis, linting, structured testing, code generation, an interactive REPL, a GDB-style debugger, and more.

The **brainfunct** dialect extends standard brainfuck with named functions via `@fn`/`@call`/`@import` macros, a compile-time preprocessor, and a built-in standard library.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Brainfunct Macros](#brainfunct-macros)
- [Standard Library](#standard-library)
- [Project Manifest](#project-manifest)
- [Commands](#commands)
- [Test File Format](#test-file-format)
- [Examples](#examples)
- [Building from Source](#building-from-source)

---

## Installation

Requires [Rust and Cargo](https://rustup.rs). Install directly from GitHub:

```sh
cargo install --git https://github.com/trentzz/ogre
```

Once installed, the `ogre` binary is available on your PATH.

---

## Quick Start

```sh
# Run a brainfuck file
ogre run hello.bf

# Format a file in-place
ogre format hello.bf

# Analyse a file for issues
ogre analyse hello.bf

# Compile to a native binary
ogre compile hello.bf -o hello

# Open the interactive REPL
ogre start

# Scaffold a new project
ogre new myproject

# Use the standard library
echo '@import "std/io.bf"' > hello.bf
echo '@call print_newline' >> hello.bf
ogre run hello.bf
```

---

## Brainfunct Macros

Brainfunct extends brainfuck with compile-time macros handled by the preprocessor.

```brainfuck
@import "lib/utils.bf"
@import "std/io.bf"

@fn greet {
    @call print_newline
}

@const REPEAT 5

++ @call greet [-]
@use REPEAT
```

| Directive | Description |
|-----------|-------------|
| `@import "path"` | Import `@fn` definitions from another file (relative to importing file) |
| `@import "std/module.bf"` | Import a built-in standard library module |
| `@fn name { body }` | Define a named macro function |
| `@call name` | Inline-expand a function at the call site |
| `@const NAME value` | Define a numeric constant |
| `@use NAME` | Expand a constant to N `+` characters |
| `@doc text` | Docstring above `@fn`, shown by `ogre doc` |

Top-level code in imported files is discarded (only `@fn` definitions are kept).
Cycle detection prevents `A->B->A` or `A->A` recursion.

---

## Standard Library

ogre ships with a built-in standard library of reusable functions. Import modules with `@import "std/module.bf"`.

| Module | Description |
|--------|-------------|
| `std/io.bf` | I/O utilities: `print_newline`, `print_space`, `read_char`, `print_char`, `flush_input`, and 25+ character printers |
| `std/math.bf` | Arithmetic: `inc`, `dec`, `double`, `multiply`, `square`, `modulo`, `divmod_10`, `min`, `max`, `clamp`, and more |
| `std/memory.bf` | Memory ops: `clear`, `swap`, `dup`, `copy_right`, `rotate3`, `reverse3`, `fill_5`, `shift_right_3`, and more |
| `std/string.bf` | String/text: `read_line`, `read_word`, `read_decimal`, `print_string`, `compare_char`, `skip_spaces`, `skip_line` |
| `std/logic.bf` | Boolean logic: `not`, `and`, `or`, `xor`, `nand`, `equal`, `greater_than`, `less_than`, `if_nonzero` |
| `std/ascii.bf` | ASCII utilities: `to_upper`, `to_lower`, `is_digit`, `is_alpha`, `is_upper`, `is_lower`, `is_printable` |
| `std/debug.bf` | Debugging: `dump_decimal`, `dump_hex`, `dump_range_5`, `separator`, `marker_start`, `marker_end` |
| `std/cli.bf` | CLI toolkit: `skip_dashes`, `read_flag_char`, `read_arg`, `match_char`, `print_error_prefix` |
| `std/convert.bf` | Format conversion: `print_decimal`, `print_hex_digit`, `print_binary_8`, `atoi_single`, `itoa_single` |

```sh
ogre stdlib list          # List all modules
ogre stdlib show io       # View a module's source
```

See [stdlibdocs/](stdlibdocs/) for per-function documentation, or [docs/stdlib-reference.md](docs/stdlib-reference.md) for an overview.

---

## Project Manifest

ogre projects use an `ogre.toml` manifest file:

```toml
[project]
name = "myproject"
version = "0.1.0"
description = "My brainfuck project"
author = "Alice"
entry = "src/main.bf"

[build]
include = [
    "src/",
    "lib/utils.bf",
]

[[tests]]
name = "Basic"
file = "tests/basic.json"
```

When a file argument is omitted from any command, ogre walks the CWD upward looking for `ogre.toml` and uses the project configuration.

---

## Commands

### Core

| Command | Description |
|---------|-------------|
| `ogre run [file]` | Preprocess and interpret a brainfuck file |
| `ogre compile [file] [-o output] [-k]` | Compile to native binary via C |
| `ogre build [-o output] [-k]` | Build a project (loads `ogre.toml`) |
| `ogre start` | Interactive REPL with memory display |
| `ogre debug [file]` | GDB-style interactive debugger |

### Code Quality

| Command | Description |
|---------|-------------|
| `ogre format [file] [options]` | Format brainfuck source in-place |
| `ogre analyse [file] [--verbose] [--in-place]` | Static analysis and linting |
| `ogre check [file]` | Validate brackets, `@call` references, and imports |

### Testing & Benchmarking

| Command | Description |
|---------|-------------|
| `ogre test [test-file.json]` | Run structured JSON test suites |
| `ogre bench [file]` | Count instructions executed and report wall time |
| `ogre trace [file]` | Print tape state after every instruction |

### Project Management

| Command | Description |
|---------|-------------|
| `ogre new <name>` | Scaffold a new project directory |
| `ogre init` | Initialize `ogre.toml` in the current directory |
| `ogre pack [file]` | Output fully preprocessed pure brainfuck |
| `ogre doc [file] [--stdlib]` | Generate documentation for functions |

### Code Generation & Stdlib

| Command | Description |
|---------|-------------|
| `ogre generate helloworld [-o file]` | Generate Hello World program |
| `ogre generate string <str> [-o file]` | Generate code to print a string |
| `ogre generate loop <n> [-o file]` | Generate a counted loop scaffold |
| `ogre stdlib list` | List standard library modules |
| `ogre stdlib show <module>` | View a module's source code |

See [docs/cli-reference.md](docs/cli-reference.md) for the full CLI reference with all flags and options.

---

## Test File Format

Test files are JSON arrays. Each object describes one test case:

```json
[
  {
    "name": "hello world",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "Hello World!\n"
  }
]
```

| Field | Type | Description |
|---|---|---|
| `name` | string | Human-readable test name |
| `brainfuck` | string | Path to the `.bf` file (relative to JSON file's directory) |
| `input` | string | Data fed to stdin |
| `output` | string | Expected stdout output |
| `output_regex` | string | Regex pattern to match against output (alternative to `output`) |

---

## Examples

The `examples/` directory contains complete ogre projects:

| Example | Description |
|---------|-------------|
| `hello/` | Classic Hello World |
| `cat/` | Cat program (echo stdin) |
| `fibonacci/` | Fibonacci number printer |
| `multifile/` | Multi-file project with `@import` |
| `stdlib-demo/` | Standard library usage demo |
| `convert/` | CLI tool with argument parsing (`--upper`, `--lower`, `--reverse`, `--ascii-to-decimal`, `--decimal-to-ascii`) |

Run any example:

```sh
cd examples/hello
ogre run
ogre test
```

---

## Building from Source

```sh
git clone https://github.com/trentzz/ogre
cd ogre
cargo build --release
# Binary is at target/release/ogre
```

Run the test suites:

```sh
cargo test                          # Rust unit tests
cd stdlibtests && ogre test         # Standard library tests (105 tests)
cd examples/convert && ogre test    # Convert example tests
```
