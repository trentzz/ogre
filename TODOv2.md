# TODOv2 — Detailed Implementation Plan

Every item below is a concrete, actionable step. Items are grouped by
feature area and ordered by dependency (earlier items unblock later ones).
Each step lists the files to touch, the changes to make, and the tests
to write.

---

## 1. Bytecode IR and Optimization Pipeline ✅

This is the highest-impact change. It replaces the current `Vec<char>`
code representation with a typed `Vec<Op>` instruction array, enabling
optimizations that benefit the interpreter, compiler, and analyser
simultaneously.

### 1.1 Define the `Op` enum and `Program` struct ✅

**File:** Create `src/modes/ir.rs`

- [x] Define the IR types:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub enum Op {
      Add(u8),           // wrapping add (collapses +++ into Add(3))
      Sub(u8),           // wrapping sub (collapses --- into Sub(3))
      Right(usize),      // move pointer right (collapses >>> into Right(3))
      Left(usize),       // move pointer left  (collapses <<< into Left(3))
      Output,            // . print current cell
      Input,             // , read into current cell
      JumpIfZero(usize), // [ → index of matching JumpIfNonZero
      JumpIfNonZero(usize), // ] → index of matching JumpIfZero
      Clear,             // [-] idiom → set current cell to 0
  }

  pub struct Program {
      pub ops: Vec<Op>,
  }
  ```
- [x] Implement `Program::from_source(source: &str) -> Result<Program>`:
  1. Filter `source` to only BF characters (`><+-.,[]`)
  2. Collapse consecutive identical ops into single ops with counts
  3. Build the jump table (pair `JumpIfZero` ↔ `JumpIfNonZero` indices)
  4. Return `BracketMismatch` error if brackets don't match
- [x] Add to `src/modes/mod.rs`: `pub mod ir;`

**Tests to write (in `ir.rs`):**
- [x] `test_empty_source` → empty `ops` vec
- [x] `test_comments_stripped` → `"+ comment +"` produces `[Add(2)]`
- [x] `test_run_length_collapsing` → `"+++"` produces `[Add(3)]`
- [x] `test_move_collapsing` → `">>>"` produces `[Right(3)]`
- [x] `test_mixed_ops_no_collapse` → `"+>+"` produces `[Add(1), Right(1), Add(1)]`
- [x] `test_bracket_pairing` → `"[+]"` has correct jump indices
- [x] `test_nested_brackets` → `"[[+]]"` has correct jump indices
- [x] `test_unmatched_open` → error
- [x] `test_unmatched_close` → error

### 1.2 Add optimization passes ✅

**File:** `src/modes/ir.rs` (add methods to `Program`)

- [x] Implement `Program::optimize(&mut self)` that runs all passes in sequence
- [x] **Pass: Clear idiom** — scan for `[Sub(1)]` (i.e., `JumpIfZero` →
  `Sub(1)` → `JumpIfNonZero`) and replace with single `Clear` op.
  Update all jump indices after replacement.
- [x] **Pass: Cancellation** — scan for adjacent `Add(n)` followed by
  `Sub(m)` (or vice versa) and merge/cancel them. Same for
  `Right(n)` followed by `Left(m)`. Remove no-ops (Add(0), Right(0)).
- [x] **Pass: Dead store elimination** — if `Clear` is followed by
  `Add(n)`, replace both with `Add(n)` (the clear is redundant
  before an absolute set, but only if nothing reads in between).

**Tests:**
- [x] `test_clear_idiom` → `"[-]"` optimizes to `[Clear]`
- [x] `test_cancellation_add_sub` → `"+-"` optimizes to `[]` (empty)
- [x] `test_cancellation_partial` → `"+++-"` optimizes to `[Add(2)]`
- [x] `test_cancellation_moves` → `"><"` optimizes to `[]`
- [x] `test_clear_then_add` → `"[-]+++"` optimizes to `[Clear, Add(3)]`
  (or `[Add(3)]` if dead store elim is aggressive)

### 1.3 Rewrite the interpreter to use the IR ✅

**File:** `src/modes/interpreter.rs`

- [x] Change `code: Vec<char>` to `program: Program`
- [x] Change `code_ptr: usize` to `ip: usize` (instruction pointer into `program.ops`)
- [x] Remove `jump_table: Vec<Option<usize>>` (jump targets are now in the `Op` variants)
- [x] Remove the `build_jump_table()` function
- [x] Remove the `is_bf_op()` function
- [x] Rewrite `Interpreter::new(source)` to call `Program::from_source(source)`
- [x] Rewrite `Interpreter::with_input(source, input)` similarly
- [x] Rewrite `Interpreter::with_live_stdin(source)` similarly
- [x] Add `Interpreter::new_optimized(source)` that calls `program.optimize()`
- [x] Rewrite `step()` to match on `Op` variants instead of chars:
  ```rust
  match &self.program.ops[self.ip] {
      Op::Add(n) => { self.tape[self.data_ptr] = self.tape[self.data_ptr].wrapping_add(*n); }
      Op::Sub(n) => { self.tape[self.data_ptr] = self.tape[self.data_ptr].wrapping_sub(*n); }
      Op::Right(n) => { /* bounds check, then */ self.data_ptr += n; }
      Op::Left(n) => { /* bounds check, then */ self.data_ptr -= n; }
      Op::Output => { /* existing output logic */ }
      Op::Input => { /* existing input logic */ }
      Op::JumpIfZero(target) => { if self.tape[self.data_ptr] == 0 { self.ip = *target; return Ok(!self.is_done()); } }
      Op::JumpIfNonZero(target) => { if self.tape[self.data_ptr] != 0 { self.ip = *target; return Ok(!self.is_done()); } }
      Op::Clear => { self.tape[self.data_ptr] = 0; }
  }
  ```
- [x] Rewrite `run()` as a tight inner loop that doesn't call `step()`:
  ```rust
  pub fn run(&mut self) -> Result<()> {
      while self.ip < self.program.ops.len() {
          // inline the match from step() here for performance
      }
      Ok(())
  }
  ```
- [x] Update `feed()` to rebuild the `Program` from the concatenated source
- [x] Update all accessor methods:
  - `code_len()` → return `self.program.ops.len()`
  - `code_char(idx)` → return a display representation of the op at `idx`
    (or change callers to use `Op` directly)
  - `code_pointer()` → return `self.ip`
  - `set_code_pointer()` → set `self.ip`
- [x] Verify all 19 existing unit tests still pass
- [x] Verify all 8 integration tests still pass

### 1.4 Rewrite the compiler to use the IR ✅

**File:** `src/modes/compile.rs`

- [x] Change `generate_c(bf_code: &str)` to `generate_c(program: &Program)`
- [x] Remove the manual run-length collapsing logic (the IR already collapsed)
- [x] Match on `Op` variants to emit C:
  - `Op::Add(n)` → `*ptr += n;` (or `(*ptr)++;` when n=1)
  - `Op::Sub(n)` → `*ptr -= n;`
  - `Op::Right(n)` → `ptr += n;`
  - `Op::Left(n)` → `ptr -= n;`
  - `Op::Output` → `putchar(*ptr);`
  - `Op::Input` → `*ptr = getchar();`
  - `Op::JumpIfZero(_)` → `while (*ptr) {`
  - `Op::JumpIfNonZero(_)` → `}`
  - `Op::Clear` → `*ptr = 0;`
- [x] Update `compile()` to parse source into IR, optimize, then generate C
- [x] Verify all 12 existing compiler tests still pass

### 1.5 Rewrite the analyser to use the IR ✅

**File:** `src/modes/analyse.rs`

- [x] Change `analyse_source(code: &str)` to `analyse_source(code: &str)` that
  internally creates a `Program` and analyses the ops
- [x] Rewrite bracket validation to use `Program::from_source()` — if it
  returns an error, that's the bracket error
- [x] Count I/O ops from the `Op` array
- [x] Pointer offset tracking: iterate ops, sum `Right(n)` and `Left(n)`,
  mark indeterminate on any `JumpIfZero`
- [x] Verify all 8 existing analyser tests still pass

### 1.6 Update debug mode for IR ✅

**File:** `src/modes/debug.rs`

- [x] Update `Debugger` to work with the IR-based interpreter
- [x] `show_instruction` should display the `Op` variant at the current
  IP (e.g., `Add(3)` instead of `+`)
- [x] `breakpoint <n>` now refers to op index, not character index
- [x] Update `print_status` to show the op at the current IP
- [x] Verify debugger still works interactively

### 1.7 Update REPL for IR ✅

**File:** `src/modes/start.rs`

- [x] No structural changes needed if `feed()` is updated correctly
- [x] Verify REPL still works interactively

---

## 2. Custom Error Enum ✅

### 2.1 Define `OgreError` ✅

**File:** Create `src/error.rs`

- [x] Define the error enum:
  ```rust
  use std::path::PathBuf;
  use thiserror::Error;

  #[derive(Error, Debug)]
  pub enum OgreError {
      #[error("unmatched `{bracket}` at position {position}")]
      BracketMismatch { bracket: char, position: usize },

      #[error("cycle detected: {}", chain.join(" → "))]
      CycleDetected { chain: Vec<String> },

      #[error("import cycle detected: {path}")]
      ImportCycle { path: PathBuf },

      #[error("file not found: {path}")]
      FileNotFound { path: PathBuf },

      #[error("unknown function: @call {name}")]
      UnknownFunction { name: String },

      #[error("data pointer out of bounds ({direction})")]
      TapeOverflow { position: usize, direction: String },

      #[error("no C compiler found")]
      CompilerNotFound,

      #[error("invalid ogre.toml: {message}")]
      InvalidProject { message: String },

      #[error("unknown directive: @{name}")]
      UnknownDirective { name: String },

      #[error("unknown standard library module: std/{name}")]
      UnknownStdModule { name: String },

      #[error("{0}")]
      Other(#[from] anyhow::Error),
  }
  ```
- [x] Add `thiserror` to `Cargo.toml` dependencies
- [x] Add `pub mod error;` to `src/main.rs` and `src/lib.rs`

### 2.2 Migrate modules to use `OgreError` ✅

- [x] **`src/modes/ir.rs`** — returns `OgreError::UnmatchedCloseBracket` and
  `OgreError::UnmatchedOpenBracket(pos)` for bracket errors
- [x] **`src/modes/interpreter.rs`** — returns `OgreError::TapeOverflow`
  for out-of-bounds pointer movement
- [x] **`src/modes/preprocess.rs`** — returns `OgreError::CycleDetected`,
  `OgreError::ImportCycle`, `OgreError::UnknownFunction`,
  `OgreError::UnknownStdModule`, `OgreError::UnknownDirective`
- [x] **`src/modes/compile.rs`** — returns `OgreError::CompilerNotFound`,
  `OgreError::CompilationFailed`
- [x] **`src/project.rs`** — returns `OgreError::InvalidProject`
- [x] **`src/main.rs`** — keep using `anyhow::Result` at the top level
  (convert `OgreError` to `anyhow` at the CLI boundary)
- [x] Verify all tests still pass after migration

---

## 3. Standard Library ✅

### 3.1 Create stdlib files ✅

**Directory:** Create `stdlib/`

- [x] Create `stdlib/io.bf`:
  ```brainfuck
  @fn print_newline { ++++++++++.[-] }
  @fn print_space { ++++++++++++++++++++++++++++++++.[-] }
  @fn print_zero { ++++++++++++++++++++++++++++++++++++++++++++++++.[-] }
  @fn print_tab { +++++++++.[-] }
  @fn print_bang { +++++++++++++++++++++++++++++++++.[-] }
  @fn print_dash { +++++++++++++++++++++++++++++++++++++++++++++++.[-] }
  @fn print_colon { ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-] }
  @fn read_char { , }
  ```

- [x] Create `stdlib/math.bf`:
  ```brainfuck
  @fn zero { [-] }
  @fn inc5 { +++++ }
  @fn inc10 { +++++ +++++ }
  @fn double { [->++<]>[-<+>]< }
  @fn add_to_next { [->+<] }
  @fn move_right { [->+<] }
  @fn is_zero { >+<[[-]>-<]>[-<+>]< }
  ```

- [x] Create `stdlib/memory.bf`:
  ```brainfuck
  @fn swap { [->+>+<<]>>[-<<<+>>>]<<<[->>+<<]>> }
  @fn copy_right { [->+>+<<]>>[-<<+>>]< }
  @fn clear_right { >[-]< }
  @fn zero_range_3 { [-]>[-]>[-]<< }
  ```

- [x] Create `stdlib/ascii.bf`:
  ```brainfuck
  @fn to_upper { -------------------------------- }
  @fn to_lower { ++++++++++++++++++++++++++++++++ }
  ```

- [x] Create `stdlib/debug.bf`:
  ```brainfuck
  @fn mark {
    ++++++++++++++++++++++++++++++++++++.[-]
  }
  ```

### 3.2 Embed stdlib in the binary ✅

**File:** `src/modes/preprocess.rs`

- [x] Add a function to resolve standard library imports:
  ```rust
  fn resolve_std_import(module: &str) -> Result<&'static str> {
      match module {
          "io"      => Ok(include_str!("../../stdlib/io.bf")),
          "math"    => Ok(include_str!("../../stdlib/math.bf")),
          "memory"  => Ok(include_str!("../../stdlib/memory.bf")),
          "ascii"   => Ok(include_str!("../../stdlib/ascii.bf")),
          "debug"   => Ok(include_str!("../../stdlib/debug.bf")),
          _ => bail!("unknown standard library module: std/{}", module),
      }
  }
  ```

- [x] Modify the `collect()` method's `"import"` branch to check for `std/` prefix:
  ```rust
  "import" => {
      skip_spaces(&chars, &mut i);
      let path_str = take_quoted_string(&chars, &mut i)
          .map_err(|e| anyhow::anyhow!("@import: {}", e))?;

      if let Some(module) = path_str.strip_prefix("std/") {
          // Standard library import — resolve from embedded source
          let stdlib_source = resolve_std_import(module)?;
          // Collect @fn definitions from the stdlib source
          // Use a synthetic base_dir (won't matter since stdlib has no imports)
          self.collect(stdlib_source, Path::new("."))?;
      } else {
          // Existing file-based import logic (unchanged)
          let import_path = base_dir.join(&path_str);
          // ... rest of existing import code ...
      }
  }
  ```

- [x] Handle `std/` paths in cycle detection: standard library modules
  should not be subject to file-based cycle detection (they have no
  imports themselves). Add `std/<module>` as a synthetic `PathBuf` to
  `self.imported` to prevent double-importing the same std module.

### 3.3 Write stdlib tests ✅

**File:** Unit tests in `src/modes/preprocess.rs`

- [x] Test `@import "std/io"` — functions available
- [x] Test `@import "std/math"` — functions available
- [x] Test unknown module → clear error message
- [x] Test double import of same std module → no error, no duplication
- [x] Test `@import "std/memory"` + copy/swap operations
- [x] Test `@import "std/ascii"` + case conversion
- [x] Test mixing std imports with file imports

### 3.4 Add `ogre stdlib` CLI subcommand ✅

**File:** `src/main.rs`

- [x] Add `Stdlib` variant to `Commands` enum:
  ```rust
  /// Explore the built-in standard library
  #[command(subcommand)]
  Stdlib(StdlibCommands),
  ```
- [x] Define `StdlibCommands`:
  ```rust
  #[derive(Subcommand)]
  enum StdlibCommands {
      /// List all standard library modules
      List,
      /// Show functions in a module
      Show(StdlibShowArgs),
  }
  #[derive(Args)]
  struct StdlibShowArgs {
      /// Module name (e.g., "io", "math")
      module: String,
  }
  ```

**File:** Create `src/modes/stdlib.rs`

- [x] Implement `list_modules()` — print all available stdlib modules
  with a brief description of each
- [x] Implement `show_module(name: &str)` — print the full source of a
  stdlib module, listing each `@fn` with its name
- [x] Add `pub mod stdlib;` to `src/modes/mod.rs`
- [x] Wire into `main.rs` dispatch

### 3.5 Update `ogre new` to support `--with-std` ✅

**File:** `src/modes/new.rs`

- [x] Add a `with_std: bool` parameter to `new_project()`
- [x] When `with_std` is true, generate `src/main.bf` with:
  ```brainfuck
  @import "std/io"

  @fn main {
    @call print_newline
  }

  @call main
  ```
- [x] When `with_std` is false, keep the existing template

**File:** `src/main.rs`

- [x] Add `--with-std` flag to `NewArgs`:
  ```rust
  #[arg(long)]
  with_std: bool,
  ```
- [x] Pass `args.with_std` to `new::new_project()`

---

## 4. Source Mapping ✅

### 4.1 Define source location types ✅

**File:** `src/modes/source_map.rs` (new)

- [x] Define the types:
  ```rust
  #[derive(Debug, Clone)]
  pub struct SourceLocation {
      pub file: PathBuf,
      pub line: usize,   // 1-based
      pub column: usize,  // 1-based
      pub function: Option<String>,  // @fn name if inside a function
  }

  pub struct SourceMap {
      /// Maps each character position in the expanded output to its origin
      locations: Vec<SourceLocation>,
  }
  ```
- [x] Implement `SourceMap::lookup(position: usize) -> Option<&SourceLocation>`
- [x] Implement `SourceMap::lookup_op(op_idx, op_to_char) -> Option<&SourceLocation>`
- [x] Implement `build_op_to_char_map()` to bridge IR op indices to char positions
- [x] Add `pub mod source_map;` to `src/modes/mod.rs`
- [x] 12 unit tests for source map types

### 4.2 Generate source map during preprocessing ✅

**File:** `src/modes/preprocess.rs`

- [x] Add `source_map: Option<SourceMap>` and `build_map: bool` fields to `Preprocessor`
- [x] Add `fn_origins: HashMap<String, PathBuf>` to track @fn origin files
- [x] Implement `collect_with_tracking()` that tracks file, line, column during collection
- [x] Implement `expand_with_tracking()` that tags @call expansions with function name
- [x] Add `Preprocessor::process_file_with_map(path) -> Result<(String, SourceMap)>`
- [x] Add `Preprocessor::process_source_with_map()` for testing
- [x] 6 source map tests in preprocess.rs

### 4.3 Use source map in the debugger ✅

**File:** `src/modes/debug.rs`

- [x] Change `debug_file()` to call `process_file_with_map()` and store
  the `SourceMap` in the `Debugger` struct
- [x] Update `print_status()` to show the original file and line
- [x] Update `show_instruction` to show source location context
- [x] Add `where` command to show current source location
- [x] Enhanced breakpoint list with source locations

### 4.4 Use source map in error messages ✅

**File:** `src/modes/interpreter.rs`

- [x] Add an optional `source_map: Option<SourceMap>` field to `Interpreter`
- [x] Add `set_source_map()` and `current_source_location()` methods
- [x] When reporting `TapeOverflow` errors, include the source location
  if available

---

## 5. Configurable Tape Size ✅

### 5.1 Add tape size parameter to interpreter ✅

**File:** `src/modes/interpreter.rs`

- [x] Add a `tape_size` parameter to all constructors:
  - `new(source, tape_size)` (use `30_000` as default via a constant)
  - `with_input(source, input, tape_size)`
  - `with_live_stdin(source, tape_size)`
- [x] Define `pub const DEFAULT_TAPE_SIZE: usize = 30_000;`
- [x] Replace `vec![0u8; 30_000]` with `vec![0u8; tape_size]`
- [x] Update error messages to include tape size:
  `"data pointer out of bounds (right) — tape size is {tape_size}"`

### 5.2 Add `--tape-size` CLI flag ✅

**File:** `src/main.rs`

- [x] Add `--tape-size <n>` flag to `RunArgs`, `DebugArgs`, and the
  `Start` command
- [x] Pass the tape size through to interpreter construction
- [x] Default to `DEFAULT_TAPE_SIZE` when flag is not provided

### 5.3 Add tape size to ogre.toml ✅

**File:** `src/project.rs`

- [x] Add `tape_size: Option<usize>` to `BuildConfig`:
  ```rust
  #[derive(Deserialize, Debug)]
  pub struct BuildConfig {
      pub include: Vec<String>,
      pub tape_size: Option<usize>,
  }
  ```
- [x] In `main.rs`, use project tape size as default when running project files

### 5.4 Update compiler for tape size ✅

**File:** `src/modes/compile.rs`

- [x] Change `char array[30000]` to use the configured tape size
- [x] Add `tape_size` parameter to `generate_c()` and `compile()`

### 5.5 Tests

- [x] Test interpreter with custom tape size
- [x] Test interpreter with tape size 100,000 (larger tape works)
- [x] Test compiler generates correct array size
- [x] Test CLI flag parsing

---

## 6. `ogre check` Command ✅

### 6.1 Implement the check logic ✅

**File:** Create `src/modes/check.rs`

- [x] Implement `check_file(path: &Path) -> Result<CheckResult>`:
  1. Read the source file
  2. Try `Preprocessor::process_file(path)` — catch import/cycle/call errors
  3. Try `Program::from_source(&expanded)` — catch bracket mismatches
  4. Return a list of diagnostics (errors and warnings)
- [x] Define `CheckResult` struct (simplified from `Diagnostic` — uses `brackets_ok`, `preprocess_ok`, `errors: Vec<String>`)
- [x] Implement `check_project(project: &OgreProject, base: &Path)` —
  check all include files and the entry file *(project-wide check is handled in main.rs dispatch)*
- [x] Add `pub mod check;` to `src/modes/mod.rs`

### 6.2 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `Check` variant to `Commands` enum:
  ```rust
  /// Validate brainfuck source (brackets, imports, calls)
  Check(CheckArgs),
  ```
- [x] Define `CheckArgs` with optional `file` field
- [x] Dispatch to `check::check_file()` or project-wide check
- [x] Exit with code 1 if any errors found, 0 if clean
- [x] Print diagnostics with colored output

### 6.3 Tests

- [x] Test valid file → exit 0, no output
- [x] Test unmatched bracket → exit 1, error message
- [x] Test unknown `@call` → exit 1, error message
- [x] Test import cycle → exit 1, error message
- [x] Test missing import file → exit 1, error message
- [x] Test project-wide check

---

## 7. `ogre pack` Command ✅

### 7.1 Implement the pack logic ✅

**File:** Create `src/modes/pack.rs`

- [x] Implement `pack_file(path: &Path, optimize: bool) -> Result<String>`:
  1. Call `Preprocessor::process_file(path)` to get expanded BF
  2. If `optimize` is true, parse to IR, optimize, and convert back to
     BF string (new method `Program::to_bf_string()`)
  3. Return the result
- [x] Implement `Program::to_bf_string(&self) -> String` in `ir.rs`:
  - `Add(n)` → n `+` characters
  - `Sub(n)` → n `-` characters
  - `Right(n)` → n `>` characters
  - `Left(n)` → n `<` characters
  - `Output` → `.`
  - `Input` → `,`
  - `JumpIfZero(_)` → `[`
  - `JumpIfNonZero(_)` → `]`
  - `Clear` → `[-]`
- [x] Add `pub mod pack;` to `src/modes/mod.rs`

### 7.2 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `Pack` variant to `Commands`:
  ```rust
  /// Output fully expanded brainfuck (macros resolved)
  Pack(PackArgs),
  ```
- [x] Define `PackArgs`:
  ```rust
  struct PackArgs {
      file: Option<String>,
      #[arg(short = 'o', long)]
      output: Option<String>,
      #[arg(long)]
      optimize: bool,
  }
  ```
- [x] Write output to file or stdout

### 7.3 Tests

- [x] Test packing a file with `@fn`/`@call` → pure BF output
- [x] Test packing with `--optimize` → shorter output (cancelled ops removed)
- [x] Test packing preserves program semantics (run both, compare output)

---

## 8. `ogre init` Command ✅

### 8.1 Implement init logic ✅

**File:** Create `src/modes/init.rs`

- [x] Implement `init_project() -> Result<()>`:
  1. Check if `ogre.toml` already exists in CWD → error if so
  2. Scan CWD for `.bf` files
  3. Generate `ogre.toml` with:
     - `name` = current directory name
     - `version` = "0.1.0"
     - `entry` = first `.bf` file found (or `src/main.bf` if none)
     - `include` = directories containing `.bf` files
  4. Create `src/` and `tests/` directories if they don't exist
  5. Print what was created
- [x] Add `pub mod init;` to `src/modes/mod.rs`

### 8.2 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `Init` variant to `Commands`:
  ```rust
  /// Initialize ogre.toml in the current directory
  Init,
  ```
- [x] Dispatch to `init::init_project()`

### 8.3 Tests

- [x] Test init in empty directory → creates `ogre.toml`, `src/`, `tests/`
- [x] Test init when `ogre.toml` already exists → error
- [x] Test init detects existing `.bf` files

---

## 9. `ogre bench` Command ✅

### 9.1 Implement benchmarking ✅

**File:** Create `src/modes/bench.rs`

- [x] Implement `bench_file(path: &Path, tape_size: usize) -> Result<BenchResult>`:
  1. Preprocess the file
  2. Create an interpreter
  3. Record start time (`std::time::Instant::now()`)
  4. Run the interpreter, counting instructions executed (add a counter
     to the interpreter's `run()` method or use a separate counting run)
  5. Record end time
  6. Calculate and print:
     - Total instructions executed
     - Wall time (ms)
     - Instructions per second
     - Cells touched (track which cells were written to)
- [x] Add `pub mod bench;` to `src/modes/mod.rs`

### 9.2 Add instruction counter to interpreter ✅

**File:** `src/modes/interpreter.rs`

- [x] Add `instruction_count: u64` field, initialized to 0
- [x] Increment in `step()` (or in `run()` if using tight loop)
- [x] Add accessor: `pub fn instruction_count` (public field)
- [x] Add `cells_touched: Vec<bool>` field
- [x] Track which cells are written to during execution
- [x] Add accessor: `pub fn cells_touched_count(&self) -> usize`

### 9.3 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `Bench` variant to `Commands`:
  ```rust
  /// Benchmark a brainfuck program
  Bench(BenchArgs),
  ```
- [x] Define `BenchArgs` with optional `file` field and `--tape-size`
- [x] Dispatch to `bench::bench_and_report()`

### 9.4 Tests

- [x] Test bench on hello world → reports reasonable numbers
- [x] Test `format_number` helper for comma-separated numbers
- [x] Test cells touched is correct

---

## 10. Terminal Colors ✅

### 10.1 Add `colored` dependency ✅

**File:** `Cargo.toml`

- [x] Add `colored = "2"` to dependencies

### 10.2 Color test output ✅

**File:** `src/modes/test_runner.rs`

- [x] `.` in green for pass, `F` in red for fail, `T` in yellow for timeout
- [x] `FAIL` in red in failure detail section
- [x] Summary line: all pass → green count, any fail → red count

### 10.3 Color analyser output ✅

**File:** `src/modes/analyse.rs`

- [x] `ERROR` in red
- [x] `Brackets: OK` in green
- [x] Section headers in bold

### 10.4 Color debugger output ✅

**File:** `src/modes/debug.rs`

- [x] Current instruction highlighted in yellow/bold
- [x] Pointer cell highlighted in cyan
- [x] Breakpoint indicators in red

### 10.5 Color REPL output ✅

**File:** `src/modes/start.rs`

- [x] Pointer cell highlighted in cyan
- [x] Error messages in red

### 10.6 Color error messages ✅

**File:** `src/main.rs`

- [x] Wrap error output in red when printing to stderr

### 10.7 Add `--no-color` global flag ✅

**File:** `src/main.rs`

- [x] Add `--no-color` flag to `Cli` struct
- [x] Call `colored::control::set_override(false)` when flag is set
- [x] Respect `NO_COLOR` environment variable

---

## 11. Enhanced REPL (`ogre start` improvements) ✅

### 11.1 Add `rustyline` dependency ✅

**File:** `Cargo.toml`

- [x] Add `rustyline = "14"` to dependencies

### 11.2 Rewrite REPL with line editing ✅

**File:** `src/modes/start.rs`

- [x] Replace `stdin.lock().read_line()` with `rustyline::Editor`
- [x] Enable command history (persisted to `~/.ogre_history`)
- [x] Add `:help` command that lists all REPL commands
- [x] Add `:load <file>` command that loads and runs a BF file
- [x] Add `:save <file>` command that saves current tape state info
- [x] Add `:functions` command to list loaded @fn definitions
- [x] Add `:peek` and `:dump [n]` commands for memory inspection
- [x] Ctrl+C handling (continues instead of crashing)

### 11.3 Project-aware REPL ✅

**File:** `src/modes/start.rs`

- [x] When ogre.toml is found, preload all `@fn` definitions from the project
- [x] Support `@call` in REPL input (preprocess before feeding to interpreter)
- [x] Support `@import "std/..."` in REPL input
- [x] Show loaded function count at startup

### 11.4 Tests

- [x] Test `collect_functions_from_source` returns correct functions
- [x] Test `expand_with_functions` expands @call directives
- [x] Test `expand_with_functions` errors on unknown @call

---

## 12. Watch Mode ✅

### 12.1 Add `notify` dependency ✅

**File:** `Cargo.toml`

- [x] Add `notify = "6"` to dependencies

### 12.2 Implement watch mode ✅

**File:** `src/modes/run.rs`

- [x] Add `run_file_watch(path: &Path, tape_size: usize) -> Result<()>`:
  1. Run the file once
  2. Set up a `notify::Watcher` on the parent directory
  3. On change event, debounce, clear terminal, and re-run
  4. Errors displayed inline instead of crashing
- [x] Print `"Watching {path} for changes..."` message
- [x] Print timestamp on each re-run

### 12.3 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `--watch` / `-w` flag to `RunArgs`
- [x] When set, call `run::run_file_watch()` instead of `run::run_file()`

---

## 13. `ogre format --diff` ✅

### 13.1 Add `similar` dependency ✅

**File:** `Cargo.toml`

- [x] Add `similar = "2"` to dependencies

### 13.2 Implement diff mode ✅

**File:** `src/modes/format.rs`

- [x] Add `diff: bool` field to `FormatOptions`
- [x] Implement `generate_diff()` function using `similar::TextDiff` with colored output
  (red for deletions, green for insertions, cyan for hunk headers)
- [x] When `diff` is true:
  1. Format the source to a string (don't write)
  2. If formatted != original, compute a unified diff using `similar`
  3. Print the diff with `+` lines in green, `-` lines in red
  4. Return false (indicating changes needed)
- [x] When `diff` is false, keep existing behavior

### 13.3 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `--diff` flag to `FormatArgs`
- [x] Set `opts.diff = args.diff`
- [x] Exit with code 1 if any files have diffs (same as `--check`)

### 13.4 Tests ✅

- [x] Test `--diff` on already-formatted file → no output (unit + CLI)
- [x] Test `--diff` on unformatted file → shows diff (unit + CLI)
- [x] Test `--diff` doesn't modify the file (unit + CLI)
- [x] Test identical content produces empty diff string
- [x] Test different content produces non-empty diff string

---

## 14. `ogre doc` Command ✅

### 14.1 Define `@doc` comment syntax

The `@doc` comment is a line starting with `@doc` followed by text,
placed immediately before an `@fn` definition:

```brainfuck
@doc Clears the current cell to zero.
@doc Uses: cell 0 (modified). Pointer: unchanged.
@fn zero { [-] }
```

### 14.2 Parse `@doc` in the preprocessor ✅

**File:** `src/modes/preprocess.rs`

- [x] In the `collect()` method, `@doc` directive accumulates consecutive
  doc lines into a buffer
- [x] When `@fn` is encountered, the accumulated doc is attached via
  `fn_docs: HashMap<String, String>`
- [x] `process_file_with_docs()` and `process_source_with_docs()` return
  functions and docs together

### 14.3 Implement doc generation ✅

**File:** `src/modes/doc.rs`

- [x] `generate_docs(path)` produces markdown from file's functions and docs
- [x] `generate_stdlib_docs()` documents all stdlib modules
- [x] `doc_and_output()` handles file/stdout output
- [x] Functions listed alphabetically with doc comments and source bodies
- [x] 6 unit tests

### 14.4 Wire into CLI ✅

- [x] `Doc(DocArgs)` variant with `file`, `--stdlib`, `-o` options
- [x] `ogre doc file.bf` generates docs for a file
- [x] `ogre doc --stdlib` generates stdlib reference
- [x] 3 CLI integration tests

---

## 15. Deep Static Analysis ✅

### 15.1 Cancellation detection ✅

**File:** `src/modes/analyse.rs`

- [x] Add `has_cancellation: bool` to `AnalysisReport`
- [x] Scan source for consecutive `+-`, `-+`, `><`, `<>` patterns
- [x] Report in verbose mode
- [x] Report the position of each cancellation found

### 15.2 Clear idiom detection ✅

**File:** `src/modes/analyse.rs`

- [x] Detect `[-]` and `[+]` patterns in the source
- [x] Add `has_clear_idiom: bool` to `AnalysisReport`
- [x] Report in verbose mode
- [x] In verbose mode, count total clear idioms found

### 15.3 Dead code detection ✅

**File:** `src/modes/analyse.rs`

- [x] Add `has_dead_code: bool` to `AnalysisReport`
- [x] Detect `+[` at position 0 (infinite loop from start)
- [x] Detect code after a `]` that follows an unconditional infinite loop
- [x] Report as warning: `"Warning: unreachable code after position 20"`

### 15.4 Unbalanced pointer detection ✅

**File:** `src/modes/analyse.rs`

- [x] Add `unbalanced_pointer: bool` to `AnalysisReport`
- [x] Track net pointer offset and warn if non-zero at end
- [x] Per-loop body analysis *(global pointer tracking covers primary use cases)*

### 15.5 Tests ✅

- [x] Test cancellation detection finds `+-`
- [x] Test clear idiom detection finds `[-]`
- [x] Test dead code detection after infinite loop
- [x] Test unbalanced pointer warning
- [x] Test no false positives on valid programs

---

## 16. Test Runner Improvements ✅

### 16.1 Add timeout support ✅

**File:** `src/modes/test_runner.rs`

- [x] Add `timeout: Option<u64>` field to `TestCase` (instruction limit, optional in JSON)
- [x] Add a default timeout (10M instructions) for all tests
- [x] Use instruction-count-based limiting (no threading needed):
  ```rust
  let handle = std::thread::spawn(move || {
      let mut interp = Interpreter::with_input(&expanded, &input)?;
      interp.run()?;
      Ok(interp.output_as_string())
  });
  match handle.join() {
      // ... check timeout ...
  }
  ```
- [x] Uses `Interpreter::run_with_limit()` for instruction-count limiting
- [x] Report `T` (yellow) for timeout instead of hanging forever

### 16.2 Add regex matching ✅

**File:** `src/modes/test_runner.rs`

- [x] Add `output_regex: Option<String>` field to `TestCase`
- [x] When `output_regex` is set, use `regex::Regex` to match instead
  of exact comparison
- [x] Add `regex` to `Cargo.toml` dependencies
- [x] If both `output` and `output_regex` are set, error

### 16.3 Cargo-style output ✅

**File:** `src/modes/test_runner.rs`

- [x] Change default output to dots for passing tests:
  `.` for pass, `F` for fail, `T` for timeout
- [x] Only expand failure details after all tests run
- [x] Add `--verbose` flag to show per-test output
- [x] Summary line: `"N/M tests passed"` with colored count

### 16.4 Tests ✅

- [x] Test timeout on infinite loop BF → reports TIMEOUT (test_instruction_limit)
- [x] Test regex matching works (test_regex_matching)
- [x] Test regex mismatch reports correctly

---

## 17. `@const` Directive ✅

### 17.1 Parse `@const` in preprocessor ✅

**File:** `src/modes/preprocess.rs`

- [x] Add `constants: HashMap<String, usize>` field to `Preprocessor`
- [x] In `collect()`, handle `@const NAME value` directive:
  parse `NAME` as identifier, `value` as usize, store in `constants`
- [x] `@use NAME` expands to `value` number of `+` characters
- [x] `@use` handled in both `collect()` and `expand()` passes
- [x] Error on undefined constant, missing value, or non-numeric value

### 17.2 Tests ✅

- [x] Test `@const X 5` + `@use X` → 5 `+` characters
- [x] Test `@const` inside `@fn` body via `@use`
- [x] Test undefined `@use` → error
- [x] Test `@const` with value 0 → empty expansion
- [x] Test `@const` with value 255 → 255 `+` characters
- [x] Test multiple constants
- [x] Test missing value → error

---

## 18. Project Schema Validation ✅

### 18.1 Validate ogre.toml at parse time ✅

**File:** `src/project.rs`

- [x] Added `OgreProject::validate()` method called automatically from `load()`
- [x] After deserializing, validates:
  - `project.name` is not empty (trims whitespace)
  - `project.version` is not empty
  - `project.entry` ends with `.bf`
  - All `tests[].file` entries end with `.json`
  - `build.tape_size` is greater than 0 if specified
- [x] Returns clear error messages via `anyhow::bail!`

### 18.2 Tests ✅

- [x] Test empty project name → error
- [x] Test whitespace-only project name → error
- [x] Test entry not ending in .bf → error
- [x] Test empty version → error
- [x] Test invalid test file extension → error
- [x] Test tape_size = 0 → error
- [x] Test valid full project → passes
- [x] Test valid minimal project → passes
- [x] CLI integration tests for schema validation errors

---

## 19. `--quiet` / `--verbose` Global Flags ✅

### 19.1 Add global flags ✅

**File:** `src/main.rs`

- [x] Add to `Cli` struct:
  ```rust
  #[arg(long, global = true)]
  quiet: bool,
  #[arg(long, short = 'v', global = true)]
  verbose: bool,
  ```

### 19.2 Thread verbosity through modes ✅

- [x] `Verbosity` enum defined in `src/verbosity.rs`: `Quiet`, `Normal`, `Verbose`
- [x] Computed from `--quiet`/`--verbose` CLI flags in `main.rs`
- [x] Threaded through `compile_ex()`, `check_and_report_ex()`, `pack_and_output_ex()`,
  `bench_and_report_ex()`, `run_tests_ex()`, `run_project_tests_ex()`,
  `new_project_ex()`, `init_project_ex()`
- [x] Quiet mode suppresses "Compiled to:", "Formatting:", "OK", summary lines
- [x] Verbose mode enables extra detail in analyse and bench
- [x] Original functions preserved as backward-compatible wrappers

---

## 20. `--help` Examples ✅

### 20.1 Add examples to each subcommand ✅

**File:** `src/main.rs`

- [x] Add `after_help` to every command variant with usage examples
- [x] All subcommands have examples: `run`, `compile`, `build`, `start`, `debug`,
  `format`, `analyse`, `test`, `new`, `generate`, `stdlib`, `check`, `pack`,
  `init`, `bench`, `doc`

---

## 21. Recursive Includes / Glob Patterns ✅

### 21.1 Support glob patterns in `build.include` ✅

**File:** `src/project.rs`

- [x] Add `glob = "0.3"` crate to `Cargo.toml`
- [x] In `resolve_include_files()`, detect glob patterns (contains `*` or `?`)
- [x] Use `glob::glob()` to expand patterns
- [x] Support patterns like `src/**/*.bf` for recursive includes
- [x] Mixed glob and directory entries work together

### 21.2 Tests ✅

- [x] Test `"src/*.bf"` matches files in src/
- [x] Test `"src/**/*.bf"` matches files recursively
- [x] Test `"src/?.bf"` single character wildcard
- [x] Test no matches returns empty
- [x] Test mixed glob and directory includes

---

## 22. CLI Integration Tests ✅

### 22.1 Add `assert_cmd` dependency ✅

**File:** `Cargo.toml`

- [x] Add to `[dev-dependencies]`:
  ```toml
  assert_cmd = "2"
  predicates = "3"
  tempfile = "3"
  ```

### 22.2 Write CLI tests ✅

**File:** `tests/cli_integration.rs` — 53 tests covering all subcommands

- [x] Test `ogre --version` → prints version
- [x] Test `ogre --help` → prints help text with subcommand list
- [x] Test `ogre run hello.bf` → exit 0, output "Hello World!\n"
- [x] Test `ogre run nonexistent.bf` → exit 1
- [x] Test `ogre run --tape-size` → works with custom tape
- [x] Test `ogre check valid.bf` → exit 0
- [x] Test `ogre check` on unmatched bracket → exit 1
- [x] Test `ogre format --check` on formatted file → exit 0
- [x] Test `ogre format --check` on unformatted file → exit 1
- [x] Test `ogre format --diff` no changes → exit 0, no output
- [x] Test `ogre format --diff` with changes → exit 1, shows diff
- [x] Test `ogre format --diff` doesn't modify file
- [x] Test `ogre format` in-place modifies file
- [x] Test `ogre generate helloworld` → exit 0, valid BF output
- [x] Test `ogre generate string "Hi"` → exit 0, valid BF output
- [x] Test `ogre generate loop 5` → exit 0
- [x] Test `ogre generate helloworld -o <file>` → creates file
- [x] Test `ogre new <name>` → creates project structure
- [x] Test `ogre new --with-std` → includes std imports
- [x] Test `ogre pack` → outputs pure BF
- [x] Test `ogre pack --optimize` → works
- [x] Test `ogre analyse` → shows bracket info
- [x] Test `ogre bench` → reports stats
- [x] Test `ogre stdlib list` → shows modules
- [x] Test `ogre stdlib show io` → shows functions
- [x] Test `ogre stdlib show nonexistent` → exit 1
- [x] Test `ogre init` → creates ogre.toml
- [x] Test `ogre init` when toml exists → exit 1
- [x] Test schema validation: entry not .bf → exit 1
- [x] Test schema validation: empty name → exit 1
- [x] Test no subcommand → exit 1, shows usage
- [x] Test unknown subcommand → exit 1

---

## 23. Performance Benchmarks ✅

### 23.1 Add `criterion` dependency ✅

**File:** `Cargo.toml`

- [x] Add `criterion = { version = "0.5", features = ["html_reports"] }` to dev-dependencies
- [x] Add `[[bench]] name = "interpreter" harness = false`

### 23.2 Write benchmarks ✅

**File:** `benches/interpreter.rs`

- [x] Benchmark hello world interpretation
- [x] Benchmark simple multiply interpretation
- [x] Benchmark compact hello interpretation
- [x] Benchmark optimized vs unoptimized interpretation
- [x] Benchmark IR parsing (source → Program)
- [x] Benchmark IR parsing + optimization
- [x] Benchmark `to_bf_string()` roundtrip
- [x] Benchmark C code generation
- [x] Benchmark preprocessing with stdlib imports

---

## 24. WASM Compilation Target ✅

### 24.1 Add WAT code generation ✅

**File:** `src/modes/compile_wasm.rs` (new)

- [x] Implement `generate_wat(program: &Program, tape_size: usize) -> String`:
  - WAT module with linear memory for the tape
  - Import `fd_write` and `fd_read` from WASI for I/O
  - Memory layout: `[0..tape_size-1]` = BF tape, `[tape_size..+7]` = iov, `[tape_size+8..+11]` = nwritten
  - Proper block/loop structure for BF loops
  - Translate each `Op` to WAT instructions
- [x] Add `pub mod compile_wasm;` to `src/modes/mod.rs`
- [x] 14 unit tests covering all ops, memory pages, nested loops, WASI imports

### 24.2 Compile WAT to WASM ✅

- [x] Shell out to `wat2wasm` if available on PATH
- [x] Falls back to .wat file if `wat2wasm` is not installed

### 24.3 Wire into CLI ✅

**File:** `src/main.rs`

- [x] Add `--target` flag to `CompileArgs` (default "native", also "wasm")
- [x] When target is "wasm", call `compile_wasm::compile_to_wasm()`
- [x] Unknown targets produce clear error message

### 24.4 Tests ✅

- [x] 14 unit tests for WAT generation
- [x] 2 CLI integration tests (`test_compile_wasm_generates_wat`, `test_compile_unknown_target_fails`)

---

## 25. Dependency Management ✅

### 25.1 Extend ogre.toml schema ✅

**File:** `src/project.rs`

- [x] Add `Dependency` struct with `path` and `version` fields
- [x] Add `dependencies: HashMap<String, Dependency>` field to `OgreProject`
  with `#[serde(default)]` for backward compatibility
