# Standard Library Reference

The ogre standard library is a collection of reusable brainfunct functions
embedded in the ogre binary. Import modules with `@import "std/module.bf"`.

Run `ogre stdlib list` to see available modules, or
`ogre stdlib show <module>` to view a module's source.

## std/io — Input/Output

| Function | Description | Cell usage |
|----------|-------------|------------|
| `print_newline` | Print ASCII newline (10) | Modifies cell 0, resets to 0 |
| `print_space` | Print ASCII space (32) | Modifies cell 0, resets to 0 |
| `print_zero` | Print ASCII '0' (48) | Modifies cell 0, resets to 0 |
| `print_tab` | Print ASCII tab (9) | Modifies cell 0, resets to 0 |
| `print_bang` | Print ASCII '!' (33) | Modifies cell 0, resets to 0 |
| `print_dash` | Print ASCII '-' (45) | Modifies cell 0, resets to 0 |
| `print_colon` | Print ASCII ':' (58) | Modifies cell 0, resets to 0 |
| `read_char` | Read one character from stdin | Modifies cell 0 |

**Pattern:** Most I/O functions add the ASCII value to cell 0, print it,
then clear with `[-]`. The data pointer remains at cell 0.

**Important:** These functions assume cell 0 starts at 0. If cell 0 has a
non-zero value, the output character will be offset.

## std/math — Arithmetic

| Function | Description | Cell usage |
|----------|-------------|------------|
| `zero` | Clear cell 0 to zero | Modifies cell 0 |
| `inc5` | Add 5 to cell 0 | Modifies cell 0 |
| `inc10` | Add 10 to cell 0 | Modifies cell 0 |
| `double` | Double the value of cell 0 | Uses cell 1 as scratch |
| `add_to_next` | Move cell 0's value to cell 1 | Zeros cell 0 |
| `move_right` | Move cell 0's value to cell 1 | Zeros cell 0 |
| `is_zero` | Set cell 1 to 1 if cell 0 is 0 | Uses cell 1 |

**Pattern:** Arithmetic functions that use loops (`double`, `add_to_next`)
consume (zero) the source cell. Use `copy_right` from std/memory first if
you need to preserve the value.

## std/memory — Memory/Tape Utilities

| Function | Description | Cell usage |
|----------|-------------|------------|
| `swap` | Swap values of cell 0 and cell 1 | Uses cell 2 as temp |
| `copy_right` | Copy cell 0 to cell 1 (non-destructive) | Uses cell 2 as temp |
| `clear_right` | Zero cell 1 | Modifies cell 1 |
| `zero_range_3` | Zero cells 0, 1, and 2 | Modifies cells 0-2 |

**Pattern:** Functions that need scratch space use the cell(s) immediately
to the right. The `swap` function requires cell 2 to be zero initially.

## std/ascii — ASCII Utilities

| Function | Description | Cell usage |
|----------|-------------|------------|
| `to_upper` | Convert lowercase letter to uppercase | Subtracts 32 from cell 0 |
| `to_lower` | Convert uppercase letter to lowercase | Adds 32 to cell 0 |

**Warning:** These functions blindly add/subtract 32. They only work correctly
for ASCII letters (a-z / A-Z). Applying `to_upper` to a non-lowercase
character will produce garbage.

Additional ASCII functions for printing specific characters:
- `print_A`, `print_B`, `print_colon`, `print_dash`, `print_dot`,
  `print_excl`, `print_space`

## std/debug — Debugging Helpers

| Function | Description | Cell usage |
|----------|-------------|------------|
| `mark` | Print '#' character as a debug marker | Modifies cell 0, resets to 0 |

## Usage Example

```brainfuck
@import "std/io.bf"
@import "std/math.bf"
@import "std/memory.bf"

Set cell 0 to 65 (ASCII 'A')
@const A_VALUE 65
@use A_VALUE

Copy to cell 1 before printing
@call copy_right

Print 'A'
.

Move to cell 1 and convert to lowercase
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
4. Add tests to verify the function's behavior
