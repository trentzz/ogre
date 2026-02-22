# Project Management Guide

ogre uses `ogre.toml` as its project manifest, analogous to `Cargo.toml` in Rust. This guide covers project structure, manifest configuration, dependencies, testing, and CI.

## 1. Project Structure

```
myproject/
  ogre.toml            # project manifest (required)
  src/
    main.bf            # entry point
    utils.bf           # additional source files
  lib/
    io_helpers.bf      # shared library code
  tests/
    basic.json         # test suite
    advanced.json      # additional test suite
```

The only hard requirements are `ogre.toml` and the file referenced by `project.entry`. The `src/`, `lib/`, and `tests/` directories are conventional.

## 2. ogre.toml Schema

```toml
[project]
name = "myproject"                       # required, must not be empty
version = "0.1.0"                        # required, must not be empty
description = "A brainfuck project"      # optional
author = "Alice"                         # optional
entry = "src/main.bf"                    # required, must end with .bf

[build]
include = [                              # source paths for format/analyse/check
    "src/",                              # trailing / = all .bf files (non-recursive)
    "lib/utils.bf",                      # specific file
    "src/**/*.bf",                       # glob pattern (recursive)
]
tape_size = 30000                        # optional, default 30000, must be > 0

[[tests]]
name = "Basic"                           # optional display name
file = "tests/basic.json"               # required, must end with .json

[[tests]]
name = "Advanced"
file = "tests/advanced.json"

[dependencies]
my-lib = { path = "../my-lib" }          # path-based dependency
```

| Section | Field | Required | Type | Notes |
|---|---|---|---|---|
| `[project]` | `name` | yes | string | Default binary name for `ogre build` |
| `[project]` | `version` | yes | string | Free-form version string |
| `[project]` | `description` | no | string | Shown on `ogre build` output |
| `[project]` | `author` | no | string | Shown on `ogre build` output |
| `[project]` | `entry` | yes | string | Entry `.bf` file, relative to `ogre.toml` |
| `[build]` | `include` | yes (if section present) | array | Directories, files, or globs |
| `[build]` | `tape_size` | no | integer | Tape size for run/debug/compile; must be > 0 |
| `[[tests]]` | `name` | no | string | Section label printed during test runs |
| `[[tests]]` | `file` | yes | string | Path to JSON test file, relative to `ogre.toml` |
| `[dependencies]` | `<name>` | -- | table | Must have `path` or `version` |

## 3. Project Discovery

When a file argument is omitted from any command (`ogre run`, `ogre test`, `ogre format`, etc.), ogre walks upward from CWD looking for `ogre.toml`. The first one found is loaded and validated. This means you can run `ogre build` from any subdirectory.

If none is found: `error: no ogre.toml found. Run 'ogre new <name>' to create a project, or supply a file argument.`

## 4. Include Resolution

The `[build].include` array controls which files are processed by `ogre format`, `ogre analyse`, and `ogre check` when run without a file argument. Three kinds of entries are supported:

**Directory entries** (trailing `/`) -- collects all `.bf` files directly inside the directory, non-recursively: `"src/"` includes `src/main.bf` but not `src/sub/foo.bf`.

**Explicit file paths** -- includes a single named file: `"lib/utils.bf"`. Errors if the file does not exist.

**Glob patterns** -- patterns containing `*` or `?` are expanded using standard glob syntax. `*.bf` matches in one directory, `**/*.bf` matches recursively, `?.bf` matches single-character names. Results are sorted alphabetically.

All three can be mixed: `include = ["src/", "lib/utils.bf", "extras/**/*.bf"]`

## 5. Schema Validation

ogre validates the manifest immediately after parsing:

| Check | Error |
|---|---|
| `project.name` empty or whitespace | `project.name must not be empty` |
| `project.version` empty or whitespace | `project.version must not be empty` |
| `project.entry` missing `.bf` suffix | `project.entry must end with .bf` |
| `[[tests]].file` missing `.json` suffix | `tests[N].file must end with .json` |
| `build.tape_size` is 0 | `build.tape_size must be greater than 0` |
| dependency lacks `path` and `version` | `dependency "name" must have a 'path' or 'version' field` |

Filesystem checks (do include paths exist? do test files exist?) happen lazily when actually needed.

## 6. Dependencies

A dependency is another ogre project directory (containing its own `ogre.toml`) whose `@fn` definitions become available to your project.

```toml
[dependencies]
my-lib = { path = "../my-lib" }   # path relative to your ogre.toml
```

