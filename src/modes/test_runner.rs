use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs;

use super::interpreter::Interpreter;

#[derive(Deserialize)]
pub struct TestCase {
    pub name: String,
    pub brainfuck: String, // path to .bf file
    pub input: String,
    pub output: String,
}

pub fn run_tests(test_file: &str) -> Result<()> {
    let json = fs::read_to_string(test_file)?;
    let cases: Vec<TestCase> = serde_json::from_str(&json)?;

    let total = cases.len();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for case in &cases {
        let source = fs::read_to_string(&case.brainfuck)
            .map_err(|e| anyhow::anyhow!("Test '{}': failed to read '{}': {}", case.name, case.brainfuck, e))?;

        match Interpreter::with_input(&source, &case.input) {
            Err(e) => {
                println!("FAIL  {}", case.name);
                println!("      parse error: {}", e);
                failed += 1;
            }
            Ok(mut interp) => {
                match interp.run() {
                    Err(e) => {
                        println!("FAIL  {}", case.name);
                        println!("      runtime error: {}", e);
                        failed += 1;
                    }
                    Ok(()) => {
                        let actual = interp.output_as_string();
                        if actual == case.output {
                            println!("PASS  {}", case.name);
                            passed += 1;
                        } else {
                            println!("FAIL  {}", case.name);
                            println!("      expected: {:?}", case.output);
                            println!("      actual:   {:?}", actual);
                            failed += 1;
                        }
                    }
                }
            }
        }
    }

    println!("\n{}/{} tests passed", passed, total);
    if failed > 0 {
        bail!("{} test(s) failed", failed);
    }
    Ok(())
}

pub fn run_test_cases(cases: &[TestCase]) -> Result<()> {
    let total = cases.len();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for case in cases {
        let source = fs::read_to_string(&case.brainfuck)
            .map_err(|e| anyhow::anyhow!("Test '{}': failed to read '{}': {}", case.name, case.brainfuck, e))?;

        match run_single_case(&source, &case.input, &case.output) {
            Ok(true) => {
                println!("PASS  {}", case.name);
                passed += 1;
            }
            Ok(false) => {
                println!("FAIL  {}", case.name);
                failed += 1;
            }
            Err(e) => {
                println!("FAIL  {} — {}", case.name, e);
                failed += 1;
            }
        }
    }

    println!("\n{}/{} tests passed", passed, total);
    if failed > 0 {
        bail!("{} test(s) failed", failed);
    }
    Ok(())
}

/// Run a single test case in-memory (source code provided directly, not via file path).
pub fn run_inline_case(source: &str, input: &str, expected_output: &str) -> Result<bool> {
    run_single_case(source, input, expected_output)
}

fn run_single_case(source: &str, input: &str, expected_output: &str) -> Result<bool> {
    let mut interp = Interpreter::with_input(source, input)?;
    interp.run()?;
    let actual = interp.output_as_string();
    Ok(actual == expected_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_case_pass() {
        // hello world one-liner
        let hw = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let result = run_inline_case(hw, "", "Hello World!\n").unwrap();
        assert!(result);
    }

    #[test]
    fn test_inline_case_fail() {
        let result = run_inline_case("+.", "", "wrong output").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_inline_case_with_input() {
        let result = run_inline_case(",[.,]", "abc", "abc").unwrap();
        assert!(result);
    }

    #[test]
    fn test_inline_case_invalid_bf_errors() {
        let result = run_inline_case("[unclosed", "", "");
        assert!(result.is_err());
    }
}
