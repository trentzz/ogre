# Testing Guide

## Test Organization

ogre has 228+ tests across multiple levels:

| Category | Count | Location |
|----------|-------|----------|
| Unit tests (ir, interpreter, preprocess, etc.) | ~160 | `src/modes/*.rs` |
| CLI integration tests | 32 | `tests/cli_integration.rs` |
| Preprocessor integration | 11 | `tests/preprocess_integration.rs` |
| Code generation integration | 10 | `tests/generate_integration.rs` |
| Interpreter integration | 8 | `tests/interpreter_integration.rs` |
| Format integration | 7 | `tests/format_integration.rs` |

## Running Tests

```bash
# Run all tests
cargo test

# Run a specific test by name
cargo test test_clear_idiom

# Run tests in a specific module
cargo test modes::ir

# Run only CLI integration tests
cargo test --test cli_integration

# Run tests with output visible
cargo test -- --nocapture
```

## Unit Test Coverage by Module

### ir.rs (21 tests)
- Parsing: empty source, comments stripped, run-length collapsing
- Bracket pairing: simple, nested, unmatched open/close
- Optimization: clear idiom, cancellation (add/sub, moves), dead store
- Back-conversion: to_bf_string roundtrip

### interpreter.rs (19 tests)
- Basic operations: increment, decrement, move, I/O
- Loops: simple, nested, skip-if-zero
- Edge cases: wrapping arithmetic (255+1=0), cell initialization
- Features: instruction count, cells touched, run with limit

### preprocess.rs (22 tests)
- Functions: define, call, nested calls, empty body
- Cycles: direct, self-referential
- Imports: file-based, standard library, double import
- Constants: basic, zero, large, inside @fn, undefined, missing value
- Error cases: unknown directive, missing brace, nonexistent import

### format.rs (12 tests)
- Formatting: basic BF, loop indentation, nested loops
- Options: indent size, line width, grouping
- Directives: @fn/@call preserved
- Features: diff generation, check mode, idempotency

### analyse.rs (14 tests)
- Bracket validation: valid, unmatched open/close
- I/O counting: inputs, outputs, mixed
- Patterns: clear idiom count, cancellation positions, dead code
- Pointer tracking: unbalanced pointer detection

### check.rs (6 tests)
- Valid source, unmatched brackets (open/close)
- Empty source, nested brackets
- File-based check with hello_world.bf

### compile.rs (14 tests)
- Code generation: each op type, comments ignored
- Structure: includes, main function, return
- Features: collapsed ops, clear idiom, custom tape size, nested indentation

### test_runner.rs (5 tests)
- Inline test cases: pass, fail, with input
- Error handling: invalid BF
- Features: instruction limit timeout, regex matching

### bench.rs (4 tests)
- Number formatting helper
- Benchmarking: instruction count, cells touched, elapsed time

### pack.rs (4 tests)
- Comment stripping
- File packing: basic, with optimization
- Optimization produces shorter output

### doc.rs (6 tests)
- Empty file, with functions, with @doc comments
- Multi-line doc comments
- Stdlib documentation generation
- Alphabetical function sorting

### generate.rs (7 tests)
- Hello world: produces valid BF that outputs "Hello World!"
- String: ASCII characters, non-ASCII error
- Loop: correct iteration count, zero iterations

## CLI Integration Tests (32 tests)

Tests use `assert_cmd` for process invocation and `predicates` for output
assertions. `tempfile` provides isolated temporary directories.

Categories:
- **Version/help:** --version, --help, no subcommand, unknown subcommand
- **Run:** valid file, nonexistent file, custom tape size
- **Check:** valid file, unmatched bracket
- **Format:** --check, --diff, in-place modification
- **Generate:** helloworld, string, loop, file output
- **New/Init:** project creation, --with-std, duplicate detection
- **Pack:** basic, --optimize
- **Analyse/Bench:** output verification
- **Stdlib:** list, show, unknown module
- **Schema validation:** empty name, wrong entry extension

## Writing Tests for ogre

### Unit test pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }
}
```

### CLI integration test pattern

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_feature() {
    let dir = tempfile::tempdir().unwrap();
    // ... create test files ...

    Command::cargo_bin("ogre")
        .unwrap()
        .args(["subcommand", "arg"])
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

### ogre test runner (JSON tests)

```json
[
  {
    "name": "test description",
    "brainfuck": "path/to/program.bf",
    "input": "input string",
    "output": "expected output"
  },
  {
    "name": "regex test",
    "brainfuck": "path/to/program.bf",
    "input": "",
    "output": "",
    "output_regex": "pattern.*here"
  }
]
```

Fields:
- `name`: Test description (shown in failure messages)
- `brainfuck`: Path to BF file (relative to test file's directory)
- `input`: Stdin input for the program
- `output`: Expected stdout output (exact match)
- `output_regex`: Regex pattern for output (mutually exclusive with `output`)
- `timeout`: Instruction limit override (default: 10,000,000)
