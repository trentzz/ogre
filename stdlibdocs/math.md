# math -- Arithmetic and Numeric Operations

```brainfuck
@import "std/math.bf"
```

The `math` module provides arithmetic primitives for brainfuck: increment, decrement, multiplication, division/modulo, comparison, and value manipulation. All operations work on unsigned 8-bit cell values (0--255 with wrapping).

## Function Reference

### Basic Arithmetic

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `zero` | Set cell 0 to 0 | c0 | c0 |
| `inc` | Increment cell 0 by 1 | c0 | c0 |
| `dec` | Decrement cell 0 by 1 | c0 | c0 |
| `inc5` | Increment cell 0 by 5 | c0 | c0 |
| `dec5` | Decrement cell 0 by 5 | c0 | c0 |
| `inc10` | Increment cell 0 by 10 | c0 | c0 |
| `dec10` | Decrement cell 0 by 10 | c0 | c0 |

### Multiplication

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `double` | Multiply cell 0 by 2 | c0, c1 as scratch | c0 |
| `triple` | Multiply cell 0 by 3 | c0, c1 as scratch | c0 |
| `multiply_by_10` | Multiply cell 0 by 10 | c0, c1 as scratch | c0 |
| `multiply` | Multiply c0 by c1, result in c0. c1 zeroed. | c0, c1, c2 as scratch | c0 |
| `square` | Square cell 0 (c0 = c0 * c0). | c0, c1--c4 as scratch | c0 |

### Division and Modulo

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `divmod_10` | Divide c0 by 10. Quotient in c0, remainder in c1. | c0, c1--c4 as scratch | c0 |
| `modulo` | Compute c0 mod c1, result in c0. c1 zeroed. | c0, c1, c2--c4 as scratch | c0 |

### Movement and Copying

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `add_to_next` | Move c0 value into c1 (c0 zeroed, c1 += old c0) | c0, c1 | c0 |
| `move_right` | Move c0 to c1 (c0 zeroed) | c0, c1 | c1 |
| `move_left` | Move c0 to c(-1) (c0 zeroed) | c0, c(-1) | c(-1) |
| `copy_right` | Copy c0 to c1 (c0 preserved) | c0, c1, c2 as scratch | c0 |

### Comparison and Tests

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `is_zero` | Set c0 to 1 if it was 0, else 0 | c0, c1, c2 as scratch | c0 |
| `is_nonzero` | Set c0 to 1 if nonzero, 0 if zero | c0, c1 | c0 |
| `is_positive` | Same as `is_nonzero` (boolean cast) | c0, c1 | c0 |
| `abs_diff` | Absolute difference of c0 and c1, result in c0 | c0, c1 | c0 |
| `min` | Minimum of c0 and c1, result in c0. Both consumed. | c0, c1, c2, c3 as scratch | c0 |
| `max` | Maximum of c0 and c1, result in c0. Both consumed. | c0, c1, c2, c3 as scratch | c0 |

### Advanced

| Function | Description | Cells Used | Pointer After |
|---|---|---|---|
| `negate` | Negate c0 (256 - c0, mod 256) | c0, c1, c2 as scratch | c0 |
| `clamp` | Clamp c0 between c1 (min) and c2 (max). Result in c0. c1, c2 zeroed. | c0, c1, c2, c3--c6 as scratch | c0 |

## Usage Example

```brainfuck
@import "std/math.bf"

Set cell 0 to 7, double it to get 14:
+++++++
@call double

Multiply cell 0 (14) by cell 1 (3):
>+++<
@call multiply
Result: cell 0 = 42

Compute 42 mod 10:
>++++++++++<
@call modulo
Result: cell 0 = 2
```
