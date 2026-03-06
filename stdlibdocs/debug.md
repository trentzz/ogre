# debug -- Debugging and Diagnostic Output

```brainfuck
@import "std/debug.bf"
```

The `debug` module provides functions for inspecting cell values during program execution. It can print cells as decimal numbers, hexadecimal, or raw byte values, and includes markers and separators for structuring debug output.

## Function Reference

### Raw Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `dump_cell` | Print the raw byte value of c0 (single `.` instruction). Does not clear c0. | c0 (read-only) | c0 |
| `dump_and_newline` | Print the raw byte value of c0, then print a newline. c0 cleared. | c0 | c0 |

### Formatted Decimal Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `dump_decimal` | Print c0 as a decimal number (0--255) followed by a newline. Handles 1--3 digit numbers with leading zero suppression. c0 cleared. | c0, c1--c6 as scratch | c0 |
| `dump_range_5` | Print 5 cells (c0--c4) as decimal values separated by spaces, followed by newline. All 5 cells cleared. | c0--c4 (consumed), scratch cells after c4 | c0 |

### Formatted Hex Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `dump_hex` | Print c0 as a 2-digit hex value (e.g., `4F`). c0 cleared. | c0, c1--c9 as scratch | c0 |

### Markers and Separators

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `marker_start` | Print `<` character (debug start marker). Uses c1 as scratch; c0 unchanged. | c1 as scratch | c0 |
| `marker_end` | Print `>` character (debug end marker). Uses c1 as scratch; c0 unchanged. | c1 as scratch | c0 |
| `separator` | Print `---` followed by newline as a visual separator. c0 cleared. | c0 | c0 |

## Usage Example

```brainfuck
@import "std/debug.bf"

Store a value and inspect it:
+++++++++++++    Set c0 = 13
@call dump_decimal
Prints: 13

Print a hex value:
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
c0 = 79 (letter 'O')
@call dump_hex
Prints: 4F

Use markers around a value:
++++++++++
@call marker_start
.
@call marker_end
Prints: <(raw byte)>

Print a separator line:
@call separator
Prints: ---
```
