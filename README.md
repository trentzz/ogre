# ogre

A fully featured brainfu(ck/nct) tool. Sorta. Hopefully one day.

## Install

Will be active eventually.

```bash
pipx install ogre
```

## Commands

| Command    | Description                                 |
| ---------- | ------------------------------------------- |
| `compile`  | Compiles a brainfuck script into a binary.  |
| `debug`    | Enters debugging mode for a brainfuck file. |
| `format`   | Formats a brainfuck file.                   |
| `generate` | Generates brainfuck code.                   |
| `run`      | Runs a brainfuck script.                    |
| `start`    | Starts a brainfuck interpreter instance.    |

#### Compile

Compiles a brainfuck script into a binary. This is done by code generating C
code and compiling that.

```bash
ogre compile <file> -o <output-file>
```

### Debug

Enters debugging mode for a brainfuck file. Allows you to step through
instructions, peek at memory, and much more. This is done through interpreting
the brainfuck script.

```bash
ogre debug <file>
```

```bash
(ogre debug) <command>
```

| Command                 | Description                                                                                    |
| ----------------------- | ---------------------------------------------------------------------------------------------- |
| `step`                  | Steps one instruction.                                                                         |
| `step <n>`              | Steps n instructions.                                                                          |
| `peek`                  | Shows the values around the current memory pointer location.                                   |
| `peek <n>`              | Shows the values around the memory pointer location n.                                         |
| `breakpoint`            | Sets a break point at the current instruction.                                                 |
| `breakpoint <n>`        | Sets a break point at instruction number n (0 indexed).                                        |
| `breakpoint list`       | Shows a list of break point numbers and associated instruction numbers.                        |
| `breakpoint delete <n>` | Removes break point number n. Note, this is the break point number and not instruction number. |
| `show instruction`      | Shows current instruction.                                                                     |
| `show instruction <n>`  | Shows instruction number n (0 indexed).                                                        |

### Format

Formats a brainfuck script.

```bash
ogre format <file> --indent 4
                   --linewidth 80
                   --grouping 5
                   --label-functions
                   --preserve-comments
```

| Option                | Description                                                                                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--indent`            | Changes the indentation amount in spaces.                                                                                                                           |
| `--linewidth`         | Sets the maximum linewidth.                                                                                                                                         |
| `--grouping`          | Sets the number of instructions to group together, e.g. `+++++ +++++` for a grouping of 5.                                                                          |
| `--label-functions`   | For brainfunct: Labels and numbers functions with comments that provide some information on a newline above the start of the function, e.g. `FUNCTION 1, 2, 3` etc. |
| `--preserve-comments` | Preserves any non-brainfuck instructions (i.e. comments) although the location of these comments relative to pre-existing code is not guaranteed.                   |

### Generate

Generates some brainfuck code.

```bash
ogre generate helloworld -o <file>
              string <string-to-print> -o <file>
              loop <n> -o <file>
```

| Option            | Description                                              |
| ----------------- | -------------------------------------------------------- |
| `helloworld`      | Generates a script that prints `Hello World!` to stdout. |
| `string <string>` | Generates a script that prints `<string>` to stdout.     |
| `loop <n>`        | Generates a script that loops n times.                   |

Note: If output file `-o <file>` is not specified, ogre generate will print the generated script to stdout.
