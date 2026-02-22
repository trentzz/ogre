# ogre CLI Reference

Complete command-line reference for ogre, a Cargo-like all-in-one brainfuck toolchain.

---

## Global Flags

These flags can be passed to any subcommand.

| Flag | Short | Description |
|------|-------|-------------|
| `--quiet` | `-q` | Suppress non-essential output. Only errors and explicitly requested data are printed. |
| `--verbose` | `-v` | Enable verbose output. Shows extra detail such as instruction counts, timing, per-file status, and expanded analysis sections. |
| `--no-color` | | Disable colored terminal output. Also respected via the `NO_COLOR` environment variable. |
| `--version` | | Print the ogre version and exit. |
| `--help` | `-h` | Print help information. Available on every subcommand as well. |

---

## Project Resolution

When a file argument is omitted from any command that accepts one, ogre walks the current working directory upward looking for `ogre.toml` and uses the project configuration (entry file, include paths, test suites, etc.). If no `ogre.toml` is found and no file is given, ogre exits with an error suggesting `ogre new <name>` or providing a file argument.

---

## Commands

### ogre run

Preprocess and interpret a brainfuck file.

```
ogre run [file] [--tape-size <n>] [-w/--watch]
```

**Description:** Preprocesses the source (expanding `@import`, `@fn`, `@call` directives), then interprets the resulting pure brainfuck. Stdin and stdout are connected directly to the terminal for interactive I/O.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml`. |

**Flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--tape-size <n>` | | 30000 | Number of cells in the brainfuck tape. Can also be set via `build.tape_size` in `ogre.toml`. |
| `--watch` | `-w` | off | Watch the file for changes and automatically re-run on save. The terminal is cleared between runs. Press Ctrl+C to stop. |

**Examples:**

```bash
ogre run hello.bf
ogre run --tape-size 60000 big.bf
ogre run --watch hello.bf
ogre run                          # uses project entry from ogre.toml
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Program executed successfully. |
| 1 | Preprocessing, parsing, or runtime error. |

---

### ogre compile

Compile brainfuck to a native binary (via C) or to WebAssembly.

```
ogre compile [file] [-o <output>] [-k/--keep] [--target native|wasm]
```

**Description:** Preprocesses the source, parses it into an optimized intermediate representation, then generates C code (or WAT for WebAssembly) and invokes the system C compiler (cc, gcc, or clang) to produce a binary. The intermediate file is deleted unless `--keep` is passed.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml`. |

**Flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `-o <output>` | `-o` | derived from input filename | Name of the output binary. |
| `--keep` | `-k` | off | Keep the intermediate `.c` file (or `.wat` for WASM) instead of deleting it after compilation. |
| `--target <target>` | | `native` | Compilation target. `native` compiles to a native binary via C. `wasm` generates WebAssembly (requires `wat2wasm` from the WABT toolkit for binary `.wasm` output; otherwise produces a `.wat` text file). |

**Examples:**

```bash
ogre compile hello.bf -o hello
ogre compile hello.bf --keep
ogre compile hello.bf --target wasm
ogre compile hello.bf --target wasm -o hello
ogre compile                               # uses project entry
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Compilation succeeded. |
| 1 | Preprocessing error, C compiler not found, or compilation failed. |

**Notes:**
- The native target searches for `cc`, `gcc`, or `clang` on `PATH` in that order.
- The WASM target uses the IR optimizer before generating WAT. If `wat2wasm` is not available, the `.wat` file is kept and a hint is printed.
- The C compiler is invoked with `-O2` optimization.

---

### ogre build

Build the current project to a native binary. Requires `ogre.toml`.

```
ogre build [-o <output>] [-k/--keep]
```

**Description:** Loads the project manifest, preprocesses the entry file (resolving all includes and dependencies), and compiles to a native binary. The output binary is named after `project.name` unless overridden with `-o`. Prints project metadata (name, version, description, author) on success.

**Flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `-o <output>` | `-o` | `project.name` from `ogre.toml` | Override the output binary name. |
| `--keep` | `-k` | off | Keep the intermediate `.c` file. |

**Examples:**

```bash
ogre build
ogre build -o myapp
ogre build --keep
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Build succeeded. |
| 1 | No `ogre.toml` found, preprocessing error, or compilation failure. |

