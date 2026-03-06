# convert -- Number Format Conversion and Display

```brainfuck
@import "std/convert.bf"
```

The `convert` module provides functions for converting cell values between different representations (decimal, hexadecimal, binary) and displaying them as formatted text. It also includes single-character ASCII/numeric conversion helpers.

## Function Reference

### Decimal Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `print_decimal` | Print c0 as a decimal number (0--255) with leading zero suppression. c0 cleared. | c0, c1--c6 as scratch | c0 |
| `print_decimal_padded` | Print c0 as a 3-digit zero-padded decimal (000--255). c0 cleared. | c0, c1--c6 as scratch | c0 |

### Hexadecimal Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `print_hex_digit` | Print c0 (value 0--15) as a single hex character (0--9, A--F). c0 cleared. | c0, c1--c5 as scratch | c0 |

### Binary Output

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `print_binary_8` | Print c0 as an 8-bit binary string (e.g., `01010011`). c0 cleared. | c0, c1--c13 as scratch | c0 |

### Single-Character Conversion

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `atoi_single` | Convert ASCII digit in c0 to numeric value (subtract 48). c0 modified in place. | c0 | c0 |
| `itoa_single` | Convert numeric value 0--9 in c0 to ASCII digit (add 48). c0 modified in place. | c0 | c0 |

## Usage Example

```brainfuck
@import "std/convert.bf"
@import "std/io.bf"

Print a number in decimal:
++++++++++++++++++++++++++++++++++++++++++  c0 = 42
@call print_decimal
Prints: 42

Print with zero-padding:
+++++++    c0 = 7
@call print_decimal_padded
Prints: 007

Print a value in binary:
+++++++++++++++++++++++++++++++++++++++++++++++++++++++  c0 = 53
@call print_binary_8
Prints: 00110101

Convert a digit character to its numeric value:
+++++++++++++++++++++++++++++++++++++++++++++++++++  c0 = 51 (ASCII '3')
@call atoi_single
c0 is now 3

Convert a numeric value back to ASCII:
@call itoa_single
c0 is now 51 (ASCII '3')
```
