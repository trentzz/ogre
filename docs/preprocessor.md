# Preprocessor Design

The preprocessor (`src/modes/preprocess.rs`) transforms brainfunct source code
into pure brainfuck by resolving macros and imports at compile time.

## Directive Reference

| Directive | Syntax | Description |
|-----------|--------|-------------|
| `@import` | `@import "path.bf"` | Include function definitions from another file |
| `@fn` | `@fn name { body }` | Define a named macro |
| `@call` | `@call name` | Inline a macro's body at the call site |
| `@const` | `@const NAME value` | Define a named numeric constant |
| `@use` | `@use NAME` | Expand a constant to N `+` characters |
| `@doc` | `@doc description text` | Attach documentation to the next `@fn` |

## Two-Pass Architecture

### Pass 1: Collect

The collect pass walks the source character by character, handling directives:

1. **`@import "path"`** — Reads the imported file and recursively collects
   its function definitions. Top-level code in imported files is discarded.
   Import cycle detection uses a `HashSet<PathBuf>` of canonical paths.

2. **`@import "std/module.bf"`** — Resolves standard library modules from
   embedded source (`include_str!()`). Uses synthetic path entries
   (`<stdlib:module>`) for deduplication.

3. **`@fn name { body }`** — Stores the function body in
   `HashMap<String, String>`. The `{`/`}` delimiters are consumed but not
   included in the body.

4. **`@call name`** — Preserved in the top-level output as `@call name`
   markers for Pass 2 to process.

5. **`@const NAME value`** — Parses the identifier and numeric value,
   stores in `HashMap<String, usize>`.

6. **`@use NAME`** — Looks up the constant value and expands to N `+`
   characters inline.

7. **`@doc text`** — Accumulates doc lines. When the next `@fn` is
   encountered, the accumulated doc is attached to the function.

All non-directive characters are appended to the top-level output string.

### Pass 2: Expand

The expand pass walks the top-level output from Pass 1 and replaces
`@call name` markers with the corresponding function bodies, recursively
expanding any `@call` references within those bodies.

**Cycle detection:** A `Vec<String>` call stack tracks the current expansion
chain. If a function name already appears in the stack, expansion halts with
a cycle error: `"cycle detected: a -> b -> a"`.

## Import Resolution

### File imports

```
@import "relative/path.bf"
```

Paths are resolved relative to the directory containing the importing file.
Canonical paths (via `fs::canonicalize()`) are used for cycle detection,
with a fallback to the raw path if canonicalization fails.

### Standard library imports

```
@import "std/io.bf"
@import "std/math"     (both forms work)
```

The `std/` prefix triggers resolution from embedded source code. Available
modules: io, math, memory, ascii, debug.

## Error Handling

| Error | Cause |
|-------|-------|
| `OgreError::ImportCycle` | File A imports B which imports A |
| `OgreError::CycleDetected` | Function A calls B which calls A |
| `OgreError::UnknownFunction` | `@call name` where `name` is not defined |
| `OgreError::UnknownStdModule` | `@import "std/nonexistent"` |
| `OgreError::UnknownDirective` | `@xyz` where `xyz` is not a known directive |

## Example

Input:
```brainfuck
@const NEWLINE 10
@import "std/math.bf"

@doc Prints a newline character.
@fn print_nl {
    @use NEWLINE
    .[-]
}

@call zero
@call print_nl
```

After Pass 1 (top-level output):
```
@call zero
@call print_nl
```

After Pass 2 (expanded):
```
[-]
++++++++++.[-]
```

The `@use NEWLINE` inside the `@fn` body is expanded during Pass 2 when
`print_nl` is inlined.