---

### ogre start

Launch an interactive brainfuck interpreter REPL.

```
ogre start [--tape-size <n>]
```

**Description:** Opens an interactive read-eval-print loop. Type brainfuck code at the `>>>` prompt; it is executed immediately and the tape state around the data pointer is displayed after each input. If run inside a project directory, all `@fn` definitions from the project and its dependencies are pre-loaded and available via `@call`.

The REPL supports `@call`, `@fn`, `@const`, `@use`, and `@import` directives inline. Command history is saved to `~/.ogre_history`.

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--tape-size <n>` | 30000 | Number of cells in the tape. Also configurable via `build.tape_size` in `ogre.toml`. |

**REPL Commands:**

| Command | Description |
|---------|-------------|
| `:help` | Show the list of REPL commands. |
| `:load <file>` | Load and execute a brainfuck file. Any `@fn` definitions in the file are added to the environment. |
| `:save <file>` | Save the current tape state (pointer position, cell values) to a file. |
| `:functions` | List all loaded `@fn` definitions with a short body preview. |
| `:peek` | Show the memory window around the current data pointer. |
| `:dump [n]` | Dump the first `n` tape cells (default 20). The current pointer position is highlighted. |
| `:reset` | Reset the tape and interpreter to initial state (all cells zeroed, pointer at 0). |
| `:quit` / `:exit` | Exit the REPL. Also accepts `quit`, `exit`, or Ctrl+D (EOF). |

**Examples:**

```bash
ogre start
ogre start --tape-size 60000
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Normal exit. |

---

### ogre debug

Launch a GDB-style interactive debugger for a brainfuck program.

```
ogre debug [file] [--tape-size <n>]
```

**Description:** Loads and preprocesses a brainfuck file, then pauses before the first instruction. The debugger prints the current instruction, data pointer, cell value, and a short tape window after every pause. If a source map is available (from preprocessing), the debugger shows the original file, line, and function context.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml`. |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--tape-size <n>` | 30000 | Number of cells in the tape. |

**Debugger Commands:**

| Command | Description |
|---------|-------------|
| `step [n]` | Execute 1 instruction (or `n` instructions) and pause. |
| `continue` / `c` | Run until the next breakpoint or end of program. |
| `breakpoint <n>` | Set a breakpoint at op index `n`. |
| `breakpoint list` | List all breakpoints with their op index and instruction. |
| `breakpoint delete <n>` | Remove the breakpoint at op index `n`. |
| `jump <n>` | Move the code pointer to op index `n` without executing any instructions. |
| `peek [n]` | Show a memory window around the current data pointer (or around cell `n`). |
| `show instruction [n]` | Show the current instruction (or the instruction at index `n`) with surrounding context. |
| `show memory` | Dump a wider range of memory cells around the data pointer. |
| `where` | Show the current source location (file, line, function) if a source map is loaded. |
| `exit` / `quit` / `q` | Quit the debugger. |

**Examples:**

```bash
ogre debug hello.bf
ogre debug --tape-size 60000 big.bf
ogre debug                             # debug project entry
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Normal exit from debugger. |
| 1 | File not found or preprocessing error. |

---

### ogre format

Format brainfuck source files in-place.

```
ogre format [file] [--indent <n>] [--linewidth <n>] [--grouping <n>]
            [--label-functions] [-p/--preserve-comments]
            [--check] [--diff]