**Resolution process**: ogre verifies the directory and its `ogre.toml` exist, loads and validates it, collects all `@fn` definitions from the dependency's include files and entry file, then recursively resolves the dependency's own `[dependencies]`. All `@fn` bodies merge into a single map (last-loaded wins on name conflicts).

**Usage**: collected functions are injected into the preprocessor, so you can `@call` them directly without `@import`:

```brainfuck
@call dep_hello
@call dep_util
```

**Example workspace layout**:
```
workspace/
  my-app/ogre.toml          # [dependencies] shared-lib = { path = "../shared-lib" }
  my-app/src/main.bf         # can @call print_char directly
  shared-lib/ogre.toml
  shared-lib/src/io.bf       # @fn print_char { ... }
```

## 7. Testing

Test suites are JSON arrays. Each case specifies a brainfuck file, input, and expected output.

```json
[
  {
    "name": "hello world",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "Hello World!"
  },
  {
    "name": "with regex",
    "brainfuck": "src/greet.bf",
    "input": "",
    "output": "",
    "output_regex": "^Hello.*!$",
    "timeout": 5000000
  }
]
```

| Field | Required | Type | Description |
|---|---|---|---|
| `name` | yes | string | Display name |
| `brainfuck` | yes | string | Path to `.bf` file |
| `input` | yes | string | Stdin input |
| `output` | yes | string | Expected output (exact match) |
| `output_regex` | no | string | Regex to match output (mutually exclusive with non-empty `output`) |
| `timeout` | no | integer | Instruction limit (default: 10,000,000); reports TIMEOUT if exceeded |

**Path resolution**: `ogre test tests/basic.json` resolves `brainfuck` paths relative to the JSON file's directory. Project-wide `ogre test` resolves relative to the project base.

**Commands**: `ogre test tests/basic.json` (single file), `ogre test` (all project suites), `ogre test --verbose` (per-case PASS/FAIL instead of dots).

## 8. CI Integration

Three commands are designed for CI pipelines, all exiting with code 1 on failure:

```bash
ogre check              # validate brackets, imports, and calls
ogre format --check     # exit 1 if any file needs formatting (no modifications)
ogre format --diff      # print unified diff of what would change, exit 1 if any
ogre test               # run all [[tests]] suites
```

**Example GitHub Actions workflow**:

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install ogre
        run: cargo install --path .
      - run: ogre check
      - run: ogre format --check
      - run: ogre test
```

## 9. Creating Projects

**`ogre new <name>`** creates a new directory with `ogre.toml`, `src/main.bf`, and `tests/basic.json`. Refuses to run if the directory exists. Pass `--with-std` to include standard library imports in the starter file.

**`ogre init`** creates `ogre.toml` in the current directory (deriving the project name from the directory name), plus `src/` and `tests/` with starter files. Existing files are never overwritten. Refuses to run if `ogre.toml` already exists.

| Scenario | Command |
|---|---|
| Brand new project from scratch | `ogre new myproject` |
| Adding ogre to an existing directory | `ogre init` |

## 10. Example ogre.toml Files

**Minimal** -- single file, no build config, no tests:
```toml
[project]
name = "hello"
version = "0.1.0"
entry = "hello.bf"
```

**Standard project with tests**:
```toml
[project]
name = "myproject"
version = "0.1.0"
description = "A brainfuck application"
author = "Alice"
entry = "src/main.bf"

[build]
include = ["src/"]

[[tests]]
name = "Basic"
file = "tests/basic.json"
```

**Multi-file with globs and custom tape size**:
```toml
[project]
name = "big-project"
version = "2.0.0"
entry = "src/main.bf"

[build]
include = ["src/**/*.bf", "lib/**/*.bf"]
tape_size = 60000

[[tests]]
name = "Unit"
file = "tests/unit.json"

[[tests]]
name = "Integration"
file = "tests/integration.json"
```

**With dependencies**:
```toml
[project]
name = "my-app"
version = "1.0.0"
entry = "src/main.bf"

[build]
include = ["src/"]

[dependencies]
bf-stdlib = { path = "../bf-stdlib" }
io-utils  = { path = "../io-utils" }

[[tests]]
name = "All"
file = "tests/all.json"
```

**Using the embedded standard library** (no `[dependencies]` needed):
```toml
[project]
name = "stdlib-demo"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
```
With `src/main.bf`:
```brainfuck
@import "std/io.bf"
@import "std/math.bf"

@fn main {
    @call print_newline
}

@call main
```

The standard library modules (`io`, `math`, `memory`, `ascii`, `debug`) are embedded in the ogre binary and resolved automatically by the preprocessor.
