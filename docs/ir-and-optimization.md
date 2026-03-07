# Bytecode IR and Optimization

## The Op Enum

The intermediate representation is a flat array of `Op` values:

```rust
pub enum Op {
    Add(u8),              // wrapping addition (collapses +++ into Add(3))
    Sub(u8),              // wrapping subtraction
    Right(usize),         // move data pointer right (collapses >>> into Right(3))
    Left(usize),          // move data pointer left
    Output,               // . — print current cell
    Input,                // , — read into current cell
    JumpIfZero(usize),    // [ — jump to target if cell is zero
    JumpIfNonZero(usize), // ] — jump to target if cell is non-zero
    Clear,                // [-] idiom — set current cell to zero
    MoveAdd(isize),       // [->+<] move-add pattern (add current cell to offset, zero current)
    MoveSub(isize),       // [->-<] move-sub pattern (sub current cell from offset, zero current)
    Set(u8),              // Set cell to a specific value (Clear + Add folded)
    ScanRight,            // [>] scan right until zero cell found
    ScanLeft,             // [<] scan left until zero cell found
    MultiplyMove(Vec<(isize, u8)>), // Multi-target multiply-move: distribute current cell
}
```

## Parsing (from_source)

`Program::from_source(source)` converts BF text to `Vec<Op>`:

1. **Filter:** Non-BF characters (`+-><.,[]`) are ignored (comments).
2. **Run-length encode:** Consecutive identical operations are collapsed:
   - `+++` -> `Add(3)`
   - `>>>` -> `Right(3)`
   - `---` -> `Sub(3)`
   - `<<<` -> `Left(3)`
3. **Bracket pairing:** A stack pairs each `[` with its matching `]`.
   Jump targets are stored directly in the Op variants:
   - `JumpIfZero(close_idx)` — index of the matching `]`
   - `JumpIfNonZero(open_idx)` — index of the matching `[`

**Errors:**
- `OgreError::UnmatchedCloseBracket` — `]` with no matching `[`
- `OgreError::UnmatchedOpenBracket(pos)` — `[` at position `pos` with no matching `]`

## Optimization Passes

`Program::optimize()` runs seven passes in sequence:

### 1. Clear Idiom Recognition

Detects `JumpIfZero -> Sub(1) -> JumpIfNonZero` patterns (i.e., `[-]`) and
replaces them with a single `Clear` op.

**Before:** `[JumpIfZero(2), Sub(1), JumpIfNonZero(0)]` (3 ops)
**After:** `[Clear]` (1 op)

This also handles `[+]` which is functionally equivalent to `[-]` for
u8-wrapping cells.

### 2. Move Idiom Recognition

Detects single-target move patterns like `[->+<]` and `[->-<]` and replaces
them with `MoveAdd(offset)` or `MoveSub(offset)` ops.

### 3. Scan Idiom Recognition

Detects `[>]` and `[<]` patterns and replaces them with `ScanRight` or
`ScanLeft` ops. These scan the tape in one direction until a zero cell is found.

**Before:** `[JumpIfZero(2), Right(1), JumpIfNonZero(0)]` (3 ops)
**After:** `[ScanRight]` (1 op)

### 4. Multiply-Move Recognition

Detects complex loop patterns that distribute the current cell's value to
multiple target cells with multiplication factors. For example, the pattern
`[->>+++>++<<<]` adds `current * 3` to cell+2 and `current * 2` to cell+3,
then zeros the current cell.

**Before:** A loop with multiple offset/add pairs (7+ ops)
**After:** `MultiplyMove(vec![(2, 3), (3, 2)])` (1 op)

The parser validates that the pointer movement returns to the origin (net zero
offset) and that the source cell is decremented by exactly 1 per iteration.

### 5. Cancellation

Merges or cancels adjacent operations that undo each other:

| Pattern | Result |
|---------|--------|
| `Add(n)` followed by `Sub(m)` | `Add(n-m)` if n>m, `Sub(m-n)` if m>n, removed if equal |
| `Sub(n)` followed by `Add(m)` | Same logic, reversed |
| `Right(n)` followed by `Left(m)` | `Right(n-m)` if n>m, `Left(m-n)` if m>n, removed if equal |
| `Left(n)` followed by `Right(m)` | Same logic, reversed |

Zero-valued ops (`Add(0)`, `Right(0)`) are removed. The pass runs iteratively
until no more changes are found.

### 6. Set Idiom Recognition

If a `Clear` (or `Set(0)`) is immediately followed by `Add(n)`, the two ops are
folded into a single `Set(n)`. This also converts standalone `Clear` ops to
`Set(0)` for consistency.

**Before:** `[Clear, Add(5)]` (2 ops)
**After:** `[Set(5)]` (1 op)

### 7. Jump Reindexing

After optimization passes may change the number of ops, all
`JumpIfZero`/`JumpIfNonZero` targets are recomputed to maintain correct
pairing.

## Back-Conversion (to_bf_string)

`Program::to_bf_string()` converts the IR back to BF text:

| Op | BF Output |
|----|-----------|
| `Add(n)` | n `+` characters |
| `Sub(n)` | n `-` characters |
| `Right(n)` | n `>` characters |
| `Left(n)` | n `<` characters |
| `Output` | `.` |
| `Input` | `,` |
| `JumpIfZero(_)` | `[` |
| `JumpIfNonZero(_)` | `]` |
| `Clear` | `[-]` |
| `Set(n)` | `[-]` followed by n `+` characters |
| `ScanRight` | `[>]` |
| `ScanLeft` | `[<]` |
| `MoveAdd(n)` | `[-` + n `>` + `+` + n `<` + `]` (for positive n) |
| `MoveSub(n)` | `[-` + n `>` + `-` + n `<` + `]` (for positive n) |
| `MultiplyMove(targets)` | Loop with multi-target distribution |

This is used by the `ogre pack --optimize` command to output optimized
pure BF.

## Usage in Modes

| Mode | How it uses the IR |
|------|-------------------|
| `run` | `new_optimized()` parses + optimizes, then executes |
| `compile` | Parses + optimizes, generates C from Op variants |
| `analyse` | Parses without optimization, inspects op patterns |
| `check` | Uses `from_source()` for bracket validation |
| `pack` | Optionally optimizes, then converts back to BF |
| `bench` | Uses optimized interpreter for benchmarking |
| `debug` | Parses without optimization for faithful debugging |
