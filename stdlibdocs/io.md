# io -- Input/Output Primitives

```brainfuck
@import "std/io.bf"
```

The `io` module provides functions for printing individual characters and symbols, reading input, and flushing input buffers. Each `print_*` function sets the current cell to the appropriate ASCII value, outputs it, then clears the cell back to zero.

## Function Reference

### Output Functions

All `print_*` functions operate on cell 0. They add the target ASCII value to cell 0, print it with `.`, then clear cell 0 with `[-]`. **Precondition:** cell 0 must be 0 before calling. **Postcondition:** cell 0 is 0; pointer at cell 0.

| Function | Character | ASCII |
|---|---|---|
| `print_newline` | `\n` (newline) | 10 |
| `print_tab` | `\t` (tab) | 9 |
| `print_space` | ` ` (space) | 32 |
| `print_bang` | `!` | 33 |
| `print_quote` | `"` | 34 |
| `print_hash` | `#` | 35 |
| `print_percent` | `%` | 37 |
| `print_ampersand` | `&` | 38 |
| `print_lparen` | `(` | 40 |
| `print_rparen` | `)` | 41 |
| `print_star` | `*` | 42 |
| `print_plus` | `+` | 43 |
| `print_comma` | `,` | 44 |
| `print_minus` | `-` | 45 |
| `print_dot` | `.` | 46 |
| `print_slash` | `/` | 47 |
| `print_zero` | `0` | 48 |
| `print_semicolon` | `;` | 59 |
| `print_lt` | `<` | 60 |
| `print_eq` | `=` | 61 |
| `print_gt` | `>` | 62 |
| `print_question` | `?` | 63 |
| `print_at` | `@` | 64 |
| `print_lbracket` | `[` | 91 |
| `print_backslash` | `\` | 92 |
| `print_rbracket` | `]` | 93 |
| `print_caret` | `^` | 94 |
| `print_underscore` | `_` | 95 |
| `print_lbrace` | `{` | 123 |
| `print_pipe` | `\|` | 124 |
| `print_rbrace` | `}` | 125 |
| `print_tilde` | `~` | 126 |

### Input Functions

| Function | Description | Cells | Pointer After |
|---|---|---|---|
| `read_char` | Read one character from stdin into cell 0 | Reads into c0 | c0 |
| `flush_input` | Read and discard all remaining input until EOF (cell reads as 0) | c0 | c0 (value 0) |

### Character Output

| Function | Description | Cells | Pointer After |
|---|---|---|---|
| `print_char` | Print whatever value is currently in cell 0 (does not clear) | Reads c0 | c0 |

## Usage Example

```brainfuck
@import "std/io.bf"

Print a greeting followed by a newline:
+++++++++[>++++++++<-]>+.     Set up and print 'H'
[-]                            Clear
@call print_space
@call print_newline

Read a character and echo it back:
@call read_char
@call print_char
@call print_newline
```
