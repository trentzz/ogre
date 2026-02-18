use anyhow::{bail, Result};
use std::fs;

pub struct FormatOptions {
    pub indent: usize,
    pub linewidth: usize,
    pub grouping: usize,
    pub label_functions: bool,
    pub preserve_comments: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: 4,
            linewidth: 80,
            grouping: 5,
            label_functions: false,
            preserve_comments: false,
        }
    }
}

pub fn format_source(code: &str, opts: &FormatOptions) -> Result<String> {
    let mut lines: Vec<String> = vec![String::new()];
    let mut depth: usize = 0;

    // Pre-check: if nesting too deep, bail early
    let max_depth = code.chars().filter(|&c| c == '[').count();
    if max_depth * opts.indent + 10 > opts.linewidth {
        // Only an error if we actually reach that depth
        // We'll check inline below
    }

    // Track run of same operator for grouping
    let mut last_op: Option<char> = None;
    let mut run_len: usize = 0;

    let bf_ops = ['>', '<', '+', '-', '.', ',', '[', ']'];
    let is_bf = |c: char| bf_ops.contains(&c);

    let push_char = |lines: &mut Vec<String>, depth: usize, ch: char, opts: &FormatOptions| {
        let indent_str = " ".repeat(depth * opts.indent);
        let last = lines.last_mut().unwrap();
        if last.trim().is_empty() {
            // Line is empty/indent only — start fresh with proper indent
            *last = format!("{}{}", indent_str, ch);
        } else if last.len() + 1 <= opts.linewidth {
            last.push(ch);
        } else {
            // Wrap to new line
            lines.push(format!("{}{}", indent_str, ch));
        }
    };

    for ch in code.chars() {
        match ch {
            '[' => {
                // Check depth won't be unreadable
                let new_depth = depth + 1;
                if new_depth * opts.indent + 10 > opts.linewidth {
                    bail!(
                        "nesting depth {} × indent {} exceeds linewidth {} - 10",
                        new_depth,
                        opts.indent,
                        opts.linewidth
                    );
                }

                // Flush grouping state
                last_op = None;
                run_len = 0;

                // Start a new line for [
                let indent_str = " ".repeat(depth * opts.indent);
                if !lines.last().unwrap().trim().is_empty() {
                    lines.push(format!("{}[", indent_str));
                } else {
                    *lines.last_mut().unwrap() = format!("{}[", indent_str);
                }
                depth += 1;
                lines.push(String::new());
            }
            ']' => {
                // Flush grouping state
                last_op = None;
                run_len = 0;

                if depth > 0 {
                    depth -= 1;
                }
                let indent_str = " ".repeat(depth * opts.indent);
                if !lines.last().unwrap().trim().is_empty() {
                    lines.push(format!("{}]", indent_str));
                } else {
                    *lines.last_mut().unwrap() = format!("{}]", indent_str);
                }
                lines.push(String::new());
            }
            c if is_bf(c) => {
                // Handle grouping
                if Some(c) == last_op {
                    run_len += 1;
                } else {
                    last_op = Some(c);
                    run_len = 1;
                }

                // Insert space for grouping boundary
                if opts.grouping > 0 && run_len > 1 && (run_len - 1) % opts.grouping == 0 {
                    let indent_str = " ".repeat(depth * opts.indent);
                    let last = lines.last_mut().unwrap();
                    if last.trim().is_empty() {
                        *last = format!("{}{}", indent_str, c);
                    } else if last.len() + 2 <= opts.linewidth {
                        last.push(' ');
                        last.push(c);
                    } else {
                        lines.push(format!("{}{}", indent_str, c));
                    }
                } else {
                    push_char(&mut lines, depth, c, opts);
                }
            }
            c => {
                // Non-BF character
                if opts.preserve_comments {
                    push_char(&mut lines, depth, c, opts);
                }
                // If not preserving comments, discard
            }
        }
    }

    // Remove trailing empty lines
    while lines.last().map(|l: &String| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }

    Ok(lines.join("\n") + "\n")
}

pub fn format_file(path: &str, opts: &FormatOptions) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let formatted = format_source(&source, opts)?;
    fs::write(path, formatted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_loop() {
        let opts = FormatOptions {
            indent: 4,
            ..Default::default()
        };
        let result = format_source("[+]", &opts).unwrap();
        // The + should be indented by 4 spaces
        let lines: Vec<&str> = result.lines().collect();
        let inner = lines.iter().find(|l| l.contains('+')).unwrap();
        assert!(inner.starts_with("    "), "inner should be indented: {:?}", inner);
    }

    #[test]
    fn test_comments_stripped_by_default() {
        let opts = FormatOptions::default();
        let result = format_source("+ this is a comment +", &opts).unwrap();
        assert!(!result.contains("this"));
        assert!(!result.contains("comment"));
    }

    #[test]
    fn test_preserve_comments() {
        let opts = FormatOptions {
            preserve_comments: true,
            ..Default::default()
        };
        let result = format_source("+ comment +", &opts).unwrap();
        assert!(result.contains("comment"));
    }

    #[test]
    fn test_grouping() {
        let opts = FormatOptions {
            grouping: 5,
            ..Default::default()
        };
        // 10 + signs → should insert a space after 5th
        let result = format_source("++++++++++", &opts).unwrap();
        assert!(result.contains("+++++ +++++"), "got: {:?}", result);
    }

    #[test]
    fn test_nested_loop_indent() {
        let opts = FormatOptions {
            indent: 2,
            linewidth: 80,
            ..Default::default()
        };
        let result = format_source("[[+]]", &opts).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        let inner_plus = lines.iter().find(|l| l.contains('+')).unwrap();
        // depth 2 = 4 spaces indent
        assert!(inner_plus.starts_with("    "), "got: {:?}", inner_plus);
    }

    #[test]
    fn test_linewidth_wrap() {
        let opts = FormatOptions {
            indent: 0,
            linewidth: 10,
            grouping: 0,
            ..Default::default()
        };
        // 15 + should wrap since linewidth=10
        let result = format_source("+++++++++++++++", &opts).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1, "should have wrapped");
    }

    #[test]
    fn test_depth_exceeds_linewidth_errors() {
        let opts = FormatOptions {
            indent: 40,
            linewidth: 80,
            ..Default::default()
        };
        // depth 2 * 40 = 80, which ≥ 80 - 10 = 70
        assert!(format_source("[[+]]", &opts).is_err());
    }

    #[test]
    fn test_grouping_zero_no_spaces() {
        let opts = FormatOptions {
            grouping: 0,
            ..Default::default()
        };
        let result = format_source("++++++++++", &opts).unwrap();
        assert!(!result.contains(' ') || result.trim().chars().all(|c| c == '+' || c == '\n'));
    }
}
