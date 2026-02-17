# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ogre` is a Cargo-like all-in-one brainfuck (and brainfunct) tool written in Rust. The goal is a single CLI that covers the full development lifecycle for brainfuck: running, compiling to a native binary, formatting, static analysis, code suggestions/linting, structured testing, code generation, a live interactive interpreter, and a GDB-style debugger.

The brainfunct dialect extends standard brainfuck with named functions, which `format` and `analyse` handle specially (e.g. `--label-functions`).

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

## Subcommands and Intended Behaviour

### `run <file>`
Interprets and executes a brainfuck file directly.

### `compile <file> [-o <output>] [-k/--keep]`
Compiles brainfuck to C source, then invokes `gcc` to produce a native binary.
The intermediate `.c` file is deleted unless `-k`/`--keep` is passed.
If `-o` is not given, the output name is derived from the input filename.

### `start`
An interactive interpreter REPL, analogous to running `python` with no arguments.
The user types brainfuck instructions one at a time (or pastes snippets) and they
execute immediately. After each instruction (or line), the REPL displays the
current memory state centred on the data pointer — a window of cells showing
addresses, values, and an arrow marking the pointer. This makes it easy to
experiment and understand what code is doing step by step.

### `debug <file>`
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

After every pause the debugger should automatically print the current instruction
and a short memory summary.

### `format <file> [options]`
Formats a brainfuck file in-place.

| Flag | Default | Description |
|---|---|---|
| `--indent <n>` | 4 | Indentation per loop level in spaces |
| `--linewidth <n>` | 80 | Maximum line width |
| `--grouping <n>` | 5 | Group consecutive identical operators, e.g. `+++++ +++++` |
| `--label-functions` | off | (brainfunct) insert comment labels above each function |
| `-p`/`--preserve-comments` | off | Keep non-BF characters in place as comments (may reduce formatting quality) |

The formatter indents loop bodies (`[`…`]`), wraps long lines, and groups
runs of the same operator. If nesting depth multiplied by indent exceeds
`linewidth - 10`, it errors rather than producing unreadable output.

### `analyse <file> [options]`
Static analysis of a brainfuck script. Produces a report covering:
- Where the data pointer starts and ends
- Net effect on each memory cell (which cells are read/written)
- Total number of input (`,`) and output (`.`) operations
- Detection of obvious issues (unmatched brackets, infinite loops, etc.)
- Suggestions for how to write the code more idiomatically or efficiently

Supports section markers (`===`) in the source to split analysis by section.

| Flag | Description |
|---|---|
| `--in-place` | Embed the analysis as comments directly in the source file |
| `--verbose` | Extra detail per section |

### `test <test-file.json>`
Runs structured tests defined in a JSON file. Each entry specifies a brainfuck
script, optional stdin input, and expected stdout output. ogre runs the
interpreter against each case and reports pass/fail.

Test file schema (one object per test case):
```json
[
  {
    "name": "hello world",
    "brainfuck": "scripts/hello_world.bf",
    "input": "",
    "output": "Hello World!"
  }
]
```

### `new` (planned)
Scaffolds a new brainfuck project directory with a sample `.bf` file and a
starter test JSON.

### `generate` (planned)
Generates brainfuck code for common patterns:

| Subcommand | Description |
|---|---|
| `generate helloworld [-o <file>]` | Script that prints `Hello World!` |
| `generate string <str> [-o <file>]` | Script that prints an arbitrary string |
| `generate loop <n> [-o <file>]` | Script that loops n times |

Output goes to stdout if `-o` is not given.

## Core Interpreter Design

`src/modes/interpreter.rs` is the shared engine used by `run`, `debug`, `start`, and `test`.

Key design points:
- 30,000 cell tape, initialised to zero (standard brainfuck)
- Before execution, a jump table is pre-compiled: for every `[` and `]`,
  the matching bracket's index is stored so jumps are O(1) at runtime
- All 8 brainfuck operators are handled; all other characters are ignored
  (treated as comments) unless `--preserve-comments` is active in `format`
- The interpreter exposes a `step()` method (single instruction) as well as
  `run_code()` (run to completion) so that `debug` and `start` can drive
  execution one instruction at a time

## Architecture

```
src/
  main.rs              — clap CLI definition and subcommand dispatch
  modes/
    interpreter.rs     — core BF interpreter (tape, jump table, step/run)
    run.rs             — run mode (thin wrapper around interpreter)
    compile.rs         — BF → C code generation + gcc invocation
    format.rs          — in-place source formatter
    analyse.rs         — static analyser and linter
    debug.rs           — interactive GDB-style debug REPL
    interpreter.rs     — interactive start REPL with memory display
    new.rs             — project scaffolding
    test.rs            — JSON-driven test runner
```

The `debug` and `start` modes both depend on the core interpreter exposing
stepped execution; keep that interface stable when modifying `interpreter.rs`.

## Dependencies

- `clap` — CLI argument parsing (derive feature)
- `anyhow` — Error propagation
- `simple_logger` — Logging
