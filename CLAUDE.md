# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ogre` is a Cargo-like all-in-one brainfuck (and brainfunct) tool written in Rust. The goal is a single CLI that covers the full development lifecycle for brainfuck: running, compiling to a native binary, formatting, static analysis, code suggestions/linting, structured testing, code generation, a live interactive interpreter, and a GDB-style debugger.

The brainfunct dialect extends standard brainfuck with named functions via `@fn`/`@call`/`@import` macros handled by a compile-time preprocessor.

## Commands

```bash
cargo build
cargo build --release
cargo run -- <subcommand> [args]
cargo test
cargo test <test_name>   # run a single test by name
cargo fmt
cargo clippy
```

## ogre.toml — Project Manifest

When a file argument is omitted from any command, ogre walks the CWD upward
looking for `ogre.toml` and uses the project configuration.

```toml
[project]
name = "myproject"
version = "0.1.0"
description = "My brainfuck project"
author = "Alice"
entry = "src/main.bf"          # file whose top-level code is the executable entry point

[build]
include = [                    # files and/or directories that are part of the project
    "src/",                    # all .bf files in src/ (trailing slash = directory, non-recursive)
    "lib/utils.bf",            # specific file
]

[[tests]]
name = "Basic"
file = "tests/basic.json"

[[tests]]
name = "Advanced"
file = "tests/advanced.json"
```

- `entry` resolved relative to the directory containing `ogre.toml`
- `include`: directory entries (trailing `/`) collect all `.bf` files non-recursively
- `[[tests]]` is an array — each entry points to a separate `.json` test file
- `name` on `[[tests]]` is optional

## @fn / @call / @import Macro Syntax

```brainfuck
@import "lib/io.bf"

@fn print_newline {
    ++++++++++.[-]
}

@fn greet {
    @call print_hello
    @call print_newline
}

++ @call greet [-]
```

Rules:
- `@import "path"` — relative to the importing file; pulls in all `@fn` definitions;
  top-level code in imported files is **not** executed
- `@fn name { body }` — defines a named macro; `{`/`}` are delimiters (not valid BF)
- `@call name` — inlines the body at call site (compile-time expansion, recursive)
- Cycle detection: A→B→A or A→A → preprocessor error
- Unknown `@call` → preprocessor error
- Import cycle → preprocessor error
- Final output passed to interpreter/compiler is pure BF (all directives stripped)

The `@` character is **not** treated as a BF comment in formatter/analyser; it
always introduces a directive.

## Subcommands and Intended Behaviour

All file-based subcommands accept an optional `file` argument. When omitted,
ogre looks for `ogre.toml` and uses the project configuration.

### `run [file]`
Preprocesses and interprets a brainfuck file (or project entry).

### `compile [file] [-o <output>] [-k/--keep]`
Preprocesses brainfuck, then compiles to C and invokes `gcc`.
The intermediate `.c` file is deleted unless `-k`/`--keep` is passed.
If `-o` is not given, the output name is derived from the input filename.

### `build [-o <output>] [-k/--keep]`
Project-only: loads `ogre.toml`, preprocesses entry file, compiles to binary
named after `project.name` (or `-o` override).

### `start`
An interactive interpreter REPL. The user types BF instructions one at a time;
after each line the REPL shows the memory window centred on the data pointer.

### `debug [file]`
A GDB-style interactive debugger that loads a brainfuck file and pauses before
execution. Intended commands:

| Command | Description |
|---|---|
| `step` / `step <n>` | Execute 1 (or n) instruction(s) and pause |
| `continue` | Run until the next breakpoint or end of program |
| `breakpoint <n>` | Set a breakpoint at instruction index n |
| `breakpoint list` | List all breakpoints (index and instruction) |
| `breakpoint delete <n>` | Remove breakpoint n |
| `jump <n>` | Move the code pointer to instruction n without executing |
| `peek` / `peek <n>` | Show the memory window around the current pointer (or cell n) |
| `show instruction` / `show instruction <n>` | Show the current (or nth) instruction in context |
| `show memory` | Dump a range of memory cells |
| `exit` | Quit the debugger |

After every pause the debugger prints the current instruction and a short memory summary.

### `format [file] [options]`
Formats a brainfuck file in-place (or all project `include` files).
`@import`, `@fn`, `@call` directives are preserved verbatim on their own lines;
BF content inside `@fn` bodies is formatted normally.

| Flag | Default | Description |
|---|---|---|
| `--indent <n>` | 4 | Indentation per loop level in spaces |
| `--linewidth <n>` | 80 | Maximum line width |
| `--grouping <n>` | 5 | Group consecutive identical operators, e.g. `+++++ +++++` |
| `--label-functions` | off | (brainfunct) insert comment labels above each function |
| `-p`/`--preserve-comments` | off | Keep non-BF characters in place as comments |