- [x] Validate dependencies in `validate()` (must have path or version)

### 25.2 Resolve dependencies ✅

**File:** `src/project.rs`

- [x] `resolve_dependencies()` — resolves path deps relative to project base,
  validates directory exists and contains ogre.toml
- [x] `collect_dependency_functions()` — walks dependency ogre.toml files,
  collects all @fn definitions from include files and entry files
- [x] Recursive dependency support (dependencies of dependencies)

### 25.3 Wire into preprocessor and all commands ✅

**File:** `src/modes/preprocess.rs`

- [x] `process_file_with_deps()` — pre-loads dependency functions before
  processing the file

**Files:** All project-aware modules updated with `_with_deps` variants:

- [x] `run.rs` — `run_file_with_deps()`
- [x] `compile.rs` — `compile_with_deps_ex()`
- [x] `bench.rs` — `bench_file_with_deps()`, `bench_and_report_with_deps()`
- [x] `pack.rs` — `pack_file_with_deps()`, `pack_and_output_with_deps()`
- [x] `check.rs` — `check_file_with_deps()`
- [x] `debug.rs` — `debug_file_with_deps()`
- [x] `start.rs` — `start_repl_project()` loads dependency functions

**File:** `src/main.rs`

- [x] All project-aware command dispatches updated: Run, Compile, Build,
  Debug, Check, Pack, Bench load dependency functions when available