```

**Description:** Reads a brainfuck source file, formats the BF content with indentation, operator grouping, and line wrapping, and writes the result back to the file. `@import`, `@fn`, and `@call` directives are preserved on their own lines; BF code inside `@fn` bodies is formatted normally. When no file is given, formats all files matched by the project's `build.include` paths.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, formats all project include files. |

**Flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--indent <n>` | | 4 | Number of spaces per loop nesting level. |
| `--linewidth <n>` | | 80 | Maximum line width before wrapping. |
| `--grouping <n>` | | 5 | Insert a space every `n` consecutive identical operators (e.g., `+++++ +++++`). Set to 0 to disable grouping. |
| `--label-functions` | | off | (brainfunct) Insert comment labels above each function definition. |
| `--preserve-comments` | `-p` | off | Keep non-BF characters in place as comments instead of stripping them. |
| `--check` | | off | Check if files are already formatted without modifying them. Prints status per file and exits with code 1 if any file would be reformatted. Useful for CI. |
| `--diff` | | off | Show a colored unified diff of what the formatter would change, without modifying files. Exits with code 1 if any file would be reformatted. |

**Examples:**

```bash
ogre format hello.bf
ogre format --check hello.bf
ogre format --diff hello.bf
ogre format --indent 2 --grouping 10 hello.bf
ogre format -p hello.bf
ogre format                                      # format all project files
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Formatting succeeded (or files are already formatted when using `--check`/`--diff`). |
| 1 | Files would be reformatted (`--check` or `--diff` mode), or an error occurred. |

---

### ogre analyse

Perform static analysis on a brainfuck file.

```
ogre analyse [file] [--in-place] [--verbose]
```

**Description:** Preprocesses the source, then performs static analysis on the expanded brainfuck. Reports bracket matching, I/O operation counts, data pointer tracking, and various code quality findings. The file is preprocessed first so that analysis covers the fully expanded program.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, analyses all project include files. |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--in-place` | off | Embed the analysis report as comments (`# ...`) at the top of the source file instead of printing to stdout. |
| `--verbose` | off | Show extra sections: per-operator counts, complexity metrics (max loop depth, total instructions, optimized IR ops, optimization reduction percentage), and deep analysis (clear idioms, cancellations, dead code, unbalanced pointer warnings). Also activated by the global `-v` flag. |

**Analysis output includes:**

- Bracket validation (matched/unmatched `[` and `]`)
- Input (`,`) and output (`.`) operation counts
- Data pointer net offset (or "indeterminate" if loops are present)
- (verbose) Per-operator breakdown (`>`, `<`, `+`, `-`, `[`, `]`)
- (verbose) Complexity metrics: max loop nesting depth, total BF instructions, optimized IR op count, optimization reduction percentage
- (verbose) Deep analysis: clear idiom (`[-]`/`[+]`) count, operator cancellations (`+-`, `><`), unreachable dead code, unbalanced pointer warnings

**Examples:**

```bash
ogre analyse hello.bf
ogre analyse --verbose hello.bf
ogre analyse --in-place hello.bf
ogre analyse                         # analyse all project files
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Analysis completed (even if issues were found; issues are reported in output). |
| 1 | File not found or preprocessing error. |

---

### ogre test

Run structured tests from a JSON test file or all project test suites.

```
ogre test [test-file.json] [--verbose]
```

**Description:** Loads test cases from a JSON file and runs each one: preprocesses the referenced `.bf` file, interprets it with the given input, and compares the output against the expected value. Supports exact string matching, regex matching, and instruction-count timeouts.

When no file argument is given, runs all test suites defined in the `[[tests]]` sections of `ogre.toml`.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `test-file.json` | Path to a JSON test file. If omitted, runs all project test suites. |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--verbose` | off | Show per-test PASS/FAIL status lines instead of compact dot notation. |

**Test file JSON schema:**

