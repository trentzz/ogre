use anyhow::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;

const HISTORY_FILE: &str = ".ogre_history";

/// Get the history file path (~/.ogre_history).
fn history_path() -> Option<std::path::PathBuf> {
    dirs_path().map(|p| p.join(HISTORY_FILE))
}

fn dirs_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}

pub struct StartRepl {
    interp: Interpreter,
    tape_size: usize,
    functions: HashMap<String, String>,
}

impl StartRepl {
    pub fn new() -> Result<Self> {
        Self::with_tape_size(super::interpreter::DEFAULT_TAPE_SIZE)
    }

    pub fn with_tape_size(tape_size: usize) -> Result<Self> {
        Ok(Self {
            interp: Interpreter::with_tape_size("", tape_size)?,
            tape_size,
            functions: HashMap::new(),
        })
    }

    /// Load all @fn definitions from a file into the REPL environment.
    pub fn load_functions_from_file(&mut self, path: &Path) -> Result<usize> {
        let fns = Preprocessor::collect_functions_from_file(path)?;
        let count = fns.len();
        self.functions.extend(fns);
        Ok(count)
    }

    /// Load functions from all project include files and entry.
    pub fn load_project_functions(
        &mut self,
        project: &crate::project::OgreProject,
        base: &std::path::Path,
    ) -> Result<usize> {
        let entry = project.entry_path(base);
        let mut total = 0;

        // Load from entry file
        if entry.exists() {
            total += self.load_functions_from_file(&entry)?;
        }

        // Load from include files
        let files = project.resolve_include_files(base)?;
        for f in &files {
            if f != &entry && f.exists() {
                let fns = Preprocessor::collect_functions_from_file(f)?;
                total += fns.len();
                self.functions.extend(fns);
            }
        }

        Ok(total)
    }

    fn print_memory(&self) {
        let dp = self.interp.data_pointer();
        let start = dp.saturating_sub(3);
        let end = (dp + 4).min(self.interp.tape().len());
        let cells: Vec<String> = (start..end)
            .map(|i| {
                if i == dp {
                    format!(
                        "{}",
                        format!(">{}:{}<", i, self.interp.tape_value(i))
                            .cyan()
                            .bold()
                    )
                } else {
                    format!("{}:{}", i, self.interp.tape_value(i))
                }
            })
            .collect();
        println!("  tape: [ {} ]", cells.join("  "));
    }

    fn print_help() {
        println!("{}", "ogre REPL commands:".bold());
        println!("  {}      — Reset the tape and interpreter", ":reset".cyan());
        println!(
            "  {} — Load and run a brainfuck file",
            ":load <file>".cyan()
        );
        println!(
            "  {} — Save tape state info to a file",
            ":save <file>".cyan()
        );
        println!(
            "  {}  — Show loaded @fn definitions",
            ":functions".cyan()
        );
        println!(
            "  {}       — Show memory around pointer",
            ":peek".cyan()
        );
        println!(
            "  {} — Show tape cells from start to end",
            ":dump [n]".cyan()
        );
        println!("  {}       — Show this help message", ":help".cyan());
        println!(
            "  {}       — Quit the REPL",
            ":quit / :exit".cyan()
        );
        println!();
        println!("  Type any brainfuck code to execute it.");
        println!(
            "  @call, @fn, @const, @use, and @import directives are supported."
        );
    }

    /// Preprocess REPL input, expanding @call/@use directives against loaded functions.
    fn preprocess_input(&mut self, code: &str) -> Result<String> {
        if !code.contains('@') {
            return Ok(code.to_string());
        }
        // Parse any new @fn/@const/@import definitions + expand @call/@use
        Preprocessor::expand_with_functions(code, &self.functions)
    }

