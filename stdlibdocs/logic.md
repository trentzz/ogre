# logic -- Boolean and Comparison Operations

```brainfuck
@import "std/logic.bf"
```

The `logic` module provides boolean logic gates, comparison operators, and conditional selection. All boolean functions treat 0 as false and nonzero as true, producing 0 or 1 as output.

## Function Reference

### Boolean Gates

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `not` | Logical NOT of c0. Result: 1 if c0 was 0, else 0. | c0, c1 as scratch | c0 |
| `bool` | Boolean cast: set c0 to 1 if nonzero, 0 if zero. | c0, c1 as scratch | c0 |
| `and` | Logical AND of c0 and c1. Result in c0. c1 zeroed. | c0, c1 | c0 |
| `or` | Logical OR of c0 and c1. Result in c0. c1 zeroed. | c0, c1 | c0 |
| `xor` | Exclusive OR of c0 and c1. Result in c0. c1 zeroed. Assumes boolean inputs (0 or 1). | c0, c1 | c0 |
| `nand` | NAND of c0 and c1. Result in c0. c1 zeroed. Assumes boolean inputs (0 or 1). | c0, c1, c2 as scratch | c0 |

### Comparison

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `equal` | Set c0 to 1 if c0 == c1, else 0. Both cells consumed. | c0, c1, c2--c3 as scratch | c0 |
| `greater_than` | Set c0 to 1 if c0 > c1, else 0. Both cells consumed. Correct for unsigned 8-bit values. | c0, c1, c2--c5 as scratch | c0 |
| `less_than` | Set c0 to 1 if c0 < c1, else 0. Both cells consumed. Correct for unsigned 8-bit values. | c0, c1, c2--c5 as scratch | c0 |

### Conditional

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `if_nonzero` | If c0 is nonzero, result is c1; else result is 0. Result placed in c0. Both c0 and c1 consumed. | c0, c1, c2 as scratch | c0 |

## Usage Example

```brainfuck
@import "std/logic.bf"

Check if cell 0 equals cell 1:
+++++          Set c0 = 5
>+++++<        Set c1 = 5
@call equal
Cell 0 is now 1 (they are equal)

Logical NOT:
[-]+           Set c0 = 1
@call not
Cell 0 is now 0

Compare: is 10 greater than 7?
++++++++++     Set c0 = 10
>+++++++<      Set c1 = 7
@call greater_than
Cell 0 is now 1
```
