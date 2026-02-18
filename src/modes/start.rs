use anyhow::Result;
use std::io::{self, BufRead, Write};

use super::interpreter::Interpreter;

pub struct StartRepl {
    tape: Vec<u8>,
    data_ptr: usize,
}

impl StartRepl {
    pub fn new() -> Self {
        Self {
            tape: vec![0u8; 30_000],
            data_ptr: 0,
        }
    }

    fn print_memory(&self) {
        let start = self.data_ptr.saturating_sub(3);
        let end = (self.data_ptr + 4).min(self.tape.len());
        let cells: Vec<String> = (start..end)
            .map(|i| {
                if i == self.data_ptr {
                    format!(">{}:{}<", i, self.tape[i])
                } else {
                    format!("{}:{}", i, self.tape[i])
                }
            })
            .collect();
        println!("  tape: [ {} ]", cells.join("  "));
    }

    pub fn run(&mut self) -> Result<()> {
        println!("ogre interactive interpreter — type BF code, 'reset' to clear, 'exit' to quit");

        let stdin = io::stdin();
        loop {
            print!(">>> ");
            io::stdout().flush()?;

            let mut line = String::new();
            if stdin.lock().read_line(&mut line)? == 0 {
                break;
            }
            let line = line.trim();

            match line {
                "exit" | "quit" => {
                    println!("Goodbye.");
                    break;
                }
                "reset" => {
                    self.tape = vec![0u8; 30_000];
                    self.data_ptr = 0;
                    println!("Tape reset.");
                    continue;
                }
                "" => continue,
                code => {
                    // Build a source that starts with an existing tape snapshot
                    // We run the snippet on the current tape state by constructing
                    // a fresh interpreter, copying our tape state in, and running.
                    match Interpreter::new(code) {
                        Err(e) => {
                            println!("Parse error: {}", e);
                            continue;
                        }
                        Ok(mut interp) => {
                            // Copy current tape state into interpreter
                            interp.tape = self.tape.clone();
                            interp.data_ptr = self.data_ptr;

                            match interp.run() {
                                Err(e) => println!("Runtime error: {}", e),
                                Ok(()) => {
                                    // Print any output
                                    if !interp.output.is_empty() {
                                        print!("{}", interp.output_as_string());
                                        io::stdout().flush()?;
                                    }
                                    // Save tape state back
                                    self.tape = interp.tape;
                                    self.data_ptr = interp.data_ptr;
                                }
                            }
                        }
                    }
                    self.print_memory();
                }
            }
        }
        Ok(())
    }
}

pub fn start_repl() -> Result<()> {
    let mut repl = StartRepl::new();
    repl.run()
}
