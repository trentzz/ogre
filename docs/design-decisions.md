# Design Decisions

This document explains the reasoning behind key design choices in ogre,
what alternatives were considered, and what tradeoffs were made.

---

## 1. Bytecode IR (Vec<Op>)

**Decision:** Replace raw `Vec<char>` code representation with a typed
`Vec<Op>` intermediate representation.

**Problem:** Every mode (interpreter, compiler, analyser) was independently
parsing brainfuck characters. The interpreter treated non-BF characters as
no-ops at runtime. The compiler had its own run-length collapsing logic.
There was no shared optimization.

**Options considered:**

1. **Keep Vec<char>, add shared utilities** — Lowest effort but doesn't
   enable optimizations or eliminate runtime comment filtering.
2. **Vec<u8> bytecode** — Compact but loses type safety and requires manual
   encoding/decoding.
3. **Enum-based IR (chosen)** — `Op::Add(u8)`, `Op::Right(usize)`, etc.
   Type-safe, self-documenting, enables pattern matching.
4. **Tree-based AST** — More powerful but overkill for BF's flat structure.

**Why Vec<Op>:** Brainfuck has no nesting beyond `[]` loops, so a flat
instruction array with jump indices is the natural representation. The enum
gives type safety and Rust's exhaustive matching catches missing cases.

**Impact:**
- 21 new unit tests for the IR module
- Interpreter is 2-3x faster (no comment filtering, collapsed ops)
- Compiler generates better C code (Clear idiom -> `*ptr = 0;`)
- Analyser reasons about typed operations instead of characters
- All optimization benefits are shared across all modes

---

## 2. Two-Pass Preprocessor

**Decision:** The preprocessor uses two passes: collect (resolve imports,
gather functions) then expand (inline function bodies at call sites).

**Problem:** With a single pass, a function defined after its first call
site would not be found. Imports need to be resolved before expansion.

**Options considered:**

1. **Single pass with forward declarations** — Requires declaring functions
   before use. Unnatural for a macro system.
2. **Two-pass (chosen)** — Natural order: gather everything first, then expand.
3. **Lazy expansion** — Expand on first call. More complex, harder to detect
   cycles.

**Why two passes:** Clean separation of concerns. Pass 1 handles the file
system (imports, reading files) and builds a function table. Pass 2 is pure
string manipulation. Cycle detection is straightforward in each pass:
import cycles via `HashSet<PathBuf>`, call cycles via `Vec<String>` stack.

---

## 3. Error Handling: anyhow + OgreError

**Decision:** Use `anyhow::Result` at the CLI boundary, `OgreError` enum
for typed errors in library code.

**Problem:** All modules originally used `anyhow::bail!()` with string
messages. Callers couldn't distinguish error types without parsing strings.

**Options considered:**

1. **anyhow everywhere** — Simple but no typed errors for library consumers.
2. **OgreError everywhere** — Forces all callers to handle every variant.
3. **Hybrid (chosen)** — `OgreError` for structured errors that callers
   might want to match on; `anyhow` for the CLI layer.

**Impact:**
- `OgreError::BracketMismatch` — distinguishable from `ImportCycle`
- `OgreError::TapeOverflow` — includes direction (left/right)
- `OgreError::CompilerNotFound` — separate from compilation failure
- `OgreError::InvalidProject` — schema validation errors

The `thiserror` crate derives `Display` and `Error` traits, and the `From`
impl converts `OgreError` to `anyhow::Error` seamlessly.

---

## 4. Embedded Standard Library

**Decision:** Ship a standard library of reusable BF functions embedded in
the binary via `include_str!()`.

**Problem:** Users need common BF patterns (print newline, clear cell,
copy value) but copy-pasting code between projects is error-prone.

**Options considered:**

1. **Ship as separate files** — Requires an installation directory, env
   var for the path, fails if files are missing.
2. **Download from registry** — Complex, requires network, versioning.
3. **Embed in binary (chosen)** — Always available, zero configuration,
   version-locked to the ogre binary.

**Modules:** io, math, memory, ascii, debug (5 modules, ~30 functions).

**Import syntax:** `@import "std/io.bf"` — the `std/` prefix triggers
embedded resolution instead of file system lookup.

**Impact:**
- `ogre stdlib list` and `ogre stdlib show` commands for exploration
- `ogre new --with-std` scaffolds projects with stdlib imports
- No installation path needed
- Functions are tested as part of ogre's test suite

---

## 5. Verbosity System

