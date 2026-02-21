use anyhow::{bail, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use super::preprocess::Preprocessor;

pub fn generate_c(bf_code: &str) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n");
    out.push_str("int main() {\n");
    out.push_str("    char array[30000] = {0};\n");
    out.push_str("    char *ptr = array;\n");

    let mut indent_level: usize = 1;
    let chars: Vec<char> = bf_code.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        let indent = "    ".repeat(indent_level);
        match ch {
            '>' | '<' | '+' | '-' => {
                // Collapse runs of identical ops
                let mut count = 1usize;
                while i + count < chars.len() && chars[i + count] == ch {
                    count += 1;
                }
                i += count;
                match ch {
                    '>' => {
                        if count == 1 {
                            out.push_str(&format!("{}ptr++;\n", indent));
                        } else {
                            out.push_str(&format!("{}ptr += {};\n", indent, count));
                        }
                    }
                    '<' => {
                        if count == 1 {
                            out.push_str(&format!("{}ptr--;\n", indent));
                        } else {
                            out.push_str(&format!("{}ptr -= {};\n", indent, count));
                        }
                    }
                    '+' => {
                        if count == 1 {
                            out.push_str(&format!("{}(*ptr)++;\n", indent));
                        } else {
                            out.push_str(&format!("{}*ptr += {};\n", indent, count));
                        }
                    }
                    '-' => {
                        if count == 1 {
                            out.push_str(&format!("{}(*ptr)--;\n", indent));
                        } else {
                            out.push_str(&format!("{}*ptr -= {};\n", indent, count));
                        }
                    }
                    _ => unreachable!(),
                }
                continue;
            }
            '.' => out.push_str(&format!("{}putchar(*ptr);\n", indent)),
            ',' => out.push_str(&format!("{}*ptr = getchar();\n", indent)),
            '[' => {
                out.push_str(&format!("{}while (*ptr) {{\n", indent));
                indent_level += 1;
            }
            ']' => {
                if indent_level > 1 {
                    indent_level -= 1;
                }
                let indent = "    ".repeat(indent_level);
                out.push_str(&format!("{}}}\n", indent));
            }
            _ => {} // comments ignored
        }
        i += 1;
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
    bail!("no C compiler found. Install gcc, clang, or ensure 'cc' is available on PATH")
}

pub fn compile(file: &Path, output: Option<&str>, keep: bool) -> Result<()> {
    let expanded = Preprocessor::process_file(file)?;
    let c_code = generate_c(&expanded);

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
        bail!("{} compilation failed", compiler);
    }

    if !keep {
        fs::remove_file(&c_path)?;
    }

    println!("Compiled to: {}", out_path);
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
        assert!(c.contains("while (*ptr) {"));
        assert!(c.contains("(*ptr)++;"));
    }

    #[test]
    fn test_generate_c_structure() {
        let c = generate_c("+");
        assert!(c.contains("#include <stdio.h>"));
        assert!(c.contains("int main()"));
        assert!(c.contains("char array[30000] = {0}"));
        assert!(c.contains("return 0;"));
    }

    #[test]
    fn test_generate_c_comments_ignored() {
        let c_with = generate_c("+this is comment+");
        let c_without = generate_c("++");
        // Both should produce collapsed increment
        assert!(c_with.contains("*ptr += 2;") || c_with.matches("(*ptr)++;").count() == 2);
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
}
