use anyhow::Result;
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

use super::directive_parser::{skip_spaces, skip_whitespace, take_brace_body, take_identifier};

pub struct FormatOptions {
    pub indent: usize,
    pub linewidth: usize,
    pub grouping: usize,
    pub label_functions: bool,
    pub preserve_comments: bool,
    pub check: bool,
    pub diff: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: 4,
            linewidth: 80,
            grouping: 5,
            label_functions: false,
            preserve_comments: false,
            check: false,
            diff: false,
        }
    }
}

// ---- Segment types for directive-aware formatting ----

enum SourceSegment {
    /// Pure BF code (no directives).
    BF(String),
    /// A complete directive to emit verbatim on its own line (`@import`, `@call`).
    Directive(String),
    /// An `@fn name { body }` definition — name and raw body.
    FnDef { name: String, body: String },
}

/// Parse a source string into alternating BF and directive segments.
fn parse_segments(source: &str) -> Result<Vec<SourceSegment>> {
    let mut segments: Vec<SourceSegment> = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut current_bf = String::new();

    while i < chars.len() {
        if chars[i] == '@' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
            // Flush accumulated BF
            if !current_bf.is_empty() {
                segments.push(SourceSegment::BF(current_bf.clone()));
                current_bf.clear();
            }

            i += 1; // skip '@'
            let keyword = take_identifier(&chars, &mut i);

            match keyword.as_str() {
                "import" => {
                    skip_spaces(&chars, &mut i);
                    if i < chars.len() && chars[i] == '"' {
                        i += 1;
                        let mut path = String::new();
                        while i < chars.len() && chars[i] != '"' {
                            path.push(chars[i]);
                            i += 1;
                        }
                        if i < chars.len() {
                            i += 1; // closing "
                        }
                        segments.push(SourceSegment::Directive(format!("@import \"{}\"", path)));
                    } else {
                        segments.push(SourceSegment::Directive("@import".to_string()));
                    }
                }
                "fn" => {
                    skip_spaces(&chars, &mut i);
                    let name = take_identifier(&chars, &mut i);
                    skip_whitespace(&chars, &mut i);
                    // Expect '{'
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let body = take_brace_body(&chars, &mut i)?;
                        segments.push(SourceSegment::FnDef { name, body });
                    } else {
                        // Malformed — treat remainder as BF comment
                        current_bf.push_str(&format!("@fn {}", name));
                    }
                }
                "call" => {
                    skip_spaces(&chars, &mut i);
                    let name = take_identifier(&chars, &mut i);
                    segments.push(SourceSegment::Directive(format!("@call {}", name)));
                }
                other => {
                    // Unknown directive — pass through as BF comment text
                    current_bf.push_str(&format!("@{}", other));
                }
            }
        } else {
            current_bf.push(chars[i]);
            i += 1;
        }
    }

    if !current_bf.is_empty() {
        segments.push(SourceSegment::BF(current_bf));
    }

    Ok(segments)
}

// ---- Core BF-only formatter ----

