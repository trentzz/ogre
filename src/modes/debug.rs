use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;
use super::source_map::{build_op_to_char_map, SourceMap};

/// A conditional breakpoint: breaks when a condition on a cell is met.
#[derive(Debug, Clone)]
struct ConditionalBreakpoint {
    op_index: usize,
    cell: usize,
    condition: BreakCondition,
}

#[derive(Debug, Clone)]
enum BreakCondition {
    Equals(u8),
    NotEquals(u8),
    GreaterThan(u8),
    LessThan(u8),
}

impl BreakCondition {
    fn matches(&self, value: u8) -> bool {
        match self {
            BreakCondition::Equals(v) => value == *v,
            BreakCondition::NotEquals(v) => value != *v,
            BreakCondition::GreaterThan(v) => value > *v,
            BreakCondition::LessThan(v) => value < *v,
        }
    }

    fn display(&self) -> String {
        match self {
            BreakCondition::Equals(v) => format!("== {}", v),
            BreakCondition::NotEquals(v) => format!("!= {}", v),
            BreakCondition::GreaterThan(v) => format!("> {}", v),
            BreakCondition::LessThan(v) => format!("< {}", v),
        }
    }
}

/// A watchpoint: breaks when a cell's value changes.
#[derive(Debug, Clone)]
struct Watchpoint {
    cell: usize,
    last_value: u8,
}

pub struct Debugger {
    interp: Interpreter,
    breakpoints: HashSet<usize>,
    conditional_breakpoints: Vec<ConditionalBreakpoint>,
    watchpoints: Vec<Watchpoint>,
    /// Optional source map for showing original file/line/function context.
    source_map: Option<SourceMap>,
    /// Maps IR op indices to character positions in expanded source (for source map lookup).
    op_to_char: Vec<usize>,
}

impl Debugger {
    pub fn new_live(source: &str) -> Result<Self> {
        let op_to_char = build_op_to_char_map(source);
        Ok(Self {
            interp: Interpreter::with_live_stdin(source)?,
            breakpoints: HashSet::new(),
            conditional_breakpoints: Vec::new(),
            watchpoints: Vec::new(),
            source_map: None,
            op_to_char,
        })
    }

    pub fn new_live_with_tape_size(source: &str, tape_size: usize) -> Result<Self> {
        let op_to_char = build_op_to_char_map(source);
        Ok(Self {
            interp: Interpreter::with_live_stdin_and_tape_size(source, tape_size)?,
            breakpoints: HashSet::new(),
            conditional_breakpoints: Vec::new(),
            watchpoints: Vec::new(),
            source_map: None,
            op_to_char,
        })
    }

    pub fn new_live_with_source_map(
        source: &str,
        tape_size: usize,
        source_map: SourceMap,
    ) -> Result<Self> {
        let op_to_char = build_op_to_char_map(source);
        Ok(Self {
            interp: Interpreter::with_live_stdin_and_tape_size(source, tape_size)?,
            breakpoints: HashSet::new(),
            conditional_breakpoints: Vec::new(),
            watchpoints: Vec::new(),
            source_map: Some(source_map),
            op_to_char,
        })
    }

    /// Get the source location string for an op index, if source map is available.
    fn source_location_str(&self, op_idx: usize) -> Option<String> {
        self.source_map
            .as_ref()
            .and_then(|sm| sm.lookup_op(op_idx, &self.op_to_char))
            .map(|loc| loc.display_short())
    }

