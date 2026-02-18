use anyhow::Result;
use std::fs;

use super::interpreter::Interpreter;

pub fn run_file(path: &str) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let mut interp = Interpreter::with_live_stdin(&source)?;
    interp.run()?;
    print!("{}", interp.output_as_string());
    Ok(())
}
