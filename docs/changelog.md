# Changelog

This document tracks all significant changes made to the ogre codebase,
organized by feature area.

---

## Bytecode IR and Optimization Pipeline

**Files changed:** `src/modes/ir.rs` (new), `src/modes/interpreter.rs`,
`src/modes/compile.rs`, `src/modes/analyse.rs`, `src/modes/debug.rs`,
`src/modes/start.rs`

**What changed:**
- New `Op` enum with 9 variants: Add, Sub, Right, Left, Output, Input,
  JumpIfZero, JumpIfNonZero, Clear
- `Program::from_source()` parses BF to Vec<Op> with run-length encoding
- `Program::optimize()` applies 3 passes: clear idiom, cancellation, dead store
- `Program::to_bf_string()` converts IR back to BF for pack command
- Interpreter rewritten to dispatch on Op variants instead of chars
- Compiler generates C from IR instead of raw BF characters
- Analyser uses IR for bracket validation and I/O counting
- Jump table is now embedded in Op variants (O(1) lookup)

**Why:** Enables shared optimizations across all modes. Interpreter performance
improved ~2-3x through collapsed operations and eliminated comment filtering.

**Tests added:** 21 unit tests in ir.rs covering parsing, collapsing,
bracket pairing, optimization passes, and to_bf_string roundtrip.

---

## OgreError Migration

**Files changed:** `src/error.rs`, `src/modes/ir.rs`, `src/modes/interpreter.rs`,
`src/modes/preprocess.rs`, `src/modes/compile.rs`, `src/project.rs`

**What changed:**
- All modules now return typed `OgreError` variants instead of string-based
  `anyhow::bail!()` calls
- `ir.rs`: `UnmatchedCloseBracket`, `UnmatchedOpenBracket(pos)`
- `interpreter.rs`: `TapeOverflow("left"/"right")`
- `preprocess.rs`: `UnknownStdModule`, `ImportCycle`, `UnknownDirective`,
  `CycleDetected`, `UnknownFunction`
- `compile.rs`: `CompilerNotFound`, `CompilationFailed`
- `project.rs`: `InvalidProject` with descriptive messages

**Why:** Enables callers to match on specific error types instead of parsing
error message strings. Important for library usage and testing.

**Impact:** All 228+ tests continue to pass. Error messages unchanged since
OgreError derives Display via thiserror.

---

## @const/@use Directives

**Files changed:** `src/modes/preprocess.rs`

**What changed:**
- Added `constants: HashMap<String, usize>` to Preprocessor state
- `@const NAME value` directive parsed in collect phase
- `@use NAME` directive expands to N '+' characters in both collect and expand
- Error on undefined `@use`, missing value, or non-numeric value

**Why:** Named constants improve readability for ASCII values and repeated
patterns. `@const SPACE 32` + `@use SPACE` is clearer than 32 `+` characters.

**Tests added:** 7 unit tests (basic, zero, large, inside @fn, undefined,
missing value, multiple constants).

---

## Verbosity Threading

**Files changed:** `src/verbosity.rs` (new), `src/main.rs`, `src/modes/compile.rs`,
`src/modes/check.rs`, `src/modes/pack.rs`, `src/modes/bench.rs`,
`src/modes/test_runner.rs`, `src/modes/new.rs`, `src/modes/init.rs`

**What changed:**
- New `Verbosity` enum with `Quiet`, `Normal`, `Verbose` variants
- Computed from `--quiet`/`--verbose` CLI flags in main.rs
- Threaded through all mode functions via `_ex` variants
- Quiet mode suppresses informational output (but not errors or requested data)
- Verbose mode enables extra detail in analyse and bench

**Why:** The `--quiet` and `--verbose` flags existed but had no effect.
Quiet mode is essential for CI scripts; verbose mode helps debugging.

---

## ogre doc Command

**Files changed:** `src/modes/doc.rs` (new), `src/modes/preprocess.rs`,
`src/modes/mod.rs`, `src/main.rs`

