use anyhow::Result;
use std::fs;
use std::path::Path;

use super::preprocess::Preprocessor;

pub struct AnalysisReport {
    pub bracket_errors: Vec<String>,
    pub total_inputs: usize,
    pub total_outputs: usize,
    pub ptr_end_offset: Option<i64>, // None if loops prevent static analysis
}

pub fn analyse_source(code: &str) -> AnalysisReport {
    let mut bracket_errors: Vec<String> = Vec::new();
    let mut total_inputs = 0usize;
    let mut total_outputs = 0usize;
    let mut ptr_offset: i64 = 0;
    let mut ptr_indeterminate = false;

    // Validate brackets
    let mut stack: Vec<usize> = Vec::new();
    for (i, ch) in code.chars().enumerate() {
        match ch {
            '[' => stack.push(i),
            ']' => {
                if stack.pop().is_none() {
                    bracket_errors.push(format!("unmatched `]` at position {}", i));
                }
                // Any loop makes pointer analysis indeterminate
                ptr_indeterminate = true;
            }
            _ => {}
        }
    }
    for pos in stack {
        bracket_errors.push(format!("unmatched `[` at position {}", pos));
    }

    // Count I/O and track pointer (for loop-free code)
    for ch in code.chars() {
        match ch {
            ',' => total_inputs += 1,
            '.' => total_outputs += 1,
            '>' if !ptr_indeterminate => ptr_offset += 1,
            '<' if !ptr_indeterminate => ptr_offset -= 1,
            _ => {}
        }
    }

    let ptr_end_offset = if ptr_indeterminate {
        None
    } else {
        Some(ptr_offset)
    };

    AnalysisReport {
        bracket_errors,
        total_inputs,
        total_outputs,
        ptr_end_offset,
    }
}

pub fn analyse_file(path: &Path, verbose: bool, in_place: bool) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let expanded = Preprocessor::process_file(path)?;
    let report = analyse_source(&expanded);

    let mut output_lines: Vec<String> = Vec::new();

    if !report.bracket_errors.is_empty() {
        output_lines.push("=== ERRORS ===".to_string());
        for err in &report.bracket_errors {
            output_lines.push(format!("  ERROR: {}", err));
        }
    } else {
        output_lines.push("Brackets: OK".to_string());
    }

    output_lines.push(format!("Input operations (,):  {}", report.total_inputs));
    output_lines.push(format!("Output operations (.): {}", report.total_outputs));

    match report.ptr_end_offset {
        Some(offset) => output_lines.push(format!("Data pointer net offset: {}", offset)),
        None => {
            output_lines.push("Data pointer: indeterminate (program contains loops)".to_string())
        }
    }

    if verbose {
        output_lines.push(String::new());
        output_lines.push("=== VERBOSE ===".to_string());
        let counts = count_ops(&source);
        output_lines.push(format!("  > (move right): {}", counts.right));
        output_lines.push(format!("  < (move left):  {}", counts.left));
        output_lines.push(format!("  + (increment):  {}", counts.inc));
        output_lines.push(format!("  - (decrement):  {}", counts.dec));
        output_lines.push(format!("  [ (loop open):  {}", counts.open));
        output_lines.push(format!("  ] (loop close): {}", counts.close));
    }

    if in_place {
        // Strip any existing analysis comments from top of file
        let stripped: String = source
            .lines()
            .skip_while(|line| line.starts_with("# ") || line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let stripped = if stripped.is_empty() {
            stripped
        } else {
            format!("{}\n", stripped)
        };
        // Embed the report as comments at the top of the source file
        let comment_block: String = output_lines.iter().map(|l| format!("# {}\n", l)).collect();
        let new_source = format!("{}\n{}", comment_block, stripped);
        fs::write(path, new_source)?;
    } else {
        for line in &output_lines {
            println!("{}", line);
        }
    }

    Ok(())
}

struct OpCounts {
    right: usize,
    left: usize,
    inc: usize,
    dec: usize,
    open: usize,
    close: usize,
}

fn count_ops(code: &str) -> OpCounts {
    let mut counts = OpCounts {
        right: 0,
        left: 0,
        inc: 0,
        dec: 0,
        open: 0,
        close: 0,
    };
    for ch in code.chars() {
        match ch {
            '>' => counts.right += 1,
            '<' => counts.left += 1,
            '+' => counts.inc += 1,
            '-' => counts.dec += 1,
            '[' => counts.open += 1,
            ']' => counts.close += 1,
            _ => {}
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_errors_valid_brackets() {
        let report = analyse_source("[+]");
        assert!(report.bracket_errors.is_empty());
    }

    #[test]
    fn test_unmatched_open_bracket() {
        let report = analyse_source("[+");
        assert!(!report.bracket_errors.is_empty());
        assert!(report.bracket_errors[0].contains('['));
    }

    #[test]
    fn test_unmatched_close_bracket() {
        let report = analyse_source("+]");
        assert!(!report.bracket_errors.is_empty());
        assert!(report.bracket_errors[0].contains(']'));
    }

    #[test]
    fn test_io_counting() {
        let report = analyse_source(",,..");
        assert_eq!(report.total_inputs, 2);
        assert_eq!(report.total_outputs, 2);
    }

    #[test]
    fn test_ptr_tracking_no_loops() {
        let report = analyse_source(">>><");
        assert_eq!(report.ptr_end_offset, Some(2));
    }

    #[test]
    fn test_ptr_indeterminate_with_loops() {
        let report = analyse_source("[>]");
        assert_eq!(report.ptr_end_offset, None);
    }

    #[test]
    fn test_ptr_offset_zero() {
        let report = analyse_source("><");
        assert_eq!(report.ptr_end_offset, Some(0));
    }

    #[test]
    fn test_empty_program() {
        let report = analyse_source("");
        assert!(report.bracket_errors.is_empty());
        assert_eq!(report.total_inputs, 0);
        assert_eq!(report.total_outputs, 0);
        assert_eq!(report.ptr_end_offset, Some(0));
    }
}
