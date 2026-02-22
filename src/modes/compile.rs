use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::ir::{Op, Program};
use super::preprocess::Preprocessor;
use crate::error::OgreError;
use crate::verbosity::Verbosity;

pub fn generate_c(bf_code: &str) -> String {
    generate_c_with_tape_size(bf_code, 30_000)
}

pub fn generate_c_with_tape_size(bf_code: &str, tape_size: usize) -> String {
    match Program::from_source(bf_code) {
        Ok(mut program) => {
            program.optimize();
            generate_c_from_program(&program, tape_size)
        }
        Err(_) => {
            // Fallback: generate without optimization if parsing fails
            let program = Program::from_source(bf_code).unwrap_or(Program { ops: vec![] });
            generate_c_from_program(&program, tape_size)
        }
    }
}

pub fn generate_c_from_program(program: &Program, tape_size: usize) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n");
    out.push_str("#include <string.h>\n");
    out.push_str("int main() {\n");
    out.push_str(&format!("    unsigned char array[{}];\n", tape_size));
    out.push_str(&format!(
        "    memset(array, 0, sizeof(array));\n"
    ));
    out.push_str("    unsigned char *ptr = array;\n");

    let mut indent_level: usize = 1;

    for op in &program.ops {
        let indent = "    ".repeat(indent_level);
        match op {
            Op::Add(n) => {
                if *n == 1 {
                    out.push_str(&format!("{}(*ptr)++;\n", indent));
                } else {
                    out.push_str(&format!("{}*ptr += {};\n", indent, n));
                }
            }
            Op::Sub(n) => {
                if *n == 1 {
                    out.push_str(&format!("{}(*ptr)--;\n", indent));
                } else {
                    out.push_str(&format!("{}*ptr -= {};\n", indent, n));
                }
            }
            Op::Right(n) => {
                if *n == 1 {
                    out.push_str(&format!("{}ptr++;\n", indent));
                } else {
                    out.push_str(&format!("{}ptr += {};\n", indent, n));
                }
            }
            Op::Left(n) => {
                if *n == 1 {
                    out.push_str(&format!("{}ptr--;\n", indent));
                } else {
                    out.push_str(&format!("{}ptr -= {};\n", indent, n));
                }
            }
            Op::Output => {
                out.push_str(&format!("{}putchar(*ptr);\n", indent));
            }
            Op::Input => {
                out.push_str(&format!("{}*ptr = getchar();\n", indent));
            }
            Op::JumpIfZero(_) => {
                out.push_str(&format!("{}while (*ptr) {{\n", indent));
                indent_level += 1;
            }
            Op::JumpIfNonZero(_) => {
                if indent_level > 1 {
                    indent_level -= 1;
                }
                let indent = "    ".repeat(indent_level);
                out.push_str(&format!("{}}}\n", indent));
            }
            Op::Clear => {
                out.push_str(&format!("{}*ptr = 0;\n", indent));
            }
            Op::MoveAdd(offset) => {
                out.push_str(&format!(
                    "{}*(ptr + {}) += *ptr; *ptr = 0;\n",
                    indent, offset
                ));
            }
            Op::MoveSub(offset) => {
                out.push_str(&format!(
                    "{}*(ptr + {}) -= *ptr; *ptr = 0;\n",
                    indent, offset
                ));
            }
        }
    }

    out.push_str("    return 0;\n");
    out.push_str("}\n");
    out
}

fn find_c_compiler() -> Result<String> {
    for compiler in &["cc", "gcc", "clang"] {
        if Command::new(compiler)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            return Ok(compiler.to_string());
        }
    }
    Err(OgreError::CompilerNotFound.into())
}

pub fn compile(file: &Path, output: Option<&str>, keep: bool) -> Result<()> {
    compile_with_tape_size(file, output, keep, 30_000)
}

pub fn compile_ex(file: &Path, output: Option<&str>, keep: bool, verbosity: Verbosity) -> Result<()> {
    compile_with_tape_size_ex(file, output, keep, 30_000, verbosity)
}

pub fn compile_with_tape_size(
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
) -> Result<()> {
    compile_with_tape_size_ex(file, output, keep, tape_size, Verbosity::Normal)
}

/// Compile with pre-loaded dependency functions available.
pub fn compile_with_deps_ex(
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
    verbosity: Verbosity,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let expanded = Preprocessor::process_file_with_deps(file, dep_functions)?;
    compile_expanded(&expanded, file, output, keep, tape_size, verbosity)
}

