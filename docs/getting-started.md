# Getting Started with ogre

ogre is a Cargo-like all-in-one toolchain for brainfuck development, written
in Rust. It provides a single binary that covers the full development lifecycle:
interpreting, compiling to native binaries (or WASM), formatting, static analysis,
structured testing, code generation, an interactive REPL, and a GDB-style debugger.

ogre also supports **brainfunct**, a superset of brainfuck that adds named
functions, imports, constants, and a built-in standard library through a
compile-time preprocessor.

---

## 1. Installation

ogre is built from source using Cargo (the Rust package manager).

```bash
# Clone the repository
git clone https://github.com/your-org/ogre.git
cd ogre

# Build an optimized release binary
cargo build --release

# The binary is at target/release/ogre
# Copy it somewhere on your PATH
cp target/release/ogre ~/.local/bin/

# Verify the installation
ogre --version
ogre --help
```

### Requirements

- Rust toolchain (1.70+) -- install from <https://rustup.rs>
- `gcc` -- required only for `ogre compile` (native compilation via C)
- `wasmtime` -- required only for running WASM output from `ogre compile --target wasm`

---

## 2. Quick Start

### Create a new project

```bash
ogre new hello
cd hello
```

This scaffolds the following directory structure:

```
hello/
  ogre.toml           -- project manifest
  src/
    main.bf           -- entry point (starter template)
  tests/
    basic.json        -- example test suite
```

### Write a Hello World program

Open `src/main.bf` and replace its contents with a brainfuck program that
prints "Hello World!":

```brainfuck
@fn main {
    ++++++++++[>+++++++>++++++++++>+++>+<<<<-]
    >++.>+.+++++++..+++.>++.<<+++++++++++++++.
    >.+++.------.--------.>+.>.
}

@call main
```

Or generate one automatically:

```bash
ogre generate helloworld -o src/main.bf
```

### Run it

```bash
ogre run
```

When you omit the file argument inside a project directory, ogre finds
`ogre.toml` and uses the configured entry point (`src/main.bf` by default).

You can also run a standalone file directly:

```bash
ogre run src/main.bf
```

---

## 3. Project Structure

### The ogre.toml manifest

Every ogre project is defined by an `ogre.toml` file at its root:

```toml
[project]
name = "hello"
version = "0.1.0"
description = "My first brainfuck project"
author = "Your Name"
entry = "src/main.bf"

[build]
include = [
    "src/",            # all .bf files in src/ (non-recursive)
    "lib/utils.bf",    # a specific file
    "src/**/*.bf",     # glob patterns are supported
]
tape_size = 30000      # optional: override default tape size

[[tests]]
name = "Basic"
file = "tests/basic.json"

[[tests]]
name = "Edge cases"
file = "tests/edge.json"

[dependencies]
mylib = { path = "../mylib" }   # path-based dependency on another ogre project
```

Key fields:

- **`entry`** -- the file whose top-level code is the executable entry point.
  Resolved relative to the directory containing `ogre.toml`.
- **`include`** -- files and directories that belong to the project. Used by
  `format`, `analyse`, and `check` when run without a file argument. Supports
  directory paths (trailing `/`), specific files, and glob patterns (`*`, `**`, `?`).
- **`[[tests]]`** -- an array of test suites. Each entry points to a JSON file
  containing test cases.
- **`[dependencies]`** -- path-based dependencies on other ogre projects.
  Functions defined in dependency projects are available to `@call` in your code.

### Directory conventions

```
myproject/
  ogre.toml
  src/
    main.bf          -- entry point
    helpers.bf       -- additional source files
  lib/
    utils.bf         -- library functions
  tests/
    basic.json       -- test suites
    advanced.json
```

---

## 4. Common Workflows

### Running a brainfuck file

```bash
# Run a standalone file
ogre run hello.bf

# Run the project entry point (finds ogre.toml automatically)
ogre run

# Run with a larger tape
ogre run --tape-size 60000 hello.bf
```

### Compiling to a native binary

ogre compiles brainfuck to C, then invokes `gcc` to produce a native executable.