```json
[
  {
    "name": "test case name",
    "brainfuck": "path/to/file.bf",
    "input": "stdin input string",
    "output": "expected stdout output",
    "output_regex": "optional regex pattern",
    "timeout": 10000000
  }
]
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Human-readable name for the test case. |
| `brainfuck` | yes | Path to the `.bf` file to test. Resolved relative to the directory containing the JSON file (or the project base directory for project tests). |
| `input` | yes | String to provide as stdin to the program. Use `""` for no input. |
| `output` | yes | Expected stdout output (exact string match). Use `""` if using `output_regex` instead. |
| `output_regex` | no | Regex pattern to match against stdout instead of exact comparison. Mutually exclusive with a non-empty `output` field. |
| `timeout` | no | Maximum number of instructions before the test is aborted. Default: 10,000,000. |

**Output format:**

- Default: compact dot notation (`.` = pass, `F` = fail, `T` = timeout)
- Verbose: one line per test with `PASS`, `FAIL`, or `TIMEOUT` status
- Failed tests print a summary with the test name and failure detail (expected vs. actual output, or regex mismatch)

**Examples:**

```bash
ogre test tests/basic.json
ogre test tests/basic.json --verbose
ogre test                              # run all project test suites
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | All tests passed. |
| 1 | One or more tests failed or timed out. |

---

### ogre new

Scaffold a new brainfuck project directory.

```
ogre new <name> [--with-std]
```

**Description:** Creates a new project directory with the standard ogre project structure. Fails if the directory already exists.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `name` | Project name. Used as the directory name and the `project.name` in `ogre.toml`. |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--with-std` | off | Include a standard library import (`@import "std/io.bf"`) in the starter `src/main.bf` and set up the test case accordingly. |

**Generated structure:**

```
<name>/
  ogre.toml            # project manifest
  src/
    main.bf            # starter file with @fn main {} and @call main
  tests/
    basic.json         # template test case pointing at src/main.bf
```

**Examples:**

```bash
ogre new myproject
ogre new myproject --with-std
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Project created successfully. |
| 1 | Directory already exists. |

---

### ogre init

Initialize `ogre.toml` in the current directory.

```
ogre init
```

**Description:** Creates an `ogre.toml` in the current directory, deriving `project.name` from the directory name. Also creates `src/` and `tests/` directories (and starter files `src/main.bf`, `tests/basic.json`) if they do not already exist. Fails if `ogre.toml` already exists.

Unlike `ogre new`, this command works in an existing directory without creating a new one.

**Examples:**

```bash
cd myproject && ogre init
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Initialization succeeded. |
| 1 | `ogre.toml` already exists in the current directory. |

---

### ogre generate

Generate brainfuck code for common patterns.

```
ogre generate helloworld [-o <file>]
ogre generate string <str> [-o <file>]
ogre generate loop <n> [-o <file>]
```

**Description:** Generates brainfuck source code for frequently needed patterns. Output goes to stdout unless `-o` is given, in which case it is written to a file.

#### ogre generate helloworld

Generate the classic "Hello World!" brainfuck program.

```
ogre generate helloworld [-o <file>]
```

**Flags:**

| Flag | Short | Description |
|------|-------|-------------|
| `-o <file>` | `-o` | Write output to a file instead of stdout. |

#### ogre generate string

Generate brainfuck code that prints an arbitrary ASCII string.

```
ogre generate string <str> [-o <file>]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `str` | The string to generate code for. Must be ASCII only. |

**Flags:**

| Flag | Short | Description |
|------|-------|-------------|
| `-o <file>` | `-o` | Write output to a file instead of stdout. |

#### ogre generate loop

Generate a brainfuck loop scaffold that executes exactly `n` times.

```
ogre generate loop <n> [-o <file>]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `n` | Number of loop iterations. For values up to 255, generates a simple counter loop. For larger values, uses multiplication to minimize code size. |

**Flags:**

| Flag | Short | Description |
|------|-------|-------------|
| `-o <file>` | `-o` | Write output to a file instead of stdout. |

**Examples:**

```bash
ogre generate helloworld
ogre generate string "Hello!" -o hello.bf
ogre generate loop 10
ogre generate loop 256 -o loop256.bf
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Code generated successfully. |
| 1 | Error (e.g., non-ASCII string). |

---

### ogre check

Validate brainfuck source files for correctness.

```
ogre check [file]
```

**Description:** Preprocesses the source (resolving all `@import`, `@fn`, and `@call` directives) and validates that:

