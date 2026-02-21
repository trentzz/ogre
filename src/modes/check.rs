use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use super::ir::Program;
use super::preprocess::Preprocessor;

pub struct CheckResult {
    pub brackets_ok: bool,
    pub preprocess_ok: bool,
    pub errors: Vec<String>,
}

pub fn check_file(path: &Path) -> Result<CheckResult> {
    let mut result = CheckResult {
        brackets_ok: true,
        preprocess_ok: true,
        errors: Vec::new(),
    };

    // Check preprocessing (imports, calls, cycles)
    let expanded = match Preprocessor::process_file(path) {
        Ok(s) => s,
        Err(e) => {
            result.preprocess_ok = false;
            result.errors.push(format!("preprocess: {}", e));
            return Ok(result);
        }
    };

    // Check bracket matching via IR
    match Program::from_source(&expanded) {
        Ok(_) => {}
        Err(e) => {
            result.brackets_ok = false;
            result.errors.push(format!("brackets: {}", e));
        }
    }

    Ok(result)
}

pub fn check_and_report(path: &Path) -> Result<bool> {
    let result = check_file(path)?;

    if result.errors.is_empty() {
        println!("{}: {}", path.display(), "OK".green());
        Ok(true)
    } else {
        for err in &result.errors {
            println!("{}: {} {}", path.display(), "ERROR".red(), err);
        }
        Ok(false)
    }
}
