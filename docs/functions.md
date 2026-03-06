# Functions in Brainfunct

Brainfunct extends standard brainfuck with named functions via three
compile-time directives: `@fn`, `@call`, and `@import`. These are
resolved entirely at preprocessing time — the interpreter and compiler
never see them. The final output is always pure brainfuck.

---

## Table of Contents

- [Defining Functions](#defining-functions)
- [Calling Functions](#calling-functions)
- [Importing Functions](#importing-functions)
- [How Expansion Works](#how-expansion-works)
- [The Standard Library](#the-standard-library)
- [Memory Conventions](#memory-conventions)
- [Writing Effective Functions](#writing-effective-functions)
- [Composing Functions](#composing-functions)
- [Error Handling](#error-handling)
- [Advanced Directives](#advanced-directives)
- [Patterns and Idioms](#patterns-and-idioms)

---

## Defining Functions

Use `@fn` to define a named function:

```brainfuck
@fn print_newline {
    ++++++++++.[-]
}
```

The name must be alphanumeric plus underscores. The body is enclosed in
`{ }` (which are not valid brainfuck characters, so there is no
ambiguity). The body can contain any brainfuck instructions and even
`@call` directives to other functions.

A function definition does not emit any code on its own. It only
registers the name so that `@call` can reference it later.

### Naming conventions

Use `snake_case` for function names. Choose names that describe what
the function does and, when relevant, hint at the memory layout:

```brainfuck
@fn clear        { [-] }
@fn move_right   { [>+<-]> }
@fn copy_right   { [>+>+<<-]>>[<<+>>-]<< }
@fn add_to_next  { [>+<-] }
```

### Documenting functions

Use `@doc` on the line(s) immediately before `@fn` to attach
documentation. The analyser (`ogre analyse --verbose`) will display
these docstrings.

```brainfuck
@doc Add 48 to cell 0 (numeric value 0-9 to ASCII digit character)
@fn digit_to_char {
    ++++++++++++++++++++++++++++++++++++++++++++++++
}

@doc Subtract 48 from cell 0 (ASCII digit character to numeric value)
@fn char_to_digit {
    ------------------------------------------------
}
```

Multiple `@doc` lines are joined together:

```brainfuck
@doc Divide cell 0 by 10.
@doc Quotient goes to cell 0, remainder to cell 1.
@doc Uses cells 2-4 as scratch.
@fn divmod_10 {
    ...
}
```

---

## Calling Functions

Use `@call` to inline a function's body at the call site:

```brainfuck
@fn greet {
    ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]
    @call print_newline
}

@call greet
```

`@call` is a compile-time text substitution. The preprocessor replaces
`@call greet` with the full expanded body of `greet`. There is no call
stack, no return address, no overhead — it is pure inlining.

This means:

1. **Pointer position matters.** The function body executes starting at
   whatever cell the pointer is on at the call site.
2. **Cell state matters.** The function sees and modifies the same tape
   cells as the caller.
3. **There is no isolation.** A function that moves the pointer and does
   not move it back will leave the pointer in a new position for the
   caller.

---

## Importing Functions

Use `@import` to bring function definitions in from another file:

```brainfuck
@import "lib/utils.bf"
@import "std/io.bf"

@call print_star
@call print_newline
```

### File imports

Paths are resolved relative to the directory of the importing file:

```
project/
  src/
    main.bf         ← @import "../lib/utils.bf"
  lib/
    utils.bf        ← defines @fn print_star
```

Only `@fn` definitions from the imported file are brought in. Top-level
brainfuck code in the imported file is silently discarded — it will not
execute.

### Standard library imports

Paths starting with `std/` resolve to ogre's built-in standard library:

```brainfuck
@import "std/io.bf"
@import "std/math.bf"
@import "std/memory.bf"
@import "std/ascii.bf"
@import "std/logic.bf"
@import "std/string.bf"
@import "std/debug.bf"
```

These are embedded in the ogre binary at compile time and are always
available without any external files.

### Import rules

- **Deduplication:** Importing the same file twice is silently allowed
  (the second import is a no-op). This means it is safe for two
  libraries to both import a shared dependency.
- **Cycle detection:** A imports B imports A is an error.
- **Transitive imports:** If `a.bf` imports `b.bf` and `b.bf` defines
  `@fn helper`, then `a.bf` can `@call helper`.

---

## How Expansion Works

The preprocessor uses a two-pass algorithm:

### Pass 1: Collect

Walk the source character by character. When an `@` directive is
encountered:

- `@import` — recursively process the target file, collecting its
  `@fn` definitions into a shared function table. Top-level code from
  imports is discarded.
- `@fn name { body }` — store the raw body string (which may itself
  contain `@call` markers) in the function table. Nothing is emitted.
- `@call name` — emit a marker into the top-level code stream.

After this pass, we have a function table (`name -> body`) and a stream
of top-level brainfuck with `@call` markers embedded in it.

### Pass 2: Expand

Walk the top-level stream. When a `@call name` marker is encountered:

1. Look up `name` in the function table.
2. Push `name` onto a call stack (for cycle detection).
3. Recursively expand the body (which may contain its own `@call`
   markers).
4. Pop `name` from the call stack.
5. Insert the fully expanded brainfuck in place of the marker.

The result is a pure brainfuck string with every directive resolved.

### Example

Source:
```brainfuck
@fn clear { [-] }
@fn reset_and_inc { @call clear + }

@call reset_and_inc
```

After pass 1:
- Function table: `clear -> "[-]"`, `reset_and_inc -> "@call clear +"`
- Top-level stream: `"@call reset_and_inc"`

After pass 2:
- Expand `reset_and_inc` → expand its body `"@call clear +"` → expand
  `clear` → `"[-]"` → body becomes `"[-]+"` → final output: `[-]+`

---

## The Standard Library

ogre ships with seven built-in modules. Import them with
`@import "std/<module>.bf"`. See `docs/stdlib-reference.md` for the
complete API reference.

| Module     | Purpose                                     |
|------------|---------------------------------------------|
| `io.bf`    | Print specific characters (newline, space, digits, punctuation) |
| `math.bf`  | Arithmetic: increment, double, copy, move, divide, min/max |
| `memory.bf`| Cell manipulation: clear, swap, push, pull, rotate |
| `ascii.bf` | Character classification and conversion (upper/lower, digit check) |
| `logic.bf` | Boolean operations: not, bool, and, or, equal |
| `string.bf`| Input parsing: skip characters, read decimal numbers |
| `debug.bf` | Debugging helpers: dump cell value, print markers |

### Quick example

```brainfuck
@import "std/io.bf"
@import "std/math.bf"

Set cell 0 to 5
+++++

Double it (cell 0 = 10)
@call double

Add 48 for ASCII digit display (cell 0 = 58 = ':')
++++++++++++++++++++++++++++++++++++++++++++++++
.

@call print_newline
```

---

## Memory Conventions

Because functions share the tape with their caller, it is critical to
document and follow memory layout conventions.

### The pointer contract

Every function should document:

1. **Entry pointer position:** Which cell is the pointer on?
2. **Exit pointer position:** Where does the pointer end up?
3. **Which cells are read, written, or used as scratch?**

Most stdlib functions follow this convention:

- **Operates on cell 0** (the cell the pointer is on at call time).
- **Returns the pointer to cell 0** after finishing.
- **Uses cells to the right as scratch** (cell 1, cell 2, etc.) and
  leaves them zeroed on exit.

For example, `@fn copy_right` from `std/math.bf`:
```brainfuck
@fn copy_right { [>+>+<<-]>>[<<+>>-]<< }
```

- Entry: pointer at cell 0, value = N.
- Exit: pointer at cell 0, cell 0 = N, cell 1 = N.
  Cell 2 used as scratch and left at 0.

### Scratch cell discipline

Always document which cells a function uses beyond the primary
operands. If your function uses cells 1 and 2 as scratch, the caller
must ensure those cells are zero before calling (or not care about
their contents).

A common pattern is to clear scratch cells as part of the function:

```brainfuck
@doc Swap cells 0 and 1. Uses cell 2 as scratch (must be 0 on entry).
@fn swap {
    [>>+<<-]     Move cell 0 to cell 2
    >[<+>-]      Move cell 1 to cell 0
    >[<+>-]      Move cell 2 to cell 1
    <<           Return pointer to cell 0
}
```

### Layout planning

For programs that use multiple values, plan your cell layout up front:

```
Cell 0: counter
Cell 1: accumulator
Cell 2: temp/scratch
Cell 3: temp/scratch
Cell 4-7: reserved for stdlib functions
```

When calling a stdlib function, move the pointer to the cell you want
to operate on first, call the function, then navigate back:

```brainfuck
@import "std/math.bf"

Set up: cell 0 = 3, cell 1 = 7
+++>+++++++

Double cell 1 (need pointer on cell 1)
> @call double <
Now cell 0 = 3, cell 1 = 14
```

---

## Writing Effective Functions

### Keep functions small and focused

Each function should do one thing. Small functions are easier to reason
about in terms of pointer position and cell usage:

```brainfuck
Good: single responsibility
@fn clear     { [-] }
@fn move_right { [>+<-]> }
@fn inc5      { +++++ }

Avoid: too many side effects in one function
@fn do_everything { [-]>+++++[>+<-]>... }
```

### Always return the pointer

If your function moves the pointer, move it back to a predictable
position. The most common convention is to return to the starting cell:

```brainfuck
@fn double {
    [>++<-]      Move cell 0 * 2 to cell 1
    >[<+>-]      Move cell 1 back to cell 0
    <            Return pointer to cell 0
}
```

If a function intentionally moves the pointer (like `move_right`),
document that clearly:

```brainfuck
@doc Move cell 0 to cell 1, then advance pointer to cell 1.
@fn move_right { [>+<-]> }
```

### Prefer destructive operations with clear semantics

In brainfuck, moving a value is simpler than copying it (copying
requires scratch space). Design your API so that the common case is
a destructive move, and provide separate copy variants when needed:

```brainfuck
@fn add_to_next  { [>+<-] }      Destructive: cell 0 is zeroed
@fn copy_right   { [>+>+<<-]>>[<<+>>-]<< }  Non-destructive: cell 0 preserved
```

### Test your functions

Write a test for each function using ogre's test runner. A test file
pairs a brainfuck source with expected output:

```json
[
  {
    "name": "double works",
    "brainfuck": "src/test_double.bf",
    "input": "",
    "output": "6"
  }
]
```

The test source file:
```brainfuck
@import "std/math.bf"
+++
@call double
++++++++++++++++++++++++++++++++++++++++++++++++.
```

Cell 0 = 3, doubled to 6, add 48 for ASCII '6', print. Expected: `"6"`.

---

## Composing Functions

Functions can call other functions. This is how you build complex
behaviour from simple pieces.

### Chaining calls

```brainfuck
@import "std/math.bf"
@import "std/io.bf"

@fn print_digit {
    ++++++++++++++++++++++++++++++++++++++++++++++++
    .[-]
}

@fn triple_and_print {
    @call triple
    @call print_digit
}

+++                       Cell 0 = 3
@call triple_and_print    Prints '9'
```

### Nested function definitions

Functions defined across different files compose naturally through
imports:

```
lib/display.bf:
    @import "std/io.bf"
    @fn print_digit { ++++++++++++++++++++++++++++++++++++++++++++++++.[-] }
    @fn show_value  { @call print_digit @call print_newline }

src/main.bf:
    @import "../lib/display.bf"
    +++++ @call show_value
```

Here `show_value` calls `print_digit` (defined in the same file) and
`print_newline` (from `std/io.bf` which `display.bf` imported). All of
these are available at the call site because imports are transitive.

### Avoiding cycles

The preprocessor will reject circular calls:

```brainfuck
@fn ping { @call pong }
@fn pong { @call ping }
@call ping   Error: cycle detected: ping -> pong -> ping
```

Self-recursion is also rejected:

```brainfuck
@fn loop_forever { @call loop_forever }
Error: cycle detected: loop_forever -> loop_forever
```

This is intentional: since `@call` is compile-time inlining, infinite
recursion would produce an infinitely long program.

---

## Error Handling

The preprocessor provides helpful error messages:

### Unknown function

```
error: unknown function: 'prnt_newline'. Did you mean 'print_newline'?
  hint: 'print_newline' is defined in std/io.bf. Add: @import "std/io.bf"
```

The error includes Levenshtein distance suggestions from known
functions and a stdlib hint if the function exists in an unimported
module.

### Unknown stdlib module

```
error: unknown stdlib module: 'std/maths.bf'. Did you mean 'math'?
  available modules: ascii, debug, io, logic, math, memory, string
```

### Import cycle

```
error: import cycle detected: src/a.bf -> lib/b.bf -> src/a.bf
```

### Call cycle

```
error: cycle detected: ping -> pong -> ping
```

### Unmatched braces

```
error: unclosed '{' in @fn definition
```

---

## Advanced Directives

### Constants with `@const` and `@use`

Define numeric constants that expand to repeated `+` instructions:

```brainfuck
@const NEWLINE 10
@const SPACE 32
@const ZERO_CHAR 48

@use ZERO_CHAR      Expands to 48 '+' characters
.[-]
@use NEWLINE         Expands to 10 '+' characters
.[-]
```

This is useful for documenting magic numbers. Instead of counting
plus signs, you give the value a name.

---

## Patterns and Idioms

### Print a specific ASCII character

The most common use of functions: generate N `+` signs, print, clear:

```brainfuck
@fn print_A {
    +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]
}
```

The stdlib `io.bf` and `ascii.bf` modules provide many of these.

### Convert and display a value

To print a numeric cell value as a readable digit:

```brainfuck
@import "std/ascii.bf"

+++++                     Cell 0 = 5
@call digit_to_char       Cell 0 = 53 (ASCII '5')
.[-]                      Print and clear
```

### Clear before reuse

When reusing cells across multiple function calls, clear them first:

```brainfuck
@import "std/memory.bf"
@import "std/math.bf"

+++++ @call double .[-]   Cell 0: 5 -> 10 -> print -> 0
+++ @call triple .[-]     Cell 0: 3 -> 9 -> print -> 0
```

The `[-]` (or `@call clear`) pattern after printing ensures the cell
is ready for the next operation.

### Conditional execution

Use `@fn` to name conditional patterns:

```brainfuck
@import "std/logic.bf"

+++++                     Cell 0 = 5
@call bool                Cell 0 = 1 (normalised to boolean)
```

The `not`, `bool`, `and`, `or`, and `equal` functions from
`std/logic.bf` all operate on cell 0 and return 0 or 1.

### Working with multiple values

Plan your cell layout, move the pointer explicitly, and call functions
on the appropriate cell:

```brainfuck
@import "std/math.bf"
@import "std/memory.bf"

Cell layout: [A] [B] [scratch...]
+++           Cell 0 = 3 (A)
>+++++++      Cell 1 = 7 (B)
<             Back to cell 0

@call swap    Now cell 0 = 7 (B), cell 1 = 3 (A)
```
