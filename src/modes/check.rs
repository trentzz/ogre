use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::verbosity::Verbosity;

use super::ir::Program;
use super::preprocess::Preprocessor;

pub struct CheckResult {
    pub brackets_ok: bool,
    pub preprocess_ok: bool,
    pub errors: Vec<String>,
}

pub fn check_file(path: &Path) -> Result<CheckResult> {
    let expanded = match Preprocessor::process_file(path) {
        Ok(s) => s,
        Err(e) => {
            return Ok(CheckResult {
                brackets_ok: true,
                preprocess_ok: false,
                errors: vec![format!("preprocess: {}", e)],
            });
        }
    };
    check_expanded(&expanded)
}

/// Check a file with pre-loaded dependency functions.
pub fn check_file_with_deps(
    path: &Path,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<CheckResult> {
    let expanded = match Preprocessor::process_file_with_deps(path, dep_functions) {
        Ok(s) => s,
        Err(e) => {
            return Ok(CheckResult {
                brackets_ok: true,
                preprocess_ok: false,
                errors: vec![format!("preprocess: {}", e)],
            });
        }
    };
    check_expanded(&expanded)
}

fn check_expanded(expanded: &str) -> Result<CheckResult> {
    let mut result = CheckResult {
        brackets_ok: true,
        preprocess_ok: true,
        errors: Vec::new(),
    };

    // Check bracket matching via IR
    match Program::from_source(expanded) {
        Ok(_) => {}
        Err(e) => {
            result.brackets_ok = false;
            result.errors.push(format!("brackets: {}", e));
        }
    }

    Ok(result)
}

pub fn check_source(source: &str) -> CheckResult {
    let mut result = CheckResult {
        brackets_ok: true,
        preprocess_ok: true,
        errors: Vec::new(),
    };

    match Program::from_source(source) {
        Ok(_) => {}
        Err(e) => {
            result.brackets_ok = false;
            result.errors.push(format!("brackets: {}", e));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_valid_source() {
        let result = check_source("[+]");
        assert!(result.brackets_ok);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_check_unmatched_open() {
        let result = check_source("[+");
        assert!(!result.brackets_ok);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_check_unmatched_close() {
        let result = check_source("+]");
        assert!(!result.brackets_ok);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_check_empty_source() {
        let result = check_source("");
        assert!(result.brackets_ok);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_check_nested_brackets_valid() {
        let result = check_source("[[[]]]");
        assert!(result.brackets_ok);
    }

    #[test]
    fn test_check_file_hello_world() {
        let result = check_file(Path::new("tests/brainfuck_scripts/hello_world.bf")).unwrap();
        assert!(result.brackets_ok);
        assert!(result.preprocess_ok);
        assert!(result.errors.is_empty());
    }
}

pub fn check_and_report(path: &Path) -> Result<bool> {
    check_and_report_ex(path, Verbosity::Normal)
}

pub fn check_and_report_ex(path: &Path, verbosity: Verbosity) -> Result<bool> {
    let result = check_file(path)?;

    if result.errors.is_empty() {
        if !verbosity.is_quiet() {
            println!("{}: {}", path.display(), "OK".green());
        }
        Ok(true)
    } else {
        for err in &result.errors {
            println!("{}: {} {}", path.display(), "ERROR".red(), err);
        }
        Ok(false)
    }
}
