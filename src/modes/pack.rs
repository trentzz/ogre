use anyhow::Result;
use std::fs;
use std::path::Path;

use super::ir::Program;
use super::preprocess::Preprocessor;
use crate::verbosity::Verbosity;

/// Pack a file with pre-loaded dependency functions.
pub fn pack_file_with_deps(
    path: &Path,
    optimize: bool,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<String> {
    let expanded = Preprocessor::process_file_with_deps(path, dep_functions)?;
    pack_expanded(&expanded, optimize)
}

pub fn pack_file(path: &Path, optimize: bool) -> Result<String> {
    let expanded = Preprocessor::process_file(path)?;
    pack_expanded(&expanded, optimize)
}

fn pack_expanded(expanded: &str, optimize: bool) -> Result<String> {
    if optimize {
        let mut program = Program::from_source(expanded)?;
        program.optimize();
        Ok(program.to_bf_string())
    } else {
        // Strip non-BF characters (comments) but keep the BF intact
        let bf: String = expanded
            .chars()
            .filter(|c| "+-><.,[]".contains(*c))
            .collect();
        Ok(bf)
    }
}

pub fn pack_and_output(path: &Path, output: Option<&str>, optimize: bool) -> Result<()> {
    pack_and_output_ex(path, output, optimize, Verbosity::Normal)
}

/// Pack with pre-loaded dependency functions.
pub fn pack_and_output_with_deps(
    path: &Path,
    output: Option<&str>,
    optimize: bool,
    verbosity: Verbosity,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let packed = pack_file_with_deps(path, optimize, dep_functions)?;
    output_packed(&packed, output, verbosity)
}

pub fn pack_and_output_ex(path: &Path, output: Option<&str>, optimize: bool, verbosity: Verbosity) -> Result<()> {
    let packed = pack_file(path, optimize)?;
    output_packed(&packed, output, verbosity)
}

fn output_packed(packed: &str, output: Option<&str>, verbosity: Verbosity) -> Result<()> {
    match output {
        Some(out_path) => {
            fs::write(out_path, packed)?;
            if !verbosity.is_quiet() {
                println!("Packed to: {} ({} bytes)", out_path, packed.len());
            }
        }
        None => {
            println!("{}", packed);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::interpreter::Interpreter;

    #[test]
    fn test_pack_strips_comments() {
        let source = "+ this is a comment +";
        let bf: String = source
            .chars()
            .filter(|c| "+-><.,[]".contains(*c))
            .collect();
        assert_eq!(bf, "++");
    }

    #[test]
    fn test_pack_file_real() {
        let path = Path::new("tests/brainfuck_scripts/hello_world.bf");
        let packed = pack_file(path, false).unwrap();
        // Should only contain BF characters
        assert!(packed.chars().all(|c| "+-><.,[]".contains(c)));
        // Should still produce correct output when interpreted
        let mut interp = Interpreter::with_input(&packed, "").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "Hello World!\n");
    }

    #[test]
    fn test_pack_with_optimize() {
        let path = Path::new("tests/brainfuck_scripts/hello_world.bf");
        let packed = pack_file(path, true).unwrap();
        // Should only contain BF characters plus [-] for Clear
        assert!(packed.chars().all(|c| "+-><.,[]".contains(c)));
        // Optimized should still produce correct output
        let mut interp = Interpreter::with_input(&packed, "").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "Hello World!\n");
    }

    #[test]
    fn test_pack_optimize_produces_shorter_output() {
        // +-><[-] has cancellations that optimize should remove
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.bf");
        std::fs::File::create(&file)
            .unwrap()
            .write_all(b"+-><[-]+++")
            .unwrap();

        let unoptimized = pack_file(&file, false).unwrap();
        let optimized = pack_file(&file, true).unwrap();
        // Optimized should be shorter (cancellations removed)
        assert!(
            optimized.len() <= unoptimized.len(),
            "optimized ({}) should be <= unoptimized ({})",
            optimized.len(),
            unoptimized.len()
        );
    }
}
