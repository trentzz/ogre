use anyhow::{bail, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn generate_c(bf_code: &str) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n");
    out.push_str("int main() {\n");
    out.push_str("    char array[30000] = {0};\n");
    out.push_str("    char *ptr = array;\n");

    let mut indent_level: usize = 1;

    for ch in bf_code.chars() {
        let indent = "    ".repeat(indent_level);
        match ch {
            '>' => out.push_str(&format!("{}ptr++;\n", indent)),
            '<' => out.push_str(&format!("{}ptr--;\n", indent)),
            '+' => out.push_str(&format!("{}(*ptr)++;\n", indent)),
            '-' => out.push_str(&format!("{}(*ptr)--;\n", indent)),
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
    }

    out.push_str("    return 0;\n");
    out.push_str("}\n");
    out
}

pub fn compile(file: &str, output: Option<&str>, keep: bool) -> Result<()> {
    let source = fs::read_to_string(file)?;
    let c_code = generate_c(&source);

    let input_path = Path::new(file);
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let c_path = format!("{}.c", stem);
    let out_path = output.unwrap_or(stem).to_string();

    fs::write(&c_path, &c_code)?;

    let status = Command::new("gcc")
        .args([&c_path, "-o", &out_path])
        .status()?;

    if !status.success() {
        if !keep {
            let _ = fs::remove_file(&c_path);
        }
        bail!("gcc compilation failed");
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
        // Both should have same C structure
        assert_eq!(
            c_with.matches("(*ptr)++;").count(),
            c_without.matches("(*ptr)++;").count()
        );
    }

    #[test]
    fn test_generate_c_nested_loop_indentation() {
        let c = generate_c("[[+]]");
        // Inner + should be indented more than outer [
        let lines: Vec<&str> = c.lines().collect();
        let inner_plus = lines.iter().find(|l| l.contains("(*ptr)++;")).unwrap();
        assert!(inner_plus.starts_with("            ")); // 3 levels = 12 spaces
    }
}
