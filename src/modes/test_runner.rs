use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;
use crate::project::OgreProject;

#[derive(Deserialize)]
pub struct TestCase {
    pub name: String,
    pub brainfuck: String, // path to .bf file
    pub input: String,
    pub output: String,
}

/// Run all tests from a single JSON test file.
/// `base_dir` is used to resolve relative .bf paths in each test case.
pub fn run_tests_from_file(
    test_file: &Path,
    section_name: Option<&str>,
    base_dir: &Path,
) -> Result<(usize, usize)> {
    let json = fs::read_to_string(test_file)?;
    let cases: Vec<TestCase> = serde_json::from_str(&json)?;

    if let Some(name) = section_name {
        println!("=== {} ===", name);
    }

    let total = cases.len();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for case in &cases {
        // Resolve .bf path relative to base_dir
        let bf_path: PathBuf = if Path::new(&case.brainfuck).is_absolute() {
            PathBuf::from(&case.brainfuck)
        } else {
            base_dir.join(&case.brainfuck)
        };

        let expanded = match Preprocessor::process_file(&bf_path) {
            Err(e) => {
                println!("FAIL  {}", case.name);
                println!("      preprocess error: {}", e);
                failed += 1;
                continue;
            }
            Ok(s) => s,
        };

        match Interpreter::with_input(&expanded, &case.input) {
            Err(e) => {
                println!("FAIL  {}", case.name);
                println!("      parse error: {}", e);
                failed += 1;
            }
            Ok(mut interp) => match interp.run() {
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
            },
        }
    }

    println!("{}/{} tests passed", passed, total);
    Ok((passed, failed))
}

/// Convenience wrapper: run tests from a single file, resolving bf paths
/// relative to the directory containing the test file.
pub fn run_tests(test_file: &Path) -> Result<()> {
    let base_dir = test_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let (_, failed) = run_tests_from_file(test_file, None, &base_dir)?;
    if failed > 0 {
        bail!("{} test(s) failed", failed);
    }
    Ok(())
}

/// Run all test suites defined in an ogre project.
pub fn run_project_tests(project: &OgreProject, base: &Path) -> Result<()> {
    if project.tests.is_empty() {
        println!("No tests defined in ogre.toml.");
        return Ok(());
    }

    let mut total_passed = 0usize;
    let mut total_failed = 0usize;

    for test_ref in &project.tests {
        let test_path = base.join(&test_ref.file);
        let section_name = test_ref.name.as_deref();

        // Resolve bf paths relative to the project base
        let (p, f) = run_tests_from_file(&test_path, section_name, base)?;
        total_passed += p;
        total_failed += f;
    }

    println!(
        "\nTotal: {}/{} tests passed",
        total_passed,
        total_passed + total_failed
    );
    if total_failed > 0 {
        bail!("{} test(s) failed", total_failed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_inline_case(source: &str, input: &str, expected_output: &str) -> Result<bool> {
        let mut interp = Interpreter::with_input(source, input)?;
        interp.run()?;
        Ok(interp.output_as_string() == expected_output)
    }

    #[test]
    fn test_inline_case_pass() {
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