- All brackets `[` and `]` are properly matched
- All `@import` paths resolve to existing files
- All `@call` references resolve to defined `@fn` names
- No import or call cycles exist

Prints `OK` for valid files and `ERROR` with details for invalid ones.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, checks all project include files. |

**Examples:**

```bash
ogre check hello.bf
ogre check                  # check all project files
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | All files are valid. |
| 1 | One or more files have errors. |

---

### ogre pack

Output fully preprocessed and expanded brainfuck.

```
ogre pack [file] [-o <output>] [--optimize]
```

**Description:** Preprocesses the source (expanding all `@import`, `@fn`, `@call` directives), strips all non-BF characters (comments), and outputs a single pure brainfuck string. Useful for sharing programs or feeding them to other tools.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml`. |

**Flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `-o <output>` | `-o` | stdout | Write the packed output to a file. If omitted, prints to stdout. When writing to a file, reports the file path and byte count. |
| `--optimize` | | off | Apply IR optimizations before outputting. This collapses repeated operators (`+++` to a single add), eliminates cancellations (`+-`), and converts clear idioms (`[-]`) and move-add patterns (`[->+<]`). The optimized output is still valid brainfuck. |

**Examples:**

```bash
ogre pack hello.bf
ogre pack hello.bf --optimize -o packed.bf
ogre pack                                      # pack project entry
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Packed successfully. |
| 1 | File not found or preprocessing error. |

---

### ogre bench

Benchmark a brainfuck program.

```
ogre bench [file] [--tape-size <n>]
```

**Description:** Runs the program with the optimized interpreter and reports execution statistics: instruction count, cells touched, output bytes, wall time, and throughput in MIPS (millions of instructions per second).

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml`. |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--tape-size <n>` | 30000 | Number of cells in the tape. |

**Output fields:**

| Field | Description |
|-------|-------------|
| Instructions executed | Total number of interpreter steps, formatted with comma separators. |
| Cells touched | Number of distinct tape cells that were read or written. |
| Output bytes | Number of bytes produced by `.` operations. |
| Wall time | Elapsed real time in milliseconds. |
| Throughput | Millions of instructions per second (shown when wall time > 0). |
| Tape size | (verbose only) The tape size used for the benchmark. |

**Examples:**

```bash
ogre bench hello.bf
ogre bench --tape-size 60000 big.bf
ogre bench                              # benchmark project entry
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Benchmark completed. |
| 1 | File not found, preprocessing error, or runtime error. |

---

### ogre doc

Generate documentation from `@doc` comments and `@fn` definitions.

```
ogre doc [file] [--stdlib] [-o <output>]
```

**Description:** Extracts `@doc` comments and `@fn` definitions from a brainfuck source file and generates Markdown documentation. Each function is listed with its doc comment (if any) and its body shown in a fenced code block. Functions are sorted alphabetically.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml` (unless `--stdlib` is used). |

**Flags:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--stdlib` | | off | Generate documentation for all built-in standard library modules instead of a user file. |
| `-o <output>` | `-o` | stdout | Write documentation to a file. If omitted, prints to stdout. |

**Doc comment syntax:**

```brainfuck
@doc Adds three to the current cell.
@doc Works on any cell value.
@fn add3 { +++ }
```

Multiple consecutive `@doc` lines before an `@fn` are concatenated into a single doc block.

**Examples:**

```bash
ogre doc hello.bf
ogre doc --stdlib
ogre doc hello.bf -o docs.md
ogre doc --stdlib -o stdlib-docs.md
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Documentation generated successfully. |
| 1 | File not found, no file or `--stdlib` provided, or preprocessing error. |

---

### ogre trace

Trace execution of a brainfuck program, printing tape state per instruction.

```
ogre trace [file] [--tape-size <n>] [--every <n>]
```

**Description:** Runs the program step by step and prints a trace line for each instruction (or every N instructions), showing the step number, operation, data pointer, current cell value, and a window of surrounding tape cells. The data pointer position is marked with `*` in the tape window.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `file` | Path to a `.bf` file. If omitted, uses the project entry from `ogre.toml`. |

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--tape-size <n>` | 30000 | Number of cells in the tape. |
| `--every <n>` | 1 | Print a trace line every `n` instructions. Setting this higher reduces output volume for long-running programs. |

