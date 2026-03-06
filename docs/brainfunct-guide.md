# The Brainfunct Language Guide

Brainfunct is an extension of the brainfuck esoteric programming language that
adds named functions, file imports, named constants, and documentation comments.
It is designed to make brainfuck programs modular, reusable, and maintainable
while remaining fully compatible with standard brainfuck. All brainfunct
features are handled by a compile-time preprocessor; the final output is always
pure brainfuck.

This guide covers standard brainfuck, every brainfunct extension, the standard
library, the preprocessor architecture, and practical patterns for writing
brainfunct code.

---

## 1. Standard Brainfuck

Brainfuck operates on an array of 30,000 memory cells (the "tape"), each
initialized to zero. Each cell holds an unsigned 8-bit value (0--255) that wraps
on overflow and underflow. A data pointer starts at cell 0. The language has
exactly eight instructions:

| Operator | Description                                               |
|----------|-----------------------------------------------------------|
| `+`      | Increment the cell at the data pointer (wraps 255 -> 0)   |
| `-`      | Decrement the cell at the data pointer (wraps 0 -> 255)   |
| `>`      | Move the data pointer one cell to the right               |
| `<`      | Move the data pointer one cell to the left                |
| `[`      | If the current cell is zero, jump past the matching `]`   |
| `]`      | If the current cell is nonzero, jump back to matching `[` |
| `.`      | Output the current cell as an ASCII character             |
| `,`      | Read one byte of input into the current cell              |

All other characters are ignored and serve as comments. Brackets must be
balanced; unmatched `[` or `]` is an error.

### Hello World in Standard Brainfuck

```brainfuck
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.
+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

This prints `Hello World!\n`. Each character is produced by arithmetic on cells,
using nested loops as multipliers.

---

## 2. Brainfunct Extensions

Standard brainfuck is deliberately minimal, which makes nontrivial programs hard
to read and impossible to share as libraries. Brainfunct addresses this with
four compile-time directives:

| Directive             | Purpose                                      |
|-----------------------|----------------------------------------------|
| `@fn name { body }`  | Define a named function (macro)              |
| `@call name`         | Inline a function's body at the call site    |
| `@import "path"`     | Import function definitions from another file|
| `@const NAME value`  | Define a named numeric constant              |
| `@use NAME`          | Expand a constant to N `+` characters        |
| `@doc text`          | Attach documentation to the next `@fn`       |

Every directive begins with `@`. The `@` character always introduces a
directive; it is never treated as a comment. After preprocessing, all directives
are stripped and the output is pure brainfuck suitable for any standard
interpreter or compiler.

---

## 3. @fn / @call -- Defining and Calling Functions

### Syntax

Define a function with `@fn`, then invoke it with `@call`:

```brainfunct
@fn print_newline {
    ++++++++++.[-]
}

@fn greet {
    ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.
    [-]
    @call print_newline
}

@call greet
```

Rules:
- `@fn name { body }` defines a named function. The braces `{` and `}` are
  delimiters, not brainfuck operators.
- `name` must be an identifier (letters, digits, underscores).
- `@call name` inlines the full body of `name` at the call site. This is a
  compile-time macro expansion, not a runtime call. There is no call stack and
  no return address.
- A function must be defined before it is called, or imported from another file.
- Function definitions do not produce any output on their own. Only code at the
  top level (or inlined via `@call`) is executed.

### Functions Are Compile-Time Macros

When the preprocessor encounters `@call greet`, it replaces that marker with the
literal body of `greet`. If `greet` itself contains `@call print_newline`, that
inner call is also expanded. The final result is a single flat string of
brainfuck operators.

Given:

```brainfunct
@fn inc3 { +++ }
@fn inc6 { @call inc3 @call inc3 }

@call inc6 .
```

After preprocessing, the expanded output is:

```brainfuck
 +++  +++  .
```

(Whitespace from function bodies is preserved but harmless -- brainfuck ignores
non-operator characters.)

### Nested Calls

Functions can call other functions to any depth:

```brainfunct
@fn a { + }
@fn b { @call a @call a }
@fn c { @call b @call b }