fn format_bf_only(code: &str, opts: &FormatOptions) -> Result<String> {
    let _ = opts.label_functions;

    let mut lines: Vec<String> = vec![String::new()];
    let mut depth: usize = 0;

    let mut last_op: Option<char> = None;
    let mut run_len: usize = 0;

    let bf_ops = ['>', '<', '+', '-', '.', ',', '[', ']'];
    let is_bf = |c: char| bf_ops.contains(&c);

    let push_char = |lines: &mut Vec<String>, depth: usize, ch: char, opts: &FormatOptions| {
        let indent_str = " ".repeat(depth * opts.indent);
        let last = lines.last_mut().unwrap();
        if last.trim().is_empty() {
            *last = format!("{}{}", indent_str, ch);
        } else if last.len() < opts.linewidth {
            last.push(ch);
        } else {
            lines.push(format!("{}{}", indent_str, ch));
        }
    };

    for ch in code.chars() {
        match ch {
            '[' => {
                last_op = None;
                run_len = 0;

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
                last_op = None;
                run_len = 0;

                depth = depth.saturating_sub(1);
                let indent_str = " ".repeat(depth * opts.indent);
                if !lines.last().unwrap().trim().is_empty() {
                    lines.push(format!("{}]", indent_str));
                } else {
                    *lines.last_mut().unwrap() = format!("{}]", indent_str);
                }
                lines.push(String::new());
            }
            c if is_bf(c) => {
                if Some(c) == last_op {
                    run_len += 1;
                } else {
                    last_op = Some(c);
                    run_len = 1;
                }

                if opts.grouping > 0 && run_len > 1 && (run_len - 1).is_multiple_of(opts.grouping) {
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
                if opts.preserve_comments {
                    push_char(&mut lines, depth, c, opts);
                }
            }
        }
    }

    while lines
        .last()
        .map(|l: &String| l.trim().is_empty())
        .unwrap_or(false)
    {
        lines.pop();
    }

    Ok(lines.join("\n") + "\n")
}

// ---- Public API ----

/// Format a brainfuck source string (may contain @fn/@call/@import directives).
///
/// Directives are preserved verbatim on their own lines.  BF segments (and @fn
/// bodies) are formatted with indentation, grouping, and line-wrapping.
pub fn format_source(code: &str, opts: &FormatOptions) -> Result<String> {
    // Fast path for pure BF (no directives)
    if !code.contains('@') {
        return format_bf_only(code, opts);
    }

    let segments = parse_segments(code)?;
    let mut output = String::new();

    for seg in segments {
        match seg {
            SourceSegment::BF(bf) => {
                let formatted = format_bf_only(&bf, opts)?;
                let trimmed = formatted.trim_end_matches('\n');
                if !trimmed.is_empty() {
                    if !output.is_empty() && !output.ends_with('\n') {
                        output.push('\n');
                    }
                    output.push_str(trimmed);
                    output.push('\n');
                }
            }
            SourceSegment::Directive(d) => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(&d);
                output.push('\n');
            }
            SourceSegment::FnDef { name, body } => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(&format!("@fn {} {{\n", name));
                let formatted_body = format_bf_only(&body, opts)?;
                for line in formatted_body.lines() {
                    if !line.trim().is_empty() {
                        output.push_str(&" ".repeat(opts.indent));
                        output.push_str(line);
                        output.push('\n');
                    }
                }
                output.push_str("}\n");
            }
        }
    }

    // Normalise to single trailing newline
    while output.ends_with("\n\n") {
        output.pop();
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }

    Ok(output)
}

/// Generate a colored unified diff between two strings.
/// Returns the diff string, or an empty string if they are identical.
pub fn generate_diff(original: &str, formatted: &str, filename: &str) -> String {
    if original == formatted {
        return String::new();
    }

    let diff = TextDiff::from_lines(original, formatted);
    let mut output = String::new();

    output.push_str(&format!("--- {}\n", filename).red().to_string());
    output.push_str(&format!("+++ {}\n", filename).green().to_string());

    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        output.push_str(&format!("{}", hunk.header()).cyan().to_string());
        for change in hunk.iter_changes() {
            let line = match change.tag() {
                ChangeTag::Delete => format!("-{}", change).red().to_string(),
                ChangeTag::Insert => format!("+{}", change).green().to_string(),
                ChangeTag::Equal => format!(" {}", change).to_string(),
            };
            output.push_str(&line);
            if change.missing_newline() {
                output.push('\n');
            }
        }
    }

    output
}