**Trace output format:**

```
step=1      op=Add(3)              dp=0     cell[0]=3   | [*3 0 0 0 0]
step=2      op=Right(1)            dp=1     cell[1]=0   | [3 *0 0 0 0]
```

At completion, prints the total number of instructions executed.

**Examples:**

```bash
ogre trace hello.bf
ogre trace --every 100 hello.bf
ogre trace --tape-size 1000 hello.bf
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Trace completed. |
| 1 | File not found, preprocessing error, or runtime error. |

---

### ogre stdlib

Browse the built-in standard library.

```
ogre stdlib list
ogre stdlib show <module>
```

**Description:** The ogre standard library provides reusable `@fn` definitions that can be imported with `@import "std/<module>.bf"`. This command lets you explore available modules and view their source code.

#### ogre stdlib list

List all available standard library modules with a short description.

```
ogre stdlib list
```

**Available modules:**

| Module | Description |
|--------|-------------|
| `std/io.bf` | I/O utilities: `print_newline`, `print_space`, `read_char`, `print_char`, `print_zero` |
| `std/math.bf` | Arithmetic: `zero`, `inc`, `dec`, `inc10`, `double`, `add_to_next`, `move_right`, `move_left`, `copy_right` |
| `std/memory.bf` | Memory operations: `clear`, `clear2`, `clear3`, `swap`, `push_right`, `pull_left` |
| `std/ascii.bf` | ASCII character output: `print_A`, `print_B`, `print_exclaim`, `print_dash`, `print_colon` |
| `std/debug.bf` | Debugging helpers: `dump_cell`, `dump_and_newline`, `marker_start`, `marker_end` |

#### ogre stdlib show

Display the full source of a standard library module.

```
ogre stdlib show <module>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `module` | Module name without the `std/` prefix or `.bf` extension (e.g., `io`, `math`, `memory`, `ascii`, `debug`). |

**Examples:**

```bash
ogre stdlib list
ogre stdlib show io
ogre stdlib show math
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Module listed or shown successfully. |
| 1 | Unknown module name. |

---

## ogre.toml Reference

The project manifest file. All paths are resolved relative to the directory containing `ogre.toml`.

```toml
[project]
name = "myproject"             # required, must not be empty
version = "0.1.0"              # required, must not be empty
description = "My BF project"  # optional
author = "Alice"               # optional
entry = "src/main.bf"          # required, must end with .bf

[build]
include = [                    # files and directories that are part of the project
    "src/",                    # trailing slash = all .bf files in directory (non-recursive)
    "lib/utils.bf",            # specific file
    "src/**/*.bf",             # glob pattern (recursive)
    "src/*.bf",                # glob pattern (non-recursive)
]
tape_size = 30000              # optional, default tape size for run/start/debug

[[tests]]
name = "Basic"                 # optional display name
file = "tests/basic.json"      # required, must end with .json

[[tests]]
name = "Advanced"
file = "tests/advanced.json"

[dependencies]
mylib = { path = "../mylib" }  # path-based dependency (must have ogre.toml)
```

**Validation rules:**
- `project.name` must not be empty or whitespace-only
- `project.version` must not be empty
- `project.entry` must end with `.bf`
- Each `tests[].file` must end with `.json`
- `build.tape_size` must be greater than 0 if specified
- Each dependency must have at least a `path` or `version` field

**Include path resolution:**
- Paths ending with `/` collect all `.bf` files directly inside the directory (non-recursive)
- Paths containing `*` or `?` are expanded as glob patterns
- All other paths are treated as specific file references

**Dependencies:**
- Path-based dependencies are resolved relative to the project root
- Each dependency directory must contain its own `ogre.toml`
- All `@fn` definitions from dependency include files and entry points are collected and made available to the main project
- Dependencies are resolved recursively (a dependency can have its own dependencies)
