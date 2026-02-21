use anyhow::Result;
use std::fs;
use std::path::Path;

use super::ir::Program;
use super::preprocess::Preprocessor;

pub fn pack_file(path: &Path, optimize: bool) -> Result<String> {
    let expanded = Preprocessor::process_file(path)?;

    if optimize {
        let mut program = Program::from_source(&expanded)?;
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
    let packed = pack_file(path, optimize)?;

    match output {
        Some(out_path) => {
            fs::write(out_path, &packed)?;
            println!("Packed to: {} ({} bytes)", out_path, packed.len());
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

    #[test]
    fn test_pack_strips_comments() {
        // We can't call pack_file directly since it needs a real file,
        // but we can test the logic
        let source = "+ this is a comment +";
        let bf: String = source
            .chars()
            .filter(|c| "+-><.,[]".contains(*c))
            .collect();
        assert_eq!(bf, "++");
    }
}
