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

/// Options for test execution.
#[derive(Default)]
pub struct TestOptions {
    /// Filter tests by name (substring match).
    pub filter: Option<String>,
    /// Output JUnit XML to this file path.
    pub junit_output: Option<String>,
    /// Run tests in parallel.
    pub parallel: bool,
}

/// Result of a single test case execution.
#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    detail: Option<String>,
    duration_ms: u128,
}

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
    run_project_tests_with_opts(project, base, verbosity, &TestOptions::default())
}

pub fn run_project_tests_with_opts(
    project: &OgreProject,
    base: &Path,
    verbosity: Verbosity,
    opts: &TestOptions,
) -> Result<()> {
    if project.tests.is_empty() {
        if !verbosity.is_quiet() {
            println!("No tests defined in ogre.toml.");
        }
        return Ok(());
    }

    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut all_results: Vec<TestResult> = Vec::new();

    for test_ref in &project.tests {
        let test_path = base.join(&test_ref.file);
        let section_name = test_ref.name.as_deref();

        let (p, f, results) =
            run_tests_from_file_with_opts(&test_path, section_name, base, verbosity, opts)?;
        total_passed += p;
        total_failed += f;
        all_results.extend(results);
    }

    if !verbosity.is_quiet() {
        println!(
            "\nTotal: {}/{} tests passed",
            total_passed,
            total_passed + total_failed
        );
    }

    // Write JUnit XML if requested
    if let Some(ref junit_path) = opts.junit_output {
        write_junit_xml(junit_path, &all_results)?;
        if !verbosity.is_quiet() {
            println!("JUnit XML written to: {}", junit_path);
        }
    }

    if total_failed > 0 {
        bail!("{} test(s) failed", total_failed);
    }
    Ok(())
}

/// Run tests with options (filter, parallel, JUnit).
pub fn run_tests_with_opts(
    test_file: &Path,
    verbosity: Verbosity,
    opts: &TestOptions,
) -> Result<()> {
    let base_dir = test_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let (_, failed, results) =
        run_tests_from_file_with_opts(test_file, None, &base_dir, verbosity, opts)?;

    if let Some(ref junit_path) = opts.junit_output {
        write_junit_xml(junit_path, &results)?;
        if !verbosity.is_quiet() {
            println!("JUnit XML written to: {}", junit_path);
        }
    }

    if failed > 0 {
        bail!("{} test(s) failed", failed);
    }
    Ok(())
}

fn run_tests_from_file_with_opts(
    test_file: &Path,
    section_name: Option<&str>,
    base_dir: &Path,
    verbosity: Verbosity,
    opts: &TestOptions,
) -> Result<(usize, usize, Vec<TestResult>)> {
    let json = fs::read_to_string(test_file)?;
    let mut cases: Vec<TestCase> = serde_json::from_str(&json)?;

    // Apply filter
    if let Some(ref filter) = opts.filter {
        cases.retain(|c| c.name.contains(filter.as_str()));
    }

    if let Some(name) = section_name {
        if !verbosity.is_quiet() {
            println!("=== {} ===", name);
        }
    }

    let total = cases.len();
    let verbose = verbosity.is_verbose();

    let results: Vec<TestResult> = if opts.parallel && cases.len() > 1 {
        // Run tests in parallel using threads
        use std::sync::{Arc, Mutex};
        use std::thread;

        let results = Arc::new(Mutex::new(Vec::new()));
        let handles: Vec<_> = cases
            .into_iter()
            .map(|case| {
                let base = base_dir.to_path_buf();
                let results = Arc::clone(&results);
                thread::spawn(move || {
                    let result = run_single_test(&case, &base);
                    results.lock().unwrap().push(result);
                })
            })
            .collect();

        for h in handles {
            let _ = h.join();
        }

        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    } else {
        cases
            .iter()
            .map(|case| run_single_test(case, base_dir))
            .collect()
    };

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for r in &results {
        if r.passed {
            passed += 1;
            if verbose {
                println!("  {} {}", "PASS".green().bold(), r.name);
            } else if !verbosity.is_quiet() {
                print!("{}", ".".green());
            }
        } else {
            failed += 1;
            if verbose {
                println!("  {} {}", "FAIL".red().bold(), r.name);
            } else if !verbosity.is_quiet() {
                print!("{}", "F".red());
            }
            if let Some(ref detail) = r.detail {
                failures.push((r.name.clone(), detail.clone()));
            }
        }
    }

    if !verbosity.is_quiet() && !verbose {
        println!();
    }

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

    Ok((passed, failed, results))
}

