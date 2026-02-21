use anyhow::Result;
use std::path::Path;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;

pub fn run_file(path: &Path) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let mut interp = Interpreter::with_live_stdin(&expanded)?;
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}
