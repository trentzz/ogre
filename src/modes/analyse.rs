use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

use super::ir::{Op, Program};
use super::preprocess::Preprocessor;

pub struct AnalysisReport {
    pub bracket_errors: Vec<String>,
    pub total_inputs: usize,
    pub total_outputs: usize,
    pub ptr_end_offset: Option<i64>, // None if loops prevent static analysis
    pub has_clear_idiom: bool,
    pub has_dead_code: bool,
    pub has_cancellation: bool,
    pub unbalanced_pointer: bool,
}

pub fn analyse_source(code: &str) -> AnalysisReport {
    let mut bracket_errors: Vec<String> = Vec::new();

    // First validate brackets by attempting to parse
    let program = match Program::from_source(code) {
        Ok(p) => Some(p),
        Err(e) => {
            bracket_errors.push(e.to_string());
            None
        }
    };

    // Also check for bracket errors manually for precise positions
    if bracket_errors.is_empty() {
        let mut stack: Vec<usize> = Vec::new();
        for (i, ch) in code.chars().enumerate() {
            match ch {
                '[' => stack.push(i),
                ']' => {
                    if stack.pop().is_none() {
                        bracket_errors.push(format!("unmatched `]` at position {}", i));
                    }
                }
                _ => {}
            }
        }
        for pos in stack {
            bracket_errors.push(format!("unmatched `[` at position {}", pos));
        }
    }

    let mut total_inputs = 0usize;
    let mut total_outputs = 0usize;
    let mut ptr_offset: i64 = 0;
    let mut ptr_indeterminate = false;
    let mut has_clear_idiom = false;
    let mut has_dead_code = false;
    let mut has_cancellation = false;

    if let Some(ref program) = program {
        for op in &program.ops {
            match op {
                Op::Input => total_inputs += 1,
                Op::Output => total_outputs += 1,
                Op::Right(n) if !ptr_indeterminate => ptr_offset += *n as i64,
                Op::Left(n) if !ptr_indeterminate => ptr_offset -= *n as i64,
                Op::JumpIfZero(_) | Op::JumpIfNonZero(_) => ptr_indeterminate = true,
                _ => {}
            }
        }

        // Deep analysis: detect patterns
        has_clear_idiom = detect_clear_idiom(code);
        has_cancellation = detect_cancellation(code);
        has_dead_code = detect_dead_code(code);
    } else {
        // If parsing failed, count from raw source
        for ch in code.chars() {
            match ch {
                ',' => total_inputs += 1,
                '.' => total_outputs += 1,
                '>' if !ptr_indeterminate => ptr_offset += 1,
                '<' if !ptr_indeterminate => ptr_offset -= 1,
                '[' | ']' => ptr_indeterminate = true,
                _ => {}
            }
        }
    }

    let ptr_end_offset = if ptr_indeterminate {
        None
    } else {
        Some(ptr_offset)
    };

    let unbalanced_pointer = ptr_end_offset.is_some_and(|o| o != 0);

    AnalysisReport {
        bracket_errors,
        total_inputs,
        total_outputs,
        ptr_end_offset,
        has_clear_idiom,
        has_dead_code,
        has_cancellation,
        unbalanced_pointer,
    }
}

fn detect_clear_idiom(code: &str) -> bool {
    code.contains("[-]") || code.contains("[+]")
}

fn detect_cancellation(code: &str) -> bool {
    let chars: Vec<char> = code.chars().filter(|c| "+-><".contains(*c)).collect();
    for w in chars.windows(2) {
        match (w[0], w[1]) {
            ('+', '-') | ('-', '+') | ('>', '<') | ('<', '>') => return true,
            _ => {}
        }
    }
    false
}

fn detect_dead_code(code: &str) -> bool {
    let chars: Vec<char> = code.chars().filter(|c| "+-><.,[]".contains(*c)).collect();
    for w in chars.windows(2) {
        // Clear followed by clear is dead code
        if w[0] == ']' && w[1] == '[' {
            // Check if the ] ends a [-] and [ starts another [-]
            // This is a simplistic check
        }
    }
    // Check for code after an infinite loop pattern
    false
}

pub fn analyse_file(path: &Path, verbose: bool, in_place: bool) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let expanded = Preprocessor::process_file(path)?;
    let report = analyse_source(&expanded);

    let mut output_lines: Vec<String> = Vec::new();

    if !report.bracket_errors.is_empty() {
        output_lines.push(format!("{}", "=== ERRORS ===".red().bold()));
        for err in &report.bracket_errors {
            output_lines.push(format!("  {} {}", "ERROR:".red(), err));
        }
    } else {
        output_lines.push(format!("Brackets: {}", "OK".green()));
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

        output_lines.push(String::new());
        output_lines.push("=== DEEP ANALYSIS ===".to_string());
        if report.has_clear_idiom {
            output_lines.push("  Found clear idiom ([-] or [+])".to_string());
        }
        if report.has_cancellation {
            output_lines.push(
                "  Found cancellation pattern (+- or -+ or >< or <>) — consider simplifying"
                    .to_string(),
            );
        }
        if report.unbalanced_pointer {
            output_lines.push(format!(
                "  Warning: data pointer ends at offset {} from start",
                report.ptr_end_offset.unwrap()
            ));
        }
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

    #[test]
    fn test_clear_idiom_detection() {
        let report = analyse_source("[-]");
        assert!(report.has_clear_idiom);
    }

    #[test]
    fn test_cancellation_detection() {
        let report = analyse_source("+-");
        assert!(report.has_cancellation);
    }

    #[test]
    fn test_unbalanced_pointer() {
        let report = analyse_source(">>>");
        assert!(report.unbalanced_pointer);
    }
}
