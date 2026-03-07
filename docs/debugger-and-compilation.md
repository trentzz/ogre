# Debugger and Compilation Guide

ogre ships with a GDB-style interactive debugger and two compilation backends
(native via C and WebAssembly via WAT). Both share the same IR and optimization
pipeline. This guide covers practical usage, the command reference, and the
internals that make it all work.

---

## Part 1: Debugger

### 1. Getting Started

Launch the debugger on any brainfuck file:

```
ogre debug hello.bf
```

The debugger preprocesses the file (expanding all `@import`, `@fn`, and `@call`
directives), builds a source map, then pauses before executing the first
instruction. You will see a status line and a short tape window:

```
ogre debugger -- type 'help' for commands
  (source map loaded -- showing file/line/function info)
  ip=0  op=Add(3)  dp=0  val=0  src/main.bf:1:1
  tape: [ >0:0<  1:0  2:0 ]
(ogre-dbg)
```

The `ip` is the instruction pointer (index into the IR op array), `op` is the
current IR operation, `dp` is the data pointer (current tape cell), and `val` is
the value in that cell.

If you are inside an ogre project (a directory with `ogre.toml`), you can omit
the file argument and the debugger will load the project entry point:

```
ogre debug
```

Use `--tape-size` to override the default 30,000-cell tape:

```
ogre debug --tape-size 60000 big.bf
```

### 2. Commands Reference

Type `help` at the `(ogre-dbg)` prompt for a quick summary. The full reference
follows.

#### step / step \<n\>

Execute one instruction (or `n` instructions) and pause.

```
(ogre-dbg) step
  ip=1  op=Right(1)  dp=0  val=3  src/main.bf:1:4
  tape: [ >0:3<  1:0  2:0 ]

(ogre-dbg) step 5
  ip=6  op=Sub(1)  dp=2  val=0  src/main.bf:2:8
  tape: [ 0:3  1:0  >2:0<  3:0  4:0 ]
```

If the program ends during stepping, the debugger prints "Program finished."

#### continue / c

Run until a breakpoint is hit or the program ends.

```
(ogre-dbg) breakpoint 10
Breakpoint set at op 10 (Output)

(ogre-dbg) continue
Hit breakpoint at 10.
  ip=10  op=Output  dp=0  val=72  src/main.bf:3:1
  tape: [ >0:72<  1:0  2:0 ]
```

#### breakpoint \<n\>

Set a breakpoint at IR op index `n`. The debugger confirms the breakpoint and
shows which instruction lives at that index.

```
(ogre-dbg) breakpoint 4
Breakpoint set at op 4 (JumpIfZero(8))
```

#### breakpoint list

List all active breakpoints, sorted by index. If a source map is loaded, the
original file, line, and function name are shown alongside each entry.

```
(ogre-dbg) breakpoint list
  breakpoint 4 -> JumpIfZero(8)  src/main.bf:2:1
  breakpoint 10 -> Output  src/main.bf:3:1
```

#### breakpoint delete \<n\>

Remove the breakpoint at op index `n`.

```
(ogre-dbg) breakpoint delete 4
Breakpoint 4 removed.
```

#### jump \<n\>

Move the instruction pointer to op index `n` without executing anything. This is
useful for skipping past a section or rewinding to an earlier point.

```
(ogre-dbg) jump 0
Jumped to op 0.
  ip=0  op=Add(3)  dp=0  val=72  src/main.bf:1:1
  tape: [ >0:72<  1:0  2:0 ]
```

Note that `jump` does not reset the tape or data pointer. Only the code pointer
moves.

#### peek / peek \<n\>

Show a window of tape cells centred on the current data pointer (or on cell
`n`). The default radius is 5 cells in each direction.

```
(ogre-dbg) peek
  tape: [ 0:0  1:0  2:0  3:0  4:0  >5:72<  6:0  7:0  8:0  9:0  10:0 ]

(ogre-dbg) peek 0
  tape: [ >0:72<  1:0  2:0  3:0  4:0  5:0 ]
```

The cell marked with `>...<` is the one the data pointer currently sits on.

#### show instruction / show instruction \<n\>

Display the current instruction (or instruction at index `n`) with its
surrounding context. Three ops before and after are shown, and the target
instruction is highlighted.