```bash
# Compile and name the output
ogre compile hello.bf -o hello

# Run the compiled binary
./hello

# Keep the intermediate C file for inspection
ogre compile hello.bf -o hello --keep
```

For projects, use `ogre build` which reads `ogre.toml` and names the binary
after the project:

```bash
ogre build
ogre build -o custom_name
```

### Formatting

Format brainfuck source in-place with configurable style options:

```bash
# Format a single file
ogre format hello.bf

# Format all project files (based on build.include)
ogre format

# Customize formatting
ogre format hello.bf --indent 2 --linewidth 100 --grouping 10

# Check formatting without modifying files (useful for CI)
ogre format --check hello.bf

# Show a diff of what would change
ogre format --diff hello.bf

# Preserve non-BF characters as comments
ogre format -p hello.bf

# Label brainfunct function boundaries
ogre format --label-functions hello.bf
```

### Running tests

Tests are defined in JSON files. Each test case specifies input, expected output,
and the brainfuck file to run:

```json
[
  {
    "name": "prints hello",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "Hello World!"
  },
  {
    "name": "echo input",
    "brainfuck": "src/echo.bf",
    "input": "abc",
    "output": "abc"
  }
]
```

Paths in the `brainfuck` field are resolved relative to the directory containing
the JSON file (or the project root when running project tests).

```bash
# Run a single test file
ogre test tests/basic.json

# Run all project test suites (reads [[tests]] from ogre.toml)
ogre test

# Verbose output showing each test result
ogre test --verbose
```

### Checking for errors

Validate that brackets are balanced, all `@call` references resolve, and there
are no import cycles:

```bash
# Check a single file
ogre check hello.bf

# Check all project files
ogre check
```

`ogre check` exits with code 0 if everything is valid, or code 1 if there are
errors. This is useful in CI pipelines.

### Static analysis

```bash
# Analyse a single file
ogre analyse hello.bf

# Analyse with extra detail
ogre analyse --verbose hello.bf

# Embed analysis as comments directly in the source file
ogre analyse --in-place hello.bf

# Analyse all project files
ogre analyse
```

### Benchmarking

Measure instruction count, wall time, and cells touched:

```bash
# Benchmark a file
ogre bench hello.bf

# Benchmark the project entry point
ogre bench

# Benchmark with a larger tape
ogre bench --tape-size 60000 hello.bf
```

### Execution tracing

Print the tape state as execution proceeds -- useful for understanding how a
program behaves instruction by instruction:

```bash
# Trace every instruction
ogre trace hello.bf

# Trace every 100th instruction (reduces output volume)
ogre trace --every 100 hello.bf

# Trace with a custom tape size
ogre trace --tape-size 1000 hello.bf
```

### Generating code

Quickly produce brainfuck for common patterns:

```bash
# Generate a Hello World program (prints to stdout)
ogre generate helloworld

# Generate and save to a file
ogre generate helloworld -o hello.bf

# Generate code that prints an arbitrary string
ogre generate string "Greetings!" -o greet.bf

# Generate a loop scaffold that runs N times
ogre generate loop 10 -o loop.bf
```

### Using the interactive REPL

`ogre start` launches an interactive interpreter. Type brainfuck instructions
and see the tape state after each line:

```bash
ogre start
```

```
ogre> +++
  [0] [0] [0]  3  [0] [0] [0]
            ^
ogre> >++
  [0] [0] [0] [3]  2  [0] [0]
                    ^
ogre> [<+>-]
  [0] [0] [0]  5  [0] [0] [0]
            ^
```

If you run `ogre start` inside a project directory, all `@fn` definitions from
your project files are preloaded and available to `@call` in the REPL session.

You can also specify a custom tape size:

```bash
ogre start --tape-size 60000
```

### Using the debugger

`ogre debug` provides a GDB-style interactive debugger:

```bash
ogre debug hello.bf
```

The debugger pauses before the first instruction. Available commands:

