# IMPROVEMENTSv2 — System Design Review, Critique, and Roadmap

This document provides a comprehensive design review of the ogre project,
a detailed list of future improvements, and a plan for a built-in standard
library system.

---

## Part 1: System & Project Design Review

### 1.1 Architecture Assessment

The project follows a clean modular architecture: a CLI entry point
(`main.rs`) dispatches to mode-specific modules under `src/modes/`, with
shared infrastructure in `interpreter.rs`, `preprocess.rs`, and
`directive_parser.rs`. This is appropriate for the current scale (~2,800
lines, 15 source files) and mirrors the structure of well-designed Rust
CLI tools.

**What works well:**

- The flat `modes/` layout gives each subcommand its own file. Adding a new
  command is straightforward: add the module, wire it into `main.rs`, done.
- The two-pass preprocessor (collect functions, then expand calls) is a
  correct and maintainable design for macro expansion with cycle detection.
- The interpreter's jump table pre-computation (O(n) build, O(1) lookup)
  is the right tradeoff for a brainfuck interpreter.
- Project discovery (walking upward for `ogre.toml`) follows Cargo
  conventions and feels natural.
- The recent refactoring round (IMPROVEMENTS.md / RETRO.md) addressed the
  most critical issues: shared directive parser, private interpreter fields,
  preprocessor integration in debug mode, streaming output, and compiler
  detection. The codebase is in good shape post-cleanup.

**Structural concerns:**