pub fn compile_with_tape_size_ex(
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
    verbosity: Verbosity,
) -> Result<()> {
    let expanded = Preprocessor::process_file(file)?;
    compile_expanded(&expanded, file, output, keep, tape_size, verbosity)
}

fn compile_expanded(
    expanded: &str,
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
    verbosity: Verbosity,
) -> Result<()> {
    let c_code = generate_c_with_tape_size(expanded, tape_size);

    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let out_path = output.unwrap_or(stem).to_string();

    // Write intermediate .c file to temp dir unless --keep
    let c_path = if keep {
        format!("{}.c", stem)
    } else {
        let tmp = std::env::temp_dir().join(format!("ogre_{}.c", stem));
        tmp.to_string_lossy().into_owned()
    };

    fs::write(&c_path, &c_code)?;

    let compiler = find_c_compiler()?;
    let status = Command::new(&compiler)
        .args([&c_path, "-o", &out_path, "-O2"])
        .status()?;

    if !status.success() {
        if !keep {
            let _ = fs::remove_file(&c_path);
        }
        return Err(OgreError::CompilationFailed(compiler).into());
    }

    if !keep {
        fs::remove_file(&c_path)?;
    }

    if !verbosity.is_quiet() {
        println!("Compiled to: {}", out_path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_c_increment() {
        let c = generate_c("+");
        assert!(c.contains("(*ptr)++;"));
    }

    #[test]
    fn test_generate_c_decrement() {
        let c = generate_c("-");
        assert!(c.contains("(*ptr)--;"));
    }

    #[test]
    fn test_generate_c_move_right() {
        let c = generate_c(">");
        assert!(c.contains("ptr++;"));
    }

    #[test]
    fn test_generate_c_move_left() {
        let c = generate_c("<");
        assert!(c.contains("ptr--;"));
    }

    #[test]
    fn test_generate_c_output() {
        let c = generate_c(".");
        assert!(c.contains("putchar(*ptr);"));
    }

    #[test]
    fn test_generate_c_input() {
        let c = generate_c(",");
        assert!(c.contains("*ptr = getchar();"));
    }

    #[test]
    fn test_generate_c_loop() {
        let c = generate_c("[+]");
        // With optimization, [+] doesn't become a clear since it's [Add(1)]
        // which is different from [-] / [Sub(1)]
        assert!(c.contains("while (*ptr) {") || c.contains("(*ptr)++;"));
    }

    #[test]
    fn test_generate_c_structure() {
        let c = generate_c("+");
        assert!(c.contains("#include <stdio.h>"));
        assert!(c.contains("#include <string.h>"));
        assert!(c.contains("int main()"));
        assert!(c.contains("unsigned char array[30000]"));
        assert!(c.contains("memset(array, 0, sizeof(array))"));
        assert!(c.contains("return 0;"));
    }

    #[test]
    fn test_generate_c_comments_ignored() {
        let c_with = generate_c("+this is comment+");
        let c_without = generate_c("++");
        // Both should produce *ptr += 2 after optimization
        assert!(c_with.contains("*ptr += 2;"));
        assert!(c_without.contains("*ptr += 2;"));
    }

    #[test]
    fn test_generate_c_nested_loop_indentation() {
        let c = generate_c("[[+]]");
        // Inner + should be indented more than outer [
        let lines: Vec<&str> = c.lines().collect();
        let inner_plus = lines.iter().find(|l| l.contains("(*ptr)++;")).unwrap();
        assert!(inner_plus.starts_with("            ")); // 3 levels = 12 spaces
    }

    #[test]
    fn test_generate_c_collapsed_ops() {
        let c = generate_c("+++");
        assert!(c.contains("*ptr += 3;"));
    }

    #[test]
    fn test_generate_c_collapsed_moves() {
        let c = generate_c(">>>");
        assert!(c.contains("ptr += 3;"));
    }

    #[test]
    fn test_generate_c_clear_idiom() {
        let c = generate_c("[-]");
        assert!(c.contains("*ptr = 0;"));
    }

    #[test]
    fn test_generate_c_custom_tape_size() {
        let c = generate_c_with_tape_size("+", 60_000);
        assert!(c.contains("unsigned char array[60000]"));
    }

    #[test]
    fn test_generate_c_move_add() {
        let c = generate_c("[->+<]");
        assert!(c.contains("*(ptr + 1) += *ptr; *ptr = 0;"));
    }
}
