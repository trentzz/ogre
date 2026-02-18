use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, Write};

use super::interpreter::Interpreter;

pub struct Debugger {
    pub interp: Interpreter,
    pub breakpoints: HashSet<usize>,
}

impl Debugger {
    pub fn new(source: &str) -> Result<Self> {
        Ok(Self {
            interp: Interpreter::new(source)?,
            breakpoints: HashSet::new(),
        })
    }

    pub fn new_live(source: &str) -> Result<Self> {
        Ok(Self {
            interp: Interpreter::with_live_stdin(source)?,
            breakpoints: HashSet::new(),
        })
    }

    pub fn with_input(source: &str, input: &str) -> Result<Self> {
        Ok(Self {
            interp: Interpreter::with_input(source, input)?,
            breakpoints: HashSet::new(),
        })
    }

    fn print_status(&self) {
        let ip = self.interp.code_ptr;
        let op = if ip < self.interp.code.len() {
            self.interp.code[ip]
        } else {
            '∎'
        };
        let dp = self.interp.data_ptr;
        let val = self.interp.tape[dp];
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
        if !self.interp.output.is_empty() {
            print!("{}", String::from_utf8_lossy(&self.interp.output));
            let _ = io::stdout().flush();
            self.interp.output.clear();
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
                            let op = if bp < self.interp.code.len() {
                                self.interp.code[bp]
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
                    if idx <= self.interp.code.len() {
                        self.interp.code_ptr = idx;
                        println!("Jumped to instruction {}.", idx);
                        self.print_status();
                    } else {
                        println!("Index {} out of range (max {}).", idx, self.interp.code.len());
                    }
                }
                ["peek"] => {
                    let window = self.interp.peek_window(self.interp.data_ptr, 5);
                    self.print_window(&window);
                }
                ["peek", n] => {
                    let center: usize = n.parse().unwrap_or(self.interp.data_ptr);
                    let window = self.interp.peek_window(center, 5);
                    self.print_window(&window);
                }
                ["show", "instruction"] => {
                    self.show_instruction(self.interp.code_ptr);
                }
                ["show", "instruction", n] => {
                    let idx: usize = n.parse().unwrap_or(self.interp.code_ptr);
                    self.show_instruction(idx);
                }
                ["show", "memory"] => {
                    let window = self.interp.peek_window(self.interp.data_ptr, 10);
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
            if self.breakpoints.contains(&self.interp.code_ptr) {
                println!("Hit breakpoint at {}.", self.interp.code_ptr);
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
        if idx >= self.interp.code.len() {
            println!("Index {} out of range.", idx);
            return;
        }
        let start = idx.saturating_sub(3);
        let end = (idx + 4).min(self.interp.code.len());
        let context: String = self.interp.code[start..end]
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                if start + i == idx {
                    format!("[{}]", c)
                } else {
                    c.to_string()
                }
            })
            .collect();
        println!("  instruction {}: {} (context: {})", idx, self.interp.code[idx], context);
    }
}

pub fn debug_file(path: &str) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let mut dbg = Debugger::new_live(&source)?;
    dbg.run_repl()
}