@call c
```

This expands to four `+` operators. Each `@call` is recursively expanded until
only brainfuck operators remain.

### Cycle Detection

The preprocessor detects and rejects recursive call cycles. Both direct and
indirect recursion are errors:

```brainfunct
Direct self-recursion:
@fn loop_forever { @call loop_forever }
@call loop_forever
Error: cycle detected: loop_forever -> loop_forever

Indirect mutual recursion:
@fn ping { @call pong }
@fn pong { @call ping }
@call ping
Error: cycle detected: ping -> pong -> ping
```

The cycle detector maintains a call stack during expansion. If a function name
appears twice on the stack, the preprocessor reports the full chain and aborts.

---

## 4. @import -- Importing from Files and the Standard Library

### File Imports

Use `@import` to pull function definitions from another `.bf` file:

```brainfunct
@import "lib/helpers.bf"

@call some_helper
```

Import rules:
- The path is resolved relative to the directory of the importing file.
- Only `@fn` definitions from the imported file are collected. Top-level
  brainfuck code in the imported file is discarded (with a warning).
- Imported files may themselves contain `@import` directives, which are resolved
  recursively.
- Importing the same file twice from different locations is detected and
  raises an import cycle error.

### Standard Library Imports

Ogre ships with a built-in standard library. Import stdlib modules with the
`std/` prefix:

```brainfunct
@import "std/io"
@import "std/math"
@import "std/memory"
@import "std/ascii"
@import "std/debug"
```

The `.bf` extension is optional for stdlib imports -- both `"std/io"` and
`"std/io.bf"` work. Stdlib modules are embedded in the ogre binary, so no
external files are required.

Duplicate stdlib imports are silently ignored (they are idempotent).

### Import Semantics

To reinforce the key point: importing a file makes its `@fn` definitions
available for `@call`, but does **not** execute any top-level code from that
file. This is deliberate -- imports are for reusable functions, not for side
effects.

```
lib/utils.bf:
    @fn helper { +++ }
    +++.    <-- this top-level code is discarded on import

main.bf:
    @import "lib/utils.bf"
    @call helper             <-- expands to +++
```

### Import Cycle Detection

The preprocessor tracks all imported file paths (canonicalized) in a set. If a
file is encountered a second time during recursive import resolution, the
preprocessor raises an import cycle error:

```
a.bf:  @import "b.bf"
b.bf:  @import "a.bf"    <-- Error: import cycle detected for a.bf
```

---

## 5. @const / @use -- Named Numeric Constants

### Defining Constants

`@const` binds a name to a numeric value:

```brainfunct
@const NEWLINE 10
@const SPACE 32
@const LETTER_A 65
```

The value must be a non-negative integer.

### Using Constants

`@use` expands a constant name to that many `+` operators:

```brainfunct
@const LETTER_H 72

@use LETTER_H .[-]
```

After preprocessing, this becomes:

```brainfuck
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ .[-]
```

That is, 72 `+` characters followed by `.[-]`, which prints `H` and clears the
cell.

### Use Cases

Constants are useful for:

- **ASCII values**: Give readable names to character codes instead of counting
  `+` operators by hand.
- **Cell initialization**: Set cells to known values with a meaningful label.
- **Configuration**: Define a value once and reference it in multiple functions.

```brainfunct
@const WIDTH 80
@const HEIGHT 24

@fn set_width { @use WIDTH }
@fn set_height { @use HEIGHT }
```

### Zero-Valued Constants

A constant with value 0 expands to zero `+` characters (i.e., nothing):

```brainfunct
@const ZERO 0
@use ZERO     produces no output
```

### Undefined Constants

Using `@use` with an undefined name is an error:

```brainfunct
@use UNDEFINED
Error: undefined constant: @use UNDEFINED
```

---

## 6. @doc -- Documentation Comments

### Syntax

Place `@doc` lines immediately before an `@fn` definition:

```brainfunct
@doc Prints a newline character (ASCII 10) and clears the cell.
@fn print_newline {
    ++++++++++.[-]
}
```

Multiple `@doc` lines are concatenated:

```brainfunct
@doc Doubles the value of the current cell.
@doc Uses cell+1 as scratch space. Pointer returns to original position.
@fn double {
    [>++<-]>[<+>-]<
}
```

### How Documentation Is Used

The `ogre doc` command reads `@doc` annotations and generates formatted
documentation for all functions in a file. Example output:

```
## @fn double

