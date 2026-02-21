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

## 2. Custom Error Enum (partial — enum defined, not yet wired into modules)

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

### 2.2 Migrate modules to use `OgreError` (partial)

- [ ] **`src/modes/ir.rs`** — return `OgreError::BracketMismatch` for
  unmatched brackets in `Program::from_source()` *(enum defined, not yet wired in — modules still use anyhow)*
- [ ] **`src/modes/interpreter.rs`** — return `OgreError::TapeOverflow`
  for out-of-bounds pointer movement
- [ ] **`src/modes/preprocess.rs`** — return `OgreError::CycleDetected`,
  `OgreError::ImportCycle`, `OgreError::UnknownFunction`,
  `OgreError::FileNotFound`, `OgreError::UnknownDirective`
- [ ] **`src/modes/compile.rs`** — return `OgreError::CompilerNotFound`
- [ ] **`src/project.rs`** — return `OgreError::InvalidProject`
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
- [ ] Test `@import "std/memory"` + copy/swap operations
- [ ] Test `@import "std/ascii"` + case conversion
- [ ] Test mixing std imports with file imports

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

## 4. Source Mapping

### 4.1 Define source location types

**File:** `src/modes/source_map.rs` (new)

- [ ] Define the types:
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
- [ ] Implement `SourceMap::lookup(position: usize) -> Option<&SourceLocation>`
- [ ] Add `pub mod source_map;` to `src/modes/mod.rs`

### 4.2 Generate source map during preprocessing

**File:** `src/modes/preprocess.rs`

- [ ] Add a `source_map: Vec<SourceLocation>` field to `Preprocessor`
- [ ] During `collect()`, track current file, line, and column
- [ ] When appending characters to `top_level`, push corresponding
  `SourceLocation` entries
- [ ] During `expand()`, when expanding `@call`, push source locations
  with the `function` field set to the function name
- [ ] Add `Preprocessor::process_file_with_map(path) -> Result<(String, SourceMap)>`
  as a new public method (keep existing `process_file` unchanged for
  backward compatibility)

### 4.3 Use source map in the debugger

**File:** `src/modes/debug.rs`

- [ ] Change `debug_file()` to call `process_file_with_map()` and store
  the `SourceMap` in the `Debugger` struct
- [ ] Update `print_status()` to show the original file and line:
  ```
  ip=47  @fn greet+3  (src/greet.bf:5:12)  op='+'  dp=0  val=3
  ```
- [ ] Update `show_instruction` to show source context from the
  original file

### 4.4 Use source map in error messages

**File:** `src/modes/interpreter.rs`

- [ ] Add an optional `source_map: Option<SourceMap>` field to `Interpreter`
- [ ] When reporting `TapeOverflow` errors, include the source location
  if available:
  ```
  data pointer out of bounds (right) at src/main.bf:12:5 (@fn process+7)
  ```

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

### 5.5 Tests (partial)

- [x] Test interpreter with custom tape size
- [ ] Test interpreter with tape size 100,000 (larger tape works)
- [x] Test compiler generates correct array size
- [ ] Test CLI flag parsing

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
- [ ] Implement `check_project(project: &OgreProject, base: &Path)` —
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

- [ ] Test valid file → exit 0, no output
- [ ] Test unmatched bracket → exit 1, error message
- [ ] Test unknown `@call` → exit 1, error message
- [ ] Test import cycle → exit 1, error message
- [ ] Test missing import file → exit 1, error message
- [ ] Test project-wide check

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

- [ ] Test packing a file with `@fn`/`@call` → pure BF output
- [ ] Test packing with `--optimize` → shorter output (cancelled ops removed)
- [ ] Test packing preserves program semantics (run both, compare output)

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

- [ ] Test init in empty directory → creates `ogre.toml`, `src/`, `tests/`
- [ ] Test init when `ogre.toml` already exists → error
- [ ] Test init detects existing `.bf` files

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

### 9.4 Tests (partial)