**What changed:**
- Added `@doc` directive handling in preprocessor collect phase
- `@doc` lines accumulate and attach to the next `@fn` definition
- New `fn_docs: HashMap<String, String>` in Preprocessor
- `process_file_with_docs()` and `process_source_with_docs()` methods
- `generate_docs()` produces markdown from functions and doc comments
- `generate_stdlib_docs()` documents all stdlib modules
- `ogre doc file.bf` generates docs for a file
- `ogre doc --stdlib` generates stdlib reference
- `-o` flag writes to file instead of stdout

**Why:** Documentation generation from source comments is essential for
any language toolchain. The `@doc` directive is simple and familiar.

**Tests added:** 6 unit tests (empty file, with functions, with doc comments,
multi-line docs, stdlib docs, alphabetical sorting).

---

## format --diff Mode

**Files changed:** `Cargo.toml`, `src/modes/format.rs`, `src/main.rs`

**What changed:**
- Added `similar = "2"` dependency for diff generation
- New `diff: bool` field in `FormatOptions`
- `generate_diff()` function produces colored unified diffs
- `--diff` flag in CLI, exits 1 if changes needed
- File is never modified in diff mode

**Why:** Users need to preview formatting changes before applying them.
CI pipelines need to check formatting without modifying files.

**Tests added:** 5 unit tests + 3 CLI integration tests.

---

## CLI Integration Tests

**Files changed:** `Cargo.toml`, `tests/cli_integration.rs` (new)

**What changed:**
- Added `assert_cmd`, `predicates`, `tempfile` dev-dependencies
- 32 CLI integration tests covering all subcommands
- Tests: run, check, format (--check, --diff, in-place), generate, new,
  pack, analyse, bench, stdlib, init, doc, version/help
- Error cases: nonexistent files, unmatched brackets, schema validation

**Why:** Unit tests can't catch CLI wiring bugs. Integration tests verify
the full pipeline from command-line invocation to exit code.

---

## Project Schema Validation

**Files changed:** `src/project.rs`

**What changed:**
- Added `validate()` method called from `load()`
- Validates: name not empty, entry ends with .bf, version not empty,
  test files end with .json, tape_size > 0
- Returns `OgreError::InvalidProject` with descriptive messages

**Why:** Invalid project configurations caused confusing downstream errors.
Early validation provides clear, actionable error messages.

**Tests added:** 8 unit tests + 2 CLI integration tests.

---

## Colored Output

**Files changed:** `src/modes/debug.rs`, `src/modes/start.rs`, `src/main.rs`,
`src/modes/analyse.rs`

**What changed:**
- Debugger: yellow/bold current instruction, cyan pointer cell, red breakpoints
- REPL: cyan pointer cell in memory display, red error messages
- Main: errors printed in red to stderr
- Analyser: colored warnings for cancellation, clear idioms, dead code

**Why:** Color significantly improves readability of output, especially for
test results, error messages, and memory displays.

---

## Help Examples

**Files changed:** `src/main.rs`

**What changed:**
- Added `after_help` with examples to all subcommands
- Examples show common usage patterns
- Format: `ogre <command> [args]` with brief descriptions

**Why:** `--help` output is often the first documentation users see.
Concrete examples demonstrate typical usage patterns.

---

## Analysis Improvements

**Files changed:** `src/modes/analyse.rs`

**What changed:**
- `clear_idiom_count` — counts total `[-]` and `[+]` occurrences
- `cancellation_positions` — returns positions of `+-`, `-+`, `><`, `<>` pairs
- `dead_code_positions` — detects code after unconditional infinite loops
- Verbose output shows counts and positions with colored warnings

**Why:** Position-level reporting helps users locate and fix issues.
Counts help assess code quality at a glance.

**Tests added:** 6 new tests for enhanced detection functions.

---

## Example Projects

**Files created:** `examples/hello/`, `examples/fibonacci/`, `examples/cat/`,
`examples/multifile/`, `examples/stdlib-demo/`

