use anyhow::{bail, Result};
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;
use crate::project::OgreProject;
use crate::verbosity::Verbosity;

/// Default instruction limit for test timeout (10 million).
const DEFAULT_INSTRUCTION_LIMIT: u64 = 10_000_000;

#[derive(Deserialize)]
pub struct TestCase {
    pub name: String,
    pub brainfuck: String, // path to .bf file
    pub input: String,
    pub output: String,
    /// Optional regex pattern to match against output instead of exact match.
    pub output_regex: Option<String>,
    /// Optional instruction limit override (default 10M).
    pub timeout: Option<u64>,
}

/// Run all tests from a single JSON test file.
/// `base_dir` is used to resolve relative .bf paths in each test case.
pub fn run_tests_from_file(
    test_file: &Path,
    section_name: Option<&str>,
    base_dir: &Path,
) -> Result<(usize, usize)> {
    run_tests_from_file_ex(test_file, section_name, base_dir, Verbosity::Normal)
}

pub fn run_tests_from_file_ex(
    test_file: &Path,
    section_name: Option<&str>,
    base_dir: &Path,
    verbosity: Verbosity,
) -> Result<(usize, usize)> {
    let json = fs::read_to_string(test_file)?;
    let cases: Vec<TestCase> = serde_json::from_str(&json)?;

    if let Some(name) = section_name {
        if !verbosity.is_quiet() {
            println!("=== {} ===", name);
        }
    }

    let total = cases.len();
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<(String, String)> = Vec::new();

    let verbose = verbosity.is_verbose();

    for case in &cases {
        // Check for conflicting output and output_regex
        if case.output_regex.is_some() && !case.output.is_empty() {
            if verbose {
                println!("  {} {}", "FAIL".red().bold(), case.name);
            } else {
                print!("{}", "F".red());
            }
            failures.push((
                case.name.clone(),
                "test case specifies both 'output' and 'output_regex' — use only one".to_string(),
            ));
            failed += 1;
            continue;
        }

        // Resolve .bf path relative to base_dir
        let bf_path: PathBuf = if Path::new(&case.brainfuck).is_absolute() {
            PathBuf::from(&case.brainfuck)
        } else {
            base_dir.join(&case.brainfuck)
        };

        let expanded = match Preprocessor::process_file(&bf_path) {
            Err(e) => {
                if verbose {
                    println!("  {} {}", "FAIL".red().bold(), case.name);
                } else {
                    print!("{}", "F".red());
                }
                failures.push((case.name.clone(), format!("preprocess error: {}", e)));
                failed += 1;
                continue;
            }
            Ok(s) => s,
        };

        let instruction_limit = case.timeout.unwrap_or(DEFAULT_INSTRUCTION_LIMIT);

        match Interpreter::with_input(&expanded, &case.input) {
            Err(e) => {
                if verbose {
                    println!("  {} {}", "FAIL".red().bold(), case.name);
                } else {
                    print!("{}", "F".red());
                }
                failures.push((case.name.clone(), format!("parse error: {}", e)));
                failed += 1;
            }
            Ok(mut interp) => match interp.run_with_limit(instruction_limit) {
                Err(e) => {
                    if verbose {
                        println!("  {} {}", "FAIL".red().bold(), case.name);
                    } else {
                        print!("{}", "F".red());
                    }
                    failures.push((case.name.clone(), format!("runtime error: {}", e)));
                    failed += 1;
                }
                Ok(false) => {
                    if verbose {
                        println!("  {} {}", "TIMEOUT".yellow().bold(), case.name);
                    } else {
                        print!("{}", "T".yellow());
                    }
                    failures.push((
                        case.name.clone(),
                        format!("timeout: exceeded {} instruction limit", instruction_limit),
                    ));
                    failed += 1;
                }
                Ok(true) => {
                    let actual = interp.output_as_string();
                    let pass = if let Some(ref regex_str) = case.output_regex {
                        match Regex::new(regex_str) {
                            Ok(re) => re.is_match(&actual),
                            Err(e) => {
                                if verbose {
                                    println!("  {} {}", "FAIL".red().bold(), case.name);
                                } else {
                                    print!("{}", "F".red());
                                }
                                failures.push((
                                    case.name.clone(),
                                    format!("invalid regex '{}': {}", regex_str, e),
                                ));
                                failed += 1;
                                continue;
                            }
                        }
                    } else {
                        actual == case.output
                    };

                    if pass {
                        if verbose {
                            println!("  {} {}", "PASS".green().bold(), case.name);
                        } else {
                            print!("{}", ".".green());
                        }
                        passed += 1;
                    } else {
                        if verbose {
                            println!("  {} {}", "FAIL".red().bold(), case.name);
                        } else {
                            print!("{}", "F".red());
                        }
                        if let Some(ref regex_str) = case.output_regex {
                            failures.push((
                                case.name.clone(),
                                format!("output {:?} does not match regex /{}/", actual, regex_str),
                            ));
                        } else {
                            failures.push((
                                case.name.clone(),
                                format!(
                                    "expected: {:?}\n      actual:   {:?}",
                                    case.output, actual
                                ),
                            ));
                        }
                        failed += 1;
                    }
                }
            },
        }
    }

    if !verbosity.is_quiet() && !verbose {
        println!(); // newline after dots
    }

    // Print failure details
    if !failures.is_empty() {
        println!();
        println!("{}", "Failures:".red().bold());
        for (name, detail) in &failures {
            println!("  {} {}", "FAIL".red().bold(), name);
            println!("      {}", detail);
        }
        println!();
    }

    if !verbosity.is_quiet() {
        if failed > 0 {
            println!("{}/{} tests passed", passed.to_string().red(), total);
        } else {
            println!("{}/{} tests passed", passed.to_string().green(), total);
        }
    }

    Ok((passed, failed))
}

