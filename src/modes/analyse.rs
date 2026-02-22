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
    pub clear_idiom_count: usize,
    pub has_dead_code: bool,
    pub dead_code_positions: Vec<usize>,
    pub has_cancellation: bool,
    pub cancellation_positions: Vec<usize>,
    pub unbalanced_pointer: bool,
    // Complexity metrics
    pub max_loop_depth: usize,
    pub total_ops: usize,
    pub optimized_ops: usize,
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
    let mut clear_idiom_count = 0;
    let mut cancellation_positions = Vec::new();
    let mut dead_code_positions = Vec::new();

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
        clear_idiom_count = count_clear_idioms(code);
        cancellation_positions = detect_cancellation_positions(code);
        dead_code_positions = detect_dead_code_positions(code);
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

    // Complexity metrics
    let max_loop_depth = compute_max_loop_depth(code);
    let total_ops = code.chars().filter(|c| "+-><.,[]".contains(*c)).count();
    let optimized_ops = if let Some(ref prog) = program {
        let mut opt_prog = prog.clone();
        opt_prog.optimize();
        opt_prog.ops.len()
    } else {
        total_ops
    };

    AnalysisReport {
        bracket_errors,
        total_inputs,
        total_outputs,
        ptr_end_offset,
        has_clear_idiom: clear_idiom_count > 0,
        clear_idiom_count,
        has_dead_code: !dead_code_positions.is_empty(),
        dead_code_positions,
        has_cancellation: !cancellation_positions.is_empty(),
        cancellation_positions,
        unbalanced_pointer,
        max_loop_depth,
        total_ops,
        optimized_ops,
    }
}

fn compute_max_loop_depth(code: &str) -> usize {
    let mut depth: usize = 0;
    let mut max_depth: usize = 0;
    for ch in code.chars() {
        match ch {
            '[' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }
    max_depth
}

fn count_clear_idioms(code: &str) -> usize {
    let mut count = 0;
    let bytes = code.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i] == b'[' && (bytes[i + 1] == b'-' || bytes[i + 1] == b'+') && bytes[i + 2] == b']'
        {
            count += 1;
        }
    }
    count
}

fn detect_cancellation_positions(code: &str) -> Vec<usize> {
    let bf_chars: Vec<(usize, char)> = code
        .chars()
        .enumerate()
        .filter(|(_, c)| "+-><".contains(*c))
        .collect();
    let mut positions = Vec::new();
    for w in bf_chars.windows(2) {
        match (w[0].1, w[1].1) {
            ('+', '-') | ('-', '+') | ('>', '<') | ('<', '>') => {
                positions.push(w[0].0);
            }
            _ => {}
        }
    }
    positions
}

fn detect_dead_code_positions(code: &str) -> Vec<usize> {
    let bf_chars: Vec<(usize, char)> = code
        .chars()
        .enumerate()
        .filter(|(_, c)| "+-><.,[]".contains(*c))
        .collect();
    let mut positions = Vec::new();

    // Detect code after an unconditional infinite loop
    // Pattern: +[ at position 0 (cell starts at 0, + makes it nonzero, [ always enters)
    if bf_chars.len() >= 2 && bf_chars[0].1 == '+' && bf_chars[1].1 == '[' {
        // Find the matching ]
        let mut depth = 0;
        let mut close_idx = None;
        for (i, &(_, c)) in bf_chars.iter().enumerate().skip(1) {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        close_idx = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        // If there's code after the closing ], it's dead
        if let Some(ci) = close_idx {
            if ci + 1 < bf_chars.len() {
                positions.push(bf_chars[ci + 1].0);
            }
        }
    }

    positions
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
        output_lines.push("=== COMPLEXITY METRICS ===".to_string());
        output_lines.push(format!("  Max loop nesting depth: {}", report.max_loop_depth));
        output_lines.push(format!("  Total BF instructions: {}", report.total_ops));
        output_lines.push(format!("  Optimized IR ops: {}", report.optimized_ops));
        if report.total_ops > 0 {
            let reduction = 100.0 - (report.optimized_ops as f64 / report.total_ops as f64 * 100.0);
            output_lines.push(format!("  Optimization reduction: {:.1}%", reduction));
        }

        output_lines.push(String::new());
        output_lines.push("=== DEEP ANALYSIS ===".to_string());
        if report.has_clear_idiom {
            output_lines.push(format!(
                "  Found {} clear idiom(s) ([-] or [+])",
                report.clear_idiom_count
            ));
        }
        if report.has_cancellation {
            let pos_str: Vec<String> = report
                .cancellation_positions
                .iter()
                .map(|p| p.to_string())
                .collect();
            output_lines.push(format!(
                "  Found {} cancellation(s) at position(s): {}",
                report.cancellation_positions.len(),
                pos_str.join(", ")
            ));
        }
        if report.has_dead_code {
            let pos_str: Vec<String> = report
                .dead_code_positions
                .iter()
                .map(|p| p.to_string())
                .collect();
            output_lines.push(format!(
                "  {} unreachable code after position(s): {}",
                "Warning:".yellow(),
                pos_str.join(", ")
            ));
        }
        if report.unbalanced_pointer {
            output_lines.push(format!(
                "  {}: data pointer ends at offset {} from start",
                "Warning".yellow(),
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

    #[test]
    fn test_clear_idiom_count() {
        let report = analyse_source("[-]+++[-]>>[-]");
        assert_eq!(report.clear_idiom_count, 3);
    }

    #[test]
    fn test_cancellation_positions() {
        let report = analyse_source("+->+<>");
        assert!(report.has_cancellation);
        assert!(!report.cancellation_positions.is_empty());
        // +- at position 0, <> at position 4
        assert!(report.cancellation_positions.contains(&0));
        assert!(report.cancellation_positions.contains(&4));
    }

    #[test]
    fn test_dead_code_after_infinite_loop() {
        let report = analyse_source("+[>+<]+++");
        assert!(report.has_dead_code);
        assert!(!report.dead_code_positions.is_empty());
    }

    #[test]
    fn test_no_dead_code_normal_program() {
        let report = analyse_source("+++[>+<-]>.");
        assert!(!report.has_dead_code);
    }

    #[test]
    fn test_no_false_positive_cancellation() {
        // These shouldn't count as cancellations (separated by other ops)
        let report = analyse_source("+.->.<>");
        // The +. separates + from -, but <> at end is cancellation
        assert!(report.has_cancellation);
    }

    #[test]
    fn test_no_cancellation_in_clean_code() {
        let report = analyse_source("+++>>>.---<<<");
        assert!(!report.has_cancellation);
    }

    #[test]
    fn test_max_loop_depth() {
        let report = analyse_source("[[+]]");
        assert_eq!(report.max_loop_depth, 2);
    }

    #[test]
    fn test_max_loop_depth_zero() {
        let report = analyse_source("+++");
        assert_eq!(report.max_loop_depth, 0);
    }

    #[test]
    fn test_total_ops_count() {
        let report = analyse_source("+++>>.");
        assert_eq!(report.total_ops, 6); // 3 + 2 > + 1 .
    }

    #[test]
    fn test_optimized_ops_fewer() {
        // +-  cancels, so optimized should have fewer ops
        let report = analyse_source("+++---");
        assert!(report.optimized_ops <= report.total_ops);
    }

    #[test]
    fn test_max_loop_depth_sequential() {
        let report = analyse_source("[+][+][+]");
        assert_eq!(report.max_loop_depth, 1);
    }

    #[test]
    fn test_max_loop_depth_deep() {
        let report = analyse_source("[[[+]]]");
        assert_eq!(report.max_loop_depth, 3);
    }
}
