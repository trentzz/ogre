# IDE & Editor Integration Ideas

## Language Server Protocol (LSP)

### Full LSP Implementation
Implement the Language Server Protocol for Brainfunct. This would provide IDE features for any editor that supports LSP (VS Code, Neovim, Emacs, Helix, Zed, etc.).

Features to support:
- **Diagnostics**: Real-time bracket matching errors, unresolved @call references, missing imports
- **Go to Definition**: Jump from @call to @fn definition, from @import to the imported file
- **Find References**: Find all @call sites for a given @fn
- **Hover Information**: Show @doc comments, function body preview, cell usage info on hover
- **Completion**: Auto-complete @fn names, @import paths, stdlib module/function names, @const names
- **Signature Help**: Show function documentation while typing @call
- **Rename**: Rename @fn across all files that @call it
- **Code Actions**: Quick fixes for common issues (add missing import, fix bracket)
- **Document Symbols**: Outline view showing all @fn definitions, @import statements, @const declarations
- **Workspace Symbols**: Search across all project files
- **Folding Ranges**: Fold @fn bodies, loop blocks
- **Semantic Tokens**: Rich syntax highlighting with semantic meaning

### Incremental Parsing for LSP
Parse only changed regions of files for fast feedback. Maintain a parse tree that can be incrementally updated. Use tree-sitter or a custom incremental parser.

---

## VS Code Extension

### Dedicated VS Code Extension
Package the LSP server with a VS Code extension. Include:
- Syntax highlighting (TextMate grammar for .bf and .bfn files)
- Bracket matching and rainbow brackets for [ ]
- Snippet support (common patterns, stdlib imports)
- Task integration (run, build, test from VS Code tasks)
- Debug Adapter Protocol (DAP) integration for the ogre debugger
- Problem matcher for ogre analyse output
- Test explorer integration for ogre test suites
- Tape visualization panel (webview showing current tape state)
- Inline cell value display (like inline type hints but showing cell values from last run)

### Debug Adapter Protocol (DAP)
Implement DAP so the ogre debugger works natively in VS Code, Neovim (nvim-dap), and other DAP-supporting editors. Support breakpoints, stepping, variable inspection (tape cells), and watch expressions.

---

## Editor-Specific Integrations

### Neovim Plugin
Lua-based Neovim plugin with:
- Tree-sitter grammar for Brainfunct syntax highlighting
- Telescope picker for stdlib functions
- Integration with nvim-dap for debugging
- Custom keymaps for common ogre commands

### Emacs Mode
Major mode for Brainfunct with:
- Syntax highlighting and indentation
- Flycheck integration for real-time diagnostics
- Company-mode completion backend
- Integration with dap-mode for debugging

### Helix/Zed Support
Tree-sitter grammar and query files for next-gen editors. These editors use tree-sitter natively, so a grammar file enables syntax highlighting, text objects, and indentation automatically.

---

## Editor-Agnostic Features

### Format on Save Hook
Provide a standardized way to run `ogre format` on save. Support editor hooks, file watchers, or integration with tools like `lefthook` or `husky`.

### Inline Diagnostics Format
Output diagnostics in a standard format (like GCC/Clang errors) that editors can parse. Support `--error-format=json` for machine-readable output and `--error-format=human` for pretty terminal output.

### Project-Aware File Detection
Automatically detect ogre projects (by ogre.toml presence) and configure editor settings accordingly. Set correct file associations, enable relevant linting, configure build tasks.