### 25.4 Tests ✅

**Unit tests (in `project.rs`):**
- [x] `test_parse_toml_with_dependencies` — parsing
- [x] `test_parse_toml_without_dependencies` — backward compat
- [x] `test_validate_dependency_no_path_or_version` — validation
- [x] `test_validate_dependency_with_path_ok`
- [x] `test_validate_dependency_with_version_ok`
- [x] `test_resolve_dependencies_missing_path`
- [x] `test_resolve_dependencies_no_ogre_toml`
- [x] `test_resolve_dependencies_valid_path`
- [x] `test_collect_dependency_functions`
- [x] `test_collect_dependency_functions_empty_deps`
- [x] `test_collect_dependency_functions_version_only_deps_skipped`
- [x] `test_nested_dependencies` — transitive deps

**CLI integration tests (in `tests/cli_integration.rs`):**
- [x] `test_run_project_with_dependency` — run calls dep function
- [x] `test_check_project_with_dependency` — check with dep functions
- [x] `test_pack_project_with_dependency` — pack expands dep functions
- [x] `test_dependency_missing_path_fails` — clear error message
- [x] `test_bench_project_with_dependency` — bench with deps

---

## 26. Example Projects ✅

### 26.1 Create example directory ✅

- [x] Create `examples/hello/`:
  - `ogre.toml` with project config
  - `src/main.bf` with hello world
  - `tests/basic.json` with test case

