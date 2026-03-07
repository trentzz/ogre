# Feature Brainstorm - Round 2

## Features to Implement

### 1. LLVM IR Backend (`compile --target llvm`)

**Priority: High | Complexity: Medium-High**

Generate LLVM IR text (`.ll` files) and compile them with `clang` or `llc + ld`. This
unlocks LLVM's full optimization pipeline (O0-O3) and supports every LLVM target
architecture without maintaining separate assembly backends.

#### Architecture

The LLVM backend follows the same pattern as the existing C backend:

1. **IR → LLVM IR text**: Walk `Vec<Op>` and emit LLVM IR instructions
2. **LLVM IR → Object**: Invoke `clang -c file.ll -o file.o -O2` (or `llc`)
3. **Object → Binary**: Link with `clang file.o -o binary`

#### LLVM IR Mapping

| Op | LLVM IR |
|----|---------|
| `Add(n)` | `%val = load i8, ptr %ptr` → `%new = add i8 %val, n` → `store i8 %new, ptr %ptr` |
| `Sub(n)` | Same with `sub` |
| `Right(n)` | `%ptr = getelementptr i8, ptr %ptr, i64 n` |
| `Left(n)` | `%ptr = getelementptr i8, ptr %ptr, i64 -n` |
| `Output` | `call i32 @putchar(i32 %val)` |
| `Input` | `%ch = call i32 @getchar()` → `store i8 %ch, ptr %ptr` |
| `JumpIfZero` | `%cmp = icmp eq i8 %val, 0` → `br i1 %cmp, label %skip, label %loop` |
| `JumpIfNonZero` | `%cmp = icmp ne i8 %val, 0` → `br i1 %cmp, label %loop, label %exit` |
| `Clear` | `store i8 0, ptr %ptr` |
| `Set(n)` | `store i8 n, ptr %ptr` |
| `ScanRight` | `call ptr @memchr` or loop with `icmp`/`br` |
| `ScanLeft` | Loop scanning left for zero |
| `MoveAdd(off)` | Load src, load dst at offset, add, store both |
| `MoveSub(off)` | Load src, load dst at offset, sub, store both |
| `MultiplyMove(targets)` | Unrolled multiply-accumulate for each target |

#### Generated LLVM IR Structure

```llvm
; ModuleID = 'program'
target triple = "x86_64-unknown-linux-gnu"

@tape = global [30000 x i8] zeroinitializer

declare i32 @putchar(i32)
declare i32 @getchar()
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)

define i32 @main() {
entry:
  %ptr = alloca ptr
  store ptr @tape, ptr %ptr
  ; ... bf ops ...
  ret i32 0
}
```

#### Advantages over C Backend
- **Better optimization**: LLVM O2/O3 can optimize BF patterns the C compiler misses
- **No C intermediate**: Cleaner pipeline, no need for gcc
- **Cross-compilation**: LLVM supports 20+ target architectures natively
- **Debug info**: Can emit DWARF debug info mapping back to BF source
- **LTO**: Natural integration with LLVM link-time optimization

#### Implementation Plan
1. New file: `src/modes/compile_llvm.rs`
2. `LlvmCompiler` struct with `generate_ir()` → String method
3. Add `--target llvm` option to compile command
4. Invoke `clang` for final compilation
5. Support `--keep` flag to preserve `.ll` file
6. Add `--opt-level` flag (O0, O1, O2, O3)
7. Tests: verify IR generation, compilation, correct output

---

### 2. Execution Profiler (`ogre profile`)

**Priority: Medium | Complexity: Medium**

Profile BF program execution to identify hotspots and optimization opportunities.

#### Output
```
=== Execution Profile ===
Total instructions: 1,234,567
Unique cells accessed: 42

Instruction Mix:
  Add/Sub:    45.2%  (558,024)
  Move:       23.1%  (285,185)
  I/O:         0.3%    (3,703)
  Loops:      31.4%  (387,655)

Hot Cells (top 10):
  Cell 0:   234,567 accesses (19.0%)
  Cell 1:   198,234 accesses (16.1%)
  ...

Loop Analysis:
  Loop at op 5:   12,345 iterations (avg 2.3 per entry)
  Loop at op 12:     789 iterations (avg 1.0 per entry)

Memory Heatmap (first 50 cells):
  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

---

### 3. Lint Command (`ogre lint`)

**Priority: Medium | Complexity: Low-Medium**

Configurable static linting rules for BF/Brainfunct code.

#### Built-in Rules
- `unbalanced-pointer`: Function doesn't return pointer to starting position
- `missing-doc`: @fn without preceding @doc
- `deep-nesting`: Loop nesting depth exceeds threshold (default 5)
- `long-function`: Function body exceeds length threshold
- `unused-import`: Imported module has no @call references
- `redundant-clear`: `[-]` on a cell known to be zero
- `suspicious-pattern`: Common mistakes (e.g., `><` no-op)

---

### 4. Arithmetic in @const

**Priority: Medium | Complexity: Low**

Allow expressions in constant definitions:
```
@const BASE 65
@const RANGE 26
@const END (BASE + RANGE)
@const DOUBLE (BASE * 2)
```

Supported operators: `+`, `-`, `*`, `/`, `%`, parentheses.
Constants can reference other constants.

---

### 5. BF Dialect Converter (`ogre convert`)

**Priority: Low-Medium | Complexity: Low**

Convert between BF and its dialect variants:

```bash
ogre convert --to ook program.bf
ogre convert --to trollscript program.bf
ogre convert --from ook program.ook
```

#### Supported Dialects
- **Ook!**: `Ook. Ook?` etc.
- **Trollscript**: `ooo` `ool` `olo` etc.
- **Blub**: `Blub. Blub?` etc.
- Custom mapping via `--map '+-><.,[]' 'abcdefgh'`

---

### 6. Project Templates (`ogre new --template`)

**Priority: Low | Complexity: Low**

Scaffold different project types:

```bash
ogre new myproject --template game      # Interactive I/O loop
ogre new myproject --template library   # Library with exports
ogre new myproject --template converter # CLI tool with arg parsing
```

---

## Ideas Considered but Deferred

- **LSP Server**: Very high complexity, needs separate crate
- **Reverse Debugging**: Requires execution snapshot infrastructure
- **Package Manager**: Needs registry/hosting infrastructure
- **Web Playground**: Needs separate frontend project
- **Fuzzing**: Complex integration with AFL/libfuzzer
- **DAP (Debug Adapter Protocol)**: Medium-high complexity, needs LSP first