| Command                     | Description                                         |
|-----------------------------|-----------------------------------------------------|
| `step` / `step <n>`        | Execute 1 (or n) instructions and pause             |
| `continue`                  | Run until the next breakpoint or end of program     |
| `breakpoint <n>`            | Set a breakpoint at instruction index n             |
| `breakpoint list`           | List all breakpoints                                |
| `breakpoint delete <n>`     | Remove breakpoint at index n                        |
| `jump <n>`                  | Move the instruction pointer without executing      |
| `peek` / `peek <n>`        | Show the memory window around the pointer (or cell n) |
| `show instruction` / `show instruction <n>` | Show the current (or nth) instruction in context |
| `show memory`               | Dump a range of memory cells                        |
| `exit`                      | Quit the debugger                                   |

After every pause, the debugger prints the current instruction and a memory
summary.

### Packing

Output the fully preprocessed and expanded brainfuck as a single pure-BF file,
with all `@fn`, `@call`, `@import`, and `@const`/`@use` directives resolved:

```bash
# Pack to stdout
ogre pack hello.bf

# Pack to a file
ogre pack hello.bf -o packed.bf

# Pack with IR optimizations applied
ogre pack hello.bf --optimize -o optimized.bf

# Pack the project entry point
ogre pack
```

This is useful for sharing brainfuck programs with people who do not have ogre
installed.

### Generating documentation

Extract `@doc` comments and `@fn` definitions into readable documentation:

```bash
# Generate docs for a file (prints to stdout)
ogre doc hello.bf

# Save documentation to a file
ogre doc hello.bf -o docs.md

# Generate documentation for the built-in standard library
ogre doc --stdlib
```

### Initializing an existing directory

If you already have brainfuck files and want to add an `ogre.toml` to the
current directory:

```bash
ogre init
```

---

## 5. Using the Standard Library

ogre ships with a built-in standard library of reusable brainfunct functions.
Import modules with the `@import` directive using the `std/` prefix:

```brainfuck
@import "std/io.bf"
@import "std/math.bf"
@import "std/memory.bf"
@import "std/ascii.bf"
@import "std/debug.bf"
```

You can also omit the `.bf` extension:

```brainfuck
@import "std/io"
```

### Available modules

| Module   | Functions                                                              |
|----------|------------------------------------------------------------------------|
| `io`     | `print_newline`, `print_space`, `read_char`, `print_char`, `print_zero` |
| `math`   | `zero`, `inc`, `dec`, `inc10`, `double`, `add_to_next`, `move_right`, `move_left`, `copy_right` |
| `memory` | `clear`, `clear2`, `clear3`, `swap`, `push_right`, `pull_left`        |
| `ascii`  | `print_A`, `print_B`, `print_exclaim`, `print_dash`, `print_colon`   |
| `debug`  | `dump_cell`, `dump_and_newline`, `marker_start`, `marker_end`         |

### Browsing the standard library

```bash
# List all available modules
ogre stdlib list

# View the source of a module
ogre stdlib show io
ogre stdlib show math
```

### Example using the standard library

```brainfuck
@import "std/io"
@import "std/math"

@fn main {
    @call inc10
    @call inc10
    @call double
    @call print_char
    @call print_newline
}

@call main
```

When scaffolding a project, use `--with-std` to include standard library imports
in the starter file:

```bash
ogre new myproject --with-std
```

---

## 6. The Brainfunct Extension

Brainfunct extends standard brainfuck with compile-time directives processed
before execution. The final output is always pure brainfuck -- directives are
fully resolved and stripped.

### Defining and calling functions: @fn / @call

```brainfuck
@fn greet {
    ++++++++++[>+++++++>++++++++++>+++>+<<<<-]
    >++.>+.+++++++..+++.>++.
}

@call greet
```

`@fn name { body }` defines a named macro. `@call name` inlines the function
body at the call site. Functions can call other functions:

```brainfuck
@fn print_hi {
    ++++++++[>+++++++++++++<-]>++.+.[-]
}

@fn greet_twice {
    @call print_hi
    @call print_hi
}

@call greet_twice
```

Cycle detection prevents infinite recursion: if `A` calls `B` and `B` calls `A`,
the preprocessor reports an error.

### Importing files: @import

```brainfuck
@import "lib/helpers.bf"
@import "std/io.bf"

@call my_helper
@call print_newline
```

`@import` pulls in all `@fn` definitions from the imported file. Top-level code
in imported files is **not** executed -- only function definitions are collected.
Import paths are resolved relative to the importing file.