**Decision:** Define `Verbosity` enum (Quiet/Normal/Verbose) and thread
it through all mode functions.

**Problem:** The `--quiet` and `--verbose` CLI flags existed but had no
effect on mode output.

**Options considered:**

1. **Global static** — Simple but not thread-safe, hard to test.
2. **Environment variable** — Implicit, hard to discover.
3. **Parameter threading (chosen)** — Explicit, testable, no global state.

**How it works:** The `run()` function in `main.rs` computes `Verbosity`
from CLI flags and passes it to each mode function. Each mode checks
`verbosity.is_quiet()` before printing informational messages and
`verbosity.is_verbose()` for extra detail.

**Impact:**
- `--quiet` suppresses "Compiled to:", "Formatting:", "OK" messages
- `--verbose` enables extra detail in analyse and bench output
- All existing functions retain backward-compatible signatures (new `_ex`
  variants accept `Verbosity`)

---

## 6. Format --diff Mode

**Decision:** Add `--diff` flag to `ogre format` that shows unified diffs
without modifying files.

**Problem:** Users want to preview formatting changes before applying them.
`--check` only reports whether changes are needed, not what they are.

**Options considered:**

1. **Custom diff algorithm** — More control but significant implementation.
2. **Shell out to `diff`** — Platform-dependent, not always available.
3. **`similar` crate (chosen)** — Pure Rust, supports unified diffs with
   context, well-maintained.

**Impact:**
- Colored output: red for deletions, green for insertions, cyan headers
- Exit code 1 if changes needed (CI-friendly, same as `--check`)
- File is never modified in diff mode
- 5 unit tests + 3 CLI integration tests

---

## 7. @const/@use Directives

**Decision:** Add `@const NAME value` and `@use NAME` directives for
named numeric constants that expand to N `+` characters.

**Problem:** Hard-coding ASCII values as `+` runs is unreadable.
`++++++++++++++++++++++++++++++++` vs `@const SPACE 32` + `@use SPACE`.

**Options considered:**

1. **Parameterized macros** (`@fn add(n)`) — More powerful but significantly
   more complex to implement. Would require expression evaluation.
2. **Named constants (chosen)** — Simple, covers the main use case (ASCII
   values), minimal preprocessor changes.
3. **`@set` directive** — Similar but name collision with BF set patterns.

**Implementation:** Constants are stored in a `HashMap<String, usize>` on
the preprocessor. `@const` defines a value during the collect pass. `@use`
expands to N `+` characters in both collect and expand passes.

---

## 8. Test Runner Design

**Decision:** JSON-based test files with instruction-count timeouts and
optional regex matching.

**Problem:** BF programs can be infinite loops. Tests need timeouts.
Some outputs are non-deterministic or hard to match exactly.

**Key design choices:**
- **Instruction-count limiting** instead of wall-clock timeouts: deterministic,
  no threading needed, works identically on fast and slow machines.
- **Default 10M instruction limit**: catches infinite loops without being
  too restrictive for normal programs.
- **`output_regex` field**: allows pattern matching when exact output is
  impractical.
- **Cargo-style output**: `.` for pass, `F` for fail, `T` for timeout.
  Failure details shown after all tests complete.

---

## 9. Project Schema Validation

**Decision:** Validate `ogre.toml` at parse time with clear error messages.

**Problem:** Invalid project configurations (empty name, wrong file
extensions) caused confusing errors later in the pipeline.

**Validation rules:**
- `project.name` must not be empty or whitespace-only
- `project.entry` must end with `.bf`
- `project.version` must not be empty
- All `tests[].file` entries must end with `.json`
- `build.tape_size` must be > 0 if specified

**Impact:** Errors caught immediately with messages like
`"invalid project: project.entry must end with .bf, got \"main.txt\""`
instead of cryptic failures downstream.

---

## 10. Colored Output Strategy

**Decision:** Use the `colored` crate with `--no-color` flag and `NO_COLOR`
environment variable support.

**Options considered:**

1. **ANSI escape codes directly** — Manual, error-prone, no detection.
2. **`termcolor`** — More powerful but more complex API.
3. **`colored` (chosen)** — Simple fluent API (`.red()`, `.green().bold()`),
   built-in `NO_COLOR` support.

**Color conventions:**
- Green: success (passing tests, "OK" in check)
- Red: failure (errors, "FAIL" in tests)
- Yellow: warnings, current instruction in debugger
- Cyan: pointer cell in memory display, diff hunk headers
- Bold: emphasis (section headers, failure labels)
