# memory -- Cell Manipulation and Memory Operations

```brainfuck
@import "std/memory.bf"
```

The `memory` module provides functions for clearing cells, moving and copying values between cells, swapping, duplicating, rotating, and shifting data on the tape.

## Function Reference

### Clearing Cells

| Function | Description | Cells Affected | Pointer After |
|---|---|---|---|
| `clear` | Clear cell 0 to zero | c0 | c0 |
| `clear2` | Clear cells 0--1 to zero | c0, c1 | c0 |
| `clear3` | Clear cells 0--2 to zero | c0, c1, c2 | c0 |
| `clear4` | Clear cells 0--3 to zero | c0, c1, c2, c3 | c0 |
| `clear5` | Clear cells 0--4 to zero | c0--c4 | c0 |
| `clear_right` | Clear cell 1 (one cell to the right) | c1 | c0 |
| `zero_range_5` | Clear cells 0--4 (same as `clear5`) | c0--c4 | c0 |
| `clear_range_10` | Clear 10 cells starting from c0 | c0--c9 | c0 |

### Moving Values

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `push_right` | Move c0 to c1 (c0 zeroed), advance pointer | c0, c1 | c1 |
| `pull_left` | Move c(-1) to c0 (c(-1) zeroed), pointer stays | c(-1), c0 | c0 |

### Copying and Duplicating

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `copy_right` | Copy c0 to c1 (c0 preserved). Uses c2 as scratch. | c0, c1, c2 | c0 |
| `copy_left` | Copy c0 to c(-1) (c0 preserved). Uses c2 as scratch. | c(-1), c0, c2 | c0 |
| `dup` | Duplicate c0 into c1 (same as `copy_right`). Uses c2 as scratch. | c0, c1, c2 | c0 |
| `fill_5` | Copy c0 into cells 0--4 (fill 5 cells with same value). Uses c5 as scratch. c0 preserved. | c0--c5 | c0 |

### Swapping

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `swap` | Swap c0 and c1. Uses c2 as scratch. | c0, c1, c2 | c0 |
| `swap_nonadj` | Swap c0 and c2 (non-adjacent). c1 used as temp and must be 0 initially. | c0, c1, c2 | c0 |

### Rotating and Shifting

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `rotate3` | Rotate cells 0, 1, 2 left: c0 <- c1, c1 <- c2, c2 <- old c0 | c0, c1, c2, c3 as scratch | c0 |
| `reverse3` | Reverse cells 0, 1, 2 (swap c0 and c2, c1 preserved). Uses c3 as scratch. | c0, c1, c2, c3 | c0 |
| `shift_right_3` | Shift cells 0,1,2 right: c2=old c1, c1=old c0, c0=0. Old c2 discarded. | c0, c1, c2 | c0 |

## Usage Example

```brainfuck
@import "std/memory.bf"

Put 5 in cell 0, 10 in cell 1, then swap them:
+++++>++++++++++<
@call swap
Cell 0 is now 10, cell 1 is now 5

Copy cell 0 to cell 1 (non-destructive):
@call clear2
++++++++
@call copy_right
Cell 0 = 8, cell 1 = 8

Fill 5 cells with the value 3:
@call clear5
+++
@call fill_5
Cells 0-4 are all 3
```
