# string -- String and Text Processing

```brainfuck
@import "std/string.bf"
```

The `string` module provides functions for reading, printing, and manipulating strings of characters from input. It includes line and word reading, decimal number parsing, character skipping, and string comparison.

## Function Reference

### Reading Input

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `read_line` | Read characters until newline (10) or EOF (0), storing in successive cells starting at c0. Newline is not stored. | c0 onward (one cell per char) | Zero-terminator cell (one past last char) |
| `read_word` | Read characters until space (32) or EOF (0), storing in successive cells starting at c0. Space is not stored. | c0 onward (one cell per char) | Zero-terminator cell |
| `read_decimal` | Read decimal digits from input, accumulating into c0. Stops at non-digit (consumed and discarded). For "123", c0 = 123. | c0, c1--c3 as scratch | c0 |
| `read_char` (via io) | Read one character into c0 | c0 | c0 |

### Skipping Input

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `skip_char` | Read and discard one character. c0 cleared. | c0 | c0 |
| `skip_spaces` | Read and discard whitespace (spaces) until a non-space char or EOF. The non-space char remains in c0. | c0, c1--c2 as scratch | c0 |
| `skip_line` | Read and discard characters until newline (10) or EOF (0). Newline consumed. c0 is 0 after. | c0, c1 as scratch | c0 |

### Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `print_string` | Print cells starting from c0 until a zero cell is hit. Advances right through the string. | c0 onward (read-only) | Zero-terminator cell |

### Comparison

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `compare_char` | Read one char from input. Set c1 to 1 if it matches c0, else 0. c0 preserved. | c0 (preserved), c1 (result), c2--c3 as scratch | c0 |

## Usage Example

```brainfuck
@import "std/string.bf"

Read a line of text from stdin, then print it back:
@call read_line

Go back to the start of the string (assuming we know the length,
or we stored the start position):
[<]

Print the stored string:
@call print_string

Read a decimal number from input:
@call read_decimal
Cell 0 now contains the numeric value
```