**What each demonstrates:**
- hello: Minimal project with Hello World
- fibonacci: Complex BF algorithm
- cat: Input/output (echo program)
- multifile: @import and @call across files
- stdlib-demo: Using the standard library

**Why:** Example projects teach users the project structure, ogre.toml format,
and test configuration through working code.

---

## Enhanced REPL (rustyline)

**Files changed:** `src/modes/start.rs`, `src/modes/preprocess.rs`, `src/main.rs`,
`Cargo.toml`

**What changed:**
- Replaced `stdin.lock().read_line()` with `rustyline::Editor` for line editing
- Command history persisted to `~/.ogre_history`
- New commands: `:help`, `:load <file>`, `:save <file>`, `:functions`,
  `:peek`, `:dump [n]`, `:reset`, `:quit`
- Project-aware mode: when `ogre.toml` found, preloads all `@fn` definitions
- `@call`/`@import`/`@const`/`@use` directives supported in REPL input
- New preprocessor methods: `collect_functions_from_file()`,
  `collect_functions_from_source()`, `expand_with_functions()`
- Ctrl+C handled gracefully (continues instead of crashing)

**Why:** Line editing and history make the REPL usable for real development.
Project awareness lets users test functions interactively.

**Tests added:** 7 new tests for preprocessor function collection and expansion.

---

## Watch Mode

**Files changed:** `src/modes/run.rs`, `src/main.rs`, `Cargo.toml`

**What changed:**
- Added `notify = "6"` dependency for filesystem events
- New `run_file_watch()` function watches parent directory for changes
- Debounced events (100ms) prevent duplicate re-runs
- Terminal cleared and timestamp shown on each re-run
- Errors displayed inline instead of crashing the watcher
- `--watch` / `-w` flag on `run` command

**Why:** Watch mode eliminates the edit-save-run cycle during development.
Debouncing prevents unnecessary re-runs when editors save in multiple steps.

---

## Glob Patterns in build.include

**Files changed:** `src/project.rs`, `Cargo.toml`

**What changed:**
- Added `glob = "0.3"` dependency
- `resolve_include_files()` now detects `*` and `?` characters in entries
- Patterns like `src/*.bf` and `src/**/*.bf` expanded via `glob::glob()`
- Mixed glob, directory, and file entries work together
- Results sorted alphabetically for deterministic ordering

**Why:** Glob patterns are more flexible than the previous directory-only
or file-only include syntax. Recursive patterns (`**/*.bf`) eliminate the
need to list each subdirectory.

**Tests added:** 5 tests covering star, recursive, question mark, empty matches,
and mixed includes.

---

## Criterion Performance Benchmarks

**Files created:** `benches/interpreter.rs`

**What changed:**
- Added `criterion` to dev-dependencies with html_reports feature
- 9 benchmarks covering: interpretation (hello world, simple multiply,
  compact hello, optimized), IR parsing, optimization, to_bf_string,
  stdlib preprocessing, and C code generation
- `[[bench]]` section added to Cargo.toml

**Why:** Performance benchmarks catch regressions and guide optimization work.
Comparing optimized vs unoptimized interpretation validates the IR pipeline.

---

## Additional CLI Integration Tests

**Files changed:** `tests/cli_integration.rs`

**What changed:**
- Added 16 new CLI tests (37 → 53 total):
  - check: unknown @call, import cycle, missing import
  - pack: with @fn/@call, output file, optimize produces shorter
  - init: creates directories
  - bench: hello world stats
  - analyse: verbose mode, unmatched bracket detection
  - test runner: pass and fail cases
  - run: @fn/@call, stdlib import, --watch flag accepted
  - --no-color flag

**Why:** Comprehensive CLI coverage ensures all subcommands work correctly
end-to-end, including error cases and edge cases.

---

## Source Mapping

**Files changed:** `src/modes/source_map.rs` (new), `src/modes/preprocess.rs`,
`src/modes/interpreter.rs`, `src/modes/debug.rs`, `src/modes/mod.rs`