### `analyse [file] [options]`
Static analysis of a brainfuck script (or all project `include` files).
The file is preprocessed first, then analysed on the expanded BF.

| Flag | Description |
|---|---|
| `--in-place` | Embed the analysis as comments directly in the source file |
| `--verbose` | Extra detail per section |

### `test [test-file.json]`
Runs structured tests (single file) or all project `[[tests]]` suites.

Test file schema:
```json
[
  {
    "name": "hello world",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "Hello World!"
  }
]
```
Paths in `brainfuck` are resolved relative to the directory containing the JSON file
(or the project base when running project tests).

### `new <name>`
Scaffolds a new brainfuck project directory:
```
<name>/
├── ogre.toml
├── src/
│   └── main.bf          (@fn main {} starter)
└── tests/
    └── basic.json       (template test case)
```

### `generate`
Generates brainfuck code for common patterns:

| Subcommand | Description |
|---|---|
| `generate helloworld [-o <file>]` | Script that prints `Hello World!` |
| `generate string <str> [-o <file>]` | Script that prints an arbitrary string |
| `generate loop <n> [-o <file>]` | Script that loops n times |

Output goes to stdout if `-o` is not given.

## Preprocessor Design (`src/modes/preprocess.rs`)

Two-pass design:

**Pass 1 — Collect**: walk source, handle `@import` (recursive, cycle-detect via
`HashSet<PathBuf>`), collect `@fn name { body }` into `HashMap<String, String>`,
return remaining top-level code with `@call name` markers preserved.

**Pass 2 — Expand**: walk top-level code, replace every `@call name` with the
body (itself recursively expanded; cycle-detect via `Vec<String>` call stack).

All modes pass source through `Preprocessor::process_file(path)` before feeding
to the interpreter/compiler/formatter.

## Core Interpreter Design

`src/modes/interpreter.rs` is the shared engine used by `run`, `debug`, `start`, and `test`.

Key design points:
- 30,000 cell tape, initialised to zero (standard brainfuck)
- Before execution, a jump table is pre-compiled: for every `[` and `]`,
  the matching bracket's index is stored so jumps are O(1) at runtime
- All 8 brainfuck operators are handled; all other characters are ignored
  (treated as comments) unless `--preserve-comments` is active in `format`
- The interpreter exposes a `step()` method (single instruction) as well as
  `run()` (run to completion) so that `debug` and `start` can drive execution
  one instruction at a time

## Architecture

```
src/
  main.rs              — clap CLI definition and subcommand dispatch
  project.rs           — OgreProject / ogre.toml TOML parsing
  modes/
    interpreter.rs     — core BF interpreter (tape, jump table, step/run)
    preprocess.rs      — @fn/@call/@import macro preprocessor
    run.rs             — run mode (thin wrapper around interpreter)
    compile.rs         — BF → C code generation + gcc invocation
    format.rs          — in-place source formatter (directive-aware)
    analyse.rs         — static analyser and linter
    debug.rs           — interactive GDB-style debug REPL
    start.rs           — interactive REPL with memory display
    new.rs             — project scaffolding (ogre.toml + src/ + tests/)
    test_runner.rs     — JSON-driven test runner (single file + project suites)
    generate.rs        — code generation (hello world, string, loop)
```

The `debug` and `start` modes both depend on the core interpreter exposing
stepped execution; keep that interface stable when modifying `interpreter.rs`.

## Dependencies

- `clap` — CLI argument parsing (derive feature)
- `anyhow` — Error propagation
- `serde` / `serde_json` — JSON test file parsing
- `toml` — `ogre.toml` parsing
- `simple_logger` — Logging

## Future Features TODO

- **`@const NAME value`** — named numeric constant, expands to `value` `+` signs
- **`@doc` comments** — docstring above `@fn`, shown by `ogre analyse --verbose`
- **Standard library** — `@import "std/io.bf"`, `@import "std/math.bf"`
- **`ogre check`** — validate all @call references resolve, no cycles, brackets match
- **`ogre pack`** — output fully preprocessed + expanded single `.bf` file for sharing
- **`ogre init`** — initialize `ogre.toml` in the current directory (vs `new`)
- **BF optimizer** — post-preprocessing: cancel `><`, simplify `[-]` on fresh cell, etc.
- **Watch mode** (`ogre run --watch`) — re-run on file save
- **`ogre fmt --check`** — exits non-zero if formatting would change anything (CI)
- **Tape tracing** (`ogre run --trace`) — print tape state after every instruction
- **`ogre bench`** — count instructions executed, report wall time
- **Named cell aliases** (`@alias varname 5`) — readable name for a tape index (debugger)
- **`ogre debug` with @fn awareness** — show `@fn greet+3` in status instead of `ip=47`
- **Project-aware `start` REPL** — REPL with all project @fn definitions pre-loaded
- **Cell size options** (`--cell-size 16/32`) — wider cell variants
- **WASM target** (`ogre compile --target wasm`)