- [x] Create `examples/fibonacci/`:
  - `ogre.toml`
  - `src/main.bf` with Fibonacci BF program
  - `tests/basic.json`

- [x] Create `examples/cat/`:
  - `ogre.toml`
  - `src/main.bf` with cat program (echo input)
  - `tests/basic.json`

- [x] Create `examples/multifile/`:
  - `ogre.toml` with includes
  - `src/main.bf` with `@import` and `@call`
  - `lib/utils.bf` with utility `@fn` definitions
  - `tests/basic.json`

- [x] Create `examples/stdlib-demo/`:
  - `ogre.toml`
  - `src/main.bf` using `@import "std/io"` and `@import "std/math"`
  - `tests/basic.json`

---

## Implementation Priority Order

For maximum impact with minimum risk, implement in this order:

1. ✅ **Standard Library** (items 3.1–3.5) — Minimal code changes, high user value
2. ✅ **`ogre check`** (item 6) — Simple, useful for CI
3. ✅ **Terminal Colors** (item 10) — Quick win, big UX improvement
4. ✅ **Bytecode IR** (items 1.1–1.7) — Largest change, unlocks everything else
5. ✅ **Custom Error Enum** (item 2) — Clean up error handling, fully migrated
6. ✅ **`ogre pack`** (item 7) — Simple, useful
7. ✅ **Test Runner Improvements** (item 16) — Timeout prevents CI hangs
8. ✅ **Deep Static Analysis** (item 15) — Builds on IR (partial: dead code stub only)
9. ✅ **`ogre bench`** (item 9) — Useful for optimization work
10. ✅ **Configurable Tape Size** (item 5) — Small, useful
11. ✅ **Enhanced REPL** (item 11) — *rustyline, :help/:load/:save, project-aware @call*
12. ✅ **`ogre format --diff`** (item 13) — *Implemented with `similar` crate, colored unified diffs*
13. ✅ **CLI Integration Tests** (item 22) — *53 tests via `assert_cmd` covering all subcommands*
14. ✅ **`--help` Examples** (item 20) — *All subcommands have after_help examples*
15. ✅ **`--quiet`/`--verbose`** (item 19) — *Verbosity enum threaded through all modes*
16. ✅ **Source Mapping** (item 4) — *SourceMap types, tracked preprocessing, debugger integration, 18 tests*
17. ✅ **`@const` Directive** (item 17) — *@const/@use fully implemented*
18. ✅ **`ogre doc`** (item 14) — *@doc parsing, markdown generation, stdlib docs*
19. ✅ **`ogre init`** (item 8) — Convenience
20. ✅ **Watch Mode** (item 12) — *notify crate, --watch/-w flag, debounced re-run*
21. ✅ **Schema Validation** (item 18) — *Validates name, version, entry, test files, tape_size*
22. ✅ **Glob Patterns** (item 21) — *glob crate, *.bf and **/*.bf patterns*
23. ✅ **Performance Benchmarks** (item 23) — *criterion, 9 benchmarks in benches/interpreter.rs*
24. ✅ **Example Projects** (item 26) — *5 example projects with ogre.toml, src, tests*
25. ✅ **WASM Target** (item 24) — *WAT code generation, WASI I/O, --target wasm, 14+2 tests*
26. ✅ **Dependency Management** (item 25) — *[dependencies] in ogre.toml, path deps, recursive, all commands updated, 12+5 tests*
