use anyhow::Result;
use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;

pub struct Debugger {
    interp: Interpreter,
    breakpoints: HashSet<usize>,
}

impl Debugger {
    pub fn new_live(source: &str) -> Result<Self> {
        Ok(Self {
            interp: Interpreter::with_live_stdin(source)?,
            breakpoints: HashSet::new(),
        })
    }

    fn print_status(&self) {
        let ip = self.interp.code_pointer();
        let op = if ip < self.interp.code_len() {
            self.interp.code_char(ip)
        } else {
            '∎'
        };
        let dp = self.interp.data_pointer();
        let val = self.interp.tape_value(dp);
        println!("  ip={} op='{}'  dp={}  val={}", ip, op, dp, val);

        // Short memory window
        let window = self.interp.peek_window(dp, 3);
        let cells: Vec<String> = window
            .iter()
            .map(|(addr, v, is_ptr)| {
                if *is_ptr {
                    format!(">{}:{}<", addr, v)
                } else {
                    format!("{}:{}", addr, v)
                }
            })
            .collect();
        println!("  tape: [ {} ]", cells.join("  "));
    }

    fn flush_output(&mut self) {
        if !self.interp.output().is_empty() {
            print!("{}", String::from_utf8_lossy(self.interp.output()));
            let _ = io::stdout().flush();
            self.interp.clear_output();
        }
    }

    pub fn run_repl(&mut self) -> Result<()> {
        println!("ogre debugger — type 'help' for commands");
        self.print_status();

        let stdin = io::stdin();
        loop {
            print!("(ogre-dbg) ");
            io::stdout().flush()?;

            let mut line = String::new();
            if stdin.lock().read_line(&mut line)? == 0 {
                break; // EOF
            }
            let line = line.trim();
            let tokens: Vec<&str> = line.split_whitespace().collect();

            if tokens.is_empty() {
                continue;
            }

            match tokens.as_slice() {
                ["exit"] | ["quit"] | ["q"] => {
                    println!("Exiting debugger.");
                    break;
                }
                ["help"] => {
                    println!("Commands:");
                    println!("  step [n]              Execute 1 or n instructions");
                    println!("  continue / c          Run until breakpoint or end");
                    println!("  breakpoint <n>        Set breakpoint at instruction index n");
                    println!("  breakpoint list       List all breakpoints");
                    println!("  breakpoint delete <n> Remove breakpoint n");
                    println!("  jump <n>              Move code pointer to n (no execution)");
                    println!("  peek [n]              Show memory around ptr (or cell n)");
                    println!("  show instruction [n]  Show current (or nth) instruction");
                    println!("  show memory           Dump memory cells");
                    println!("  exit / quit / q       Quit debugger");
                }
                ["step"] => {
                    self.do_step(1)?;
                }
                ["step", n] => {
                    let count: usize = n.parse().unwrap_or(1);
                    self.do_step(count)?;
                }
                ["continue"] | ["c"] => {
                    self.do_continue()?;
                }
                ["breakpoint", n] if n.chars().all(|c| c.is_ascii_digit()) => {
                    let idx: usize = n.parse().unwrap();
                    self.breakpoints.insert(idx);
                    println!("Breakpoint set at instruction {}", idx);
                }
                ["breakpoint", "list"] => {
                    if self.breakpoints.is_empty() {
                        println!("No breakpoints set.");
                    } else {
                        let mut bps: Vec<usize> = self.breakpoints.iter().copied().collect();
                        bps.sort_unstable();
                        for bp in bps {
                            let op = if bp < self.interp.code_len() {
                                self.interp.code_char(bp)
                            } else {
                                '?'
                            };
                            println!("  breakpoint {} → '{}'", bp, op);
                        }
                    }
                }
                ["breakpoint", "delete", n] => {
                    let idx: usize = n.parse().unwrap_or(usize::MAX);
                    if self.breakpoints.remove(&idx) {
                        println!("Breakpoint {} removed.", idx);
                    } else {
                        println!("No breakpoint at {}.", idx);
                    }
                }
                ["jump", n] => {
                    let idx: usize = n.parse().unwrap_or(0);
                    if idx <= self.interp.code_len() {
                        self.interp.set_code_pointer(idx);
                        println!("Jumped to instruction {}.", idx);
                        self.print_status();
                    } else {
                        println!(
                            "Index {} out of range (max {}).",
                            idx,
                            self.interp.code_len()
                        );
                    }
                }
                ["peek"] => {
                    let window = self.interp.peek_window(self.interp.data_pointer(), 5);
                    self.print_window(&window);
                }
                ["peek", n] => {
                    let center: usize = n.parse().unwrap_or(self.interp.data_pointer());
                    let window = self.interp.peek_window(center, 5);
                    self.print_window(&window);
                }
                ["show", "instruction"] => {
                    self.show_instruction(self.interp.code_pointer());
                }
                ["show", "instruction", n] => {
                    let idx: usize = n.parse().unwrap_or(self.interp.code_pointer());
                    self.show_instruction(idx);
                }
                ["show", "memory"] => {
                    let window = self.interp.peek_window(self.interp.data_pointer(), 10);
                    self.print_window(&window);
                }
                _ => {
                    println!("Unknown command: '{}'. Type 'help' for commands.", line);
                }
            }
        }
        Ok(())
    }

    fn do_step(&mut self, count: usize) -> Result<()> {
        for _ in 0..count {
            if self.interp.is_done() {
                println!("Program has ended.");
                break;
            }
            self.interp.step()?;
            self.flush_output();
        }
        if !self.interp.is_done() {
            self.print_status();
        } else {
            println!("Program finished.");
        }
        Ok(())
    }

    fn do_continue(&mut self) -> Result<()> {
        loop {
            if self.interp.is_done() {
                println!("Program finished.");
                break;
            }
            if self.breakpoints.contains(&self.interp.code_pointer()) {
                println!("Hit breakpoint at {}.", self.interp.code_pointer());
                self.print_status();
                break;
            }
            self.interp.step()?;
            self.flush_output();
        }
        Ok(())
    }

    fn print_window(&self, window: &[(usize, u8, bool)]) {
        let cells: Vec<String> = window
            .iter()
            .map(|(addr, v, is_ptr)| {
                if *is_ptr {
                    format!(">{}:{}<", addr, v)
                } else {
                    format!("{}:{}", addr, v)
                }
            })
            .collect();
        println!("  tape: [ {} ]", cells.join("  "));
    }

    fn show_instruction(&self, idx: usize) {
        if idx >= self.interp.code_len() {
            println!("Index {} out of range.", idx);
            return;
        }
        let start = idx.saturating_sub(3);
        let end = (idx + 4).min(self.interp.code_len());
        let context: String = (start..end)
            .map(|i| {
                let c = self.interp.code_char(i);
                if i == idx {
                    format!("[{}]", c)
                } else {
                    c.to_string()
                }
            })
            .collect();
        println!(
            "  instruction {}: {} (context: {})",
            idx,
            self.interp.code_char(idx),
            context
        );
    }
}

pub fn debug_file(path: &Path) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let mut dbg = Debugger::new_live(&expanded)?;
    dbg.run_repl()
}
