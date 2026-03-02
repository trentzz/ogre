# Standard Library Reference

The ogre standard library is a collection of reusable brainfunct functions
embedded in the ogre binary. Import modules with `@import "std/module.bf"`.

Run `ogre stdlib list` to see available modules, or
`ogre stdlib show <module>` to view a module's source.

## Conventions

- Functions that print characters assume cell 0 starts at 0. They add the
  ASCII value, print with `.`, then clear with `[-]`.
- Functions document which cells they use as scratch. Scratch cells are
  assumed to be 0 on entry and are cleared on exit.
- The data pointer returns to cell 0 unless documented otherwise.

---

## std/io.bf — Input/Output

| Function | Description | Cell usage |
|----------|-------------|------------|
| `print_newline` | Print newline (ASCII 10) | Cell 0: set and cleared |
| `print_space` | Print space (ASCII 32) | Cell 0: set and cleared |
| `print_tab` | Print tab (ASCII 9) | Cell 0: set and cleared |
| `print_zero` | Print '0' (ASCII 48) | Cell 0: set and cleared |
| `print_bang` | Print '!' (ASCII 33) | Cell 0: set and cleared |
| `print_dot` | Print '.' (ASCII 46) | Cell 0: set and cleared |
| `print_comma` | Print ',' (ASCII 44) | Cell 0: set and cleared |
| `print_hash` | Print '#' (ASCII 35) | Cell 0: set and cleared |
| `print_at` | Print '@' (ASCII 64) | Cell 0: set and cleared |
| `print_star` | Print '*' (ASCII 42) | Cell 0: set and cleared |
| `print_eq` | Print '=' (ASCII 61) | Cell 0: set and cleared |
| `print_gt` | Print '>' (ASCII 62) | Cell 0: set and cleared |
| `print_lt` | Print '<' (ASCII 60) | Cell 0: set and cleared |
| `print_quote` | Print '"' (ASCII 34) | Cell 0: set and cleared |
| `print_slash` | Print '/' (ASCII 47) | Cell 0: set and cleared |
| `print_backslash` | Print '\\' (ASCII 92) | Cell 0: set and cleared |
| `print_underscore` | Print '_' (ASCII 95) | Cell 0: set and cleared |
| `print_pipe` | Print '\|' (ASCII 124) | Cell 0: set and cleared |
| `print_lbracket` | Print '[' (ASCII 91) | Cell 0: set and cleared |
| `print_rbracket` | Print ']' (ASCII 93) | Cell 0: set and cleared |
| `print_lparen` | Print '(' (ASCII 40) | Cell 0: set and cleared |
| `print_rparen` | Print ')' (ASCII 41) | Cell 0: set and cleared |
| `read_char` | Read one byte from stdin into cell 0 | Cell 0: modified |
| `print_char` | Print cell 0 as a character | Cell 0: unchanged |

---

## std/math.bf — Arithmetic

| Function | Description | Cell usage |
|----------|-------------|------------|
| `zero` | Clear cell 0 to 0 | Cell 0 |
| `inc` | Add 1 to cell 0 | Cell 0 |
| `dec` | Subtract 1 from cell 0 | Cell 0 |
| `inc5` | Add 5 to cell 0 | Cell 0 |
| `dec5` | Subtract 5 from cell 0 | Cell 0 |
| `inc10` | Add 10 to cell 0 | Cell 0 |
| `dec10` | Subtract 10 from cell 0 | Cell 0 |
| `double` | Double cell 0 | Cell 1 as scratch |
| `triple` | Triple cell 0 | Cell 1 as scratch |
| `multiply_by_10` | Multiply cell 0 by 10 | Cell 1 as scratch |
| `divmod_10` | Divide cell 0 by 10; quotient in cell 0, remainder in cell 1 | Cells 0-5 |
| `add_to_next` | Move cell 0 to cell 1 (destructive) | Cell 0 zeroed |
| `move_right` | Move cell 0 to cell 1 and advance pointer | Pointer ends at cell 1 |
| `move_left` | Move cell 0 to cell -1 and retreat pointer | Pointer ends at cell -1 |
| `copy_right` | Copy cell 0 to cell 1 (non-destructive) | Cell 2 as scratch |
| `is_zero` | Set cell 1 to 1 if cell 0 is 0, else 0 | Cells 1-2 as scratch |
| `is_nonzero` | Set cell 0 to 1 if nonzero (destructive) | Cell 0 modified |
| `negate` | 256 minus cell 0 (wrapping negation) | Cell 1 as scratch |
| `abs_diff` | Absolute difference of cell 0 and cell 1 into cell 0 | Cell 1 zeroed |
| `min` | min(cell 0, cell 1) into cell 0 | Cells 2-4 as scratch |
| `max` | max(cell 0, cell 1) into cell 0 | Cells 2-4 as scratch |

**Note:** `double`, `triple`, `multiply_by_10`, and `add_to_next` are
destructive — they consume (zero) the source cell via loops. Use `copy_right`
first if you need to preserve the original value.

