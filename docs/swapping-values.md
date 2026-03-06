# Swapping Values in Brainfuck

Swapping two adjacent cell values is one of the fundamental operations
in brainfuck programming. It comes up constantly when reordering data,
implementing comparisons, and managing cell layouts. This document
explains how swapping works at the instruction level, the different
approaches, and practical usage.

---

## Table of Contents

- [The Problem](#the-problem)
- [Why Swapping Is Hard](#why-swapping-is-hard)
- [The Standard Swap Algorithm](#the-standard-swap-algorithm)
- [Step-by-Step Walkthrough](#step-by-step-walkthrough)
- [Using the stdlib swap](#using-the-stdlib-swap)
- [Alternative Approaches](#alternative-approaches)
- [Related Operations](#related-operations)
- [Common Pitfalls](#common-pitfalls)

---

## The Problem

Given two adjacent cells:

```
Cell:  [A] [B]
        ^
        pointer
```

We want to end up with:

```
Cell:  [B] [A]
        ^
        pointer
```

In a language with variables, this is trivial: `temp = a; a = b; b = temp;`.
In brainfuck, we have no variables, no random access, and the only way
to read a cell's value is to decrement it in a loop — which destroys it.

---

## Why Swapping Is Hard

The core difficulty is that brainfuck's loop construct `[...]` is the
only way to "read" a value, and it does so destructively. The loop
`[>+<-]` moves a value from cell 0 to cell 1, but cell 0 is now zero.
You cannot read a value and keep it in the same cell without using a
scratch cell.

This means:

- You cannot swap two values using only those two cells.
- You need at least one scratch cell (a third cell known to be zero).
- Every swap algorithm uses a temporary holding cell.

---

## The Standard Swap Algorithm

The ogre standard library (`std/memory.bf`) provides:

```brainfuck
@fn swap {
    [>>+<<-]>[<+>-]>[<+>-]<<
}
```

This swaps cell 0 and cell 1, using cell 2 as scratch. The pointer
starts and ends at cell 0.

**Requirements:**
- Pointer is at cell 0.
- Cell 2 must be zero before calling.
- After the swap, cell 2 is left at zero (cleaned up).

---

## Step-by-Step Walkthrough

Starting state:
```
Cell:    [A]  [B]  [0]
Index:    0    1    2
          ^
```

### Phase 1: Move A to the scratch cell

```brainfuck
[>>+<<-]
```

This loop runs A times. Each iteration: move right 2 (to cell 2), add
1, move left 2 (back to cell 0), subtract 1.

```
Before:  [A]  [B]  [0]    ptr=0
After:   [0]  [B]  [A]    ptr=0
```

Cell 0 is now empty. A is safely stored in cell 2.

### Phase 2: Move B to cell 0

```brainfuck
>[<+>-]
```

Move pointer right 1 to cell 1 (which holds B). The loop runs B times:
move left 1 (to cell 0), add 1, move right 1 (back to cell 1),
subtract 1.

```
Before:  [0]  [B]  [A]    ptr=1
After:   [B]  [0]  [A]    ptr=1
```

Cell 0 now holds B. Cell 1 is empty.

### Phase 3: Move A from scratch to cell 1

```brainfuck
>[<+>-]
```

Move pointer right 1 to cell 2 (which holds A). The loop runs A times:
move left 1 (to cell 1), add 1, move right 1 (back to cell 2),
subtract 1.

```
Before:  [B]  [0]  [A]    ptr=2
After:   [B]  [A]  [0]    ptr=2
```

Cell 1 now holds A. The scratch cell is clean.

### Phase 4: Return the pointer

```brainfuck
<<
```

Move left 2 back to cell 0.

```
Final:   [B]  [A]  [0]    ptr=0
```

The swap is complete. Values are exchanged, scratch cell is zero,
pointer is back where it started.

---

## Using the stdlib swap

Import the memory module and call swap:

```brainfuck
@import "std/memory.bf"

Set up cell 0 = 3, cell 1 = 7
+++>+++++++<

@call swap

Now cell 0 = 7, cell 1 = 3
```

### Swapping non-adjacent cells

The stdlib `swap` only works on adjacent cells (0 and 1 relative to
the pointer). To swap non-adjacent cells, you have two options:

**Option 1:** Move values to be adjacent, swap, move back.

```brainfuck
@import "std/memory.bf"
@import "std/math.bf"

Cell layout: [A] [X] [B]
To swap A and B:

Step 1: Move B to cell 1
>>                        Pointer at cell 2 (B)
[<<+>>-]                  Move B to cell 0... wait, that overwrites A.
```

This gets complicated quickly. For non-adjacent swaps, it is usually
easier to use a chain of moves through temporary cells.

**Option 2:** Use a rotate pattern. The stdlib provides `rotate3` for
three-cell rotation:

```brainfuck
@fn rotate3 {
    [>>>+<<<-]
    >[<<+>>-]<
    >>[<<<+>>>-]<<<
    >>[<+>-]<<
}
```

This rotates cells 0, 1, 2: `[A][B][C]` becomes `[C][A][B]`.

### Swapping at an offset

To swap cells that are not at the current pointer position, navigate
to them first:

```brainfuck
@import "std/memory.bf"

Cell layout: [X] [Y] [A] [B] [Z]
Want to swap A and B (cells 2 and 3)

>> @call swap <<

Pointer moved to cell 2, swapped cells 2 and 3, moved back to cell 0
```

---

## Alternative Approaches

### The minimal swap (same algorithm, written differently)

The standard algorithm can be written equivalently as three move
operations:

```brainfuck
Move cell 0 to cell 2:     [>>+<<-]
Move cell 1 to cell 0:     >[<+>-]
Move cell 2 to cell 1:     >[<+>-]
Return pointer:             <<
```

Some programmers write this more compactly or with different scratch
cell choices.

### Using a left scratch cell

If the cell to the left is available (cell -1 relative to the pointer),
you can swap without needing the cell to the right to be free:

```brainfuck
Swap using cell to the left as scratch
Pointer at cell 1, swapping cells 1 and 2, scratch is cell 0

[<+>-]        Move cell 1 to cell 0 (scratch)
>[<+>-]       Move cell 2 to cell 1
<[>>+<<-]     Move cell 0 (scratch) to cell 2
>>            Adjust pointer
```

The choice depends on which neighbouring cells are available as scratch
space.

### XOR swap? Not in brainfuck

In traditional programming, XOR swap (`a ^= b; b ^= a; a ^= b`)
avoids a temporary variable. This does not work in brainfuck because
there is no XOR instruction. The only operations are increment,
decrement, and loop-until-zero — all of which are additive/subtractive
rather than bitwise.

---

## Related Operations

### Move (destructive transfer)

Moving a value is simpler than swapping — it is just one loop:

```brainfuck
@fn move_right { [>+<-]> }
@fn move_left  { [<+>-]< }
@fn add_to_next { [>+<-] }
```

`move_right` transfers cell 0 to cell 1 (cell 0 becomes 0) and
advances the pointer. This is the building block of the swap algorithm.

### Copy (non-destructive transfer)

Copying requires a scratch cell because you must "read" the source
without destroying it:

```brainfuck
@fn copy_right { [>+>+<<-]>>[<<+>>-]<< }
```

This copies cell 0 to cell 1, using cell 2 as scratch:

1. `[>+>+<<-]` — distribute cell 0 into cells 1 and 2 (cell 0 is
   destroyed).
2. `>>[<<+>>-]<<` — move cell 2 back to cell 0 (restoring it).

Result: cell 0 = original value, cell 1 = copy.

### Push and pull

The stdlib provides directional moves that also shift the pointer:

```brainfuck
@fn push_right { [>+<-]> }    Move value right, pointer follows
@fn pull_left  { <[>+<-] }    Pull value from left to current cell
```

These are useful for "sliding" values along the tape.

---

## Common Pitfalls

### Forgetting to zero the scratch cell

The swap algorithm assumes cell 2 is zero. If it contains a nonzero
value, the result will be corrupted:

```
Before:  [3]  [7]  [5]    cell 2 is NOT zero
After swap:
  Phase 1: [0]  [7]  [8]  (A=3 added to existing 5)
  Phase 2: [7]  [0]  [8]
  Phase 3: [7]  [8]  [0]  WRONG: cell 1 should be 3, not 8
```

Always ensure scratch cells are zero before calling swap. Use
`@call clear` or `[-]` if in doubt.

### Losing track of the pointer

After a swap, the pointer is back at cell 0 (relative to where it was
when you called swap). If you forget to account for pointer movement in
your surrounding code, subsequent operations will target the wrong cell.

### Swapping with zero

Swapping works correctly even when one or both values are zero. The
loop `[>>+<<-]` simply does not execute if cell 0 is zero, which is
the correct behaviour (zero is already "moved").

### Cell overflow

If your cells are 8-bit (the brainfuck default, values 0-255), the
swap itself cannot cause overflow because it only moves values — it
never adds values together. However, if you are combining a swap with
arithmetic, be mindful of wrapping.