```
(ogre-dbg) show instruction
  op 10: Output (context: Sub(1) Right(1) Add(1) [Output] JumpIfNonZero(4) Right(2) Add(1))

(ogre-dbg) show instruction 4
  op 4: JumpIfZero(8) (context: Add(3) Right(1) Add(1) [JumpIfZero(8)] Sub(1) Right(1) Add(1))  src/main.bf:2:1
```

#### show memory

Dump a wider range of memory cells around the current data pointer (radius of
10). This is the same format as `peek` but with a larger window.

```
(ogre-dbg) show memory
  tape: [ 0:0  1:0  2:0  ... >15:72< ... 20:0  21:0 ]
```

#### where

Print the current source location: file path, line number, column, and the
enclosing `@fn` name if the instruction originated from a function expansion.

```
(ogre-dbg) where
  10 -> src/greet.bf:3:5 (@fn greet)
```

If no source map is loaded (for example when debugging a plain BF string), the
output falls back to just the op index:

```
(ogre-dbg) where
  ip=10 (no source map)
```

#### cbreak \<op\> \<cell\> \<cond\> \<val\>

Set a conditional breakpoint that triggers only when a specific cell meets a
condition. The debugger pauses at the given op index only if the condition is
true at that moment.

Available conditions:
- `eq` -- cell value equals val
- `ne` -- cell value does not equal val
- `gt` -- cell value is greater than val
- `lt` -- cell value is less than val

```
(ogre-dbg) cbreak 10 0 eq 72
Conditional breakpoint #0 set at op 10 when cell[0] == 72

(ogre-dbg) cbreak 5 1 gt 0
Conditional breakpoint #1 set at op 5 when cell[1] > 0

(ogre-dbg) cbreak list
  #0: op 10 when cell[0] == 72
  #1: op 5 when cell[1] > 0

(ogre-dbg) cbreak delete 0
Conditional breakpoint #0 removed.
```

Conditional breakpoints are checked during `continue`. If the instruction
pointer reaches the op index and the condition evaluates to true, execution
pauses.

#### watch \<cell\>

Set a watchpoint on a tape cell. The debugger pauses whenever the cell's value
changes during execution with `continue`.

```
(ogre-dbg) watch 0
Watchpoint #0 on cell[0] (current value: 0)

(ogre-dbg) continue
Watchpoint triggered cell[0] changed: 0 → 3
  ip=1  op=Right(1)  dp=0  val=3
  tape: [ >0:3<  1:0  2:0 ]

(ogre-dbg) watch list
  #0: cell[0] last=3 current=3

(ogre-dbg) watch delete 0
Watchpoint #0 removed.
```

Watchpoints track the last known value and trigger when the current value
differs after any instruction is executed. Multiple watchpoints can be active
simultaneously.

#### exit / quit / q

Quit the debugger immediately.

### 3. Source Mapping

When ogre preprocesses a file, the `@import`, `@fn`, and `@call` directives are
expanded into a flat brainfuck string. The debugger builds a **source map** that
tracks, for every character in the expanded output, which original file, line,
column, and function it came from.

The source map is constructed in the preprocessor during expansion
(`Preprocessor::process_file_with_map`). Each `SourceLocation` entry records:

- **file** -- the path to the original `.bf` file.
- **line / column** -- 1-based position within that file.
- **function** -- the `@fn` name, if the code was inlined from a function body.

Because the IR collapses consecutive identical characters (e.g. `+++` becomes a
single `Add(3)` op), the debugger maintains an **op-to-char map** that maps each
IR op index back to the character position of the first character that produced
it. The source map is then consulted at that character position.

This means that when you `step` through the IR, each op can be traced back to
its original location -- even across file and function boundaries.

### 4. Breakpoint Strategies

Breakpoints are set on **IR op indices**, not on source line numbers. A few tips
for working with them effectively:

- Use `show instruction 0` and step forward to discover the op index you want.
  The context display around each instruction makes it easier to find loop
  boundaries and I/O operations.
- Set breakpoints on `Output` ops to pause just before the program prints a
  character. This lets you inspect what value is about to be printed.
- Set breakpoints on `JumpIfZero` ops (loop headers) to pause each time a loop
  is entered.
- Use `where` after hitting a breakpoint to see which function the instruction
  came from. This is especially helpful when debugging brainfunct projects with
  many `@fn` definitions spread across files.
- Use `breakpoint list` regularly to review active breakpoints and clean up ones
  you no longer need with `breakpoint delete`.

### 5. Memory Inspection

The tape is a flat array of unsigned bytes (0--255). Use `peek` to see cells
near the data pointer, or `peek <n>` to look at a specific region.