    fn print_status(&self) {
        let ip = self.interp.code_pointer();
        let desc = self.interp.op_description(ip);
        let dp = self.interp.data_pointer();
        let val = self.interp.tape_value(dp);

        // Show source location if available
        match self.source_location_str(ip) {
            Some(loc) => {
                println!(
                    "  ip={}  op={}  dp={}  val={}  {}",
                    ip,
                    desc.yellow().bold(),
                    dp,
                    val,
                    loc.dimmed()
                );
            }
            None => {
                println!(
                    "  ip={}  op={}  dp={}  val={}",
                    ip,
                    desc.yellow().bold(),
                    dp,
                    val
                );
            }
        }

        // Short memory window
        let window = self.interp.peek_window(dp, 3);
        let cells: Vec<String> = window
            .iter()
            .map(|(addr, v, is_ptr)| {
                if *is_ptr {
                    format!("{}", format!(">{}:{}<", addr, v).cyan().bold())
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
        if self.source_map.is_some() {
            println!("  (source map loaded — showing file/line/function info)");
        }
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
                    println!("  step [n]                     Execute 1 or n instructions");
                    println!("  continue / c                 Run until breakpoint or end");
                    println!("  breakpoint <n>               Set breakpoint at op index n");
                    println!("  breakpoint list              List all breakpoints");
                    println!("  breakpoint delete <n>        Remove breakpoint n");
                    println!("  cbreak <op> <cell> <cond> <val>  Conditional breakpoint");
                    println!("    conditions: eq, ne, gt, lt");
                    println!("  cbreak list                  List conditional breakpoints");
                    println!("  cbreak delete <n>            Remove conditional breakpoint n");
                    println!("  watch <cell>                 Watch cell for value changes");
                    println!("  watch list                   List watchpoints");
                    println!("  watch delete <n>             Remove watchpoint n");
                    println!("  jump <n>                     Move code pointer to n");
                    println!("  peek [n]                     Show memory around ptr (or cell n)");
                    println!("  show instruction [n]         Show current (or nth) instruction");
                    println!("  show memory                  Dump memory cells");
                    println!("  where                        Show current source location");
                    println!("  exit / quit / q              Quit debugger");
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
                    let desc = self.interp.op_description(idx);
                    println!("{} set at op {} ({})", "Breakpoint".red().bold(), idx, desc);
                }
                ["breakpoint", "list"] => {
                    if self.breakpoints.is_empty() {
                        println!("No breakpoints set.");
                    } else {
                        let mut bps: Vec<usize> = self.breakpoints.iter().copied().collect();
                        bps.sort_unstable();
                        for bp in bps {
                            let desc = self.interp.op_description(bp);
                            let loc = self.source_location_str(bp).unwrap_or_default();
                            if loc.is_empty() {
                                println!("  {} {} → {}", "breakpoint".red(), bp, desc);
                            } else {
                                println!(
                                    "  {} {} → {}  {}",
                                    "breakpoint".red(),
                                    bp,
                                    desc,
                                    loc.dimmed()
                                );
                            }
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
                        println!("Jumped to op {}.", idx);
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
                ["where"] => {
                    let ip = self.interp.code_pointer();
                    match self.source_location_str(ip) {
                        Some(loc) => println!("  {} → {}", ip, loc),
                        None => println!("  ip={} (no source map)", ip),
                    }
                }
                ["cbreak", op, cell, cond, val] => {
                    let op_idx: usize = match op.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            println!("Invalid op index.");
                            continue;
                        }
                    };
                    let cell_idx: usize = match cell.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            println!("Invalid cell index.");
                            continue;
                        }
                    };
                    let value: u8 = match val.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            println!("Invalid value (0-255).");
                            continue;
                        }
                    };
                    let condition = match *cond {
                        "eq" => BreakCondition::Equals(value),
                        "ne" => BreakCondition::NotEquals(value),
                        "gt" => BreakCondition::GreaterThan(value),
                        "lt" => BreakCondition::LessThan(value),
                        _ => {
                            println!("Unknown condition. Use: eq, ne, gt, lt");
                            continue;
                        }
                    };
                    let idx = self.conditional_breakpoints.len();
                    println!(
                        "{} #{} set at op {} when cell[{}] {}",
                        "Conditional breakpoint".red().bold(),
                        idx,
                        op_idx,
                        cell_idx,
                        condition.display()
                    );
                    self.conditional_breakpoints.push(ConditionalBreakpoint {
                        op_index: op_idx,
                        cell: cell_idx,
                        condition,
                    });
                }
                ["cbreak", "list"] => {
                    if self.conditional_breakpoints.is_empty() {
                        println!("No conditional breakpoints set.");
                    } else {
                        for (i, cb) in self.conditional_breakpoints.iter().enumerate() {
                            println!(
                                "  #{}: op {} when cell[{}] {}",
                                i,
                                cb.op_index,
                                cb.cell,
                                cb.condition.display()
                            );
                        }
                    }
                }
                ["cbreak", "delete", n] => {
                    let idx: usize = n.parse().unwrap_or(usize::MAX);
                    if idx < self.conditional_breakpoints.len() {
                        self.conditional_breakpoints.remove(idx);
                        println!("Conditional breakpoint #{} removed.", idx);
                    } else {
                        println!("No conditional breakpoint #{}.", idx);
                    }
                }
                ["watch", cell] if cell.chars().all(|c| c.is_ascii_digit()) => {
                    let cell_idx: usize = cell.parse().unwrap();
                    let current_val = self.interp.tape_value(cell_idx);
                    let idx = self.watchpoints.len();
                    self.watchpoints.push(Watchpoint {
                        cell: cell_idx,
                        last_value: current_val,
                    });
                    println!(
                        "{} #{} on cell[{}] (current value: {})",
                        "Watchpoint".red().bold(),
                        idx,
                        cell_idx,
                        current_val
                    );
                }
                ["watch", "list"] => {
                    if self.watchpoints.is_empty() {
                        println!("No watchpoints set.");
                    } else {
                        for (i, wp) in self.watchpoints.iter().enumerate() {
                            let current = self.interp.tape_value(wp.cell);
                            println!(
                                "  #{}: cell[{}] last={} current={}",
                                i, wp.cell, wp.last_value, current
                            );
                        }
                    }
                }
                ["watch", "delete", n] => {
                    let idx: usize = n.parse().unwrap_or(usize::MAX);
                    if idx < self.watchpoints.len() {
                        self.watchpoints.remove(idx);
                        println!("Watchpoint #{} removed.", idx);
                    } else {
                        println!("No watchpoint #{}.", idx);
                    }
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
            let ip = self.interp.code_pointer();

            // Check simple breakpoints
            if self.breakpoints.contains(&ip) {
                println!("{} at {}.", "Hit breakpoint".red().bold(), ip);
                self.print_status();
                break;
            }

            // Check conditional breakpoints
            let mut cbreak_hit = false;
            for cb in &self.conditional_breakpoints {
                if cb.op_index == ip {
                    let val = self.interp.tape_value(cb.cell);
                    if cb.condition.matches(val) {
                        println!(
                            "{} at op {} (cell[{}]={} {})",
                            "Hit conditional breakpoint".red().bold(),
                            ip,
                            cb.cell,
                            val,
                            cb.condition.display()
                        );
                        cbreak_hit = true;
                        break;
                    }
                }
            }
            if cbreak_hit {
                self.print_status();
                break;
            }

            self.interp.step()?;
            self.flush_output();

            // Check watchpoints after execution
            let mut watch_hit = false;
            for wp in &mut self.watchpoints {
                let current = self.interp.tape_value(wp.cell);
                if current != wp.last_value {
                    println!(
                        "{} cell[{}] changed: {} → {}",
                        "Watchpoint triggered".red().bold(),
                        wp.cell,
                        wp.last_value,
                        current
                    );
                    wp.last_value = current;
                    watch_hit = true;
                }
            }
            if watch_hit {
                self.print_status();
                break;
            }
        }
        Ok(())
    }

    fn print_window(&self, window: &[(usize, u8, bool)]) {
        let cells: Vec<String> = window
            .iter()
            .map(|(addr, v, is_ptr)| {
                if *is_ptr {
                    format!("{}", format!(">{}:{}<", addr, v).cyan().bold())
                } else {
                    format!("{}:{}", addr, v)
                }
            })
            .collect();
        println!("  tape: [ {} ]", cells.join("  "));
    }

    fn show_instruction(&self, idx: usize) {
        if idx >= self.interp.code_len() {
            println!("{}", format!("Index {} out of range.", idx).red());
            return;
        }
        let desc = self.interp.op_description(idx);
        // Show context: surrounding ops
        let start = idx.saturating_sub(3);
        let end = (idx + 4).min(self.interp.code_len());
        let context: Vec<String> = (start..end)
            .map(|i| {
                let d = self.interp.op_description(i);
                if i == idx {
                    format!("{}", format!("[{}]", d).yellow().bold())
                } else {
                    d
                }
            })
            .collect();

        // Show source location if available
        let loc_str = self
            .source_location_str(idx)
            .map(|l| format!("  {}", l.dimmed()))
            .unwrap_or_default();

        println!(
            "  op {}: {} (context: {}){}",
            idx,
            desc.yellow().bold(),
            context.join(" "),
            loc_str
        );
    }
}

pub fn debug_file(path: &Path) -> Result<()> {
    debug_file_with_tape_size(path, super::interpreter::DEFAULT_TAPE_SIZE)
}

pub fn debug_file_with_tape_size(path: &Path, tape_size: usize) -> Result<()> {
    // Use source-map-aware preprocessing
    let (expanded, source_map) = Preprocessor::process_file_with_map(path)?;
    let mut dbg = Debugger::new_live_with_source_map(&expanded, tape_size, source_map)?;
    dbg.run_repl()
}

/// Debug a file with pre-loaded dependency functions.
pub fn debug_file_with_deps(
    path: &Path,
    tape_size: usize,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<()> {
    // Load deps into preprocessor before processing
    let expanded = Preprocessor::process_file_with_deps(path, dep_functions)?;
    let mut dbg = Debugger::new_live_with_tape_size(&expanded, tape_size)?;
    dbg.run_repl()
}
