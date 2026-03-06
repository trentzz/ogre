# ogre Standard Library Reference

The ogre standard library provides reusable brainfuck functions via the `@import` directive. Import any module with:

```brainfuck
@import "std/<module>.bf"
```

Then call its functions with `@call <function_name>`. All functions are inlined at compile time by the preprocessor.

## Modules

| Module | Import | Description |
|---|---|---|
| [io](io.md) | `@import "std/io.bf"` | Input/output primitives: print individual characters (newline, space, punctuation, symbols), read input, flush input buffers |
| [math](math.md) | `@import "std/math.bf"` | Arithmetic operations: increment, decrement, multiply, divide/modulo, min/max, absolute difference, square, clamp, negation |
| [memory](memory.md) | `@import "std/memory.bf"` | Cell manipulation: clear ranges, move/copy/swap/duplicate values, rotate and shift data on the tape |
| [string](string.md) | `@import "std/string.bf"` | String and text processing: read lines/words/decimal numbers, skip whitespace, print null-terminated strings, compare characters |
| [logic](logic.md) | `@import "std/logic.bf"` | Boolean logic and comparison: NOT, AND, OR, XOR, NAND, equality, greater/less than, conditional selection |
| [ascii](ascii.md) | `@import "std/ascii.bf"` | ASCII utilities: case conversion, character classification (digit, letter, space, printable), digit-to-char conversion |
| [debug](debug.md) | `@import "std/debug.bf"` | Debugging output: print cell values as decimal, hex, or raw bytes; markers and separators for structured debug output |
| [cli](cli.md) | `@import "std/cli.bf"` | Command-line argument handling: parse flags and arguments passed via `ogre run program.bf -- args`, print error/usage prefixes |
| [convert](convert.md) | `@import "std/convert.bf"` | Number format conversion: print values as decimal (with or without zero-padding), hex digits, 8-bit binary strings; ASCII/numeric single-char conversion |

## Conventions

- **Cell 0** is the primary working cell for most functions. Function descriptions refer to cells relative to the pointer position at call time (c0 = current cell, c1 = one right, etc.).
- **Scratch cells** are temporary cells used internally by a function. They are zeroed after the function completes unless otherwise noted.
- **Pointer position** after each call is documented per function. Most functions return the pointer to cell 0 (the position it was at when called).
- **Preconditions**: `print_*` character functions in `io` and `ascii` require cell 0 to be 0 before calling. Other preconditions are noted per function.
- **Destructive vs. non-destructive**: Functions that consume their input cells are marked as such. Classification functions in `ascii` (e.g., `is_digit`, `is_upper`) are non-destructive on the tested cell.

## Quick Start

```brainfuck
@import "std/io.bf"
@import "std/math.bf"
@import "std/convert.bf"

Set cell 0 to 6, square it:
++++++
@call square

Print the result (36) as decimal:
@call print_decimal

Print a newline:
@call print_newline
```