pub fn format_file(path: &Path, opts: &FormatOptions) -> Result<bool> {
    let source = fs::read_to_string(path)?;
    let formatted = format_source(&source, opts)?;
    if opts.diff {
        if source == formatted {
            return Ok(true);
        }
        let diff_output = generate_diff(&source, &formatted, &path.display().to_string());
        print!("{}", diff_output);
        return Ok(false);
    }
    if opts.check {
        let already_formatted = source == formatted;
        if already_formatted {
            println!("{}: already formatted", path.display());
        } else {
            println!("{}: would be reformatted", path.display());
        }
        return Ok(already_formatted);
    }
    fs::write(path, formatted)?;
    Ok(true)
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
        let lines: Vec<&str> = result.lines().collect();
        let inner = lines.iter().find(|l| l.contains('+')).unwrap();
        assert!(
            inner.starts_with("    "),
            "inner should be indented: {:?}",
            inner
        );
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
        let result = format_source("+++++++++++++++", &opts).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1, "should have wrapped");
    }

    #[test]
    fn test_deep_nesting_still_formats() {
        let opts = FormatOptions {
            indent: 40,
            linewidth: 80,
            ..Default::default()
        };
        // Should not error — formatter should always produce output
        let result = format_source("[[+]]", &opts);
        assert!(result.is_ok());
        assert!(result.unwrap().contains('+'));
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

    #[test]
    fn test_directive_import_preserved() {
        let opts = FormatOptions::default();
        let src = "@import \"lib/io.bf\"\n+++";
        let out = format_source(src, &opts).unwrap();
        assert!(out.contains("@import \"lib/io.bf\""), "got: {:?}", out);
        assert!(out.contains("+++"));
    }

    #[test]
    fn test_directive_call_preserved() {
        let opts = FormatOptions::default();
        let src = "@fn greet { +++.--- } @call greet";
        let out = format_source(src, &opts).unwrap();
        assert!(out.contains("@fn greet {"), "got: {:?}", out);
        assert!(out.contains("@call greet"), "got: {:?}", out);
        assert!(out.contains('}'));
    }

    #[test]
    fn test_diff_identical_returns_empty() {
        let source = "+++\n";
        let formatted = "+++\n";
        let diff = generate_diff(source, formatted, "test.bf");
        assert!(diff.is_empty(), "identical files should produce no diff");
    }

    #[test]
    fn test_diff_different_returns_content() {
        let source = "+++---\n";
        let opts = FormatOptions::default();
        let formatted = format_source(source, &opts).unwrap();
        // If formatted differs from source, we should get a non-empty diff
        if source != formatted {
            let diff = generate_diff(source, &formatted, "test.bf");
            assert!(!diff.is_empty(), "different files should produce a diff");
            // The diff should contain the filename
            assert!(diff.contains("test.bf"), "diff should mention filename");
        }
    }

    #[test]
    fn test_diff_does_not_modify_file() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.bf");
        let original = "+++[>+++<-]>.";
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(original.as_bytes())
            .unwrap();

        let opts = FormatOptions {
            diff: true,
            ..Default::default()
        };
        let _ = format_file(&file_path, &opts);

        // File should be unchanged
        let after = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(after, original, "diff mode should not modify the file");
    }

    #[test]
    fn test_diff_already_formatted_returns_true() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.bf");
        // Write already-formatted content
        let opts = FormatOptions::default();
        let formatted = format_source("+++", &opts).unwrap();
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(formatted.as_bytes())
            .unwrap();

        let diff_opts = FormatOptions {
            diff: true,
            ..Default::default()
        };
        let result = format_file(&file_path, &diff_opts).unwrap();
        assert!(result, "already-formatted file should return true");
    }

    #[test]
    fn test_diff_unformatted_returns_false() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.bf");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"+++[>+++<-]>.")
            .unwrap();

        let opts = FormatOptions {
            diff: true,
            ..Default::default()
        };
        let result = format_file(&file_path, &opts).unwrap();
        assert!(!result, "unformatted file should return false");
    }

    #[test]
    fn test_fn_body_formatted() {
        let opts = FormatOptions {
            indent: 4,
            grouping: 0,
            ..Default::default()
        };
        let src = "@fn inc { [+] }";
        let out = format_source(src, &opts).unwrap();
        assert!(out.contains("@fn inc {"), "got: {:?}", out);
        assert!(out.contains('}'));
        let lines: Vec<&str> = out.lines().collect();
        let plus_line = lines.iter().find(|l| l.contains('+'));
        assert!(plus_line.is_some(), "body should contain '+': {:?}", out);
    }
}