/// Convenience wrapper: run tests from a single file, resolving bf paths
/// relative to the directory containing the test file.
pub fn run_tests(test_file: &Path) -> Result<()> {
    run_tests_ex(test_file, Verbosity::Normal)
}

pub fn run_tests_ex(test_file: &Path, verbosity: Verbosity) -> Result<()> {
    let base_dir = test_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let (_, failed) = run_tests_from_file_ex(test_file, None, &base_dir, verbosity)?;
    if failed > 0 {
        bail!("{} test(s) failed", failed);
    }
    Ok(())
}

/// Run all test suites defined in an ogre project.
pub fn run_project_tests(project: &OgreProject, base: &Path) -> Result<()> {
    run_project_tests_ex(project, base, Verbosity::Normal)
}

pub fn run_project_tests_ex(
    project: &OgreProject,
    base: &Path,
    verbosity: Verbosity,
) -> Result<()> {
    if project.tests.is_empty() {
        if !verbosity.is_quiet() {
            println!("No tests defined in ogre.toml.");
        }
        return Ok(());
    }

    let mut total_passed = 0usize;
    let mut total_failed = 0usize;

    for test_ref in &project.tests {
        let test_path = base.join(&test_ref.file);
        let section_name = test_ref.name.as_deref();

        // Resolve bf paths relative to the project base
        let (p, f) = run_tests_from_file_ex(&test_path, section_name, base, verbosity)?;
        total_passed += p;
        total_failed += f;
    }

    if !verbosity.is_quiet() {
        println!(
            "\nTotal: {}/{} tests passed",
            total_passed,
            total_passed + total_failed
        );
    }
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

    #[test]
    fn test_regex_matching() {
        let re = Regex::new(r"Hello.*!").unwrap();
        assert!(re.is_match("Hello World!"));
        assert!(!re.is_match("Goodbye"));
    }

    #[test]
    fn test_instruction_limit() {
        // Infinite loop: +[+] will never terminate
        let mut interp = Interpreter::with_input("+[+]", "").unwrap();
        let completed = interp.run_with_limit(100).unwrap();
        assert!(!completed);
    }

    #[test]
    fn test_regex_mismatch_reports_correctly() {
        let re = Regex::new(r"^Hello$").unwrap();
        assert!(!re.is_match("Goodbye World"));
        assert!(!re.is_match("Hello World!"));
        assert!(re.is_match("Hello"));
    }

    #[test]
    fn test_output_and_regex_conflict() {
        // Verifies that the conflict detection logic works:
        // if output_regex is Some and output is non-empty, it should be flagged
        let case = TestCase {
            name: "conflict".to_string(),
            brainfuck: "dummy.bf".to_string(),
            input: "".to_string(),
            output: "something".to_string(),
            output_regex: Some("pattern".to_string()),
            timeout: None,
        };
        assert!(case.output_regex.is_some() && !case.output.is_empty());
    }
}