---

## std/memory.bf — Memory/Tape Utilities

| Function | Description | Cell usage |
|----------|-------------|------------|
| `clear` | Zero cell 0 | Cell 0 |
| `clear2` | Zero cells 0-1 | Cells 0-1 |
| `clear3` | Zero cells 0-2 | Cells 0-2 |
| `clear4` | Zero cells 0-3 | Cells 0-3 |
| `clear5` | Zero cells 0-4 | Cells 0-4 |
| `clear_right` | Zero cell 1 without moving pointer | Cell 1 |
| `swap` | Swap cell 0 and cell 1 | Cell 2 as scratch |
| `copy_right` | Copy cell 0 to cell 1 (non-destructive) | Cell 2 as scratch |
| `copy_left` | Copy cell 0 to cell -1 (non-destructive) | Cell 1 as scratch |
| `dup` | Duplicate cell 0 into cell 1 (alias for copy_right) | Cell 2 as scratch |
| `push_right` | Move cell 0 to cell 1 and advance pointer | Pointer ends at cell 1 |
| `pull_left` | Move cell -1 to cell 0 | Cell -1 zeroed |
| `zero_range_5` | Zero cells 0-4 | Cells 0-4 |
| `rotate3` | Rotate cells 0,1,2 left: 0->2, 1->0, 2->1 | Cells 0-2 modified |

---

## std/ascii.bf — ASCII Utilities

| Function | Description | Cell usage |
|----------|-------------|------------|
| `print_A` | Print 'A' (ASCII 65) | Cell 0: set and cleared |
| `print_B` | Print 'B' (ASCII 66) | Cell 0: set and cleared |
| `print_exclaim` | Print '!' (ASCII 33) | Cell 0: set and cleared |
| `print_dash` | Print '-' (ASCII 45) | Cell 0: set and cleared |
| `print_colon` | Print ':' (ASCII 58) | Cell 0: set and cleared |
| `to_upper` | Subtract 32 from cell 0 (lowercase to uppercase) | Cell 0 |
| `to_lower` | Add 32 to cell 0 (uppercase to lowercase) | Cell 0 |
| `is_digit` | Set cell 1 to 1 if cell 0 is '0'-'9' | Cells 1-3 as scratch |
| `is_space` | Set cell 1 to 1 if cell 0 is space (32) | Cells 1-2 as scratch |
| `digit_to_char` | Add 48 to cell 0 (0-9 to '0'-'9') | Cell 0 |
| `char_to_digit` | Subtract 48 from cell 0 ('0'-'9' to 0-9) | Cell 0 |

**Warning:** `to_upper` and `to_lower` blindly add/subtract 32. They only
work correctly for ASCII letters (a-z / A-Z).

---

## std/string.bf — String/Text Operations

| Function | Description | Cell usage |
|----------|-------------|------------|
| `skip_char` | Read and discard one character from stdin | Cell 0 |
| `skip_spaces` | Read and discard until non-space | Cells 0-1 |
| `skip_line` | Read and discard until newline | Cell 0 |
| `read_decimal` | Read decimal digits from stdin, accumulate into cell 0 | Cells 0-2 as scratch |

---

## std/logic.bf — Boolean/Conditional Logic

| Function | Description | Cell usage |
|----------|-------------|------------|
| `not` | If cell 0 is 0, set to 1; if nonzero, set to 0 | Cell 1 as scratch |
| `bool` | Normalize cell 0 to boolean (nonzero becomes 1) | Cell 1 as scratch |
| `and` | Logical AND of cell 0 and cell 1 into cell 0 | Cell 1 consumed |
| `or` | Logical OR of cell 0 and cell 1 into cell 0 | Cell 1 consumed |
| `equal` | Set cell 2 to 1 if cell 0 equals cell 1 | Cells 0-3 used |

---

## std/debug.bf — Debugging Helpers

| Function | Description | Cell usage |
|----------|-------------|------------|
| `dump_cell` | Print cell 0 as raw byte | Cell 0: unchanged |
| `dump_and_newline` | Print cell 0 as raw byte, then newline | Cell 0: modified |
| `marker_start` | Print '<' as a debug marker | Cell 1: set and cleared |
| `marker_end` | Print '?' as a debug marker | Cell 1: set and cleared |

---

## Usage Example

```brainfuck
@import "std/io.bf"
@import "std/math.bf"
@import "std/ascii.bf"

Set cell 0 to 65 (ASCII 'A')
@const A_VALUE 65
@use A_VALUE

Print 'A'
.

Move to cell 1 and convert to lowercase
@call copy_right
>
@call to_lower

Print 'a'
.

@call print_newline
```

## Adding to the Standard Library

Standard library modules live in `stdlib/*.bf`. Each file contains only
`@fn` definitions (no top-level code). To add a new function:

1. Edit the appropriate `stdlib/*.bf` file
2. Add `@doc` comments above the function
3. The function is automatically available after recompilation
4. Add tests to `examples/stdlib-tests/` to verify behavior