Doubles the value of the current cell.
Uses cell+1 as scratch space. Pointer returns to original position.

    [>++<-]>[<+>-]<
```

Run `ogre doc --stdlib` to view documentation for all standard library modules.

### Orphaned @doc Lines

A `@doc` line that is not followed by an `@fn` is silently discarded. It does
not appear in the preprocessor output and does not cause an error.

---

## 7. Standard Library Modules

Ogre includes five standard library modules. Each provides a set of `@fn`
definitions for common operations.

### std/io -- Input and Output

```brainfunct
@import "std/io"
```

| Function         | Description                                     |
|------------------|-------------------------------------------------|
| `print_newline`  | Print ASCII 10 (newline), then clear the cell   |
| `print_space`    | Print ASCII 32 (space), then clear the cell     |
| `read_char`      | Read one byte of input into the current cell    |
| `print_char`     | Output the current cell as a character           |
| `print_zero`     | Print ASCII 48 (the character `0`), clear cell  |

Example:

```brainfunct
@import "std/io"

++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.
@call print_newline
```

### std/math -- Arithmetic

```brainfunct
@import "std/math"
```

| Function        | Description                                          |
|-----------------|------------------------------------------------------|
| `zero`          | Clear the current cell: `[-]`                        |
| `inc`           | Increment: `+`                                       |
| `dec`           | Decrement: `-`                                       |
| `inc10`         | Add 10: `++++++++++`                                 |
| `double`        | Double cell value using cell+1 as scratch            |
| `add_to_next`   | Move current cell value into cell+1 (destructive)    |
| `move_right`    | Move value one cell right, advance pointer           |
| `move_left`     | Move value one cell left, retreat pointer             |
| `copy_right`    | Copy value to cell+1, using cell+2 as scratch        |

Example -- double a value and print it:

```brainfunct
@import "std/math"
@import "std/io"

+++++                   set cell 0 to 5
@call double            cell 0 is now 10
@call print_char        prints ASCII 10 (newline)
```

### std/memory -- Cell and Pointer Management

```brainfunct
@import "std/memory"
```

| Function      | Description                                           |
|---------------|-------------------------------------------------------|
| `clear`       | Clear the current cell                                |
| `clear2`      | Clear the current cell and the next cell              |
| `clear3`      | Clear three consecutive cells starting at pointer     |
| `swap`        | Swap the values of the current cell and the next cell |
| `push_right`  | Move value one cell to the right                      |
| `pull_left`   | Pull value from one cell to the left                  |

### std/ascii -- Character Output

```brainfunct
@import "std/ascii"
```

| Function        | Description                                |
|-----------------|--------------------------------------------|
| `print_A`       | Print the letter `A` (ASCII 65), clear     |
| `print_B`       | Print the letter `B` (ASCII 66), clear     |
| `print_exclaim` | Print `!` (ASCII 33), clear                |
| `print_dash`    | Print `-` (ASCII 45), clear                |
| `print_colon`   | Print `:` (ASCII 58), clear                |

Each function sets the current cell to the target ASCII value, outputs it, then
clears the cell back to zero.

### std/debug -- Debugging Helpers

```brainfunct
@import "std/debug"
```

| Function           | Description                                       |
|--------------------|---------------------------------------------------|
| `dump_cell`        | Output the current cell value as a raw byte       |
| `dump_and_newline` | Output the cell, then print a newline             |
| `marker_start`     | Print `<` on cell+1 as a visual start marker     |
| `marker_end`       | Print `=` on cell+1 as a visual end marker        |

These are useful for inspecting tape state during development. The marker
functions operate on the cell to the right of the pointer, preserving the
current cell.

---

## 8. Preprocessor Architecture

The brainfunct preprocessor uses a two-pass design.

### Pass 1: Collect

The collect pass walks the source character by character:

1. **`@import`**: Reads the target file (or loads the stdlib module), then
   recursively runs the collect pass on the imported source. Only `@fn`
   definitions are retained; top-level code from imports is discarded. Import
   cycle detection uses a `HashSet<PathBuf>` of canonical file paths.

2. **`@fn name { body }`**: Stores the function name and body in a
   `HashMap<String, String>`. If a `@doc` annotation precedes the `@fn`, the
   documentation text is stored in a separate map.

3. **`@const NAME value`**: Stores the constant in a `HashMap<String, usize>`.

4. **`@use NAME`**: Looks up the constant value and emits that many `+`
   characters into the top-level output.

5. **`@call name`**: Preserved as a literal `@call name` marker in the
   top-level output string for the expand pass to handle.

6. **Everything else**: Passed through to the top-level output verbatim.

After the collect pass, all `@fn` bodies are available in the function map, and
the top-level code contains raw brainfuck interleaved with `@call` markers.

### Pass 2: Expand

The expand pass walks the top-level output from pass 1:

1. **`@call name`**: Looks up the function body, pushes `name` onto a call stack
   (a `Vec<String>`), recursively expands the body, pops the stack, and appends
   the result. If `name` is already on the stack, a cycle error is raised.

2. **`@use NAME`**: Expands to `+` characters (constants defined inside function
   bodies are handled here).

3. **Everything else**: Passed through to the final output.

The result of pass 2 is pure brainfuck -- no `@` directives remain.

### Diagram

```
Source file
    |
    v
