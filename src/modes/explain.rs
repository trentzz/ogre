use anyhow::Result;
use std::path::Path;

use super::ir::Program;
use super::preprocess::Preprocessor;

/// Explain what a brainfuck program does in plain English.
pub fn explain_file(path: &Path) -> Result<String> {
    let expanded = Preprocessor::process_file(path)?;
    Ok(explain_source(&expanded))
}

pub fn explain_source(source: &str) -> String {
    let program = match Program::from_source(source) {
        Ok(p) => p,
        Err(e) => return format!("Error parsing program: {}", e),
    };

    let mut explanations = Vec::new();
    let bf_chars: Vec<char> = source.chars().filter(|c| "+-<>.,[]".contains(*c)).collect();

    // Overall statistics
    let total_ops = bf_chars.len();
    let io_input = bf_chars.iter().filter(|&&c| c == ',').count();
    let io_output = bf_chars.iter().filter(|&&c| c == '.').count();
    let loops = bf_chars.iter().filter(|&&c| c == '[').count();
    let moves_right = bf_chars.iter().filter(|&&c| c == '>').count();
    let moves_left = bf_chars.iter().filter(|&&c| c == '<').count();
    let max_cells = if moves_right > moves_left {
        moves_right - moves_left + 1
    } else {
        1
    };

    explanations.push(format!("Program has {} BF instructions.", total_ops));

    // Classify program type
    if io_input > 0 && io_output > 0 {
        explanations.push("This is an I/O program that reads input and produces output.".into());
    } else if io_output > 0 {
        explanations.push("This is an output-only program (no input reading).".into());
    } else if io_input > 0 {
        explanations.push("This program reads input but produces no output.".into());
    } else {
        explanations.push("This program has no I/O operations.".into());
    }

    // Detect common patterns
    if total_ops == 0 {
        explanations.push("The program is empty (no-op).".into());
    }

    // Detect cat program pattern: ,[.,]
    if source.contains(",[.,]") || source.contains(",.[,.]") {
        explanations.push("Pattern detected: cat (echo) — copies input to output.".into());
    }

    // Detect hello world pattern
    if io_output > 5 && io_input == 0 && loops > 0 {
        explanations
            .push("Pattern: output-generating program using loops to compute ASCII values.".into());
    }

    // Detect clear idiom usage
    let clear_count = source.matches("[-]").count();
    if clear_count > 0 {
        explanations.push(format!(
            "Uses {} clear idiom(s) ([-]) to zero cells.",
            clear_count
        ));
    }

    // Detect move patterns
    let move_right_pattern = source.matches("[->+<]").count();
    let move_left_pattern = source.matches("[-<+>]").count();
    if move_right_pattern + move_left_pattern > 0 {
        explanations.push(format!(
            "Uses {} move pattern(s) to transfer cell values.",
            move_right_pattern + move_left_pattern
        ));
    }

    // Detect scan patterns
    if source.contains("[>]") {
        explanations.push("Uses scan-right ([>]) to find a zero cell.".into());
    }
    if source.contains("[<]") {
        explanations.push("Uses scan-left ([<]) to find a zero cell.".into());
    }

    // IR-level analysis
    let mut opt_program = program;
    let pre_opt = opt_program.ops.len();
    opt_program.optimize();
    let post_opt = opt_program.ops.len();
    if pre_opt > post_opt {
        explanations.push(format!(
            "Optimization reduces {} IR ops to {} ({:.0}% reduction).",
            pre_opt,
            post_opt,
            (1.0 - post_opt as f64 / pre_opt as f64) * 100.0
        ));
    }

    // Memory usage estimate
    explanations.push(format!(
        "Estimated memory: ~{} cell(s), {} loop(s), {} input(s), {} output(s).",
        max_cells, loops, io_input, io_output
    ));

    // Try to determine output
    if total_ops < 1000 && io_input == 0 {
        if let Ok(mut interp) =
            super::interpreter::Interpreter::with_input(&opt_program.to_bf_string(), "")
        {
            if let Ok(true) = interp.run_with_limit(100_000) {
                let output = interp.output_as_string();
                if !output.is_empty() && output.len() <= 200 {
                    explanations.push(format!("Output: {:?}", output));
                }
            }
        }
    }

    explanations.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_empty() {
        let result = explain_source("");
        assert!(result.contains("0 BF instructions"));
        assert!(result.contains("empty"));
    }

    #[test]
    fn test_explain_cat() {
        let result = explain_source(",[.,]");
        assert!(result.contains("cat"));
    }

    #[test]
    fn test_explain_hello_world() {
        let hw = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let result = explain_source(hw);
        assert!(result.contains("output"));
        assert!(result.contains("Hello World!"));
    }

    #[test]
    fn test_explain_clear_idiom() {
        let result = explain_source("+++[-]");
        assert!(result.contains("clear idiom"));
    }

    #[test]
    fn test_explain_io() {
        let result = explain_source(",.");
        assert!(result.contains("I/O program"));
    }

    #[test]
    fn test_explain_no_io() {
        let result = explain_source("+++");
        assert!(result.contains("no I/O"));
    }

    #[test]
    fn test_explain_move_pattern() {
        let result = explain_source("+++[->+<]");
        assert!(result.contains("move pattern"));
    }

    #[test]
    fn test_explain_optimization() {
        let result = explain_source("+++--->><<");
        assert!(result.contains("Optimization") || result.contains("reduction"));
    }
}
