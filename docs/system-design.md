# ogre System Design Document

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Architecture](#2-architecture)
3. [Module Architecture](#3-module-architecture)
4. [Data Flow](#4-data-flow)
5. [Error Handling Strategy](#5-error-handling-strategy)
6. [Standard Library](#6-standard-library)
7. [Dependency Management](#7-dependency-management)
8. [Testing Strategy](#8-testing-strategy)
9. [Performance Considerations](#9-performance-considerations)
10. [Key Design Principles](#10-key-design-principles)

---

## 1. System Overview

### What ogre Is

ogre is a Cargo-like all-in-one brainfuck (and brainfunct) toolchain written in Rust. It provides a single CLI binary that covers the full development lifecycle for brainfuck programs: interpreting, compiling to native binaries, compiling to WebAssembly, formatting, static analysis, structured testing, code generation, an interactive REPL, a GDB-style debugger, execution tracing, benchmarking, documentation generation, and project management.

### The brainfunct Dialect

Standard brainfuck operates on an array of 30,000 byte cells with 8 instructions (`+-><.,[]`). The brainfunct dialect extends this with named functions via compile-time macros:

- `@fn name { body }` -- defines a named macro
- `@call name` -- inlines the macro body at the call site
- `@import "path"` -- imports function definitions from another file
- `@const NAME value` -- defines a named numeric constant
- `@use NAME` -- expands to `value` `+` characters
- `@doc description` -- attaches documentation to the following `@fn`

All directives are resolved at compile time by the preprocessor. The final output fed to the interpreter or compiler is pure brainfuck with no directives remaining.

### Key Goals

1. **Unified toolchain** -- a single binary replaces disparate scripts and ad-hoc tools
2. **Project management** -- `ogre.toml` manifests with dependency resolution, test suites, and build configuration
3. **Performance** -- an intermediate representation (IR) with optimization passes produces faster interpreted and compiled programs
4. **Developer experience** -- colored output, helpful error messages with source locations, watch mode, REPL with history, and a debugger with source mapping
5. **Correctness** -- comprehensive unit tests, integration tests, CLI tests, and criterion benchmarks

### Subcommand Summary

| Subcommand | Purpose |
|---|---|
| `run` | Preprocess and interpret a brainfuck file |
| `compile` | Preprocess, compile to C, invoke gcc/clang (or compile to WASM) |
| `build` | Project-only: compile using `ogre.toml` configuration |
| `start` | Interactive REPL with rustyline, project-aware |
| `debug` | GDB-style interactive debugger with source mapping |
| `format` | In-place source formatter with `--check` and `--diff` modes |
| `analyse` | Static analysis: bracket validation, complexity metrics, pattern detection |
| `test` | JSON-driven structured test runner with timeouts and regex matching |
| `check` | Validate brackets, imports, and calls (CI-friendly exit codes) |
| `pack` | Output fully preprocessed and expanded single `.bf` file |
| `bench` | Benchmark: instruction count, wall time, cells touched, MIPS |
| `trace` | Execution tracing with tape snapshots at configurable intervals |
| `doc` | Generate documentation from `@doc` comments and `@fn` definitions |
| `new` | Scaffold a new project directory |
| `init` | Initialize `ogre.toml` in the current directory |
| `generate` | Code generation for common patterns (hello world, string, loop) |
| `stdlib` | Browse the built-in standard library |

---

## 2. Architecture

### High-Level Architecture

```
                          +------------------+
                          |    CLI (clap)    |
                          |    main.rs       |
                          +--------+---------+
                                   |
                    +--------------+--------------+
                    |                             |
              +-----+------+              +------+------+
              | project.rs |              | verbosity.rs|
              | ogre.toml  |              | Quiet/Normal|
              | parsing &  |              | /Verbose    |
              | validation |              +-------------+
              +-----+------+
                    |
    +---------------+----------------+
    |               |                |
    v               v                v
+--------+   +----------+   +------------+
| error  |   | preproc  |   | directive  |
| .rs    |   | essor    |   | _parser.rs |
|OgreErr |   | 2-pass   |   | tokenizer  |
+--------+   +----+-----+   +------------+
                  |
          +-------+--------+
          |                |
          v                v
    +-----------+   +------------+
    | ir.rs     |   | source_map |
    | Op enum   |   | .rs        |
    | Program   |   | SourceLoc  |
    | optimize  |   | SourceMap  |
    +-----+-----+   +-----+------+
          |                |
+---------+------+---------+---------+
|         |      |         |         |
v         v      v         v         v
+------+ +----+ +------+ +------+ +------+
| run  | |comp| |format| |debug | |start |
| .rs  | |ile | |.rs   | |.rs   | |.rs   |
|      | |.rs | |      | |      | |REPL  |
+------+ +----+ +------+ +------+ +------+
          |
     +----+-----+
     |           |
     v           v
 +-------+  +--------+
 | C gen |  |WAT gen |
 | gcc   |  |wat2wasm|
 +-------+  +--------+

Other modes: analyse, test_runner, check, pack, bench,
             trace, doc, new, init, generate, stdlib
```

### Data Flow Overview

```
Source File (.bf)
       |
       v
  Preprocessor (2-pass)
  - Pass 1: Collect @fn, resolve @import, expand @const/@use
  - Pass 2: Expand @call, detect cycles
       |
       v
  Expanded Pure BF String
       |
       +------+------+------+------+
       |      |      |      |      |
       v      v      v      v      v
     IR    Interp  Compile Format  Analyse
   Parse   eter    (C/WAT)
       |
       v
  Program { ops: Vec<Op> }
       |
       v
  Optimization Passes
  - Clear idiom ([-] -> Clear)
  - Move idiom ([->+<] -> MoveAdd)
  - Cancellation (+- -> noop)
  - Dead store (Clear + Add -> Add)
  - Jump reindexing
       |
       v
  Optimized Program
       |
       +------+------+
       |      |      |
       v      v      v
    Interp  C gen  WAT gen
    eter           WASM
```

### Module Dependency Graph

The crate exposes both a library (`src/lib.rs`) and a binary (`src/main.rs`). The library re-exports four public modules:

```rust
pub mod error;
pub mod modes;
pub mod project;
pub mod verbosity;
```

The binary depends on the library and all mode modules. Mode modules form a layered dependency:

- **Foundation layer**: `error`, `verbosity`, `directive_parser`, `source_map`, `ir`
- **Core layer**: `preprocess` (depends on `directive_parser`, `source_map`, `error`), `interpreter` (depends on `ir`, `source_map`, `error`)
- **Command layer**: all mode modules (`run`, `compile`, `format`, `debug`, etc.) depend on the core layer

---

## 3. Module Architecture

### 3.1 main.rs -- CLI Dispatch via clap Derive

`main.rs` is the entry point. It defines the CLI structure using clap's derive API and dispatches to mode-specific functions.

**Responsibilities:**
- Define the `Cli` struct with global flags (`--no-color`, `--quiet`, `--verbose`)
- Define the `Commands` enum with one variant per subcommand
- Define per-subcommand argument structs (`RunArgs`, `CompileArgs`, etc.)
- Parse the CLI, determine `Verbosity`, and dispatch to the appropriate mode function
- Handle the `require_project()` pattern: when no file argument is provided, walk CWD upward looking for `ogre.toml`
- Handle the `NO_COLOR` environment variable and `--no-color` flag via the `colored` crate

**Key pattern -- project fallback:**

```rust
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Run(args) => {
            let file = match args.file {
                Some(f) => PathBuf::from(f),
                None => {
                    let (proj, base) = require_project()?;
                    let entry = proj.entry_path(&base);
                    // ... use entry as the file
                }
            };
            // ...
        }
    }
}
```

Every file-accepting subcommand follows this pattern: accept an optional file argument, fall back to the project entry if absent.

### 3.2 error.rs -- OgreError Typed Error Enum with thiserror

**Responsibilities:**
- Define a comprehensive error enum that covers all known failure modes
- Integrate with `anyhow` for ergonomic error propagation

```rust
#[derive(Error, Debug)]
pub enum OgreError {
    #[error("unmatched `]`")]
    UnmatchedCloseBracket,

    #[error("unmatched `[` at op index {0}")]
    UnmatchedOpenBracket(usize),

    #[error("data pointer out of bounds ({0})")]
    TapeOverflow(String),

    #[error("cycle detected: {0}")]
    CycleDetected(String),

    #[error("import cycle detected: {0}")]
    ImportCycle(String),

    #[error("unknown function: {0}")]
    UnknownFunction(String),

    #[error("unknown directive: @{0}")]
    UnknownDirective(String),

    #[error("unknown standard library module: {0}")]
    UnknownStdModule(String),

    #[error("no C compiler found. Install gcc, clang, or ensure 'cc' is available on PATH")]
    CompilerNotFound,

    #[error("compilation failed: {0}")]
    CompilationFailed(String),

    #[error("invalid project: {0}")]
    InvalidProject(String),

    #[error("timeout: instruction limit of {0} reached")]
    Timeout(u64),

    #[error("{0}")]
    Other(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Each variant is used by a specific subsystem. `UnmatchedCloseBracket` and `UnmatchedOpenBracket` come from IR parsing. `CycleDetected` and `ImportCycle` come from the preprocessor. `CompilerNotFound` comes from the native compiler. This design means callers can match on specific error types when needed while still using `anyhow::Result` for general propagation.

### 3.3 verbosity.rs -- Quiet/Normal/Verbose Enum

**Responsibilities:**
- Provide a three-level verbosity control used by all commands
- Determined by the global `--quiet` and `--verbose` flags

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,    // Only errors and requested data
    Normal,   // Default output
    Verbose,  // Extra detail (instruction counts, timing, per-file info)
}
```

The `Verbosity` value is passed down to mode functions. Functions use `is_quiet()` to suppress informational output and `is_verbose()` to add extra detail. This avoids boolean flag proliferation.

### 3.4 project.rs -- ogre.toml Parsing, Validation, Dependency Management

**Responsibilities:**
- Define the TOML schema as Rust structs with serde Deserialize
- Validate the parsed configuration (non-empty name, `.bf` entry, non-zero tape size, valid test file extensions, dependency completeness)
- Walk the filesystem upward to find `ogre.toml`
- Resolve include files (directory entries, glob patterns, explicit file paths)
- Resolve path-based dependencies and recursively collect `@fn` definitions

**Core types:**

```rust
pub struct OgreProject {
    pub project: ProjectMeta,
    pub build: Option<BuildConfig>,
    pub tests: Vec<TestFileRef>,
    pub dependencies: HashMap<String, Dependency>,
}

pub struct ProjectMeta {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub entry: String,
}

pub struct BuildConfig {
    pub include: Vec<String>,
    pub tape_size: Option<usize>,
}

pub struct Dependency {
    pub path: Option<String>,
    pub version: Option<String>,  // Reserved for future registry support
}
```

**Include resolution** supports three forms:
1. Directory entries (`"src/"`) -- collect all `.bf` files non-recursively
2. Glob patterns (`"src/**/*.bf"`) -- expand using the `glob` crate
3. Explicit files (`"lib/utils.bf"`) -- include directly

**Dependency resolution** is recursive. `collect_dependency_functions()` walks each dependency, loads its `ogre.toml`, collects `@fn` definitions from its include files and entry file, then recursively processes the dependency's own dependencies. A `HashSet` prevents processing the same dependency twice.

### 3.5 modes/ir.rs -- Op Enum, Program Struct, Optimization Passes

This is the intermediate representation that sits between raw brainfuck source and execution/compilation. It is the performance backbone of the entire system.

**The Op enum:**

```rust
pub enum Op {
    Add(u8),              // Increment current cell by n (wrapping)
    Sub(u8),              // Decrement current cell by n (wrapping)
    Right(usize),         // Move data pointer right by n
    Left(usize),          // Move data pointer left by n
    Output,               // Print current cell
    Input,                // Read into current cell
    JumpIfZero(usize),    // If cell == 0, jump to target
    JumpIfNonZero(usize), // If cell != 0, jump to target
    Clear,                // Set current cell to 0 (optimized [-])
    MoveAdd(isize),       // tape[dp+offset] += tape[dp]; tape[dp] = 0
    MoveSub(isize),       // tape[dp+offset] -= tape[dp]; tape[dp] = 0
}
```

**Parsing (`Program::from_source`):**

The parser performs two transformations simultaneously during a single pass:

1. **Run-length encoding**: consecutive identical operators are collapsed into a single op with a count (e.g., `+++` becomes `Add(3)`, `>>>` becomes `Right(3)`)
2. **Bracket pairing**: `[` and `]` are matched using a stack, with each storing the index of its partner for O(1) jumps at runtime

```rust
'+' => {
    if let Some(Op::Add(n)) = ops.last_mut() {
        *n = n.wrapping_add(1);
    } else {
        ops.push(Op::Add(1));
    }
}
```

Non-BF characters are silently ignored (treated as comments).

**Optimization passes (`Program::optimize`):**

The optimizer runs four passes in sequence, then reindexes all jump targets:

1. **Clear idiom** (`optimize_clear_idiom`): Replaces the pattern `JumpIfZero`, `Sub(1)`, `JumpIfNonZero` (i.e., `[-]`) with the single `Clear` op. This eliminates the loop entirely.

2. **Move idiom** (`optimize_move_idiom`): Detects patterns like `[->+<]` (move-add forward), `[-<+>]` (move-add backward), `[->-<]` (move-sub forward), and their backward variants. Replaces the 6-op loop with a single `MoveAdd(offset)` or `MoveSub(offset)`.

3. **Cancellation** (`optimize_cancellation`): Merges adjacent opposite operations. `Add(3)` followed by `Sub(2)` becomes `Add(1)`. `Right(3)` followed by `Left(1)` becomes `Right(2)`. Equal-magnitude opposites cancel completely. This pass runs in a loop until no more cancellations are found.

4. **Dead store** (`optimize_dead_store`): Removes `Clear` operations that are immediately followed by `Add(n)`, since the add will overwrite the cleared cell anyway.

5. **Jump reindexing** (`reindex_jumps`): After ops are inserted or removed by optimization passes, all `JumpIfZero` and `JumpIfNonZero` targets must be recalculated. This pass walks the ops array with a bracket stack and patches all targets.

**Decompilation (`Program::to_bf_string`):**

The IR can be converted back to brainfuck source. This is used by `ogre pack --optimize` to emit optimized BF. The `MoveAdd` and `MoveSub` ops are expanded back to their loop forms (e.g., `MoveAdd(2)` becomes `[->>+<<]`).

### 3.6 modes/interpreter.rs -- Tape, Step/Run, Streaming I/O, Instruction Counting

**Responsibilities:**
- Execute IR programs on a simulated tape
- Expose stepped execution for the debugger and REPL
- Track metrics (instruction count, cells touched) for benchmarking
- Support streaming output (flush each `.` immediately) and buffered output
- Support both pre-loaded input and live stdin
- Provide source-map-aware error messages

**Core state:**

```rust
pub struct Interpreter {
    tape: Vec<u8>,           // The BF tape (default 30,000 cells)
    data_ptr: usize,         // Current cell index
    program: Program,        // The IR program
    ip: usize,               // Instruction pointer (index into ops)
    output: Vec<u8>,         // Buffered output
    input: Vec<u8>,          // Pre-loaded input bytes
    input_ptr: usize,        // Current position in input buffer
    live_stdin: bool,        // Fall back to stdin after input buffer
    streaming: bool,         // Flush output immediately on '.'
    instruction_count: u64,  // Total ops executed
    cells_touched: Vec<bool>,// Track which cells were written
    source_map: Option<SourceMap>,
    op_to_char: Vec<usize>,  // Maps op indices to source positions
}
```

**The step() method:**

The `step()` method executes exactly one IR operation and returns `Ok(true)` if more instructions remain, `Ok(false)` if the program has finished, or `Err` on a runtime error (tape overflow).

Key execution semantics:
- `Add(n)` and `Sub(n)` use `wrapping_add` / `wrapping_sub` for u8 overflow semantics (standard BF behavior: 255 + 1 = 0)
- `Right(n)` and `Left(n)` check bounds and return `OgreError::TapeOverflow` with source location context
- `JumpIfZero(target)` skips to `target + 1` if the current cell is zero
- `JumpIfNonZero(target)` loops back to `target + 1` if the current cell is non-zero
- `Clear` directly sets the current cell to 0
- `MoveAdd(offset)` performs `tape[dp + offset] += tape[dp]; tape[dp] = 0` as a single atomic operation

**Constructor variants:**

The interpreter provides multiple constructors to support different use cases:
- `new(source)` -- basic construction with empty input
- `with_input(source, input)` -- pre-loaded input for testing
- `with_live_stdin(source)` -- for `run` mode (reads from real stdin)
- `with_tape_size(source, size)` -- custom tape size
- `new_optimized(source)` -- parses and runs `program.optimize()` before execution
- `new_optimized_with_input(source, input)` -- optimized with test input

**The feed() method:**

Used by the REPL to incrementally add code. It concatenates the new source with all previously entered source, re-parses the entire program, and updates the op-to-char mapping. This means the REPL maintains a coherent program state across multiple inputs.

### 3.7 modes/preprocess.rs -- Two-Pass Preprocessor

**Responsibilities:**
- Resolve `@import` directives (file and stdlib)
- Collect `@fn` definitions into a HashMap
- Expand `@call` directives by inlining function bodies
- Expand `@const` and `@use` directives
- Collect `@doc` comments and attach them to functions
- Detect and report import cycles and call cycles
- Optionally build a `SourceMap` for debugger integration

**Two-pass design:**

**Pass 1 -- Collect (`collect` method):**
- Walks the source character by character
- On `@import "path"`: reads the imported file, recursively calls `collect` on it (collecting function definitions), discards top-level code from imports (with a warning)
- On `@import "std/module"`: loads the stdlib module from embedded `include_str!` data
- On `@fn name { body }`: stores the name-body pair in `self.functions`
- On `@call name`: preserves the `@call name` marker in the top-level output string (for Pass 2)
- On `@const NAME value`: stores the constant in `self.constants`
- On `@use NAME`: expands to `value` `+` characters inline
- On `@doc text`: accumulates documentation lines in `pending_doc`, attaches them to the next `@fn`
- Import cycle detection: a `HashSet<PathBuf>` of imported files; re-importing the same canonical path is an error

**Pass 2 -- Expand (`expand` method):**
- Walks the output of Pass 1
- On `@call name`: looks up the function body, recursively expands it (functions can call other functions), and inlines the result
- Call cycle detection: a `Vec<String>` call stack; if a name appears twice in the stack, it is a cycle
- On `@use NAME`: expands constants (they may appear in function bodies)

**Source-map-aware variants:**

The preprocessor has parallel `collect_with_tracking` and `expand_with_tracking` methods that build a `SourceMap` during processing. Each character in the expanded output gets a `SourceLocation` recording its origin file, line, column, and optionally the `@fn` name it was expanded from.

**Dependency integration:**

`process_file_with_deps` pre-loads a `HashMap<String, String>` of function definitions from project dependencies before running the collect/expand passes. This allows functions from dependencies to be available for `@call` expansion without explicit `@import`.

### 3.8 modes/directive_parser.rs -- Shared Tokenizer

**Responsibilities:**
- Provide reusable parsing primitives used by both the preprocessor and the formatter

Four functions:

```rust
fn take_identifier(chars: &[char], i: &mut usize) -> String
fn skip_spaces(chars: &[char], i: &mut usize)      // horizontal only
fn skip_whitespace(chars: &[char], i: &mut usize)   // including newlines
fn take_quoted_string(chars: &[char], i: &mut usize) -> Result<String>
fn take_brace_body(chars: &[char], i: &mut usize) -> Result<String>
```

`take_brace_body` handles nested braces: it tracks brace depth and returns everything between the opening `{` (already consumed by caller) and the matching `}`. This is critical for `@fn` bodies that may contain nested brace comments or other structured content.

### 3.9 modes/source_map.rs -- SourceLocation, SourceMap for Debug/Errors

**Responsibilities:**
- Map each character in the expanded BF output back to its origin
- Bridge between IR op indices and character positions (since IR collapses consecutive characters)

**SourceLocation:**

```rust
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,        // 1-based
    pub column: usize,      // 1-based
    pub function: Option<String>,  // @fn name if inside a function expansion
}
```

Display format: `src/main.bf:5:12` or `src/greet.bf:3:5 (@fn greet)`.

**SourceMap:**

A simple `Vec<SourceLocation>` indexed by character position in the expanded output. The `lookup_op` method bridges from IR op indices to source locations using the `op_to_char` mapping built by `build_op_to_char_map`.

**build_op_to_char_map:**

Since the IR collapses e.g. `+++` into a single `Add(3)` op, we need to know which character position corresponds to each op. This function walks the source, detects when consecutive identical BF characters would be collapsed, and records the position of the first character of each op.

### 3.10 modes/compile.rs -- BF to C to gcc/clang with -O2

**Responsibilities:**
- Generate C source code from an optimized IR program
- Find a C compiler on the system (`cc`, `gcc`, `clang`)
- Invoke the compiler with `-O2` optimization
- Manage intermediate `.c` file lifecycle (temp dir or `--keep`)

**C code generation:**

The generated C uses `unsigned char` for the tape (matching BF's 8-bit cell semantics), `memset` initialization, and pointer arithmetic:

```c
#include <stdio.h>
#include <string.h>
int main() {
    unsigned char array[30000];
    memset(array, 0, sizeof(array));
    unsigned char *ptr = array;
    // ... ops ...
    return 0;
}
```

Each IR op maps directly to C:
- `Add(1)` -> `(*ptr)++;`, `Add(n)` -> `*ptr += n;`
- `Right(1)` -> `ptr++;`, `Right(n)` -> `ptr += n;`
- `JumpIfZero` -> `while (*ptr) {`
- `Clear` -> `*ptr = 0;`
- `MoveAdd(offset)` -> `*(ptr + offset) += *ptr; *ptr = 0;`

The IR is optimized before C generation, so the generated C benefits from clear-idiom elimination, move-idiom recognition, and cancellation passes.

**Compiler discovery** probes for `cc`, then `gcc`, then `clang`, using `--version` to test availability. Returns `OgreError::CompilerNotFound` if none are available.

### 3.11 modes/compile_wasm.rs -- BF to WAT to WASM via wat2wasm, WASI I/O

**Responsibilities:**
- Generate WAT (WebAssembly Text Format) from an optimized IR program
- Use WASI imports for I/O (`fd_write` for output, `fd_read` for input)
- Optionally invoke `wat2wasm` to produce binary `.wasm`

**Memory layout:**

```
[0 .. tape_size-1]        = BF tape
[tape_size .. tape_size+7] = iov buffer (ptr + len) for fd_write/fd_read
[tape_size+8 .. tape_size+11] = nwritten/nread result
```

The data pointer is stored in a WASM global `$dp`. Tape access uses `i32.load8_u` and `i32.store8` for 8-bit operations with `i32.and ... 255` masking to ensure wrapping behavior.

Loops use WASM's `block`/`loop`/`br_if` structured control flow:

```wat
(block $B
  (loop $L
    (br_if $B (i32.eqz (i32.load8_u (global.get $dp))))
    ;; loop body
    (br_if $L (i32.load8_u (global.get $dp)))
  )
)
```

If `wat2wasm` is available on PATH, ogre compiles the `.wat` to binary `.wasm` and removes the intermediate `.wat`. Otherwise it outputs the `.wat` file with an informational message.

### 3.12 modes/run.rs -- Run, Watch Mode with notify Crate

**Responsibilities:**
- Preprocess and interpret a brainfuck file with streaming output
- Implement watch mode: re-run whenever the source file changes

**Normal mode** is a thin wrapper:

```rust
pub fn run_file_with_tape_size(path: &Path, tape_size: usize) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let mut interp = Interpreter::with_live_stdin_and_tape_size(&expanded, tape_size)?;
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}
```

**Watch mode** uses the `notify` crate (version 6) with the recommended watcher. It watches the file's parent directory (more reliable than watching the file directly) in non-recursive mode. Events are debounced with a 100ms sleep to coalesce rapid filesystem updates. On each change, the terminal is cleared and the program is re-run. Errors during preprocessing or execution are printed rather than propagated, so a syntax error in the source file does not kill the watcher.

### 3.13 modes/format.rs -- In-Place Formatter, --check, --diff with similar Crate

**Responsibilities:**
- Format brainfuck source files with configurable indentation, line width, and operator grouping
- Preserve `@import`, `@fn`, and `@call` directives on their own lines
- Format `@fn` bodies with indentation
- Support `--check` mode (exit 1 if unformatted) for CI
- Support `--diff` mode (show unified diff without modifying files)

**Formatting rules:**
- `[` starts a new line and increases indent depth
- `]` decreases indent depth and starts a new line
- Lines wrap at `--linewidth` (default 80)
- Consecutive identical operators are grouped with spaces every `--grouping` characters (default 5): `+++++ +++++`
- Non-BF characters are stripped unless `--preserve-comments` is set
- Directives are placed on their own lines, verbatim

**Segment-based parsing:**

The formatter uses a segment parser that divides source into `BF`, `Directive`, and `FnDef` segments. Each segment type is formatted differently:
- `BF` segments go through the core BF formatter
- `Directive` segments are emitted verbatim on their own line
- `FnDef` segments format their body through the BF formatter with additional indentation

**Diff generation** uses the `similar` crate's `TextDiff::from_lines` with a 3-line context radius. Output is colorized using the `colored` crate (red for deletions, green for insertions, cyan for hunk headers).

### 3.14 modes/analyse.rs -- Static Analysis, Complexity Metrics, Pattern Detection

**Responsibilities:**
- Validate bracket matching
- Count I/O operations
- Track net data pointer offset (when statically determinable)
- Detect patterns: clear idioms (`[-]`), operator cancellations (`+-`), dead code after infinite loops
- Compute complexity metrics: max loop depth, total ops, optimized ops, optimization reduction percentage
- Support `--in-place` mode to embed analysis as comments in the source file

**Analysis report structure:**

```rust
pub struct AnalysisReport {
    pub bracket_errors: Vec<String>,
    pub total_inputs: usize,
    pub total_outputs: usize,
    pub ptr_end_offset: Option<i64>,  // None if loops prevent analysis
    pub has_clear_idiom: bool,
    pub clear_idiom_count: usize,
    pub has_dead_code: bool,
    pub dead_code_positions: Vec<usize>,
    pub has_cancellation: bool,
    pub cancellation_positions: Vec<usize>,
    pub unbalanced_pointer: bool,
    pub max_loop_depth: usize,
    pub total_ops: usize,
    pub optimized_ops: usize,
}
```

The analyser operates on the preprocessed (expanded) BF, so it sees the fully inlined program.

### 3.15 modes/debug.rs -- GDB-Style Debugger with Source Mapping

**Responsibilities:**
- Provide a GDB-style interactive debugger
- Execute brainfuck programs one instruction at a time
- Support breakpoints, stepping, continuing, jumping, peeking at memory
- Display source location context (file, line, column, function name) via source map

**Debugger state:**

```rust
pub struct Debugger {
    interp: Interpreter,
    breakpoints: HashSet<usize>,
    source_map: Option<SourceMap>,
    op_to_char: Vec<usize>,
}
```

**Command set:**

| Command | Description |
|---|---|
| `step [n]` | Execute 1 or n instructions |
| `continue` / `c` | Run until breakpoint or end |
| `breakpoint <n>` | Set breakpoint at op index n |
| `breakpoint list` | List all breakpoints with source locations |
| `breakpoint delete <n>` | Remove a breakpoint |
| `jump <n>` | Move code pointer without executing |
| `peek [n]` | Show memory window around pointer or cell n |
| `show instruction [n]` | Show current or nth instruction with context |
| `show memory` | Dump memory cells around pointer |
| `where` | Show current source location |
| `exit` / `quit` / `q` | Exit the debugger |

After every pause, the debugger prints status showing: instruction pointer, current op description (e.g., `Add(3)`), data pointer, current cell value, and source location if available. It also shows a tape memory window centered on the data pointer.

**Source mapping integration:**

When a file is loaded for debugging, `Preprocessor::process_file_with_map` is used instead of the regular `process_file`. This produces both the expanded BF and a `SourceMap`. The debugger uses this to display locations like `src/main.bf:5:12 (@fn greet)` after each step.

### 3.16 modes/start.rs -- REPL with rustyline, Project-Aware

**Responsibilities:**
- Provide an interactive REPL where users type BF code and see tape state after each evaluation
- Support command-line editing, history (persisted to `~/.ogre_history`), and Emacs key bindings via rustyline
- Pre-load project `@fn` definitions (including from dependencies) when run inside a project
- Support special commands: `:reset`, `:load <file>`, `:save <file>`, `:functions`, `:peek`, `:dump`, `:help`

**REPL loop:**

1. Read a line with rustyline
2. If it starts with `:`, handle as a meta-command
3. Otherwise, preprocess the input (expand `@call` against loaded functions)
4. Feed the expanded code to the interpreter via `interp.feed()`
5. Run the interpreter to completion
6. Print any output
7. Print the tape memory window

**Project awareness:**

When `start_repl_project` is called, it loads all `@fn` definitions from the project's entry file, all include files, and all dependency functions. These are displayed in the welcome message and available for `@call` expansion in REPL input.

### 3.17 modes/test_runner.rs -- JSON Test Runner, Timeouts, Regex

**Responsibilities:**
- Parse JSON test files
- Run each test case: preprocess the BF file, create an interpreter with the specified input, run with an instruction limit, compare output
- Support exact string matching and regex matching (`output_regex` field)
- Support per-test timeout overrides (default: 10 million instructions)
- Report pass/fail with colored output (dots for compact mode, per-test lines for verbose mode)

**Test case schema:**

```rust
pub struct TestCase {
    pub name: String,
    pub brainfuck: String,         // Path to .bf file
    pub input: String,
    pub output: String,
    pub output_regex: Option<String>,
    pub timeout: Option<u64>,
}
```

**Timeout mechanism:** `run_with_limit(max_instructions)` returns `Ok(false)` if the limit is reached before the program completes. This is reported as a `TIMEOUT` status distinct from `FAIL`.

**Project test suites:** `run_project_tests_ex` iterates over the `[[tests]]` array in `ogre.toml`, running each test file. BF paths within test cases are resolved relative to the project base directory.

### 3.18 modes/trace.rs -- Execution Tracing with Tape Snapshots

**Responsibilities:**
- Execute a brainfuck program with tracing output
- Print the tape state after every N instructions (configurable via `--every`)

Each trace line shows:

```
step=1      op=Add(3)             dp=0     cell[0]=3   | [*3 0 0 0 0 0 0 0 0]
```

The trace prints a window of 9 cells centered on the data pointer, with the current cell marked by `*`. This mode is useful for understanding program behavior without the interactive overhead of the debugger.

### 3.19 modes/check.rs -- Validate Brackets, Imports, and Calls

**Responsibilities:**
- Preprocess a file (catching import/call errors)
- Validate bracket matching via IR parsing
- Report results with colored OK/ERROR output
- Exit with code 1 if any errors are found (CI-friendly)

```rust
pub struct CheckResult {
    pub brackets_ok: bool,
    pub preprocess_ok: bool,
    pub errors: Vec<String>,
}
```

For projects with dependencies, `check_file_with_deps` pre-loads dependency functions so that `@call` directives referencing dependency functions are validated correctly.

### 3.20 modes/pack.rs -- Preprocessed Output

**Responsibilities:**
- Preprocess a file and output the fully expanded pure BF
- Optionally optimize via IR passes before output
- Strip all non-BF characters (comments) from the output
- Write to a file or stdout

The `--optimize` flag runs the IR optimizer and then converts back to BF using `program.to_bf_string()`. This produces shorter, more efficient BF that is semantically equivalent to the original.

### 3.21 modes/bench.rs -- Benchmarking

**Responsibilities:**
- Run a program with the optimized interpreter and measure performance
- Report instruction count, cells touched, output bytes, wall time, and throughput in MIPS

```rust
pub struct BenchResult {
    pub instruction_count: u64,
    pub cells_touched: usize,
    pub elapsed_ms: f64,
    pub output_bytes: usize,
}
```

Uses `Interpreter::new_optimized` for maximum execution speed during benchmarking. Wall time is measured with `std::time::Instant`. Throughput is computed as `instruction_count / elapsed_ms / 1000` to give millions of instructions per second.

### 3.22 modes/init.rs -- Initialize Project in Current Directory

**Responsibilities:**
- Create `ogre.toml` in the current directory (fail if it already exists)
- Derive the project name from the current directory name
- Create `src/` and `tests/` directories if they do not exist
- Create `src/main.bf` and `tests/basic.json` templates if they do not exist

Unlike `new`, which creates a new directory, `init` initializes an existing directory as an ogre project.

### 3.23 modes/new.rs -- Project Scaffolding

**Responsibilities:**
- Create a new project directory with the standard layout
- Generate `ogre.toml`, `src/main.bf`, and `tests/basic.json`
- Optionally include stdlib imports with `--with-std`

The generated project structure:

```
<name>/
  ogre.toml
  src/
    main.bf          (@fn main {} starter)
  tests/
    basic.json       (template test case)
```

### 3.24 modes/generate.rs -- Code Generation

**Responsibilities:**
- Generate brainfuck code for common patterns
- `helloworld`: the classic hello world program
- `string <str>`: generates BF that prints an arbitrary ASCII string (uses differential encoding: tracks the current cell value and generates the minimal add/subtract to reach each character)
- `loop <n>`: generates a loop scaffold that runs exactly n times (uses multiplication for values > 255)

### 3.25 modes/doc.rs -- Documentation Generation

**Responsibilities:**
- Generate markdown documentation from `@doc` comments and `@fn` definitions
- Support both user files and the built-in standard library
- Output to stdout or a file

Uses `Preprocessor::process_file_with_docs` to get the function map and documentation map. Functions are listed alphabetically, each with its `@doc` text and body in a fenced code block.

### 3.26 modes/stdlib.rs -- Standard Library Browser

**Responsibilities:**
- `list`: show all available stdlib modules with descriptions
- `show <module>`: print the source of a stdlib module

This is a thin interface over `preprocess::stdlib_modules()` and `preprocess::get_stdlib_module()`.

---

## 4. Data Flow

### 4.1 Source to Execution (ogre run)

```
hello.bf (with @fn/@call/@import directives)
    |
    v
Preprocessor::process_file("hello.bf")
    |
    +-- Pass 1: Collect
    |   - Read "hello.bf"
    |   - Encounter @import "std/io.bf" -> load embedded stdlib, collect functions
    |   - Encounter @fn greet { ... } -> store in functions HashMap
    |   - Encounter @call greet -> preserve as marker in top_level string
    |   - Encounter @const X 65 -> store in constants HashMap
    |   - Encounter @use X -> emit 65 '+' characters
    |
    +-- Pass 2: Expand
    |   - Walk top_level string
    |   - Replace @call greet with recursively-expanded body
    |   - Replace @use X with 65 '+' characters
    |   - Detect cycles via call stack
    |
    v
Expanded pure BF string (no directives)
    |
    v
Interpreter::with_live_stdin_and_tape_size(expanded, 30000)
    |
    +-- Program::from_source(expanded)
    |   - Run-length encode: +++ -> Add(3)
    |   - Pair brackets: [..] -> JumpIfZero/JumpIfNonZero with targets
    |
    v
Interpreter { tape, program, ip, dp, ... }
    |
    v
interp.set_streaming(true)  // flush output on each '.'
interp.run()                 // execute until ip >= program.ops.len()
    |
    v
Output to stdout
```

### 4.2 Source to Native Binary (ogre compile)

```
hello.bf
    |
    v
Preprocessor::process_file("hello.bf")
    |
    v
Expanded pure BF string
    |
    v
Program::from_source(expanded)
    |
    v
program.optimize()
    |  - Clear idiom: [-] -> Clear
    |  - Move idiom: [->+<] -> MoveAdd(1)
    |  - Cancellation: +- -> removed
    |  - Dead store: Clear + Add(n) -> Add(n)
    |  - Reindex jumps
    |
    v
generate_c_from_program(&program, tape_size)
    |  - Emit #include <stdio.h>
    |  - Emit unsigned char array[30000]; memset(...)
    |  - Map each Op to C statement
    |  - Clear -> *ptr = 0;
    |  - MoveAdd(n) -> *(ptr + n) += *ptr; *ptr = 0;
    |
    v
C source string
    |
    v
Write to temp file (or keep with -k)
    |
    v
find_c_compiler() -> "gcc" | "clang" | "cc"
    |
    v
gcc hello.c -o hello -O2
    |
    v
Native binary
```

### 4.3 Source to WASM (ogre compile --target wasm)

```
hello.bf
    |
    v
Preprocessor + IR + optimize (same as native)
    |
    v
generate_wat(&program, tape_size)
    |  - Emit WASI fd_write/fd_read imports
    |  - Emit linear memory with tape + I/O scratch space
    |  - Emit $dp global for data pointer
    |  - Map each Op to WAT instructions
    |  - Loops -> block/loop/br_if structure
    |
    v
WAT source string
    |
    v
Write to hello.wat
    |
    v
wat2wasm hello.wat -o hello.wasm (if available)
    |
    v
hello.wasm (WASI-compatible)
```

### 4.4 Source to Formatted Output (ogre format)

```
source.bf (with directives)
    |
    v
parse_segments(source)
    |  - Split into BF, Directive, and FnDef segments
    |
    v
For each segment:
    BF -> format_bf_only(code, opts)
        - Track loop depth for indentation
        - Group consecutive operators
        - Wrap at linewidth
    Directive -> emit verbatim on own line
    FnDef -> format body with BF formatter, add indent wrapper
    |
    v
Formatted string
    |
    +-- --check: compare with original, exit 1 if different
    +-- --diff: show unified diff with colored output
    +-- default: write back to file in-place
```

---

## 5. Error Handling Strategy

### OgreError + anyhow Hybrid

ogre uses a hybrid error strategy:

1. **`OgreError`** (defined with `thiserror`) for domain-specific, matchable errors
2. **`anyhow::Result`** for general error propagation

This gives the best of both worlds:
- Functions that need to distinguish error types can match on `OgreError` variants
- Functions that just propagate errors can use `anyhow::Result` and the `?` operator
- Additional context can be added with `anyhow::anyhow!()` and `.map_err()`

**Example: how errors flow from preprocessor to CLI:**

```rust
// In preprocess.rs -- creates a typed error
return Err(OgreError::UnknownFunction(name.clone()).into());

// In main.rs -- propagates with ?
let expanded = Preprocessor::process_file(path)?;

// In main() -- catches and displays
if let Err(e) = run(cli) {
    eprintln!("error: {}", e);
    process::exit(1);
}
```

**Source-location-enriched errors:**

The interpreter enriches `TapeOverflow` errors with source location when a source map is available:

```rust
Op::Right(n) => {
    if self.data_ptr + n >= self.tape.len() {
        let msg = match self.current_source_location() {
            Some(loc) => format!("right at {}", loc),
            None => "right".to_string(),
        };
        return Err(OgreError::TapeOverflow(msg).into());
    }
}
```

This means a tape overflow in `@fn greet` will report something like: `data pointer out of bounds (right at src/lib.bf:3:5 (@fn greet))`.

### Validation at Load Time

`OgreProject::validate()` is called immediately after parsing `ogre.toml`, catching configuration errors before any processing begins:
- Empty project name or version
- Entry file not ending in `.bf`
- Test files not ending in `.json`
- Zero tape size
- Dependencies without `path` or `version`

---

## 6. Standard Library

### Embedded via include_str!

The standard library modules are embedded directly in the binary using `include_str!`:

```rust
const STDLIB_IO: &str = include_str!("../../stdlib/io.bf");
const STDLIB_MATH: &str = include_str!("../../stdlib/math.bf");
const STDLIB_MEMORY: &str = include_str!("../../stdlib/memory.bf");
const STDLIB_ASCII: &str = include_str!("../../stdlib/ascii.bf");
const STDLIB_DEBUG: &str = include_str!("../../stdlib/debug.bf");
```

This means the standard library is always available without any file system access. It is compiled into the ogre binary at build time.

### Available Modules

| Module | Functions | Description |
|---|---|---|
| `std/io.bf` | `print_newline`, `print_space`, `read_char`, `print_char`, `print_zero` | I/O utilities |
| `std/math.bf` | `zero`, `inc`, `dec`, `inc10`, `double`, `add_to_next`, `move_right`, `move_left`, `copy_right` | Arithmetic operations |
| `std/memory.bf` | `clear`, `clear2`, `clear3`, `swap`, `push_right`, `pull_left` | Memory cell operations |
| `std/ascii.bf` | `print_A`, `print_B`, `print_exclaim`, `print_dash`, `print_colon` | ASCII character output |
| `std/debug.bf` | `dump_cell`, `dump_and_newline`, `marker_start`, `marker_end` | Debugging helpers |

### Resolution

When the preprocessor encounters `@import "std/io.bf"` (or `@import "std/io"`), it:

1. Strips the `std/` prefix and optional `.bf` suffix to get the module name
2. Calls `get_stdlib_module(name)` which returns `Option<&'static str>`
3. If found, uses a sentinel path `<stdlib:io>` for import cycle detection
4. Runs the collect pass on the module source (extracting `@fn` definitions)
5. Does not include top-level code from stdlib modules

Duplicate imports of the same stdlib module are silently ignored (idempotent).

---

## 7. Dependency Management

### [dependencies] in ogre.toml

Dependencies are declared in `ogre.toml`:

```toml
[dependencies]
mylib = { path = "../mylib" }
utils = { path = "../utils" }
```

Currently only path-based dependencies are supported. The `version` field is reserved for future registry-based resolution.

### Resolution Algorithm

1. **Resolve paths**: For each dependency, join the path with the project's base directory. Verify the directory and its `ogre.toml` exist.

2. **Collect functions**: For each dependency:
   a. Load the dependency's `ogre.toml`
   b. Resolve its include files
   c. Run `Preprocessor::collect_functions_from_file` on each include file
   d. Also collect from the dependency's entry file
   e. Recursively process the dependency's own dependencies

3. **Pre-load into preprocessor**: The collected `HashMap<String, String>` of function_name -> function_body is passed to `Preprocessor::process_file_with_deps`, which pre-loads these functions before the collect/expand passes.

4. **Cycle prevention**: A `HashSet` of visited dependency names prevents infinite recursion in circular dependency graphs.

### How Dependencies are Used

All commands that process files (`run`, `compile`, `build`, `check`, `pack`, `bench`, `debug`) check for project dependencies and pre-load them. The pattern in `main.rs`:

```rust
let (proj, base) = require_project()?;
let dep_fns = proj.collect_dependency_functions(&base)?;
if dep_fns.is_empty() {
    run::run_file_with_tape_size(&entry, ts)?;
} else {
    run::run_file_with_deps(&entry, ts, &dep_fns)?;
}
```

This means dependency functions are available for `@call` without explicit `@import` directives.

---

## 8. Testing Strategy

### Unit Tests

Every module includes `#[cfg(test)] mod tests` with targeted unit tests. Key coverage areas:

- **ir.rs**: Run-length encoding, bracket pairing, each optimization pass, decompilation roundtrip, semantics preservation after optimization
- **interpreter.rs**: All 8 BF operations, wrapping arithmetic, loop behavior, I/O, step vs run, peek window, optimized execution, instruction counting
- **preprocess.rs**: Plain BF passthrough, `@fn`/`@call` expansion, cycle detection (direct and self), stdlib imports, `@const`/`@use`, `@doc`, source maps, mixing file and stdlib imports
- **project.rs**: TOML parsing (minimal and full), validation edge cases (empty name, non-`.bf` entry, zero tape size), glob pattern resolution, dependency parsing and resolution, nested dependencies
- **format.rs**: Indentation, comment handling, grouping, line wrapping, directive preservation, diff generation
- **analyse.rs**: Bracket validation, I/O counting, pointer tracking, clear idiom detection, cancellation detection, dead code detection, complexity metrics

### Integration Tests

Located in `tests/`:

- **`interpreter_integration.rs`**: End-to-end interpreter tests using real BF scripts
- **`preprocess_integration.rs`**: Import resolution with real files, cycle detection with file pairs
- **`format_integration.rs`**: Format real BF files and verify output properties
- **`generate_integration.rs`**: Generate code and verify it produces correct output when interpreted

### CLI Integration Tests

`tests/cli_integration.rs` uses the `assert_cmd` and `predicates` crates to test the compiled binary end-to-end:

```rust
fn ogre() -> Command {
    Command::cargo_bin("ogre").unwrap()
}

#[test]
fn test_run_hello_world() {
    ogre()
        .args(["run", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success()
        .stdout("Hello World!\n");
}
```

Coverage includes:
- All subcommands (run, compile, format, check, analyse, test, bench, pack, new, init, doc, stdlib, generate, trace)
- Error cases (nonexistent files, invalid projects, unmatched brackets, unknown targets)
- Flag combinations (--quiet, --verbose, --no-color, --check, --diff, --optimize, --with-std, --target)
- Project features (dependency resolution, schema validation)
- Semantic preservation (packed output produces same runtime behavior as original)

### Criterion Benchmarks

`benches/interpreter.rs` provides micro-benchmarks using the criterion crate:

```rust
criterion_group!(
    benches,
    bench_interpret_hello_world,
    bench_interpret_simple_multiply,
    bench_interpret_compact_hello,
    bench_interpret_optimized,
    bench_ir_parse,
    bench_ir_parse_and_optimize,
    bench_ir_to_bf_string,
    bench_preprocess_with_stdlib,
    bench_compile_c_codegen,
);
```

These benchmarks measure:
- Interpreter throughput (with and without optimization)
- IR parsing speed
- Optimization pass overhead
- IR-to-BF-string decompilation speed
- Preprocessor speed with stdlib imports
- C code generation speed

Run with `cargo bench`. Results include statistical analysis with confidence intervals and HTML reports.

---

## 9. Performance Considerations

### IR Optimization Impact

The IR layer is the primary performance mechanism. Without optimization, the interpreter executes one op per raw BF character. With optimization:

1. **Run-length encoding** reduces `+++++++++` (9 iterations) to `Add(9)` (1 iteration). This is done during parsing, not as a separate pass. The reduction is proportional to the average run length of consecutive identical operators.

2. **Clear idiom** (`[-]` -> `Clear`): Eliminates a loop that would otherwise execute `cell_value` iterations. For a cell with value 255, this saves 255 loop iterations (each with a Sub, a JumpIfNonZero, and the JumpIfZero check) and replaces them with a single assignment.

3. **Move idiom** (`[->+<]` -> `MoveAdd(1)`): Eliminates a loop that transfers a cell's value to an adjacent cell. Without optimization this is `cell_value * 4` ops per transfer. With optimization it is 1 op regardless of cell value.

4. **Cancellation** (`+-` -> noop, `><` -> noop): Removes redundant operations that are common in generated or macro-expanded code. This is particularly effective after `@call` expansion, where function prologues and epilogues may cancel each other.

5. **Dead store** (`Clear` + `Add(n)` -> `Add(n)`): A cell that is cleared and immediately set to a value does not need the clear.

### Interpreter Execution Model

The interpreter uses a direct-threaded loop:

```rust
pub fn step(&mut self) -> Result<bool> {
    match &self.program.ops[self.ip] {
        Op::Add(n) => { ... }
        Op::Right(n) => { ... }
        Op::JumpIfZero(target) => {
            if self.tape[self.data_ptr] == 0 {
                self.ip = target + 1;
                return Ok(!self.is_done());
            }
        }
        ...
    }
    self.ip += 1;
    Ok(!self.is_done())
}
```

Key performance characteristics:
- Jump table is pre-computed during parsing (O(1) bracket jumps)
- Ops are stored in a contiguous `Vec<Op>` for cache-friendly iteration
- The hot loop is a simple `match` on an enum with no dynamic dispatch
- Instruction counting is a single `u64` increment per step

### C Backend Optimization

When compiling to C, the generated code benefits from both ogre's IR optimizations and the C compiler's optimizations (`-O2`):

- `Clear` emits `*ptr = 0;` which the C compiler can optimize further in context
- `MoveAdd(offset)` emits `*(ptr + offset) += *ptr; *ptr = 0;` which avoids loop overhead entirely
- Run-length operations emit `*ptr += n;` instead of n separate increments

The combination of ogre's high-level pattern recognition and gcc/clang's low-level optimization produces efficient native code.

### WASM Backend

The WASM backend generates efficient WAT code:
- Uses `i32.load8_u` and `i32.store8` for 8-bit tape access
- Uses WASI for I/O (no JavaScript glue needed)
- Loop structure maps directly to WASM's `block`/`loop`/`br_if` which WASM engines can optimize well

---

## 10. Key Design Principles

### 1. Single Binary, Full Lifecycle

ogre follows the Cargo model: one binary to rule them all. Users install one tool and get the complete brainfuck development experience. This eliminates the friction of finding, installing, and configuring separate tools for each task.

### 2. Project-First, File-Second

Every file-accepting command works in two modes: explicit file argument or project discovery. The project mode walks the directory tree upward to find `ogre.toml`, reads the manifest, resolves dependencies, and uses the configured entry point. This makes the most common case (working within a project) the easiest.

### 3. Layered Architecture

The codebase is organized in clean layers:
- **Foundation**: error types, verbosity, directive parsing, source mapping
- **Core**: preprocessor (source -> expanded BF), IR (expanded BF -> optimized ops), interpreter (ops -> execution)
- **Commands**: each subcommand is a thin wrapper that connects core components

This layering means the preprocessor does not know about the interpreter, the interpreter does not know about the compiler, and mode modules only depend downward.

### 4. Optimization at the IR Level

By introducing an intermediate representation between raw BF and execution, ogre can apply optimizations that benefit all backends (interpreter, C compiler, WASM compiler) equally. The `Op` enum is the single point of truth for semantics, and the optimization passes operate on it.

### 5. Source Maps Preserve Debug Context

After preprocessing, the connection between the expanded BF and the original source files is lost. Source maps restore this connection, enabling the debugger to show `src/lib.bf:3:5 (@fn greet)` instead of just `ip=47`. This is built into the preprocessor as an opt-in feature (used only by the debugger and for enriched error messages).

### 6. Progressive Disclosure of Complexity

- `ogre run hello.bf` -- just works, no project needed
- `ogre new myproject` -- scaffolds a project with tests
- `ogre.toml` `[dependencies]` -- opt-in dependency management
- `@import "std/io.bf"` -- opt-in standard library
- `ogre debug`, `ogre trace` -- opt-in debugging tools

Each feature is available when needed but does not impose complexity on simpler use cases.

### 7. Fail Fast with Helpful Messages

- Project validation runs immediately after parsing `ogre.toml`
- Bracket mismatches are caught during IR parsing before execution begins
- Import cycles are detected during the collect pass
- Call cycles are detected during the expand pass
- Source locations are included in error messages whenever a source map is available
- The `colored` crate provides visual distinction between errors, warnings, and success messages

### 8. Testability at Every Level

- Pure functions (`analyse_source`, `format_source`, `generate_c`, `generate_wat`) accept strings and return strings, making them trivially testable
- The interpreter has deterministic behavior (no randomness, configurable I/O)
- The preprocessor can operate on in-memory strings without filesystem access (except for `@import`)
- CLI tests use `assert_cmd` to test the compiled binary as a black box
- Criterion benchmarks provide regression detection for performance

### 9. Extensible Directive System

The `@` directive system is designed for extension. The `directive_parser` module provides reusable tokenizing primitives. Adding a new directive requires:
1. Adding a new match arm in the preprocessor's collect and/or expand passes
2. Optionally adding a new `OgreError` variant
3. Adding tests

The `@const`/`@use` and `@doc` directives were added following this pattern without modifying any existing directive handling.

### 10. Deterministic, Reproducible Builds

- `ogre pack` produces a single, self-contained BF file that can be shared
- `ogre pack --optimize` applies all optimizations and emits the result
- The generated C and WAT code is deterministic given the same input
- Test cases use exact output matching (or regex) for reproducibility
- The instruction-count-based timeout mechanism is deterministic (not wall-clock based)
