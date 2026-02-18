use anyhow::Result;
use std::fs;

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

pub fn analyse_file(path: &str, verbose: bool, in_place: bool) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let report = analyse_source(&source);

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
        None => output_lines.push("Data pointer: indeterminate (program contains loops)".to_string()),
    }

    if verbose {
        output_lines.push(String::new());
        output_lines.push("=== VERBOSE ===".to_string());
        let op_counts = count_ops(&source);
        output_lines.push(format!("  > (move right): {}", op_counts.0));
        output_lines.push(format!("  < (move left):  {}", op_counts.1));
        output_lines.push(format!("  + (increment):  {}", op_counts.2));
        output_lines.push(format!("  - (decrement):  {}", op_counts.3));
        output_lines.push(format!("  [ (loop open):  {}", op_counts.4));
        output_lines.push(format!("  ] (loop close): {}", op_counts.5));
    }

    if in_place {
        // Embed the report as comments at the top of the source file
        let comment_block: String = output_lines
            .iter()
            .map(|l| format!("# {}\n", l))
            .collect();
        let new_source = format!("{}\n{}", comment_block, source);
        fs::write(path, new_source)?;
    } else {
        for line in &output_lines {
            println!("{}", line);
        }
    }

    Ok(())
}

fn count_ops(code: &str) -> (usize, usize, usize, usize, usize, usize) {
    let mut r = 0;
    let mut l = 0;
    let mut inc = 0;
    let mut dec = 0;
    let mut open = 0;
    let mut close = 0;
    for ch in code.chars() {
        match ch {
            '>' => r += 1,
            '<' => l += 1,
            '+' => inc += 1,
            '-' => dec += 1,
            '[' => open += 1,
            ']' => close += 1,
            _ => {}
        }
    }
    (r, l, inc, dec, open, close)
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