Example workflow for understanding tape state:

```
(ogre-dbg) step 20
  ip=20  op=Output  dp=1  val=101
  tape: [ 0:72  >1:101<  2:108  3:0  4:0 ]

(ogre-dbg) peek 0
  tape: [ >0:72<  1:101  2:108  3:0  4:0  5:0 ]
```

Cell 0 holds 72 (ASCII `H`), cell 1 holds 101 (ASCII `e`), cell 2 holds 108
(ASCII `l`). The program is building the string "Hel..." across consecutive
cells.

Use `show memory` for a wider view when working with programs that spread data
across many cells.

---

## Part 2: Compilation

### 6. Native Compilation

Compile a brainfuck file to a native binary:

```
ogre compile hello.bf -o hello
```

The pipeline is:

1. **Preprocess** -- expand `@import`, `@fn`, `@call` into pure brainfuck.
2. **Parse to IR** -- convert the brainfuck string into an `Op` array with
   run-length collapsing and bracket pairing.
3. **Optimize IR** -- apply clear-idiom, move-idiom, cancellation, and
   dead-store passes (see section 8).
4. **Generate C** -- emit a self-contained C program.
5. **Invoke compiler** -- call `cc`, `gcc`, or `clang` with `-O2`.

The generated C uses `unsigned char` for cells and `memset` for tape
initialization:

```c
#include <stdio.h>
#include <string.h>
int main() {
    unsigned char array[30000];
    memset(array, 0, sizeof(array));
    unsigned char *ptr = array;
    /* ... ops ... */
    return 0;
}
```

IR operations map directly to C constructs:

| IR Op | C Output |
|---|---|
| `Add(1)` | `(*ptr)++;` |
| `Add(n)` | `*ptr += n;` |
| `Sub(1)` | `(*ptr)--;` |
| `Right(1)` | `ptr++;` |
| `Left(n)` | `ptr -= n;` |
| `Output` | `putchar(*ptr);` |
| `Input` | `*ptr = getchar();` |
| `JumpIfZero` | `while (*ptr) {` |
| `JumpIfNonZero` | `}` |
| `Clear` | `*ptr = 0;` |
| `MoveAdd(offset)` | `*(ptr + offset) += *ptr; *ptr = 0;` |
| `Set(n)` | `*ptr = n;` |
| `ScanRight` | `while (*ptr) ptr++;` |
| `ScanLeft` | `while (*ptr) ptr--;` |
| `MultiplyMove(targets)` | Multi-target distribution loop |

ogre tries `cc`, `gcc`, and `clang` in that order and uses the first one it
finds. If none is available, it reports an error.

The intermediate `.c` file is written to a temp directory and deleted after
compilation. Use `--keep` (`-k`) to retain it in the current directory:

```
ogre compile hello.bf -o hello --keep
# produces: hello (binary) + hello.c (intermediate)
```

When no `-o` is given, the output name is derived from the input filename
(e.g. `hello.bf` produces `hello`).

### 7. WASM Compilation

Compile to WebAssembly using the `--target wasm` flag:

```
ogre compile --target wasm hello.bf
```

