# CLI & Developer Experience Ideas

## New Commands

### ogre lint
Dedicated linting command (separate from analyse). Configurable rules, severity levels, and ignore patterns. Support `.ogrelint.toml` config file for per-project rules. Rules like "max function size", "no nested loops deeper than 3", "all functions must have @doc".

### ogre minify
Produce the smallest possible BF output. Strip all whitespace, comments, and non-BF characters. Optimize for code size (use shortest equivalent instruction sequences). Useful for code golf and sharing.

### ogre diff
Semantically diff two BF programs. Show differences at the instruction level, not character level. Ignore formatting differences. Highlight behavioral changes vs cosmetic changes.

### ogre convert
Convert between BF dialects:
- Standard BF (+-<>.,[ ])
- Extended BF (with # for debug)
- Ook! (Ook. Ook? Ook!)
- Whitespace-encoded BF
- Custom character mappings
- `ogre convert --from bf --to ook program.bf`

### ogre profile
Dedicated profiling command (beyond bench). Generate flamegraphs, instruction histograms, memory access patterns, and I/O timing. Output in formats consumable by external tools (perf, flamegraph.pl).

### ogre clean
Remove build artifacts, cached files, and temporary outputs. `ogre clean --all` removes everything. `ogre clean --cache` removes only cached preprocessor output.

### ogre upgrade
Self-update ogre to the latest version. Check for updates on startup (optional, can be disabled). Show changelog for new version.

### ogre doctor
Diagnose the development environment. Check for required tools (gcc, wabt for WASM), verify PATH configuration, check project structure, validate ogre.toml. Report issues and suggest fixes.

### ogre repl-import
Launch the REPL with specific stdlib modules pre-imported. `ogre start --import std/math.bf --import std/io.bf` makes all math and io functions available immediately.

### ogre tree
Display the project's file dependency tree. Show which files import which, and which functions are defined where. Graphical output using Unicode box-drawing characters.

### ogre explain
Explain what a BF program does in plain English. Use static analysis to describe the program's behavior: "This program reads a character, adds 32 to convert uppercase to lowercase, and prints the result." Could use pattern matching against known idioms.

### ogre golf
Code golf mode. Show the shortest known implementation for common tasks. Compare your solution's length against known shortest. Leaderboard of shortest programs per task.

---

## Project Management

### Workspace Support
Multi-project workspaces (like Cargo workspaces):
```toml
[workspace]
members = ["lib/", "cli/", "examples/*"]
```
Build, test, and manage multiple related projects together.

### Dependency Management
Allow projects to depend on other ogre projects:
```toml
[dependencies]
my-lib = { path = "../my-lib" }
my-remote-lib = { git = "https://github.com/user/lib" }
```
Resolve and include dependency functions automatically.

### Lock File
Generate `ogre.lock` with exact dependency versions and file hashes. Ensure reproducible builds.

### Scripts Section in ogre.toml
Custom script definitions:
```toml
[scripts]
lint = "ogre analyse src/main.bf && ogre format --check src/"
release = "ogre build && ogre test && ogre compile --release"
```
Run with `ogre run-script lint`.

---

## Developer Experience

### Shell Completions
Generate shell completion scripts for bash, zsh, fish, and PowerShell. `ogre completions bash > /etc/bash_completion.d/ogre`. Clap already supports this - just needs to be exposed.

### Colored Diff Output
When `ogre format --diff` shows changes, use syntax-aware colorization. Highlight BF operators in different colors within the diff.

### Progress Bars
Show progress bars for long operations: compilation, large test suites, code generation. Use `indicatif` crate for clean progress reporting.

### Configuration File
Global `~/.config/ogre/config.toml` for user preferences:
```toml
[format]
line_width = 100
indent = 2

[compile]
default_tape_size = 65536
keep_intermediate = false

[output]
color = "auto"
```

### Error Recovery
When a command fails, suggest the most likely fix. "Did you mean `ogre run src/main.bf`?" for common mistakes. Show relevant documentation links.

### Man Pages
Generate man pages from the CLI help. Install them with the binary. `man ogre`, `man ogre-run`, `man ogre-compile`, etc.

### Changelog Generation
Auto-generate changelog entries from git commits when releasing. Categorize changes by type (feature, fix, breaking change).

### Init Templates
`ogre new --template game` to scaffold different project types:
- `basic` - Simple hello world (current default)
- `game` - Interactive I/O loop with ANSI codes
- `converter` - CLI tool with argument parsing
- `library` - Stdlib-style library with tests
- `multi-file` - Multi-module project structure
