use anyhow::Result;
use std::path::Path;

use super::interpreter::{Interpreter, DEFAULT_TAPE_SIZE};
use super::preprocess::Preprocessor;

pub fn run_file(path: &Path) -> Result<()> {
    run_file_with_tape_size(path, DEFAULT_TAPE_SIZE)
}

pub fn run_file_with_tape_size(path: &Path, tape_size: usize) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let mut interp = Interpreter::with_live_stdin_and_tape_size(&expanded, tape_size)?;
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}
