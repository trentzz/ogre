use anyhow::Result;
use std::io::{self, BufRead, Write};

use super::interpreter::Interpreter;

pub struct StartRepl {
    interp: Interpreter,
}

impl StartRepl {
    pub fn new() -> Result<Self> {
        Ok(Self {
            interp: Interpreter::new("")?,
        })
    }

    fn print_memory(&self) {
        let dp = self.interp.data_pointer();
        let start = dp.saturating_sub(3);
        let end = (dp + 4).min(self.interp.tape().len());
        let cells: Vec<String> = (start..end)
            .map(|i| {
                if i == dp {
                    format!(">{}:{}<", i, self.interp.tape_value(i))
                } else {
                    format!("{}:{}", i, self.interp.tape_value(i))
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
                    self.interp = Interpreter::new("")?;
                    println!("Tape reset.");
                    continue;
                }
                "" => continue,
                code => {
                    match self.interp.feed(code) {
                        Err(e) => {
                            println!("Parse error: {}", e);
                            continue;
                        }
                        Ok(()) => match self.interp.run() {
                            Err(e) => println!("Runtime error: {}", e),
                            Ok(()) => {
                                if !self.interp.output().is_empty() {
                                    print!("{}", String::from_utf8_lossy(self.interp.output()));
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
        Ok(())
    }
}

pub fn start_repl() -> Result<()> {
    let mut repl = StartRepl::new()?;
    repl.run()
}