    pub fn run(&mut self) -> Result<()> {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();

        let mut rl: Editor<(), rustyline::history::FileHistory> =
            Editor::with_config(config)?;

        // Load history
        if let Some(ref hist) = history_path() {
            let _ = rl.load_history(hist);
        }

        println!(
            "{}",
            "ogre interactive interpreter".bold()
        );
        if !self.functions.is_empty() {
            println!(
                "  {} @fn definition(s) loaded",
                self.functions.len().to_string().green()
            );
        }
        println!(
            "  Type {} for commands, or enter brainfuck code.",
            ":help".cyan()
        );
        println!();

        loop {
            match rl.readline(">>> ") {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    rl.add_history_entry(&line)?;

                    match trimmed {
                        ":exit" | ":quit" | "exit" | "quit" => {
                            println!("Goodbye.");
                            break;
                        }
                        ":reset" | "reset" => {
                            self.interp =
                                Interpreter::with_tape_size("", self.tape_size)?;
                            println!("Tape reset.");
                            continue;
                        }
                        ":help" | "help" => {
                            Self::print_help();
                            continue;
                        }
                        ":functions" => {
                            if self.functions.is_empty() {
                                println!("No functions loaded.");
                            } else {
                                let mut names: Vec<&String> =
                                    self.functions.keys().collect();
                                names.sort();
                                println!(
                                    "{} function(s) available:",
                                    names.len()
                                );
                                for name in names {
                                    let body = &self.functions[name];
                                    let preview: String =
                                        body.chars().take(40).collect();
                                    let preview = preview.trim().replace('\n', " ");
                                    if body.len() > 40 {
                                        println!(
                                            "  {} — {}...",
                                            name.green(),
                                            preview
                                        );
                                    } else {
                                        println!(
                                            "  {} — {}",
                                            name.green(),
                                            preview
                                        );
                                    }
                                }
                            }
                            continue;
                        }
                        ":peek" => {
                            self.print_memory();
                            continue;
                        }
                        s if s.starts_with(":dump") => {
                            let n: usize = s
                                .strip_prefix(":dump")
                                .and_then(|rest: &str| rest.trim().parse().ok())
                                .unwrap_or(20);
                            let tape = self.interp.tape();
                            let len = n.min(tape.len());
                            let dp = self.interp.data_pointer();
                            for (i, &cell) in tape.iter().enumerate().take(len) {
                                if i == dp {
                                    print!(
                                        " {}",
                                        format!("[{}]", cell)
                                            .cyan()
                                            .bold()
                                    );
                                } else if cell != 0 {
                                    print!(" {}", cell);
                                } else {
                                    print!(" 0");
                                }
                            }
                            println!();
                            continue;
                        }
                        s if s.starts_with(":load ") => {
                            let file_path = s.strip_prefix(":load ").unwrap().trim();
                            if file_path.is_empty() {
                                println!(
                                    "{} usage: :load <file>",
                                    "Error:".red()
                                );
                                continue;
                            }
                            let path = Path::new(file_path);
                            if !path.exists() {
                                println!(
                                    "{} file not found: {}",
                                    "Error:".red(),
                                    file_path
                                );
                                continue;
                            }
                            match Preprocessor::process_file(path) {
                                Ok(expanded) => {
                                    // Also load any function definitions
                                    if let Ok(fns) =
                                        Preprocessor::collect_functions_from_file(path)
                                    {
                                        let fn_count = fns.len();
                                        self.functions.extend(fns);
                                        if fn_count > 0 {
                                            println!(
                                                "Loaded {} function(s).",
                                                fn_count
                                            );
                                        }
                                    }
                                    // Run the expanded code
                                    match self.interp.feed(&expanded) {
                                        Err(e) => {
                                            println!(
                                                "{} {}",
                                                "Parse error:".red(),
                                                e
                                            );
                                            continue;
                                        }
                                        Ok(()) => match self.interp.run() {
                                            Err(e) => println!(
                                                "{} {}",
                                                "Runtime error:".red(),
                                                e
                                            ),
                                            Ok(()) => {
                                                if !self.interp.output().is_empty() {
                                                    print!(
                                                        "{}",
                                                        String::from_utf8_lossy(
                                                            self.interp.output()
                                                        )
                                                    );
                                                    io::stdout().flush()?;
                                                    self.interp.clear_output();
                                                }
                                                println!(
                                                    "{}",
                                                    "Loaded and executed."
                                                        .green()
                                                );
                                            }
                                        },
                                    }
                                    self.print_memory();
                                }
                                Err(e) => {
                                    println!(
                                        "{} {}",
                                        "Preprocess error:".red(),
                                        e
                                    );
                                }
                            }
                            continue;
                        }
                        s if s.starts_with(":save ") => {
                            let file_path = s.strip_prefix(":save ").unwrap().trim();
                            if file_path.is_empty() {
                                println!(
                                    "{} usage: :save <file>",
                                    "Error:".red()
                                );
                                continue;
                            }
                            let dp = self.interp.data_pointer();
                            let tape = self.interp.tape();
                            // Find last non-zero cell
                            let last_nonzero = tape
                                .iter()
                                .rposition(|&c| c != 0)
                                .map(|i| i + 1)
                                .unwrap_or(0);
                            let mut info = String::new();
                            info.push_str("# ogre REPL tape state\n");
                            info.push_str(&format!(
                                "data_pointer: {}\n",
                                dp
                            ));
                            info.push_str(&format!(
                                "tape_size: {}\n",
                                tape.len()
                            ));
                            info.push_str(&format!(
                                "cells_used: {}\n",
                                last_nonzero
                            ));
                            info.push_str("tape: [");
                            let slice = &tape[..last_nonzero.max(dp + 1).min(tape.len())];
                            let vals: Vec<String> =
                                slice.iter().map(|v| v.to_string()).collect();
                            info.push_str(&vals.join(", "));
                            info.push_str("]\n");
                            match std::fs::write(file_path, &info) {
                                Ok(()) => println!(
                                    "Tape state saved to: {}",
                                    file_path
                                ),
                                Err(e) => println!(
                                    "{} {}",
                                    "Error:".red(),
                                    e
                                ),
                            }
                            continue;
                        }
                        s if s.starts_with(':') => {
                            println!(
                                "{} unknown command. Type {} for help.",
                                "Error:".red(),
                                ":help".cyan()
                            );
                            continue;
                        }
                        code => {
                            // Preprocess the input (expand @call/@use/@fn)
                            let expanded = match self.preprocess_input(code) {
                                Ok(e) => e,
                                Err(e) => {
                                    println!(
                                        "{} {}",
                                        "Preprocess error:".red(),
                                        e
                                    );
                                    continue;
                                }
                            };

                            match self.interp.feed(&expanded) {
                                Err(e) => {
                                    println!(
                                        "{} {}",
                                        "Parse error:".red(),
                                        e
                                    );
                                    continue;
                                }
                                Ok(()) => match self.interp.run() {
                                    Err(e) => println!(
                                        "{} {}",
                                        "Runtime error:".red(),
                                        e
                                    ),
                                    Ok(()) => {
                                        if !self.interp.output().is_empty() {
                                            print!(
                                                "{}",
                                                String::from_utf8_lossy(
                                                    self.interp.output()
                                                )
                                            );
                                            io::stdout().flush()?;
                                            self.interp.clear_output();
                                        }
                                    }
                                },
                            }
                            self.print_memory();
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye.");
                    break;
                }
                Err(err) => {
                    println!("{} {:?}", "Readline error:".red(), err);
                    break;
                }
            }
        }

        // Save history
        if let Some(ref hist) = history_path() {
            let _ = rl.save_history(hist);
        }

        Ok(())
    }
}

pub fn start_repl() -> Result<()> {
    let mut repl = StartRepl::new()?;
    repl.run()
}

pub fn start_repl_with_tape_size(tape_size: usize) -> Result<()> {
    let mut repl = StartRepl::with_tape_size(tape_size)?;
    repl.run()
}

/// Start REPL with project functions preloaded (including dependency functions).
pub fn start_repl_project(
    tape_size: usize,
    project: &crate::project::OgreProject,
    base: &std::path::Path,
) -> Result<()> {
    let mut repl = StartRepl::with_tape_size(tape_size)?;
    let count = repl.load_project_functions(project, base)?;
    // Also load functions from dependencies
    let dep_fns = project.collect_dependency_functions(base)?;
    let dep_count = dep_fns.len();
    repl.functions.extend(dep_fns);
    if count > 0 || dep_count > 0 {
        // Functions will be shown in the welcome message
    }
    repl.run()
}