- [ ] Test bench on hello world → reports reasonable numbers
- [x] Test `format_number` helper for comma-separated numbers
- [ ] Test cells touched is correct

---

## 10. Terminal Colors (partial — test runner, analyser, check colored; debugger/REPL not yet)

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

### 10.4 Color debugger output

**File:** `src/modes/debug.rs`

- [ ] Current instruction highlighted in yellow/bold
- [ ] Pointer cell highlighted in cyan
- [ ] Breakpoint indicators in red

### 10.5 Color REPL output

**File:** `src/modes/start.rs`

- [ ] Pointer cell highlighted in cyan
- [ ] Error messages in red

### 10.6 Color error messages

**File:** `src/main.rs`

- [ ] Wrap error output in red when printing to stderr

### 10.7 Add `--no-color` global flag ✅

**File:** `src/main.rs`

- [x] Add `--no-color` flag to `Cli` struct
- [x] Call `colored::control::set_override(false)` when flag is set
- [x] Respect `NO_COLOR` environment variable

---

## 11. Enhanced REPL (`ogre start` improvements)

### 11.1 Add `rustyline` dependency

**File:** `Cargo.toml`

- [ ] Add `rustyline = "14"` to dependencies

### 11.2 Rewrite REPL with line editing

**File:** `src/modes/start.rs`

- [ ] Replace `stdin.lock().read_line()` with `rustyline::Editor`
- [ ] Enable command history (persisted to `~/.ogre_history`)
- [ ] Add tab completion for commands (`reset`, `exit`, `help`, `load`, `save`)
- [ ] Add `:help` command that lists all REPL commands
- [ ] Add `:load <file>` command that loads and runs a BF file
- [ ] Add `:save <file>` command that saves current tape state info

### 11.3 Project-aware REPL

**File:** `src/modes/start.rs`

- [ ] When ogre.toml is found, preload all `@fn` definitions from the project
- [ ] Support `@call` in REPL input (preprocess before feeding to interpreter)
- [ ] Support `@import "std/..."` in REPL input
- [ ] Show loaded function count at startup

### 11.4 Tests

- [ ] Test `:load` with a valid file
- [ ] Test `:load` with nonexistent file → error
- [ ] Test `@call` in REPL works with preloaded functions

---

## 12. Watch Mode

### 12.1 Add `notify` dependency

**File:** `Cargo.toml`

- [ ] Add `notify = "6"` to dependencies

### 12.2 Implement watch mode

**File:** `src/modes/run.rs`

- [ ] Add `run_file_watch(path: &Path) -> Result<()>`:
  1. Run the file once
  2. Set up a `notify::Watcher` on the file (and its directory for imports)
  3. On change event, clear terminal and re-run
  4. Handle Ctrl+C for clean exit
- [ ] Print `"Watching {path} for changes..."` message
- [ ] Print timestamp on each re-run

### 12.3 Wire into CLI

**File:** `src/main.rs`

- [ ] Add `--watch` / `-w` flag to `RunArgs`
- [ ] When set, call `run::run_file_watch()` instead of `run::run_file()`

---

## 13. `ogre format --diff`

### 13.1 Add `similar` dependency

**File:** `Cargo.toml`

- [ ] Add `similar = "2"` to dependencies

### 13.2 Implement diff mode

**File:** `src/modes/format.rs`

