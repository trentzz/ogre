# TODOv2.1 — Outstanding Items from IMPROVEMENTS and IMPROVEMENTSv2

This document lists all concrete, actionable items that were identified as
outstanding from IMPROVEMENTS.md and IMPROVEMENTSv2.md, organized by priority.

---

## 1. Move/Copy IR Optimization ✅

**Source:** IMPROVEMENTSv2.md Tier 1, Item 2 — "not yet implemented"

**Status:** Implemented. `MoveAdd(isize)` and `MoveSub(isize)` ops added.
Pattern detection for `[->+<]` and `[->-<]` variants. Full pipeline support
in interpreter, C codegen, and WASM codegen.

**Files:** `src/modes/ir.rs`, `src/modes/interpreter.rs`, `src/modes/compile.rs`,
`src/modes/compile_wasm.rs`

**Changes:**
- [x] Add `Op::MoveAdd(isize)` variant
- [x] Add `Op::MoveSub(isize)` variant
- [x] Add `optimize_move_idiom()` pass detecting `[->+<]` and `[->-<]` patterns
- [x] Update interpreter `step()` for new ops
- [x] Update C code generation for new ops
- [x] Update WAT code generation for new ops
- [x] Update `to_bf_string()` for new ops
- [x] Update `op_description()` and `code_char()` in interpreter
- [x] Tests for move detection, interpreter execution, C codegen

---

## 2. `ogre trace` Command ✅

**Source:** IMPROVEMENTSv2.md Tier 5, Item 25

**Status:** Implemented. New `trace` subcommand that prints tape state after
each instruction (or every N instructions).

**Files:** `src/modes/trace.rs`, `src/modes/mod.rs`, `src/main.rs`

**Changes:**
- [x] Implement `trace_file(path, tape_size, every_n)` and `trace_source()`
- [x] Output format: `step=N op=Add(3) dp=M cell[M]=V | [0 0 *5 0 0]`
- [x] Add `--every <n>` flag to control output frequency
- [x] Wire into CLI as `Trace(TraceArgs)` subcommand
- [x] Tests for trace output formatting

---

## 3. Cell Size Options

**Source:** IMPROVEMENTSv2.md Tier 3, Item 16

**Status:** Deferred. Requires significant generic type refactoring of the
interpreter's tape (`Vec<u8>` → generic or runtime dispatch). Lower priority
given the complexity vs. benefit ratio.

**Files:** `src/modes/interpreter.rs`, `src/modes/compile.rs`, `src/main.rs`

**Changes:**
- [ ] Add `--cell-size 8|16|32` flag to `run`, `debug`, `start`, `compile`
- [ ] Interpreter: use generic tape or runtime dispatch for u8/u16/u32
- [ ] Compiler: change to `uint8_t`/`uint16_t`/`uint32_t` with `#include <stdint.h>`
- [ ] Tests for each cell size variant

---

## 4. Compiler C Codegen Improvements ✅

**Source:** IMPROVEMENTSv2.md Section 1.4

**Status:** Implemented. Changed from `char array[N] = {0}` to
`unsigned char array[N]` + `memset(array, 0, sizeof(array))`.
Added `#include <string.h>`. MoveAdd/MoveSub codegen added.

**Files:** `src/modes/compile.rs`

**Changes:**
- [x] Change tape initialization to use `memset` with `#include <string.h>`
- [x] Use `unsigned char` instead of `char`
- [x] Add MoveAdd/MoveSub C codegen
- [x] Tests for new initialization format

---

## 5. Import Warning for Dropped Top-Level Code ✅

**Source:** IMPROVEMENTSv2.md Section 1.2

**Status:** Implemented. When an imported file has top-level BF code,
a warning is emitted via `eprintln!`.

**Files:** `src/modes/preprocess.rs`

**Changes:**
- [x] During `collect()`, track whether imported file had top-level BF code
- [x] Emit warning: `"warning: top-level code in imported file '...' is discarded"`
- [x] Only warn once per file

---

## 6. Complexity Metrics in Analyser ✅

**Source:** IMPROVEMENTSv2.md Tier 6, Item 28

**Status:** Implemented. Added max loop depth, total ops, optimized ops,
and optimization ratio to analysis output.

**Files:** `src/modes/analyse.rs`

**Changes:**
- [x] Add `max_loop_depth` to `AnalysisReport`
- [x] Add `total_ops` — raw BF instruction count
- [x] Add `optimized_ops` — instruction count after IR optimization
- [x] Add optimization ratio in verbose output
- [x] Add `compute_max_loop_depth()` function
- [x] 6 new tests for each metric

---

## 7. Comprehensive Documentation ✅

**Source:** IMPROVEMENTSv2.md Tier 7, Item 31

**Status:** Implemented. 13 documentation files in `docs/` covering all
aspects of the tool.

**Files:** `docs/` directory

**Documents created/updated:**
- [x] `docs/system-design.md` — comprehensive system design (~700 lines)
- [x] `docs/getting-started.md` — installation, first project, workflows (~707 lines)
- [x] `docs/cli-reference.md` — complete reference for all 17 commands
- [x] `docs/brainfunct-guide.md` — brainfunct dialect tutorial (~310 lines)
- [x] `docs/project-management.md` — ogre.toml, dependencies, CI (~288 lines)
- [x] `docs/debugger-and-compilation.md` — debugger + compilation guide (~260 lines)
- [x] `docs/architecture.md` — existing architecture overview
- [x] `docs/design-decisions.md` — design rationale
- [x] `docs/preprocessor.md` — preprocessor internals
- [x] `docs/ir-and-optimization.md` — IR and optimization passes
- [x] `docs/testing.md` — test infrastructure
- [x] `docs/stdlib-reference.md` — standard library reference
- [x] `docs/changelog.md` — project changelog

---

## Lower Priority / Future Items

These are acknowledged but not planned for immediate implementation:

- **Cell size options** (`--cell-size 8|16|32`) — requires generic refactoring
- **`ogre lsp`** — Language Server Protocol implementation (large effort)
- **Direct x86_64/ARM64 codegen** — JIT via cranelift/dynasm (large effort)
- **Workspace support** — multi-project ogre.toml (medium effort)
- **Registry / `ogre publish`** — package registry (large effort)
- **Lock file (`ogre.lock`)** — dependency pinning (medium effort)
- **Editor integration** — VS Code extension, tree-sitter grammar (large effort)
- **Playground** — web-based BF playground via WASM (large effort)
- **Plugin system** — dynamic analysis/optimization passes (large effort)
- **Property-based tests** — proptest for formatter idempotency, etc.
- **Named cell aliases** — `@alias varname 5` for debugger
- **Parameterized macros** — `@fn add(n)` with expression evaluation
- **Security analysis** — halting detection, unbounded input detection
- **Escape sequences** in `@import` string literals
- **Library surface cleanup** — designed pub API boundary for lib.rs
- **Loop unrolling** — unroll simple counted loops in optimizer
- **Progress indication** — spinner for long operations
- **Bounds checking flag** (`--bounds-check`) for compiler
- **CopyAdd optimization** — non-destructive copy pattern `[->+>+<<]`
