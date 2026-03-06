# Standard Library Reference

The ogre standard library is a collection of reusable brainfunct functions
embedded in the ogre binary. Import modules with `@import "std/module.bf"`.

Run `ogre stdlib list` to see available modules, or
`ogre stdlib show <module>` to view a module's source.

For detailed per-function documentation, see the [stdlibdocs/](../stdlibdocs/) folder.

## Modules

| Module | Import | Description |
|--------|--------|-------------|
| io | `@import "std/io.bf"` | I/O primitives: print characters, read input, flush stdin |
| math | `@import "std/math.bf"` | Arithmetic: inc/dec, multiply, divide, square, modulo, clamp, min/max |
| memory | `@import "std/memory.bf"` | Tape manipulation: clear, swap, copy, rotate, shift, fill |
| string | `@import "std/string.bf"` | Text I/O: read lines/words/decimals, skip whitespace, print strings, compare chars |
| logic | `@import "std/logic.bf"` | Boolean logic: not, and, or, xor, nand, equal, greater/less than |
| ascii | `@import "std/ascii.bf"` | ASCII utilities: case conversion, classification (digit/letter/space/printable) |
| debug | `@import "std/debug.bf"` | Debug output: dump cells as decimal/hex, markers, separators |
| cli | `@import "std/cli.bf"` | CLI toolkit: parse flags/arguments, print error/usage prefixes |
| convert | `@import "std/convert.bf"` | Format conversion: decimal/hex/binary output, atoi/itoa |

## Conventions

- Functions that print characters assume cell 0 starts at 0. They add the
  ASCII value, print with `.`, then clear with `[-]`.
- Functions document which cells they use as scratch. Scratch cells are
  assumed to be 0 on entry and are cleared on exit.
- The data pointer returns to cell 0 unless documented otherwise.

## Usage Example

```brainfuck
@import "std/io.bf"
@import "std/math.bf"
@import "std/convert.bf"

Set cell 0 to 42
@const ANSWER 42
@use ANSWER

Print as decimal
@call print_decimal

@call print_newline
```

## Testing

The standard library has a comprehensive test suite in `stdlibtests/`:

```sh
cd stdlibtests
ogre test
```

105 tests covering all 9 modules.