- [ ] Add `diff: bool` field to `FormatOptions`
- [ ] When `diff` is true:
  1. Format the source to a string (don't write)
  2. If formatted != original, compute a unified diff using `similar`
  3. Print the diff with `+` lines in green, `-` lines in red
  4. Return false (indicating changes needed)
- [ ] When `diff` is false, keep existing behavior

### 13.3 Wire into CLI

**File:** `src/main.rs`

- [ ] Add `--diff` flag to `FormatArgs`
- [ ] Set `opts.diff = args.diff`

### 13.4 Tests

- [ ] Test `--diff` on already-formatted file → no output
- [ ] Test `--diff` on unformatted file → shows diff
- [ ] Test `--diff` doesn't modify the file

---

## 14. `ogre doc` Command

### 14.1 Define `@doc` comment syntax

The `@doc` comment is a line starting with `@doc` followed by text,
placed immediately before an `@fn` definition:

```brainfuck
@doc Clears the current cell to zero.
@doc Uses: cell 0 (modified). Pointer: unchanged.
@fn zero { [-] }
```

### 14.2 Parse `@doc` in the preprocessor

**File:** `src/modes/preprocess.rs`

- [ ] In the `collect()` method, add handling for `@doc` directive:
  - Accumulate consecutive `@doc` lines into a buffer
  - When `@fn` is encountered, attach the accumulated doc buffer to
    the function in a new `HashMap<String, String>` called `fn_docs`
- [ ] Add `fn_docs: HashMap<String, String>` field to `Preprocessor`
- [ ] Add `pub fn get_docs(&self) -> &HashMap<String, String>` accessor

### 14.3 Implement doc generation

**File:** Create `src/modes/doc.rs`

- [ ] Implement `generate_docs(path: &Path) -> Result<String>`:
  1. Run the preprocessor to collect functions and docs
  2. Generate markdown output:
     - Module name (filename)
     - For each function: name, doc comment, source body
  3. Return the markdown string
- [ ] Implement `generate_project_docs(project, base) -> Result<String>`:
  1. Process all include files
  2. Generate a table of contents
  3. Generate per-file documentation
- [ ] Implement `generate_stdlib_docs() -> Result<String>`:
  1. Process all stdlib modules
  2. Generate documentation for each
- [ ] Add `pub mod doc;` to `src/modes/mod.rs`

### 14.4 Wire into CLI

**File:** `src/main.rs`

- [ ] Add `Doc` variant to `Commands`:
  ```rust
  /// Generate documentation from @doc comments
  Doc(DocArgs),
  ```
- [ ] Define `DocArgs`:
  ```rust
  struct DocArgs {
      file: Option<String>,
      #[arg(long)]
      stdlib: bool,
      #[arg(short = 'o', long)]
      output: Option<String>,
  }
  ```

---

## 15. Deep Static Analysis (partial — cancellation, clear idiom, unbalanced pointer done; dead code stub only)

### 15.1 Cancellation detection ✅

**File:** `src/modes/analyse.rs`

- [x] Add `has_cancellation: bool` to `AnalysisReport`
- [x] Scan source for consecutive `+-`, `-+`, `><`, `<>` patterns
- [x] Report in verbose mode
- [ ] Report the position of each cancellation found

### 15.2 Clear idiom detection ✅

**File:** `src/modes/analyse.rs`

- [x] Detect `[-]` and `[+]` patterns in the source
- [x] Add `has_clear_idiom: bool` to `AnalysisReport`
- [x] Report in verbose mode
- [ ] In verbose mode, count total clear idioms found

### 15.3 Dead code detection (stub)

**File:** `src/modes/analyse.rs`

- [x] Add `has_dead_code: bool` to `AnalysisReport`
- [ ] Detect `+[` at position 0 (infinite loop from start)
- [ ] Detect code after a `]` that follows an unconditional infinite loop
- [ ] Report as warning: `"Warning: unreachable code after position 20"`

### 15.4 Unbalanced pointer detection ✅

**File:** `src/modes/analyse.rs`

- [x] Add `unbalanced_pointer: bool` to `AnalysisReport`
- [x] Track net pointer offset and warn if non-zero at end
- [ ] Per-loop body analysis (not yet implemented)

### 15.5 Tests (partial)

- [x] Test cancellation detection finds `+-`
- [x] Test clear idiom detection finds `[-]`
- [ ] Test dead code detection after infinite loop
- [x] Test unbalanced pointer warning
- [ ] Test no false positives on valid programs

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
- [ ] If both `output` and `output_regex` are set, error

### 16.3 Cargo-style output ✅

**File:** `src/modes/test_runner.rs`

- [x] Change default output to dots for passing tests:
  `.` for pass, `F` for fail, `T` for timeout
- [x] Only expand failure details after all tests run
- [ ] Add `--verbose` flag to show per-test output (current behavior)
- [x] Summary line: `"N/M tests passed"` with colored count

### 16.4 Tests ✅

- [x] Test timeout on infinite loop BF → reports TIMEOUT (test_instruction_limit)
- [x] Test regex matching works (test_regex_matching)
- [ ] Test regex mismatch reports correctly

---

## 17. `@const` Directive

### 17.1 Parse `@const` in preprocessor

**File:** `src/modes/preprocess.rs`

- [ ] Add `constants: HashMap<String, usize>` field to `Preprocessor`
- [ ] In `collect()`, handle `@const` directive:
  ```
  @const NAME value
  ```
  Parse `NAME` as identifier, `value` as usize, store in `constants`
- [ ] In `expand()`, when encountering `@const` references... actually,
  `@const` should expand to `value` number of `+` characters wherever
  the constant name is referenced via `@const NAME` inline usage
- [ ] Alternative: `@const` defines a value, and `@use NAME` expands
  to that many `+` characters. This is cleaner.
- [ ] Add handling for `@use NAME` directive in `collect()`:
  - Look up NAME in constants
  - Append `n` `+` characters to top_level

### 17.2 Tests

- [ ] Test `@const X 5` + `@use X` → `+++++`
- [ ] Test `@const` inside `@fn` body
- [ ] Test undefined `@use` → error
- [ ] Test `@const` with value 0 → empty expansion
- [ ] Test `@const` with value 255 → 255 `+` characters

---

## 18. Project Schema Validation

### 18.1 Validate ogre.toml at parse time

**File:** `src/project.rs`

- [ ] After deserializing, validate:
  - `project.name` is not empty
  - `project.version` matches semver pattern (warn if not)
  - `project.entry` ends with `.bf`
  - All `tests[].file` entries end with `.json`
  - All `build.include` entries are valid paths/globs
- [ ] Return `OgreError::InvalidProject` with specific messages
- [ ] Warn on unknown fields (requires custom deserializer or serde
  `deny_unknown_fields`)

### 18.2 Tests

- [ ] Test empty project name → error
- [ ] Test missing entry → error
- [ ] Test invalid test file extension → warning
- [ ] Test valid project → no warnings

---

## 19. `--quiet` / `--verbose` Global Flags (partial — flags added, not threaded through modes)

### 19.1 Add global flags ✅

**File:** `src/main.rs`

- [x] Add to `Cli` struct:
  ```rust
  #[arg(long, global = true)]
  quiet: bool,
  #[arg(long, short = 'v', global = true)]
  verbose: bool,
  ```

### 19.2 Thread verbosity through modes

- [ ] Define a `Verbosity` enum: `Quiet`, `Normal`, `Verbose`
- [ ] Pass `Verbosity` to `run_file()`, `compile()`, `format_file()`,
  `test_runner::run_tests()`, etc.
- [ ] In `Quiet` mode: suppress informational output ("Compiled to: ...",
  "Formatting: ...", passing test names)
- [ ] In `Verbose` mode: add extra output (instruction counts, timing,
  per-file details)

---

## 20. `--help` Examples (partial — added to key subcommands)

### 20.1 Add examples to each subcommand ✅

**File:** `src/main.rs`

- [x] Add `after_help` to each command variant:
  ```rust
  /// Interpret and execute a brainfuck file
  #[command(after_help = "\
  EXAMPLES:
    ogre run hello.bf
    ogre run                    # uses ogre.toml entry
    ogre run --tape-size 60000 big_program.bf
  ")]
  Run(RunArgs),
  ```
- [x] Add examples for: `run`, `compile`, `check`, `pack`, `init`, `bench`
- [ ] Add examples for: `build`, `format`, `analyse`, `test`, `debug`, `generate`, `new`

---

## 21. Recursive Includes / Glob Patterns

### 21.1 Support glob patterns in `build.include`

**File:** `src/project.rs`

- [ ] Add `glob` crate to `Cargo.toml`
- [ ] In `resolve_include_files()`, detect glob patterns (contains `*` or `?`)
- [ ] Use `glob::glob()` to expand patterns:
  ```rust
  if entry.contains('*') || entry.contains('?') {
      let pattern = base.join(entry).to_string_lossy().to_string();
      for path in glob::glob(&pattern)? {
          files.push(path?);
      }
  }
  ```
- [ ] Support patterns like `src/**/*.bf` for recursive includes

### 21.2 Tests

- [ ] Test `"src/*.bf"` matches files in src/
- [ ] Test `"src/**/*.bf"` matches files recursively
- [ ] Test invalid glob pattern → error

---

## 22. CLI Integration Tests

### 22.1 Add `assert_cmd` dependency

**File:** `Cargo.toml`

- [ ] Add to `[dev-dependencies]`:
  ```toml
  assert_cmd = "2"
  predicates = "3"
  ```

### 22.2 Write CLI tests

**File:** Create `tests/cli_integration.rs`

- [ ] Test `ogre run hello.bf` → exit 0, output "Hello World!\n"
- [ ] Test `ogre run nonexistent.bf` → exit 1, error message
- [ ] Test `ogre format --check` on formatted file → exit 0
- [ ] Test `ogre format --check` on unformatted file → exit 1
- [ ] Test `ogre compile hello.bf` → exit 0, creates binary
- [ ] Test `ogre new testproject` → creates directory structure
- [ ] Test `ogre generate helloworld` → exit 0, valid BF output
- [ ] Test `ogre generate string "Hi"` → exit 0, valid BF output
- [ ] Test `ogre check valid.bf` → exit 0
- [ ] Test `ogre check invalid.bf` → exit 1
- [ ] Test `ogre --version` → prints version
- [ ] Test `ogre --help` → prints help text

---

## 23. Performance Benchmarks

### 23.1 Add `criterion` dependency

**File:** `Cargo.toml`

- [ ] Add to `[dev-dependencies]`:
  ```toml
  criterion = { version = "0.5", features = ["html_reports"] }
  ```
- [ ] Add `[[bench]]` section:
  ```toml
  [[bench]]
  name = "interpreter"
  harness = false
  ```

### 23.2 Write benchmarks

**File:** Create `benches/interpreter.rs`

- [ ] Benchmark hello world interpretation
- [ ] Benchmark mandelbrot.bf interpretation (add as test fixture)
- [ ] Benchmark IR compilation (source → Program)
- [ ] Benchmark optimization passes
- [ ] Benchmark C code generation
- [ ] Benchmark preprocessing with imports

---

## 24. WASM Compilation Target

### 24.1 Add WAT code generation

**File:** Create `src/modes/compile_wasm.rs`

- [ ] Implement `generate_wat(program: &Program, tape_size: usize) -> String`:
  - WAT module with linear memory for the tape
  - Import `fd_write` from WASI for output
  - Translate each `Op` to WAT instructions
- [ ] Add `pub mod compile_wasm;` to `src/modes/mod.rs`

### 24.2 Compile WAT to WASM

- [ ] Use `wat` crate to convert WAT text to WASM binary
- [ ] Or shell out to `wat2wasm` if available
- [ ] Add `wat = "1"` to `Cargo.toml` (optional dependency)

### 24.3 Wire into CLI

**File:** `src/main.rs`

- [ ] Add `--target` flag to `CompileArgs`:
  ```rust
  #[arg(long, default_value = "native")]
  target: String,  // "native" or "wasm"
  ```
- [ ] When target is "wasm", call `compile_wasm::compile_to_wasm()`

### 24.4 Tests

- [ ] Test WAT generation for simple programs
- [ ] Test WASM binary is valid (if wasmtime available, run it)

---

## 25. Dependency Management

### 25.1 Extend ogre.toml schema

**File:** `src/project.rs`

- [ ] Add `dependencies` field:
  ```rust
  #[derive(Deserialize, Debug)]
  pub struct Dependency {
      pub path: Option<String>,   // local path dependency
      pub version: Option<String>, // for future registry support
  }

  // In OgreProject:
  pub dependencies: Option<HashMap<String, Dependency>>,
  ```

### 25.2 Resolve dependencies in preprocessor

**File:** `src/modes/preprocess.rs`

- [ ] When encountering `@import "dep/module"`, check if `dep` matches
  a key in `dependencies`
- [ ] If it's a path dependency, resolve relative to the dependency's
  directory
- [ ] Load the dependency's `ogre.toml` to find its include files
- [ ] Make all `@fn` definitions from the dependency available

### 25.3 Wire into project loading

**File:** `src/project.rs`

- [ ] When loading a project, also load all dependency projects
- [ ] Build a dependency graph, detect cycles
- [ ] Make dependency `@fn` definitions available during preprocessing

### 25.4 Tests

- [ ] Test path dependency resolution
- [ ] Test dependency cycle detection
- [ ] Test `@call` into dependency functions works

---

## 26. Example Projects

### 26.1 Create example directory

- [ ] Create `examples/hello/`:
  - `ogre.toml` with project config
  - `src/main.bf` with hello world
  - `tests/basic.json` with test case

- [ ] Create `examples/fibonacci/`:
  - `ogre.toml`
  - `src/main.bf` with Fibonacci BF program
  - `tests/basic.json`

- [ ] Create `examples/cat/`:
  - `ogre.toml`
  - `src/main.bf` with cat program (echo input)
  - `tests/basic.json`

- [ ] Create `examples/multifile/`:
  - `ogre.toml` with includes
  - `src/main.bf` with `@import` and `@call`
  - `src/utils.bf` with utility `@fn` definitions
  - `tests/basic.json`

- [ ] Create `examples/stdlib-demo/`:
  - `ogre.toml`
  - `src/main.bf` using `@import "std/io"` and `@import "std/math"`
  - `tests/basic.json`

---

## Implementation Priority Order

For maximum impact with minimum risk, implement in this order:

1. ✅ **Standard Library** (items 3.1–3.5) — Minimal code changes, high user value
2. ✅ **`ogre check`** (item 6) — Simple, useful for CI
3. ✅ **Terminal Colors** (item 10) — Quick win, big UX improvement (partial: debugger/REPL not yet)
4. ✅ **Bytecode IR** (items 1.1–1.7) — Largest change, unlocks everything else
5. ✅ **Custom Error Enum** (item 2) — Clean up error handling (partial: enum defined, not wired)
6. ✅ **`ogre pack`** (item 7) — Simple, useful
7. ✅ **Test Runner Improvements** (item 16) — Timeout prevents CI hangs
8. ✅ **Deep Static Analysis** (item 15) — Builds on IR (partial: dead code stub only)
9. ✅ **`ogre bench`** (item 9) — Useful for optimization work
10. ✅ **Configurable Tape Size** (item 5) — Small, useful
11. **Enhanced REPL** (item 11) — Nice to have
12. **`ogre format --diff`** (item 13) — Nice to have
13. **CLI Integration Tests** (item 22) — Testing infrastructure
14. ✅ **`--help` Examples** (item 20) — Polish (partial: key subcommands)
15. ✅ **`--quiet`/`--verbose`** (item 19) — Polish (partial: flags added, not threaded)
16. **Source Mapping** (item 4) — Complex, high value for debugger
17. **`@const` Directive** (item 17) — Language extension
18. **`ogre doc`** (item 14) — Documentation tooling
19. ✅ **`ogre init`** (item 8) — Convenience
20. **Watch Mode** (item 12) — Convenience
21. **Schema Validation** (item 18) — Polish
22. **Glob Patterns** (item 21) — Convenience
23. **Performance Benchmarks** (item 23) — Infrastructure
24. **Example Projects** (item 26) — Documentation
25. **WASM Target** (item 24) — Advanced feature
26. **Dependency Management** (item 25) — Advanced feature