Import cycles are detected and reported as errors. Duplicate imports of the same
file are silently ignored.

### Named constants: @const / @use

```brainfuck
@const NEWLINE 10
@const STAR 42

@use NEWLINE .[-]
@use STAR .[-]
```

`@const NAME value` defines a named numeric constant. `@use NAME` expands to
`value` number of `+` characters. This is useful for producing specific ASCII
values without counting plus signs manually.

Constants can be used inside function bodies:

```brainfuck
@const EXCLAIM 33

@fn print_exclaim {
    @use EXCLAIM .[-]
}

@call print_exclaim
```

### Documenting functions: @doc

```brainfuck
@doc Prints a newline character (ASCII 10) and clears the current cell.
@fn print_newline {
    ++++++++++.[-]
}

@doc Reads one character from stdin into the current cell.
@doc The cell must be zero before calling.
@fn safe_read {
    ,
}
```

`@doc` lines immediately above an `@fn` are attached as documentation to that
function. Multi-line documentation is supported by using consecutive `@doc`
lines. Documentation is displayed by `ogre doc` and `ogre analyse --verbose`.

---

## 7. Watch Mode

Re-run your program automatically whenever the source file changes:

```bash
ogre run --watch hello.bf
ogre run -w hello.bf
```

ogre watches the file for modifications and re-executes it each time you save.
Press Ctrl+C to stop. This works with both standalone files and project entry
points:

```bash
# Watch the project entry point
ogre run --watch
```

---

## 8. Compiling to WASM

ogre can compile brainfuck to WebAssembly (WAT format), then assemble it into
a `.wasm` binary:

```bash
# Compile to WASM
ogre compile hello.bf --target wasm -o hello.wasm

# Run with a WASI-compatible runtime
wasmtime hello.wasm
```

The generated WASM module uses WASI for I/O (`fd_write` for output, `fd_read`
for input), so it runs in any WASI-compatible runtime such as `wasmtime` or
`wasmer`.

---

## 9. Verbosity Control

All ogre commands support global flags for controlling output verbosity:

```bash
# Suppress non-essential output (errors and requested data only)
ogre --quiet run hello.bf
ogre -q run hello.bf

# Enable extra detail (timing, instruction counts, per-file info)
ogre --verbose run hello.bf
ogre -v run hello.bf

# Disable colored output
ogre --no-color run hello.bf
```

These flags can be combined with any subcommand. The `NO_COLOR` environment
variable is also respected:

```bash
NO_COLOR=1 ogre run hello.bf
```

---

## Quick Reference

| Task                          | Command                                      |
|-------------------------------|----------------------------------------------|
| Create a new project          | `ogre new myproject`                         |
| Run a file                    | `ogre run hello.bf`                          |
| Run the project entry         | `ogre run`                                   |
| Compile to native             | `ogre compile hello.bf -o hello`             |
| Build the project             | `ogre build`                                 |
| Compile to WASM               | `ogre compile hello.bf --target wasm`        |
| Format a file                 | `ogre format hello.bf`                       |
| Check formatting (CI)         | `ogre format --check`                        |
| Static analysis               | `ogre analyse hello.bf`                      |
| Validate brackets and calls   | `ogre check hello.bf`                        |
| Run tests                     | `ogre test tests/basic.json`                 |
| Run all project tests         | `ogre test`                                  |
| Benchmark                     | `ogre bench hello.bf`                        |
| Trace execution               | `ogre trace hello.bf`                        |
| Generate Hello World          | `ogre generate helloworld -o hello.bf`       |
| Generate a string printer     | `ogre generate string "Hi!" -o hi.bf`        |
| Interactive REPL              | `ogre start`                                 |
| Interactive debugger          | `ogre debug hello.bf`                        |
| Pack to single BF file        | `ogre pack hello.bf -o packed.bf`            |
| Generate documentation        | `ogre doc hello.bf`                          |
| Browse the standard library   | `ogre stdlib list`                           |
| Initialize ogre.toml here     | `ogre init`                                  |
| Watch mode                    | `ogre run --watch hello.bf`                  |