**What changed:**
- New `SourceLocation` struct: `(file, line, column, function)`
- New `SourceMap` struct: Vec-based lookup by character position
- `build_op_to_char_map()` bridges IR op indices to char positions
- `line_col_map()` for line/column lookup in source strings
- Preprocessor: `collect_with_tracking()` tracks file/line/col during collection
- Preprocessor: `expand_with_tracking()` tags @call expansions with function name
- `process_file_with_map()` and `process_source_with_map()` public methods
- Interpreter: `source_map` and `op_to_char` fields, `set_source_map()`,
  `current_source_location()`, enhanced TapeOverflow errors with location
- Debugger: source location in `print_status()`, `show_instruction()`,
  breakpoint list, new `where` command

**Why:** After preprocessing, position information is lost. Source mapping
lets the debugger and error messages show original file/line/function context
instead of raw positions in the expanded output.

**Tests added:** 12 unit tests for source map types, 6 preprocessor tests.

---

## WASM Compilation Target

**Files changed:** `src/modes/compile_wasm.rs` (new), `src/modes/mod.rs`,
`src/main.rs`, `tests/cli_integration.rs`

**What changed:**
- `generate_wat()` produces WAT (WebAssembly Text Format) from IR Program
- Uses WASI imports (`fd_write`, `fd_read`) for I/O
- Memory layout: tape + scratch space for iov buffers
- Proper block/loop nesting for BF loops
- `compile_to_wasm()` full pipeline: preprocess → optimize → WAT → optional WASM
- `--target` flag on CompileArgs (default "native", also "wasm")
- Falls back to .wat file if `wat2wasm` not installed

**Why:** WASM target allows BF programs to run in browsers and other WASM
runtimes (wasmtime, wasmer). WAT text format is human-readable and debuggable.

**Tests added:** 14 unit tests covering all ops, memory pages, nested loops,
WASI imports, collapsed ops. 2 CLI integration tests.

---

## Dependency Management

**Files changed:** `src/project.rs`, `src/modes/preprocess.rs`, `src/modes/run.rs`,
`src/modes/compile.rs`, `src/modes/bench.rs`, `src/modes/pack.rs`,
`src/modes/check.rs`, `src/modes/debug.rs`, `src/modes/start.rs`, `src/main.rs`,
`tests/cli_integration.rs`

**What changed:**
- `Dependency` struct with `path` and `version` fields
- `dependencies: HashMap<String, Dependency>` field in `OgreProject`
- `resolve_dependencies()` validates paths and ogre.toml existence
- `collect_dependency_functions()` recursively collects @fn definitions
- `process_file_with_deps()` in Preprocessor pre-loads dep functions
- `_with_deps` variants added to compile, bench, pack, check, debug
- All project-aware dispatches in main.rs updated
- Start REPL loads dependency functions alongside project functions

**Why:** Projects need to share brainfuck functions across repositories.
Path-based dependencies (like Cargo path dependencies) are the simplest
starting point. Recursive resolution handles transitive dependencies.

**Tests added:** 12 unit tests (parsing, validation, resolution, collection,
nested deps, edge cases). 5 CLI integration tests (run, check, pack, bench,
missing dep error).

---

## Final Cleanup: Missing Tests and Verbose Test Runner

**Files changed:** `src/modes/interpreter.rs`, `src/modes/test_runner.rs`,
`src/main.rs`, `tests/cli_integration.rs`

**What changed:**
- Added large tape size test (100,000 cells) in interpreter
- Added `--verbose` flag to `ogre test` command for per-test name/result output
- Verbose mode shows `PASS`/`FAIL`/`TIMEOUT` per test instead of dots
- Added regex mismatch and output/regex conflict unit tests in test runner
- Added CLI integration tests: pack preserves semantics, init detects existing
  .bf files, verbose test runner output

**Why:** Fills remaining gaps in the TODOv2.md checklist. Every single item
across all 26 sections is now complete.

**Tests added:** 1 interpreter unit test, 2 test runner unit tests,
3 CLI integration tests. Total: 322 tests passing.