This generates a `.wat` file (WebAssembly Text Format). If `wat2wasm` from the
[wabt](https://github.com/WebAssembly/wabt) toolkit is available on your PATH,
ogre automatically converts it to a `.wasm` binary and removes the intermediate
`.wat` file:

```
Compiled to: hello.wasm
```

If `wat2wasm` is not found, ogre keeps the `.wat` file and prints a hint:

```
Generated WAT: hello.wat
  (install wabt's wat2wasm to compile to .wasm binary)
```

The generated WAT module uses **WASI** (WebAssembly System Interface) for I/O:

- `fd_write` (from `wasi_snapshot_preview1`) for the `.` operator, writing to
  stdout (fd 1).
- `fd_read` (from `wasi_snapshot_preview1`) for the `,` operator, reading from
  stdin (fd 0).

**Memory layout:**

```
[0 .. tape_size-1]         BF tape (one byte per cell)
[tape_size .. tape_size+7] iov buffer (pointer + length for fd_write/fd_read)
[tape_size+8 .. +11]       nwritten/nread result word
```

The number of WASM linear memory pages is calculated from the tape size plus the
scratch area. For a 30,000-cell tape this is 1 page (64 KB).

Loops are compiled using the `block`/`loop`/`br_if` pattern:

```wat
(block $B
  (loop $L
    (br_if $B (i32.eqz (i32.load8_u (global.get $dp))))
    ;; loop body
    (br_if $L (i32.load8_u (global.get $dp)))
  )
)
```

The data pointer is a mutable global `$dp`. All cell accesses use `i32.load8_u`
and `i32.store8` with masking to 255 to match brainfuck's unsigned byte
semantics.

To run the compiled WASM, use a WASI-compatible runtime such as `wasmtime`:

```
ogre compile --target wasm hello.bf
wasmtime hello.wasm
```

### 8. IR Optimizations

Before code generation (for both native and WASM targets), the IR goes through
several optimization passes. These run in order:

#### Run-Length Encoding

During parsing, consecutive identical operators are collapsed into a single op
with a count:

```
+++       ->  Add(3)
>>>>>     ->  Right(5)
---       ->  Sub(3)
<<<       ->  Left(3)
```

This happens at parse time in `Program::from_source`, not as a separate pass.

#### Clear Idiom

The pattern `[-]` (decrement until zero) is recognized and replaced with a
single `Clear` op:

```
JumpIfZero, Sub(1), JumpIfNonZero  ->  Clear
```

In generated C, this becomes `*ptr = 0;` instead of a loop.

#### Move Pattern

Common data-movement idioms are detected and collapsed:

```
[->+<]    ->  MoveAdd(1)     // add current cell to cell+1, zero current
[->>+<<]  ->  MoveAdd(2)     // add current cell to cell+2, zero current
[-<+>]    ->  MoveAdd(-1)    // add current cell to cell-1, zero current
[->-<]    ->  MoveSub(1)     // subtract current cell from cell+1, zero current
```

The general form is `[- {move} {add|sub} {move-back}]` where the forward and
backward moves have equal distance.

In generated C, `MoveAdd(1)` becomes:

```c
*(ptr + 1) += *ptr; *ptr = 0;
```

This eliminates the loop entirely.

#### Cancellation

Adjacent opposite operations are merged or cancelled:

```
+++--     ->  Add(1)          // 3 - 2 = 1
>><<<     ->  Left(1)         // 2 right, 3 left = 1 left
+-        ->  (nothing)       // fully cancelled
><        ->  (nothing)       // fully cancelled
```

The pass also merges adjacent same-direction operations:

```
Add(3), Add(2)  ->  Add(5)
Right(2), Right(3)  ->  Right(5)
```

This runs iteratively until no more changes are found.

#### Scan Idiom

The patterns `[>]` (scan right) and `[<]` (scan left) are recognized and
replaced with `ScanRight` / `ScanLeft` ops. These move the pointer until a
zero cell is found, without looping through individual increment/decrement
steps.

```
JumpIfZero, Right(1), JumpIfNonZero  ->  ScanRight
JumpIfZero, Left(1), JumpIfNonZero   ->  ScanLeft
```

In generated C, this becomes `while (*ptr) ptr++;` or `while (*ptr) ptr--;`.

#### Multiply-Move

Complex loop patterns that distribute the current cell's value to multiple
targets with multiplication factors are detected:

```
[->>+++>++<<<]  ->  MultiplyMove(vec![(2, 3), (3, 2)])
```

This adds `current * 3` to cell+2, `current * 2` to cell+3, and zeros the
current cell -- all without a loop.

#### Set Idiom

A `Clear` (or `Set(0)`) followed immediately by `Add(n)` is folded into a
single `Set(n)`:

```
Clear, Add(3)  ->  Set(3)
```

Standalone `Clear` ops are also converted to `Set(0)` for consistency.

#### Jump Reindexing

After ops are inserted or removed by optimization passes, all `JumpIfZero` and
`JumpIfNonZero` targets are recalculated to point to the correct indices.

### 9. Build Command

The `ogre build` command is the project-level compilation workflow. It reads
`ogre.toml`, resolves the entry file, collects dependency functions from all
included files, and compiles to a native binary named after the project:

```
ogre build
# Built myproject v0.1.0 -- My brainfuck project (by Alice)
# Compiled to: myproject
```

Override the output name with `-o`:

```
ogre build -o app
```

Keep the intermediate C file with `-k` / `--keep`:

```
ogre build --keep
```

The build command uses the same IR optimization pipeline and C code generation as
`ogre compile`. The difference is that `build` reads the project manifest,
resolves all `include` paths, pre-loads `@fn` definitions from dependency files,
and names the output after `project.name` by default.