[Pass 1: Collect]
    |-- @import -> recursive collect (discard top-level)
    |-- @fn     -> store in function map
    |-- @const  -> store in constant map
    |-- @use    -> expand to +++ inline
    |-- @call   -> preserve marker
    |-- other   -> pass through
    |
    v
Top-level code (with @call markers) + function map + constant map
    |
    v
[Pass 2: Expand]
    |-- @call   -> look up body, push call stack, recurse, pop
    |-- @use    -> expand to +++ inline
    |-- other   -> pass through
    |
    v
Pure brainfuck output
```

---

## 9. Common Brainfuck Patterns

These patterns appear frequently in brainfuck and brainfunct code. Understanding
them is essential for writing and reading brainfuck.

### Clear a Cell

```brainfuck
[-]
```

Decrements the current cell until it reaches zero. This is the standard idiom
for zeroing a cell. The ogre optimizer recognizes `[-]` and compiles it to a
single `Clear` operation.

### Move a Value

Move cell 0 into cell 1 (destructive -- cell 0 becomes zero):

```brainfuck
[->+<]
```

This decrements cell 0 and increments cell 1 in a loop until cell 0 is zero.
The ogre optimizer recognizes this pattern and compiles it to a single
`MoveAdd` operation.

### Copy a Value

Copy cell 0 into cell 1, using cell 2 as scratch (nondestructive):

```brainfuck
[->+>+<<]>>[<<+>>-]<<
```

Phase 1: Move cell 0 to both cell 1 and cell 2.
Phase 2: Move cell 2 back into cell 0, restoring the original value.

### Multiply

Set cell 1 to (cell 0 * 3):

```brainfuck
[->+++<]
```

Each iteration decrements cell 0 by 1 and increments cell 1 by 3. When cell 0
reaches zero, cell 1 holds the product. This generalizes: replace `+++` with any
number of `+` operators to multiply by that constant.

### Conditional (If Nonzero)

Execute code only if the current cell is nonzero (destructive):

```brainfuck
[code[-]]
```

If the cell is zero the loop is skipped. If nonzero, `code` runs, then `[-]`
clears the cell to exit the loop.

### Simple Loop (Repeat N Times)

```brainfuck
+++++ [code -]
```

Sets the counter to 5, runs `code` five times, decrementing the counter each
iteration. Be careful: `code` must not change the cell the counter lives in.

### Addition

Add cell 0 to cell 1:

```brainfuck
[->+<]
```

This is the same as "move." After execution, cell 0 is zero and cell 1 holds
the sum of both original values.

### Print a Specific Character

To print a character, set a cell to its ASCII value and use `.`:

```brainfuck
Set cell to 72 (H) using multiplication: 8 * 9 = 72
++++++++[>+++++++++<-]>.[-]
```

Or, using brainfunct constants:

```brainfunct
@const H 72
@use H .[-]
```

---

## 10. Tips and Tricks

### Use Functions for Readability

Even simple operations benefit from named functions. Compare:

```brainfuck
[-]++++++++++.[-]
```

versus:

```brainfunct
@import "std/math"
@import "std/io"

