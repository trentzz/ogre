# ogre

A Cargo-like all-in-one brainfuck tool. One binary covering the full development lifecycle for brainfuck programs: running, compiling to a native binary, formatting, static analysis, structured testing, code generation, an interactive REPL, and a GDB-style debugger.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
  - [run](#run)
  - [compile](#compile)
  - [start](#start)
  - [debug](#debug)
  - [format](#format)
  - [analyse](#analyse)
  - [test](#test)
  - [new](#new)
  - [generate](#generate)
- [Test File Format](#test-file-format)
- [Building from Source](#building-from-source)

---

## Installation

Requires [Rust and Cargo](https://rustup.rs). Install directly from GitHub:

```sh
cargo install --git https://github.com/trentzz/ogre
```

To install from a specific branch:

```sh
cargo install --git https://github.com/trentzz/ogre --branch rust-rewrite
```

Once installed, the `ogre` binary is available on your PATH.

---

## Quick Start

```sh
# Run a brainfuck file
ogre run hello.bf

# Format a file in-place
ogre format hello.bf

# Analyse a file for issues
ogre analyse hello.bf

# Compile to a native binary
ogre compile hello.bf -o hello

# Open the interactive REPL
ogre start

# Scaffold a new project
ogre new myproject
```

---

## Commands

### `run`

Interprets and executes a brainfuck file directly. If the program reads input (`,`), it is read from stdin.

```sh
ogre run <file>
```

**Example:**

```sh
ogre run hello.bf
# Hello World!

echo "Hello" | ogre run cat.bf
# Hello
```

---

### `compile`

Compiles a brainfuck file to a native binary by first translating it to C, then invoking `gcc`. Requires `gcc` to be installed.

```sh
ogre compile <file> [-o <output>] [-k/--keep]
```

| Flag | Description |
|---|---|
| `-o <name>` | Output binary name (defaults to the input filename without extension) |
| `-k`, `--keep` | Keep the intermediate `.c` file instead of deleting it |

**Example:**

```sh
ogre compile hello.bf
# Compiled to: hello

ogre compile hello.bf -o greet --keep
# Compiled to: greet
# (greet.c is also kept)

./hello
# Hello World!
```

---

### `start`

An interactive brainfuck REPL. Type BF snippets line by line; the tape state persists between inputs. After each snippet executes, the memory window around the data pointer is printed.

```sh
ogre start
```

**Session example:**

```
ogre interactive interpreter — type BF code, 'reset' to clear, 'exit' to quit
>>> +++++
  tape: [ >0:5<  1:0  2:0  3:0 ]
>>> >+++
  tape: [ 0:5  >1:3<  2:0  3:0 ]
>>> .
  (prints ASCII 3)
  tape: [ 0:5  >1:3<  2:0  3:0 ]
>>> reset
Tape reset.
>>> exit
Goodbye.
```

Special commands: `reset` (clears the tape), `exit` / `quit`.

---

### `debug`

A GDB-style interactive debugger. Loads a brainfuck file and pauses before execution. Execution is driven by commands; after every pause the current instruction and memory state are shown.

```sh
ogre debug <file>
```

**Debugger commands:**

| Command | Description |
|---|---|
| `step` / `step <n>` | Execute 1 (or n) instruction(s) and pause |
| `continue` / `c` | Run until the next breakpoint or end of program |
| `breakpoint <n>` | Set a breakpoint at instruction index n |
| `breakpoint list` | List all breakpoints |
| `breakpoint delete <n>` | Remove breakpoint n |
| `jump <n>` | Move the instruction pointer to index n without executing |
| `peek` / `peek <n>` | Show memory around the current pointer (or cell n) |
| `show instruction` / `show instruction <n>` | Show current (or nth) instruction in context |
| `show memory` | Dump a wider range of memory cells |
| `help` | Print the command reference |
| `exit` / `quit` / `q` | Quit the debugger |

**Session example:**

```
ogre debugger — type 'help' for commands
  ip=0 op='+'  dp=0  val=0
  tape: [ >0:0<  1:0  2:0  3:0 ]
(ogre-dbg) step 3
  ip=3 op='['  dp=0  val=3
  tape: [ >0:3<  1:0  2:0  3:0 ]
(ogre-dbg) breakpoint 8
Breakpoint set at instruction 8
(ogre-dbg) continue
Hit breakpoint at 8.
  ip=8 op='-'  dp=0  val=2
  tape: [ >0:2<  1:1  2:0  3:0 ]
(ogre-dbg) peek
  tape: [ >0:2<  1:1  2:0  3:0  4:0  5:0  6:0 ]
(ogre-dbg) exit
```

---

### `format`

Formats a brainfuck file **in-place**. Loop bodies are indented, long lines are wrapped, and runs of the same operator are spaced for readability.

```sh
ogre format <file> [options]
```

| Flag | Default | Description |
|---|---|---|
| `--indent <n>` | `4` | Spaces of indentation per loop level |
| `--linewidth <n>` | `80` | Maximum line width before wrapping |
| `--grouping <n>` | `5` | Insert a space every n consecutive identical operators |
| `--label-functions` | off | *(brainfunct)* Insert comment labels above each function |
| `-p`, `--preserve-comments` | off | Keep non-BF characters as inline comments |

**Example — before:**

```
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

**After `ogre format hello.bf`:**

```
+++++ +++
[
    >++++
    [
        >++>+++>+++>+<<<<-
    ]
    >+>+>->>+
    [
        <
    ]
    <-
]
>>.>---.+++++ ++..+++.>>.<-.<.+++.----- -.----- ---.>>+.>++.
```

If the nesting depth multiplied by the indent would exceed `linewidth - 10`, ogre errors rather than producing unreadable output.

---

### `analyse`

Performs static analysis on a brainfuck file and prints a report covering:

- Bracket matching errors
- Total input (`,`) and output (`.`) operation counts
- Net data pointer movement (or reports indeterminate if loops are present)

```sh
ogre analyse <file> [--verbose] [--in-place]
```

| Flag | Description |
|---|---|
| `--verbose` | Extra detail: per-operator counts |
| `--in-place` | Embed the analysis report as comments at the top of the source file |

**Example output:**

```sh
ogre analyse hello.bf
```

```
Brackets: OK
Input operations (,):  0
Output operations (.): 13
Data pointer: indeterminate (program contains loops)
```

**With `--verbose`:**

```
Brackets: OK
Input operations (,):  0
Output operations (.): 13
Data pointer: indeterminate (program contains loops)

=== VERBOSE ===
  > (move right): 20
  < (move left):  14
  + (increment):  48
  - (decrement):  31
  [ (loop open):  4
  ] (loop close): 4
```

---

### `test`

Runs structured tests defined in a JSON file. Each entry specifies a brainfuck script, optional stdin input, and expected stdout output. ogre runs the interpreter against each case and reports pass/fail.

```sh
ogre test <test-file.json>
```

Exits with a non-zero status if any test fails, making it suitable for CI pipelines.

**Example output:**

```
PASS  hello world
PASS  cat passthrough
FAIL  multiply
      expected: "\f"
      actual:   "\v"

2/3 tests passed
Error: 1 test(s) failed
```

See [Test File Format](#test-file-format) for the JSON schema.

---

### `new`

Scaffolds a new brainfuck project directory with a starter `.bf` file and a `tests.json` template.

```sh
ogre new <name>
```

**Example:**

```sh
ogre new myproject
# Created project 'myproject':
#   myproject/myproject.bf
#   myproject/tests.json
```

The generated `tests.json` is pre-filled with a template entry pointing at the new `.bf` file, ready to be filled in.

---

### `generate`

Generates brainfuck code for common patterns. Output goes to stdout unless `-o` is given.

```sh
ogre generate <subcommand> [options]
```

#### `generate helloworld`

Outputs the classic Hello World brainfuck program.

```sh
ogre generate helloworld
# ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.

ogre generate helloworld -o hello.bf
# Written to: hello.bf
```

#### `generate string <str>`

Generates a program that prints an arbitrary string.

```sh
ogre generate string "Hi!"
# +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.-------.+++++++++++++++++++++++.

ogre generate string "Hi!" | ogre run /dev/stdin
# Hi!
```

#### `generate loop <n>`

Generates a loop scaffold that iterates exactly `n` times. Cell 0 is used as the counter (ends at 0); cell 1 accumulates the count.

```sh
ogre generate loop 5
# +++++[>+<-]
```

Use this as a starting point when you need a counted loop.

---

## Test File Format

Test files are JSON arrays. Each object describes one test case:

```json
[
  {
    "name": "hello world",
    "brainfuck": "scripts/hello_world.bf",
    "input": "",
    "output": "Hello World!\n"
  },
  {
    "name": "cat passthrough",
    "brainfuck": "scripts/cat.bf",
    "input": "Hello",
    "output": "Hello"
  }
]
```

| Field | Type | Description |
|---|---|---|
| `name` | string | Human-readable test name shown in output |
| `brainfuck` | string | Path to the `.bf` file (relative to where `ogre test` is run) |
| `input` | string | Data fed to stdin (`,` instructions) |
| `output` | string | Expected stdout output to match against |

---

## Building from Source

```sh
git clone https://github.com/trentzz/ogre
cd ogre
cargo build --release
# Binary is at target/release/ogre
```

Run the test suite:

```sh
cargo test
```