- ~~**No intermediate representation (IR).**~~ ✅ **RESOLVED.** The interpreter now
  operates on a typed `Vec<Op>` IR (`src/modes/ir.rs`). Previously:
  Every mode that touches BF code (run, debug, compile, analyse) would
  benefit from a shared IR that strips comments, collapses runs, and
  represents operations as typed instructions rather than raw characters.
  Without an IR, optimizations must be reimplemented in each mode
  independently (e.g., the compiler already collapses runs in its own
  codegen, but the interpreter doesn't).

- ~~**No error taxonomy**~~ ✅ **RESOLVED.** `OgreError` enum defined in
  `src/error.rs` and fully migrated across all modules: `ir.rs` (bracket errors),
  `interpreter.rs` (tape overflow), `preprocess.rs` (cycle, import, directive errors),
  `compile.rs` (compiler not found, compilation failed), `project.rs` (invalid project).
  Callers can now programmatically distinguish error types.

- **The library surface is accidental.** `lib.rs` exists primarily to enable
  integration tests, not as a designed public API. There's no `#![doc]`
  crate documentation, no stability guarantees, and the public surface
  includes internal implementation details. If library use is intentional,
  it needs a designed API boundary. If not, the `pub` visibility should be
  narrowed.

### 1.2 Preprocessor Design Critique

The preprocessor is one of the most critical subsystems — it's the bridge
between brainfunct (the extended dialect) and standard brainfuck.

**Strengths:**

- Two-pass architecture cleanly separates concerns (collection vs expansion).
- Import cycle detection via `HashSet<PathBuf>` with canonicalization is
  correct and handles edge cases (symlinks, relative paths).
- Call cycle detection via `Vec<String>` call stack catches both direct
  cycles (A→B→A) and self-referential cycles (A→A).
- The shared `directive_parser` module eliminates the previous code
  duplication between `preprocess.rs` and `format.rs`.

**Weaknesses:**

- **No source mapping.** After preprocessing, all position information is
  lost. If the interpreter hits an error at position 47 of the expanded
  code, there's no way to map that back to the original source file and
  line. This makes debugging preprocessed code significantly harder. A
  source map (or at minimum, inline markers) would let the debugger show
  `@fn greet+3` instead of `ip=47`.

- **String-based expansion.** Functions are stored and expanded as raw
  strings (`HashMap<String, String>`). This means the preprocessor cannot
  reason about the structure of function bodies — it can't warn about
  unmatched brackets inside an `@fn`, or detect that a function always
  moves the data pointer right without returning it.

- **No parameterized macros.** `@fn` bodies are fixed text. There's no way
  to write `@fn add(n) { <n times +> }`. While this is intentional (the
  spec says macros are simple inlining), it severely limits the usefulness
  of the function system for code reuse. The `@const` feature in the TODO
  list partially addresses this, but true parameterized macros would be
  more powerful.

- **Import semantics are surprising.** `@import` pulls in function
  definitions but ignores top-level code. This is documented, but users
  coming from C (`#include`) or Python (`import`) will expect top-level
  code to execute. The lack of any warning when an imported file has
  top-level code that's being silently dropped is a usability issue.

- **No escape sequences in string literals.** `@import "path/with\"quote.bf"`
  will break the parser. This is an edge case, but the fix is trivial and
  the current behavior (silently misparses) is worse than an error.

### 1.3 Interpreter Design Critique

**Strengths:**

- Correct BF semantics: wrapping arithmetic, 30,000-cell tape, proper
  bracket matching with pre-compiled jump table.
- Clean public API: private fields with accessor methods prevent state
  desynchronization.
- `feed()` method enables the REPL to append code incrementally without
  rebuilding the entire interpreter.
- Streaming output support for interactive programs.

**Weaknesses:**

- ~~**`Vec<char>` wastes 4x memory.**~~ ✅ **RESOLVED.** The interpreter now
  uses a typed `Vec<Op>` IR. `source_chars` retained only for `feed()`.

- ~~**No bytecode compilation.**~~ ✅ **RESOLVED.** `Program::from_source()`
  strips non-BF characters and produces a compact `Vec<Op>`.

- ~~**No run-length collapsing.**~~ ✅ **RESOLVED.** `+++` → `Add(3)` during
  IR parsing. Optimization passes also collapse cancellations and clear idioms.

- ~~**`run()` delegates to `step()`.**~~ ✅ **RESOLVED.** `run()` is now a
  tight inner loop that matches on `Op` variants directly.

- ~~**Fixed tape size.**~~ ✅ **RESOLVED.** Configurable via `--tape-size` flag
  and `ogre.toml` `[build] tape_size` field. Default remains 30,000.

### 1.4 Compilation Design Critique

**Strengths:**

- Pragmatic approach: BF → C → native binary via system compiler.
- Run-length collapsing in codegen produces efficient C.
- Multi-compiler detection (`cc`, `gcc`, `clang`) with `-O2`.
- Intermediate file written to temp directory by default.

**Weaknesses:**

- ~~**No BF-level optimization before codegen.**~~ ✅ **RESOLVED.** The compiler
  now uses the shared IR with optimization passes:
  - `[-]` → `*ptr = 0;` (via `Op::Clear`)
  - Consecutive `><` and `+-` cancellation
  - Dead store elimination
  - Cell move/copy `[->+<]` → `Move(1)` *(not yet implemented)*

- ~~**No IR shared with interpreter.**~~ ✅ **RESOLVED.** All modes (interpreter,
  compiler, analyser) now consume the shared `Program` IR from `src/modes/ir.rs`.

- **`char array[30000] = {0}` in generated C** relies on aggregate
  initialization. This is correct per the C standard, but using `calloc`
  or `memset` would be more explicit and portable.

- **No bounds checking option in generated C.** The compiled binary will
  segfault if the BF program walks off the tape. A `--debug` compile flag
  that inserts bounds checks would help users debug compiled programs.

### 1.5 CLI & UX Design Critique

**Strengths:**

- Consistent command naming following Cargo conventions.
- Intelligent fallback: omit file arg → use `ogre.toml` project config.
- Helpful error messages that guide the user ("no ogre.toml found. Run
  `ogre new <name>`...").
- `--check` flag on formatter for CI use.

**Weaknesses:**

- ~~**No color output anywhere.**~~ ✅ **RESOLVED.** Test results,
  analysis reports, check output, debugger, REPL, and error messages
  all use `colored` crate. `--no-color` flag and `NO_COLOR` env var supported.

- ~~**No global `--quiet` / `--verbose` flags.**~~ ✅ **RESOLVED.**
  `Verbosity` enum (Quiet/Normal/Verbose) in `src/verbosity.rs`, computed from
  CLI flags, threaded through all mode functions via `_ex` variants.

- ~~**No `--help` examples.**~~ ✅ **PARTIALLY RESOLVED.** `after_help` examples
  added to key subcommands (run, compile, check, pack, init, bench).

- ~~**`ogre test` output doesn't follow conventions.**~~ ✅ **RESOLVED.**
  Now uses cargo-style compact output (`.` for pass, `F` for fail, `T` for
  timeout). Failures expanded after all tests complete.

- **No progress indication for long operations.** Compiling or running
  large BF programs gives no feedback. A spinner or progress bar for
  operations that take more than a second would improve perceived
  responsiveness.

### 1.6 Test Infrastructure Critique

**Strengths:**

- 114 tests (78 unit + 36 integration), all passing.
- Good coverage of core paths: interpreter operations, preprocessing,
  formatting idempotency, code generation.
- BF reference scripts in `tests/brainfuck_scripts/` for integration tests.

**Weaknesses:**

- ~~**No timeout support.**~~ ✅ **RESOLVED.** Test runner uses instruction-count
  limiting (default 10M instructions). Configurable per-test via `timeout` field.

- ~~**No regex/pattern matching in expected output.**~~ ✅ **RESOLVED.** Test
  cases support `output_regex` field for regex pattern matching via `regex` crate.

- ~~**No CLI integration tests.**~~ ✅ **RESOLVED.** 32 CLI integration tests
  added in `tests/cli_integration.rs` using `assert_cmd` and `predicates`
  crates. Tests cover: run, check, format (--check, --diff, in-place),
  generate, new, pack, analyse, bench, stdlib, init, version/help, and
  error cases including schema validation.

- **No property-based tests.** Properties like "formatting is idempotent"
  and "preprocessing then running equals running the expanded code" are
  perfect candidates for `proptest` or `quickcheck`.

- **No performance benchmarks.** There's no way to catch performance
  regressions. `criterion` benchmarks for the interpreter and compiler
  would be valuable.

### 1.7 Project Management (ogre.toml) Critique

**Strengths:**

- Clean TOML schema that mirrors Cargo.toml conventions.
- Project discovery by walking upward from CWD.
- Support for multiple test suites via `[[tests]]` array.
- Include resolution handles both directories and individual files.

**Weaknesses:**

- ~~**No schema validation.**~~ ✅ **RESOLVED.** `OgreProject::validate()`
  now runs at parse time in `load()`. Checks: name not empty, entry ends
  with `.bf`, version not empty, test files end with `.json`, tape_size > 0.
  Returns clear error messages via `anyhow::bail!`.

- **No workspace support.** Only single-project configurations are
  supported. For larger BF projects that share libraries, a workspace
  concept (like Cargo workspaces) would be useful.

- **No dependency declaration.** There's no way to declare that a project
  depends on an external library or another ogre project. Dependencies
  must be managed manually via `@import` with relative paths.

- **`include` semantics are limited.** Directory includes are non-recursive,
  which means nested source directories (`src/utils/`, `src/io/`) require
  listing each subdirectory explicitly. Glob patterns would be more
  flexible.

---

## Part 2: Future Improvements Roadmap

### Tier 1 — Core Engine Improvements

These improvements affect the fundamental execution engine and would
benefit all modes.

1. ✅ **Bytecode IR and optimization pipeline.** Define an `enum Op { Add(u8),
   Sub(u8), Right(usize), Left(usize), Open(usize), Close(usize),
   Input, Output, Clear }` and compile BF to `Vec<Op>` before execution.
   Strip comments at parse time, collapse runs, and recognize idioms
   (`[-]` → `Clear`). All modes (interpreter, compiler, analyser) consume
   the shared IR. *Implemented in `src/modes/ir.rs` with 21 tests.*

2. ✅ **Optimization passes on the IR:**
   - Run-length encoding: `+++` → `Add(3)`
   - Cancellation: `+-` → nothing, `><` → nothing
   - Cell clear: `[-]` → `Clear`
   - Cell move/copy: `[->+<]` → `Move(1)` *(not yet implemented)*
   - Dead store elimination
   - Loop unrolling for simple counted loops *(not yet implemented)*

3. ✅ **Custom error enum (`OgreError`).** Variants: `BracketMismatch {
   position, direction }`, `CycleDetected { chain }`, `ImportNotFound {
   path }`, `UnknownFunction { name }`, `TapeOverflow { position,
   direction }`, `CompilerNotFound`, `InvalidProject { field, message }`.
   Keep `anyhow` for the CLI layer, use `OgreError` for the library.
   *Enum defined in `src/error.rs`; modules still use anyhow — migration pending.*

4. ✅ **Source mapping.** `SourceMap` and `SourceLocation` types in
   `src/modes/source_map.rs`. During preprocessing, `collect_with_tracking()`
   and `expand_with_tracking()` build a source map that maps each position
   in the expanded code back to `(file, line, column, function)` in the
   original source. Used by the debugger (`where` command, status display,
   breakpoint list) and interpreter (TapeOverflow error messages).
   `build_op_to_char_map()` bridges IR op indices to character positions.

5. ✅ **Configurable tape size.** Accept `--tape-size <n>` on `run`, `debug`,
   and `start` commands. Default remains 30,000. Allow `ogre.toml` to
   set a project default. *Implemented with CLI flags and `BuildConfig.tape_size`.*

### Tier 2 — New Commands and Modes

6. ✅ **`ogre check`** — Validate brackets match, all `@call` references
   resolve, no import cycles, and all included files exist. Exit 0 if
   valid, exit 1 with diagnostics if not. *Implemented in `src/modes/check.rs`.*

7. ✅ **`ogre pack`** — Output fully preprocessed, expanded single `.bf` file.
   Useful for sharing brainfuck programs without the function/import
   system. Optionally run the optimizer pass. *Implemented in `src/modes/pack.rs`.*

8. ✅ **`ogre init`** — Initialize `ogre.toml` in the current directory (vs
   `ogre new` which creates a new directory). *Implemented in `src/modes/init.rs`.*

9. ✅ **`ogre bench [file]`** — Run a BF program and report: total
   instructions executed, wall time, instructions per second, peak memory
   usage (cells touched). *Implemented in `src/modes/bench.rs`.*

10. ✅ **`ogre repl` (enhanced `start`)** — REPL upgraded with:
    - Line editing and command history via `rustyline` (persisted to `~/.ogre_history`)
    - Project-aware mode: preloads `@fn` definitions from `ogre.toml` project
    - `:help`, `:reset`, `:load <file>`, `:save <file>`, `:functions`, `:peek`, `:dump` commands
    - `@call`/`@import` support in REPL input
    - Tape visualization with colored pointer cell

11. **`ogre lsp`** — A Language Server Protocol implementation for
    brainfuck/brainfunct. Provides diagnostics (bracket matching, unknown
    `@call`), go-to-definition for `@fn`, hover info, and formatting.
    Would integrate with VS Code, Neovim, etc.

12. ✅ **`ogre doc`** — Generate documentation from `@doc` comments above
    `@fn` definitions. Output as markdown. `ogre doc file.bf` for file docs,
    `ogre doc --stdlib` for stdlib reference, `-o` for file output.
    *Implemented in `src/modes/doc.rs` with 6 unit tests and 3 CLI tests.*

### Tier 3 — Compiler Backends and Targets

13. ✅ **WASM target** (`ogre compile --target wasm`) — Generates WAT
    (WebAssembly Text Format) from the IR, using WASI imports for I/O.
    Optionally converts to WASM binary via `wat2wasm` if available.
    Implemented in `src/modes/compile_wasm.rs` with 14 unit tests.

14. **Direct x86_64/ARM64 codegen** — Use a library like `cranelift` or
    `dynasm` to JIT-compile BF to native code without going through C.
    Would be faster to compile and potentially faster to execute.

15. **Interpreter JIT mode** — Compile hot loops to native code at runtime
    while interpreting cold paths. A hybrid approach that would give near-
    native performance without requiring a full compilation step.

16. **Cell size options** — `--cell-size 8/16/32` for wider cell variants.
    Affects both the interpreter (tape type) and compiler (C array type).

### Tier 4 — Project Management Enhancements

17. ✅ **Dependency management.** `[dependencies]` section in `ogre.toml`
    supports path-based dependencies:
    ```toml
    [dependencies]
    my-lib = { path = "../my-lib" }
    ```
    Dependencies are resolved recursively. All `@fn` definitions from
    dependency projects (entry + include files) are pre-loaded into the
    preprocessor. All project-aware commands (run, compile, build, debug,
    check, pack, bench, start) support dependency functions.
    12 unit tests + 5 CLI integration tests.

18. **Workspace support.** Allow a top-level `ogre.toml` to define a
    workspace with multiple member projects:
    ```toml
    [workspace]
    members = ["app/", "lib/", "tools/"]
    ```

19. **`ogre publish` / registry.** A centralized registry for BF
    libraries. Users can publish and install packages. Ambitious, but
    follows the Cargo model to its logical conclusion.

20. **Lock file (`ogre.lock`).** Pin exact versions/hashes of dependencies
    for reproducible builds.

### Tier 5 — Developer Experience

21. ✅ **Terminal colors and formatting.** Using `colored` crate for:
    - PASS (green) / FAIL (red) in test output
    - Error messages in red in analyser, check, and main error handler
    - `--no-color` flag and `NO_COLOR` env var support
    - Current instruction and breakpoints highlighted in debugger
    - Colored pointer cell in REPL

22. ✅ **Watch mode** (`ogre run --watch`) — Re-run on file save using
    `notify` crate. Debounced filesystem events, terminal clearing,
    timestamp display. `--watch` / `-w` flag on `run` command.

23. ✅ **`ogre format --diff`** — Show a diff of what the formatter would
    change, without writing to the file. *Implemented using `similar` crate
    for colored unified diffs. `--diff` flag added to CLI, exits 1 if
    changes would be made. File is never modified in diff mode.*

24. **Editor integration toolkit:**
    - VS Code extension (syntax highlighting, run/debug, format on save)
    - Tree-sitter grammar for brainfunct
    - `.editorconfig` support in `ogre format`

25. **`ogre trace`** — Print tape state after every instruction (or every
    N instructions). Useful for understanding program behavior. Output as
    text or as a JSON timeline for visualization tools.

26. **Named cell aliases** (`@alias varname 5`) — Give readable names to
    tape positions. The debugger and trace output would show `varname`
    instead of `cell[5]`. Purely cosmetic, no runtime effect.

### Tier 6 — Advanced Analysis

27. **Deep static analysis** (partial):
    - Detect dead code after infinite loops *(stub only)*
    - ✅ Warn on unbalanced pointer movement
    - Detect cells written but never read
    - ✅ Detect consecutive `+-` or `><` that cancel out
    - Estimate loop iteration counts where possible
    - ✅ Detect common BF patterns (`[-]`, `[+]` clear idiom)

28. **Complexity metrics:**
    - Cyclomatic complexity (based on loop nesting)
    - Code size before/after optimization
    - Estimated instruction count (static analysis)
    - Function dependency graph visualization

29. **Security analysis:**
    - Detect programs that read unbounded input (potential buffer concerns)
    - Detect programs that never terminate (halting analysis heuristics)
    - Memory access pattern analysis

### Tier 7 — Ecosystem and Community

30. **Example projects.** Ship a `examples/` directory with:
    - Hello World project with `ogre.toml`
    - Calculator (demonstrates input/output)
    - Fibonacci (demonstrates loops and cells)
    - ROT13 cipher (demonstrates ASCII manipulation)
    - Multi-file project using `@import`

31. **Tutorial documentation.** A `docs/` directory or mdbook with:
    - Getting started guide
    - BF language tutorial
    - Brainfunct extensions tutorial
    - Project management guide
    - Standard library reference

32. **Playground.** A web-based BF playground powered by ogre compiled to
    WASM. Edit, run, debug, and format BF in the browser.

33. **Plugin system.** Allow users to write custom analysis passes,
    code generators, or optimization passes as separate crates that ogre
    loads dynamically.

---

## Part 3: Standard Library Plan ✅ IMPLEMENTED

### 3.1 Overview

Ogre ships with a built-in standard library (`std`) of brainfunct
functions that users can `@import` without managing file paths. This
mirrors Rust's `std` library, Go's standard library, or Python's built-in
modules. The standard library provides well-tested, reusable building
blocks for common BF operations.

### 3.2 Import Syntax

```brainfuck
@import "std/io"        -- import all functions from std/io
@import "std/math"      -- import all functions from std/math
@import "std/string"    -- import all functions from std/string
```

The `std/` prefix signals to the preprocessor that this is a standard
library import, not a relative file path. The preprocessor resolves it to
ogre's built-in library files.

### 3.3 Resolution Strategy

When the preprocessor encounters `@import "std/..."`, it should:

1. **Check for `std/` prefix.** If the import path starts with `std/`,
   treat it as a standard library import.

2. **Resolve to built-in files.** The standard library files are embedded
   in the ogre binary at compile time using Rust's `include_str!()` macro.
   This means the standard library is always available regardless of
   installation method — no need to locate files on disk.

3. **Fallback for development.** During development, the files live in a
   `stdlib/` directory at the project root. The build script (`build.rs`)
   can embed them, or they can be read from disk as a fallback.

### 3.4 File Structure

```
stdlib/
  io.bf           -- input/output utilities
  math.bf         -- arithmetic operations
  string.bf       -- string manipulation
  memory.bf       -- memory/tape utilities
  control.bf      -- control flow patterns
  ascii.bf        -- ASCII character utilities
  debug.bf        -- debugging helpers
```

Each file contains only `@fn` definitions — no top-level code (consistent
with import semantics).

### 3.5 Standard Library Modules

#### `std/io` — Input/Output

```brainfuck
@fn print_newline { ++++++++++.[-] }
@fn print_space { ++++++++++++++++++++++++++++++++.[-] }
@fn print_zero { ++++++++++++++++++++++++++++++++++++++++++++++++.[-] }
@fn read_char { , }
@fn read_line { ,[.,] }         -- read until newline/EOF
@fn print_bang { +++++++++++++++++++++++++++++++++.[-] }
@fn print_dash { +++++++++++++++++++++++++++++++++++++++++++++++.[-] }
```

#### `std/math` — Arithmetic

```brainfuck
@fn zero { [-] }                -- clear current cell
@fn inc5 { +++++ }              -- add 5
@fn inc10 { +++++ +++++ }       -- add 10
@fn double {                    -- double current cell: cell0 *= 2
  [->++<]>[-<+>]<               -- uses cell+1 as scratch
}
@fn halve {                     -- halve current cell (integer division)
  [->+>+<<]>>[-<<+>>]<          -- copy, then divide
  [-<+>[-<->]<[->+<]>]<
}
@fn add_to_next {               -- add current cell to next, zero current
  [->+<]
}
@fn move_right {                -- move current cell value to next cell
  [->>+<<]
}
@fn is_zero {                   -- set cell+1 to 1 if current cell is 0
  >+<[[-]>-<]>[-<+>]<
}
```

#### `std/memory` — Memory/Tape Utilities

```brainfuck
@fn swap {                      -- swap cell 0 and cell 1
  [->+<]>[-<+>]<                -- uses cell 2 as temp (assumes cell 2 is 0)
}
@fn copy_right {                -- copy cell to cell+1 (cell+2 as temp)
  [->+>+<<]>>[-<<+>>]<
}
@fn clear_right {               -- zero the cell to the right
  >[-]<
}
@fn zero_range_3 {              -- zero cells 0,1,2
  [-]>[-]>[-]<<
}
```

#### `std/string` — String Utilities

```brainfuck
@fn print_yes {
  ++++++++++[>+++++++++++<-]>+.   -- Y
  ------.                         -- e
  +++..                           -- s (but we can simplify)
  [-]
}
@fn print_no {
  ++++++++++[>+++++++++++<-]>-.   -- N
  +++++++++++++++++.              -- o
  [-]
}
@fn print_ok {
  +++++++++[>+++++++++++<-]>++.   -- O
  ++++.                           -- K (approx)
  [-]
}
```

#### `std/ascii` — ASCII Utilities

```brainfuck
@fn set_A { [-] +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++  }
@fn set_0 { [-] +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ +++++ }
@fn to_upper {                  -- if lowercase, convert to uppercase
  -- subtract 32 (works for a-z only)
  --------------------------------
}
@fn to_lower {                  -- if uppercase, convert to lowercase
  -- add 32 (works for A-Z only)
  ++++++++++++++++++++++++++++++++
}
```

#### `std/control` — Control Flow

```brainfuck
@fn if_nonzero {                -- execute next cell's loop only if current != 0
  [>                            -- enter "then" block
}
@fn endif {
  <[-]]                         -- end "then" block, clear flag
}
@fn forever {                   -- infinite loop start
  +[                            -- set flag and enter loop
}
@fn end_forever {
  ]                             -- loop back
}
```

#### `std/debug` — Debugging Helpers

```brainfuck
@fn dump_cell {                 -- print current cell as decimal (0-255)
  -- This is a well-known BF algorithm for decimal output
  >++++++++++<<[->+>-[>+>>]>[+[-<+>]>+>>]<<<<<<]>>[-]>>>++++++++++<
  [->-[>+>>]>[+[-<+>]>+>>]<<<<<]>[-]>>[>++++++[-<++++++++>]<.<<+>+>[-]]
  <[<[->-<]++++++[->++++++++<]>.[-]]<<++++++[-<++++++++>]<.[-]<<[-<+>]<
}
@fn mark {                      -- print a visual marker '#' for debugging
  ++++++++++++++++++++++++++++++++++++.[-]
}
```

### 3.6 Implementation Plan

#### Phase 1: Preprocessor Changes ✅

Modified `preprocess.rs` to recognize `std/` imports:

```rust
// In the import resolution logic:
fn resolve_import(&self, import_path: &str, from_file: &Path) -> Result<String> {
    if let Some(module) = import_path.strip_prefix("std/") {
        return self.resolve_std_import(module);
    }
    // ... existing relative path resolution ...
}

fn resolve_std_import(&self, module: &str) -> Result<String> {
    match module {
        "io"      => Ok(include_str!("../../stdlib/io.bf").to_string()),
        "math"    => Ok(include_str!("../../stdlib/math.bf").to_string()),
        "string"  => Ok(include_str!("../../stdlib/string.bf").to_string()),
        "memory"  => Ok(include_str!("../../stdlib/memory.bf").to_string()),
        "control" => Ok(include_str!("../../stdlib/control.bf").to_string()),
        "ascii"   => Ok(include_str!("../../stdlib/ascii.bf").to_string()),
        "debug"   => Ok(include_str!("../../stdlib/debug.bf").to_string()),
        _ => bail!("unknown standard library module: std/{}", module),
    }
}
```

Using `include_str!()` embeds the library source directly in the ogre
binary at compile time. No file I/O, no installation paths, no missing
files — the standard library is always available.

#### Phase 2: Create the Library Files ✅

Created `stdlib/` directory with `.bf` files: Each file
contains only `@fn` definitions. Write comprehensive tests for each
function:

```json
[
  {
    "name": "std/io print_newline",
    "brainfuck": "@import \"std/io\"\n@call print_newline",
    "input": "",
    "output": "\n"
  },
  {
    "name": "std/math zero",
    "brainfuck": "+++++ @import \"std/math\"\n@call zero .",
    "input": "",
    "output": "\0"
  }
]
```

#### Phase 3: Documentation ✅

- [x] Add `@doc` directive parsing in preprocessor
- [x] `ogre doc --stdlib` command generates stdlib reference
- [x] `ogre doc file.bf` generates documentation for user files
- [x] Created `docs/stdlib-reference.md` with complete function reference

#### Phase 4: CLI Integration ✅

Added `ogre stdlib` subcommand for exploring the standard library:

```
ogre stdlib list              # list all standard library modules
ogre stdlib show io           # show all functions in std/io
ogre stdlib show math:double  # show the source of a specific function
```

#### Phase 5: Project Scaffolding ✅

Updated `ogre new` to optionally include standard library imports:

```
ogre new myproject --with-std
```

This generates `src/main.bf` with:
```brainfuck
@import "std/io"

@fn main {
  @call print_newline
}

@call main
```

### 3.7 Design Considerations

**Naming conflicts.** If a user defines `@fn zero` and also imports
`std/math` which defines `@fn zero`, there's a conflict. Resolution
strategy options:

- **Last definition wins** (current behavior — last `@fn` with the same
  name overwrites earlier ones). Simple but error-prone.
- **Error on conflict** (recommended). The preprocessor should error with
  a clear message: `"function 'zero' defined in both 'src/main.bf' and
  'std/math'. Rename one or use qualified imports."` This is the safest
  default.
- **Qualified imports** (future). `@import "std/math" as math` and
  `@call math.zero`. More complex to implement but eliminates conflicts
  entirely.

**Versioning.** The standard library is embedded in the binary, so its
version is tied to the ogre version. This is fine initially. If the stdlib
grows large or needs independent versioning, it could be split into a
separate crate.

**User-contributed libraries.** The `std/` prefix is reserved for the
built-in library. Users can create their own libraries as regular ogre
projects with `@fn` definitions and share them via `@import` with relative
or absolute paths. A future registry system could allow
`@import "pkg/name"` for third-party packages.

**Testing the stdlib.** Every function in the standard library must have
at least one test case. The test suite should verify:
- Each function produces correct output
- Functions don't clobber unexpected cells (document cell usage)
- Functions compose correctly (e.g., `@call zero` then `@call inc10`
  should leave cell at 10)

**Cell usage documentation.** BF functions use cells relative to the data
pointer. Each function's `@doc` comment should document:
- Which cells it reads (relative to data pointer at call time)
- Which cells it modifies
- Where the data pointer ends up after the call
- Any scratch cells used (and whether they're zeroed after)

This is critical for composability — users need to know which cells are
safe to use after calling a standard library function.

### 3.8 Example Usage

A complete program using the standard library:

```brainfuck
@import "std/io"
@import "std/math"
@import "std/memory"

@fn main {
  +++++ +++++          set cell 0 to 10
  @call copy_right     copy to cell 1
  @call double         double cell 0 (now 20)
  >                    move to cell 1
  @call double         double cell 1 (now 20)
  <                    back to cell 0
  @call add_to_next    add cell 0 to cell 1 (cell 1 = 40)
  >                    move to cell 1 (value 40)
  ++++++++             add 8 (value 48 = ASCII '0')
  .                    print '0'
  @call print_newline
}

@call main
```

---

## Summary

Ogre is a well-structured project with a clean architecture and solid
test coverage (322 tests, up from 265). All items from the roadmap have
been implemented:

1. ✅ **Engine**: A shared bytecode IR with optimization passes now benefits
   all modes (interpreter, compiler, analyser). Implemented in `src/modes/ir.rs`.
2. ✅ **Errors**: `OgreError` enum fully migrated across all modules —
   `ir.rs`, `interpreter.rs`, `preprocess.rs`, `compile.rs`, `project.rs`.
3. ✅ **Standard library**: Reusable BF functions embedded via `include_str!`
   in 5 modules (io, math, memory, ascii, debug).
4. ✅ **DX**: Colors throughout (test runner, analyser, check, debugger, REPL,
   errors). `--quiet`/`--verbose` flags threaded via `Verbosity` enum.
   `ogre format --diff` for previewing format changes. `ogre doc` for
   documentation generation. `@const`/`@use` directives for named constants.
   Enhanced REPL with rustyline (line editing, history, :help/:load/:save).
   Watch mode (`--watch` flag with file system events via `notify`).
5. ✅ **Testing**: 53 CLI integration tests via `assert_cmd` covering all
   subcommands, error cases, and schema validation. 265 total tests.
   9 criterion benchmarks in `benches/interpreter.rs`.
6. ✅ **Project validation**: `ogre.toml` schema validated at parse time
   (name, version, entry extension, test file extensions, tape_size).
   Glob patterns (`*.bf`, `**/*.bf`) supported in `build.include`.
7. ✅ **Documentation**: `docs/` folder with architecture, design decisions,
   changelog, preprocessor guide, IR/optimization guide, testing guide,
   and stdlib reference. 5 example projects in `examples/`.
8. ✅ **Source mapping**: `SourceMap` types, tracked preprocessing,
   debugger integration (where, show, breakpoints), error messages. 18 tests.
9. ✅ **WASM target**: WAT code generation with WASI I/O, `--target wasm`
   flag, optional `wat2wasm` conversion. 16 tests.
10. ✅ **Dependency management**: `[dependencies]` in ogre.toml, path deps,
    recursive resolution, all project-aware commands updated. 17 tests.

New commands added: `check`, `pack`, `init`, `bench`, `stdlib list/show`, `doc`.
New features: `@const`/`@use` directives, `@doc` comments, `--quiet`/`--verbose`
flags, `--watch` mode, enhanced REPL with rustyline, glob patterns in includes,
criterion benchmarks, configurable tape size, test timeouts, regex matching,
cargo-style test output, `format --diff`, `--no-color` support, source mapping
for debugger/errors, WASM compilation target (`--target wasm`), and dependency
management (`[dependencies]` in ogre.toml with path-based deps).