@call zero
@call inc10
@call print_char
@call zero
```

The brainfunct version communicates intent. The preprocessor generates identical
brainfuck.

### Document Your Functions

Always add `@doc` annotations to nontrivial functions. Future readers (including
yourself) will need to know:
- What cell(s) the function reads
- What cell(s) the function writes
- Where the pointer ends up after the call
- Whether any scratch cells are used

```brainfunct
@doc Copy current cell value to cell+1. Uses cell+2 as scratch.
@doc Pointer returns to original position. Cell+2 is zeroed.
@fn copy_right {
    [->+>+<<]>>[<<+>>-]<<
}
```

### Mind the Pointer Position

The most common source of bugs in brainfuck is losing track of where the data
pointer is. Every `@fn` should leave the pointer in a documented position --
typically the same cell where it started. If a function moves the pointer, say
so in the `@doc`.

### Use @const for ASCII Values

Counting `+` operators by hand is error-prone. Use `@const` for any value you
need to set explicitly:

```brainfunct
@const NEWLINE 10
@const SPACE 32
@const ZERO_CHAR 48
@const A 65
@const a 97
```

### Organize Code with Imports

For anything beyond a trivial script, split your code into files:

```
project/
  ogre.toml
  src/
    main.bf        entry point with top-level code
    lib/
      output.bf    @fn definitions for output
      math.bf      @fn definitions for arithmetic
  tests/
    basic.json
```

In `main.bf`:

```brainfunct
@import "lib/output.bf"
@import "lib/math.bf"

@call setup
@call print_greeting
@call cleanup
```

### Use the Standard Library

Before writing a utility function from scratch, check whether the standard
library already provides it. Import only the modules you need:

```brainfunct
@import "std/math"
@import "std/io"
```

### Test with ogre test

Write JSON test cases to verify your functions:

```json
[
  {
    "name": "prints H",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "H"
  }
]
```

Run with `ogre test tests/basic.json`. Tests preprocess the source file through
the full brainfunct pipeline before executing.

### Use ogre check for Validation

Run `ogre check` on your file or project to verify that:
- All `@call` references resolve to defined functions
- No call cycles exist
- All brackets are matched

This catches errors before you attempt to run or compile.

### Debugging with ogre debug

The interactive debugger loads preprocessed brainfuck and lets you step through
execution one instruction at a time, set breakpoints, and inspect tape state.
This is invaluable for tracking down pointer position bugs.

### Compilation

For performance, compile brainfunct to a native binary:

```
ogre compile src/main.bf -o myprogram
```

The compiler translates brainfuck to optimized C, which is then compiled by gcc
or clang. The ogre optimizer collapses consecutive operations (e.g., `+++`
becomes `*ptr += 3`), recognizes `[-]` as a clear operation, and detects move
idioms like `[->+<]`.

---

## Quick Reference

```
Directive                 Expansion
---------                 ---------
@fn name { body }         Defines function "name" with given body
@call name                Replaced with body of "name" (recursive)
@import "path.bf"         Imports @fn definitions from file
@import "std/module"      Imports from built-in standard library
@const NAME 42            Defines constant NAME = 42
@use NAME                 Expands to 42 '+' characters
@doc text                 Attaches documentation to next @fn

Standard Library Modules
------------------------
std/io       print_newline, print_space, read_char, print_char, print_zero
std/math     zero, inc, dec, inc10, double, add_to_next, move_right,
             move_left, copy_right
std/memory   clear, clear2, clear3, swap, push_right, pull_left
std/ascii    print_A, print_B, print_exclaim, print_dash, print_colon
std/debug    dump_cell, dump_and_newline, marker_start, marker_end
```
