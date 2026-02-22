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

`Program::optimize()` runs three passes in sequence:

### 1. Clear Idiom Recognition

Detects `JumpIfZero -> Sub(1) -> JumpIfNonZero` patterns (i.e., `[-]`) and
replaces them with a single `Clear` op.

**Before:** `[JumpIfZero(2), Sub(1), JumpIfNonZero(0)]` (3 ops)
**After:** `[Clear]` (1 op)

This also handles `[+]` which is functionally equivalent to `[-]` for
u8-wrapping cells.

### 2. Cancellation

Merges or cancels adjacent operations that undo each other:

| Pattern | Result |
|---------|--------|
| `Add(n)` followed by `Sub(m)` | `Add(n-m)` if n>m, `Sub(m-n)` if m>n, removed if equal |
| `Sub(n)` followed by `Add(m)` | Same logic, reversed |
| `Right(n)` followed by `Left(m)` | `Right(n-m)` if n>m, `Left(m-n)` if m>n, removed if equal |
| `Left(n)` followed by `Right(m)` | Same logic, reversed |

Zero-valued ops (`Add(0)`, `Right(0)`) are removed.

### 3. Dead Store Elimination

If a `Clear` is followed by `Add(n)` with no reads in between, the
`Clear` is redundant (the cell will be overwritten). Both are replaced
with `Set(n)` (currently kept as `Add(n)` since the cell was just cleared).

### Jump Reindexing

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
