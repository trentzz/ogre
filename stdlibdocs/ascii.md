# ascii -- ASCII Character Utilities

```brainfuck
@import "std/ascii.bf"
```

The `ascii` module provides functions for printing specific ASCII characters, case conversion, character classification (digit, letter, space, etc.), and digit/character conversion.

## Function Reference

### Character Printing

These functions print a specific character from a zeroed cell, then clear the cell.
**Precondition:** cell 0 must be 0. **Postcondition:** cell 0 is 0; pointer at cell 0.

| Function | Character | ASCII |
|---|---|---|
| `print_A` | `A` | 65 |
| `print_B` | `B` | 66 |
| `print_exclaim` | `!` | 33 |
| `print_dash` | `-` | 45 |
| `print_colon` | `:` | 58 |
| `print_digit` | Prints the value in c0 as a digit (adds 48 to value, prints, clears). Input should be 0--9. | 48--57 |

### Case Conversion

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `to_upper` | Subtract 32 from c0 (lowercase ASCII to uppercase). | c0 | c0 |
| `to_lower` | Add 32 to c0 (uppercase ASCII to lowercase). | c0 | c0 |

### Digit/Character Conversion

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `digit_to_char` | Add 48 to c0 (numeric value 0--9 to ASCII char '0'--'9'). | c0 | c0 |
| `char_to_digit` | Subtract 48 from c0 (ASCII char '0'--'9' to numeric value 0--9). | c0 | c0 |

### Character Classification

All classification functions are **non-destructive** on cell 0 (the character being tested). The boolean result (0 or 1) is placed in cell 1.

| Function | Description | Tests For | Cells Used | Pointer After |
|---|---|---|---|---|
| `is_digit` | Set c1 to 1 if c0 is a digit (ASCII 48--57) | `0`--`9` | c0 (preserved), c1--c5 as scratch | c0 |
| `is_upper` | Set c1 to 1 if c0 is uppercase (ASCII 65--90) | `A`--`Z` | c0 (preserved), c1--c5 as scratch | c0 |
| `is_lower` | Set c1 to 1 if c0 is lowercase (ASCII 97--122) | `a`--`z` | c0 (preserved), c1--c5 as scratch | c0 |
| `is_alpha` | Set c1 to 1 if c0 is a letter (A--Z or a--z) | `A`--`Z`, `a`--`z` | c0 (preserved), c1--c6 as scratch | c0 |
| `is_space` | Set c1 to 1 if c0 is space (ASCII 32) | ` ` | c0 (preserved), c1--c2 as scratch | c0 |
| `is_newline` | Set c1 to 1 if c0 is newline (ASCII 10) | `\n` | c0 (preserved), c1--c2 as scratch | c0 |
| `is_printable` | Set c1 to 1 if c0 is printable ASCII (32--126) | printable chars | c0 (preserved), c1--c5 as scratch | c0 |

## Usage Example

```brainfuck
@import "std/ascii.bf"

Read a character and convert to uppercase if lowercase:
,
@call is_lower
>[<                   If c1 is 1 (is lowercase)
    @call to_upper    Convert to uppercase
>-]<                  Clear flag

Print the result:
.[-]

Print a digit value:
+++++                 c0 = 5
@call print_digit     Prints '5', clears c0
```
