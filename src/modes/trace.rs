use anyhow::Result;
use std::path::Path;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;

/// Trace execution of a brainfuck program, printing tape state after each instruction
/// (or every N instructions).
pub fn trace_file(path: &Path, tape_size: usize, every_n: usize) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    trace_source(&expanded, tape_size, every_n)
}

/// Trace execution from a source string.
pub fn trace_source(source: &str, tape_size: usize, every_n: usize) -> Result<()> {
    let mut interp = Interpreter::with_tape_size(source, tape_size)?;
    interp.set_streaming(true);

    let every = every_n.max(1);
    let mut step_count: u64 = 0;

    while !interp.is_done() {
        let ip = interp.code_pointer();
        let desc = interp.op_description(ip);

        interp.step()?;
        step_count += 1;

        if step_count.is_multiple_of(every as u64) {
            let dp = interp.data_pointer();
            let cell_val = interp.tape_value(dp);
            print_trace_line(step_count, &desc, dp, cell_val, interp.tape(), dp);
        }
    }

    println!("\nTrace complete: {} instructions executed", step_count);
    Ok(())
}

fn print_trace_line(step: u64, op: &str, dp: usize, cell_val: u8, tape: &[u8], center: usize) {
    // Show a window of cells around the data pointer
    let start = center.saturating_sub(4);
    let end = (center + 5).min(tape.len());

    let mut cells = String::new();
    cells.push('[');
    for (i, &val) in tape.iter().enumerate().take(end).skip(start) {
        if i > start {
            cells.push(' ');
        }
        if i == center {
            cells.push_str(&format!("*{}", val));
        } else {
            cells.push_str(&format!("{}", val));
        }
    }
    cells.push(']');

    println!(
        "step={:<6} op={:<20} dp={:<5} cell[{}]={:<3} | {}",
        step, op, dp, dp, cell_val, cells
    );
}

/// Format a trace line as a string (for testing).
pub fn format_trace_line(step: u64, op: &str, dp: usize, cell_val: u8) -> String {
    format!(
        "step={:<6} op={:<20} dp={:<5} cell[{}]={:<3}",
        step, op, dp, dp, cell_val
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_trace_line() {
        let line = format_trace_line(1, "Add(3)", 0, 3);
        assert!(line.contains("step=1"));
        assert!(line.contains("Add(3)"));
        assert!(line.contains("dp=0"));
        assert!(line.contains("cell[0]=3"));
    }

    #[test]
    fn test_trace_source_runs() {
        // Just verify it doesn't panic/error
        let result = trace_source("+++", 100, 1);
        assert!(result.is_ok());
    }
}
