# Architecture

## Overview

ogre is a Cargo-like all-in-one brainfuck toolchain written in Rust. It follows
a modular architecture where each CLI subcommand maps to a distinct mode module
under `src/modes/`.

## Directory Layout

```
src/
  main.rs              CLI definition (clap derive) and subcommand dispatch
  lib.rs               Library crate root (re-exports for integration tests)
  error.rs             OgreError enum (typed error variants)
  verbosity.rs         Verbosity enum (Quiet/Normal/Verbose)
  project.rs           ogre.toml parsing and validation (OgreProject)
  modes/
    mod.rs             Module declarations
    ir.rs              Bytecode IR (Op enum, Program, optimization passes)
    interpreter.rs     Core BF interpreter (tape, step/run, I/O)
    preprocess.rs      @fn/@call/@import/@const/@doc macro preprocessor
    directive_parser.rs Shared tokenizer for @ directives
    run.rs             `ogre run` (thin wrapper around interpreter)
    compile.rs         `ogre compile` (BF -> C -> gcc)
    format.rs          `ogre format` (in-place formatter with diff support)
    analyse.rs         `ogre analyse` (static analysis and linting)
    debug.rs           `ogre debug` (GDB-style interactive debugger)
    start.rs           `ogre start` (interactive REPL)
    test_runner.rs     `ogre test` (JSON-driven test runner)
    check.rs           `ogre check` (validation: brackets, imports, calls)
    pack.rs            `ogre pack` (expand all macros to pure BF)
    bench.rs           `ogre bench` (instruction counting and timing)
    new.rs             `ogre new` (project scaffolding)
    init.rs            `ogre init` (initialize ogre.toml in CWD)
    generate.rs        `ogre generate` (code generation: hello, string, loop)
    stdlib.rs          `ogre stdlib` (explore built-in standard library)
    doc.rs             `ogre doc` (documentation from @doc/@fn)
stdlib/
  io.bf                I/O utility functions
  math.bf              Arithmetic functions
  memory.bf            Memory/tape manipulation functions
  ascii.bf             ASCII character utilities
  debug.bf             Debugging helpers
tests/
  brainfuck_scripts/   BF test fixtures (hello_world.bf, etc.)
  cli_integration.rs   32 CLI integration tests (assert_cmd)
  preprocess_integration.rs  11 preprocessor integration tests
  interpreter_integration.rs 8 interpreter integration tests
  generate_integration.rs    10 code generation integration tests
  format_integration.rs      7 format integration tests
examples/
  hello/               Hello World example project
  fibonacci/           Fibonacci sequence example
  cat/                 Cat (echo) example
  multifile/           Multi-file brainfunct example
  stdlib-demo/         Standard library demo
```

## Data Flow

All file-based subcommands follow the same pipeline:

```
Source (.bf) -> Preprocessor -> Expanded BF -> Mode-specific processing
```

1. **Preprocessor** (`preprocess.rs`) resolves `@import`, expands `@fn`/`@call`,
   and evaluates `@const`/`@use` directives. The output is pure brainfuck with
   all macros resolved.

2. **IR Compilation** (`ir.rs`) parses the expanded BF into `Vec<Op>`, a typed
   instruction array with run-length encoding and bracket pairing.

3. **Optimization** (`ir.rs`) applies multiple passes: clear idiom detection,
   cancellation, and dead store elimination.

4. **Execution/Output** varies by mode:
   - `run`: Interpreter executes the IR directly
   - `compile`: IR is translated to C, then compiled with gcc/clang
   - `format`: Source is reformatted (directive-aware)
   - `analyse`: IR is statically analyzed for patterns
   - `debug`: IR-based interpreter with single-step support

## Key Design Decisions

### Why clap derive?

clap's derive API provides compile-time checked CLI definitions with automatic
help generation. The `#[command(after_help = "...")]` attribute adds examples
to each subcommand. Global flags (--quiet, --verbose, --no-color) are declared
once and propagated to all subcommands.

### Why anyhow + OgreError?

The CLI layer uses `anyhow::Result` for ergonomic error propagation with context.
Internal modules use `OgreError` variants for typed error handling that callers
can match on (e.g., distinguishing `BracketMismatch` from `ImportCycle`). The
`OgreError` enum derives `thiserror::Error` and converts to `anyhow::Error`
via the `Into` trait.

### Why Vec<Op> instead of Vec<char>?

A typed IR enables:
- Run-length encoding (3 bytes `+++` -> 1 op `Add(3)`)
- Idiom recognition (`[-]` -> `Clear`)
- Optimization passes (cancellation, dead store elimination)
- Correct bracket pairing at parse time (not runtime)
- Shared representation across interpreter, compiler, and analyser

### Why embed stdlib with include_str!()?

The standard library is embedded in the binary at compile time. This means:
- No installation path to configure
- No missing files at runtime
- Works identically in debug and release builds
- Zero I/O cost for stdlib imports

### Why two-pass preprocessing?

Pass 1 (collect) resolves imports and gathers function definitions. Pass 2
(expand) inlines function bodies at call sites. This separation enables:
- Import cycle detection during collection
- Call cycle detection during expansion
- Functions defined in imported files are available before the calling code

## Dependency Rationale

| Crate | Purpose | Why chosen |
|-------|---------|------------|
| clap | CLI parsing | Industry standard, derive API, auto-help |
| anyhow | Error handling | Ergonomic ? operator, context chaining |
| thiserror | Error enum | Derives Display/Error, works with anyhow |
| colored | Terminal colors | Simple API, supports NO_COLOR |
| serde/serde_json | JSON parsing | For test case files |
| toml | TOML parsing | For ogre.toml project manifest |
| similar | Diff generation | For `format --diff` unified diffs |
| regex | Pattern matching | For test output_regex matching |
