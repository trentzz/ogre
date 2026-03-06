# Language & Preprocessor Ideas

## New Directives

### @macro with Parameters
Parameterized macros that expand with substitution:
```
@macro repeat(n, body) { @use n [ body - ] }
@expand repeat(5, +++)
```
This would bring template-like metaprogramming to Brainfunct without changing the BF execution model.

### @if / @ifdef Conditional Compilation
Compile-time conditional inclusion:
```
@const DEBUG 1
@if DEBUG
  @call debug.dump_cell
@endif
```
Allows building debug and release versions from the same source. Support `@ifdef`, `@ifndef`, `@else`.

### @repeat Directive
Repeat a block N times at compile time:
```
@repeat 10 { + }   // Expands to ++++++++++
```
Cleaner than manually writing repetitive code or using @const with @use for non-increment patterns.

### @inline Hint
Mark a function for mandatory inlining at every call site (current behavior) vs `@noinline` to emit it as a labeled block. When a function is called many times, inlining bloats code. A noinline hint could emit a comment-delimited block that the optimizer treats specially.

### @assert Directive
Compile-time assertions about cell state:
```
@assert cell[0] == 0   // Error if analysis can't prove cell 0 is zero here
```
Helps catch bugs in stdlib functions where preconditions must hold.

### @test Directive
Inline test definitions:
```
@test "addition works" {
  @input ""
  @expect "A"
  +++++++++[>++++++++<-]>+.
}
```
Tests live alongside the code they test. `ogre test` discovers and runs them.

### @deprecated Directive
Mark functions as deprecated with a message:
```
@deprecated "Use convert.print_decimal instead"
@fn math.print_num { ... }
```
Emit warnings when deprecated functions are called.

### @alias Directive
Create function aliases:
```
@alias println = io.print_newline
```
Allows shorter names and migration paths when renaming functions.

---

## Module System Enhancements

### Visibility Control
Public/private function visibility:
```
@fn public my_api_function { ... }
@fn private helper_function { ... }
```
Private functions can only be called within the same file. Prevents leaking internal implementation details.

### Namespace Scoping
Allow qualified function calls without importing everything:
```
@import "std/math.bf" as math
@call math.double
```
The `as` clause gives the import a namespace prefix. Avoids name collisions between modules.

### Wildcard and Selective Imports
Import specific functions instead of entire modules:
```
@import { double, triple } from "std/math.bf"
```
Reduces preprocessor work and makes dependencies explicit.

### Re-exports
Allow a module to re-export functions from other modules:
```
@import "std/math.bf"
@export math.double   // Available to importers of this file
```
Enables facade modules that curate a public API from multiple internal modules.

---

## Type System (Lightweight)

### Cell Range Annotations
Annotate expected value ranges for documentation and analysis:
```
@fn ascii.to_upper {
  @expects cell[0] in 97..122   // lowercase a-z
  @produces cell[0] in 65..90   // uppercase A-Z
  ...
}
```
The analyser could verify these constraints statically or insert runtime checks in debug mode.

### Memory Layout Annotations
Document which cells a function uses:
```
@fn my_function {
  @uses cell[0]     // input/output
  @scratch cell[1..3]  // temporary, zeroed on exit
  ...
}
```
Enables the analyser to detect cell conflicts when composing functions.

### Pointer Position Tracking
Annotate expected pointer position:
```
@fn my_function {
  @pointer_in 0    // expects DP at cell 0
  @pointer_out 0   // leaves DP at cell 0
}
```
The analyser verifies pointer positions are consistent across @call chains.

---

## Preprocessor Improvements

### Source Map Comments
Emit `// @source file.bf:42` comments in packed output. Allows tracing optimized code back to source even without the debugger.

### Macro Expansion Tracing
`ogre pack --trace-macros` shows each macro expansion step. Useful for debugging complex nested @call chains.

### Include Guards
Prevent double-inclusion of the same file:
```
@pragma once
```
Or automatic include guard detection. Currently double-importing the same file duplicates all function definitions.

### String Literals
Support string constants that expand to BF code:
```
@string GREETING "Hello, World!\n"
@call print_string GREETING
```
Currently, string printing requires manual ASCII value computation or the generate command.

### Arithmetic in Constants
Allow basic math in @const:
```
@const NEWLINE 10
@const DOUBLE_NEWLINE (NEWLINE * 2)
```
Computed at preprocessing time.