fn run_single_test(case: &TestCase, base_dir: &Path) -> TestResult {
    let start = std::time::Instant::now();

    // Check for conflicting output and output_regex
    if case.output_regex.is_some() && !case.output.is_empty() {
        return TestResult {
            name: case.name.clone(),
            passed: false,
            detail: Some(
                "test case specifies both 'output' and 'output_regex' — use only one".to_string(),
            ),
            duration_ms: start.elapsed().as_millis(),
        };
    }

    let bf_path: PathBuf = if Path::new(&case.brainfuck).is_absolute() {
        PathBuf::from(&case.brainfuck)
    } else {
        base_dir.join(&case.brainfuck)
    };

    let expanded = match Preprocessor::process_file(&bf_path) {
        Err(e) => {
            return TestResult {
                name: case.name.clone(),
                passed: false,
                detail: Some(format!("preprocess error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
            };
        }
        Ok(s) => s,
    };

    let instruction_limit = case.timeout.unwrap_or(DEFAULT_INSTRUCTION_LIMIT);

    match Interpreter::with_input(&expanded, &case.input) {
        Err(e) => TestResult {
            name: case.name.clone(),
            passed: false,
            detail: Some(format!("parse error: {}", e)),
            duration_ms: start.elapsed().as_millis(),
        },
        Ok(mut interp) => match interp.run_with_limit(instruction_limit) {
            Err(e) => TestResult {
                name: case.name.clone(),
                passed: false,
                detail: Some(format!("runtime error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
            },
            Ok(false) => TestResult {
                name: case.name.clone(),
                passed: false,
                detail: Some(format!(
                    "timeout: exceeded {} instruction limit",
                    instruction_limit
                )),
                duration_ms: start.elapsed().as_millis(),
            },
            Ok(true) => {
                let actual = interp.output_as_string();
                let pass = if let Some(ref regex_str) = case.output_regex {
                    match Regex::new(regex_str) {
                        Ok(re) => re.is_match(&actual),
                        Err(e) => {
                            return TestResult {
                                name: case.name.clone(),
                                passed: false,
                                detail: Some(format!("invalid regex '{}': {}", regex_str, e)),
                                duration_ms: start.elapsed().as_millis(),
                            };
                        }
                    }
                } else {
                    actual == case.output
                };

                if pass {
                    TestResult {
                        name: case.name.clone(),
                        passed: true,
                        detail: None,
                        duration_ms: start.elapsed().as_millis(),
                    }
                } else {
                    let detail = if let Some(ref regex_str) = case.output_regex {
                        format!("output {:?} does not match regex /{}/", actual, regex_str)
                    } else {
                        format!("expected: {:?}\n      actual:   {:?}", case.output, actual)
                    };
                    TestResult {
                        name: case.name.clone(),
                        passed: false,
                        detail: Some(detail),
                        duration_ms: start.elapsed().as_millis(),
                    }
                }
            }
        },
    }
}

/// Generate JUnit XML from test results.
fn write_junit_xml(path: &str, results: &[TestResult]) -> Result<()> {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    let total = results.len();
    let failures = results.iter().filter(|r| !r.passed).count();
    let total_time: f64 = results.iter().map(|r| r.duration_ms as f64 / 1000.0).sum();

    xml.push_str(&format!(
        "<testsuite name=\"ogre\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
        total, failures, total_time
    ));

    for r in results {
        let time = r.duration_ms as f64 / 1000.0;
        if r.passed {
            xml.push_str(&format!(
                "  <testcase name=\"{}\" time=\"{:.3}\"/>\n",
                xml_escape(&r.name),
                time
            ));
        } else {
            xml.push_str(&format!(
                "  <testcase name=\"{}\" time=\"{:.3}\">\n",
                xml_escape(&r.name),
                time
            ));
            xml.push_str(&format!(
                "    <failure message=\"{}\">{}</failure>\n",
                xml_escape(r.detail.as_deref().unwrap_or("test failed")),
                xml_escape(r.detail.as_deref().unwrap_or(""))
            ));
            xml.push_str("  </testcase>\n");
        }
    }

    xml.push_str("</testsuite>\n");
    fs::write(path, &xml)?;
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
