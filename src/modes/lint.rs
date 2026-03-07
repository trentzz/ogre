use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

use super::preprocess::Preprocessor;

/// A lint warning with location and description.
#[derive(Debug, Clone)]
pub struct LintWarning {
    pub rule: String,
    pub message: String,
    pub line: Option<usize>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Lint a brainfunct/brainfuck source file.
pub fn lint_source(source: &str) -> Vec<LintWarning> {
    let mut warnings = Vec::new();

    // Rule: deep-nesting - loops nested deeper than 5
    check_deep_nesting(source, &mut warnings);

    // Rule: suspicious-pattern - common no-ops
    check_suspicious_patterns(source, &mut warnings);

    // Rule: redundant-clear - [-] at start or after another [-]
    check_redundant_clear(source, &mut warnings);

    // Rule: unbalanced-pointer - pointer doesn't return to origin
    check_unbalanced_pointer(source, &mut warnings);

    // Rule: missing-doc - @fn without @doc
    check_missing_doc(source, &mut warnings);

    // Rule: long-function - function body exceeds 500 BF chars
    check_long_function(source, &mut warnings);

    // Rule: unused-import - @import without corresponding @call
    check_unused_import(source, &mut warnings);

    warnings
}

fn check_deep_nesting(source: &str, warnings: &mut Vec<LintWarning>) {
    let max_depth = 5;
    let mut depth: usize = 0;
    let mut line_num: usize = 1;

    for ch in source.chars() {
        match ch {
            '\n' => line_num += 1,
            '[' => {
                depth += 1;
                if depth > max_depth {
                    warnings.push(LintWarning {
                        rule: "deep-nesting".to_string(),
                        message: format!(
                            "loop nesting depth {} exceeds maximum of {}",
                            depth, max_depth
                        ),
                        line: Some(line_num),
                        severity: Severity::Warning,
                    });
                }
            }
            ']' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
}

fn check_suspicious_patterns(source: &str, warnings: &mut Vec<LintWarning>) {
    let bf_only: String = source.chars().filter(|c| "+-><.,[]".contains(*c)).collect();
    let bytes = bf_only.as_bytes();

    // Map bf positions back to line numbers
    let line_for_bf_pos = |bf_pos: usize| -> usize {
        let mut line = 1;
        let mut bf_idx = 0;
        for ch in source.chars() {
            if ch == '\n' {
                line += 1;
            }
            if "+-><.,[]".contains(ch) {
                if bf_idx == bf_pos {
                    return line;
                }
                bf_idx += 1;
            }
        }
        line
    };

    // Check for >< and <> patterns (no-ops)
    for i in 0..bytes.len().saturating_sub(1) {
        if (bytes[i] == b'>' && bytes[i + 1] == b'<') || (bytes[i] == b'<' && bytes[i + 1] == b'>')
        {
            warnings.push(LintWarning {
                rule: "suspicious-pattern".to_string(),
                message: format!(
                    "adjacent {}{} is a no-op",
                    bytes[i] as char,
                    bytes[i + 1] as char
                ),
                line: Some(line_for_bf_pos(i)),
                severity: Severity::Warning,
            });
        }
        if (bytes[i] == b'+' && bytes[i + 1] == b'-') || (bytes[i] == b'-' && bytes[i + 1] == b'+')
        {
            warnings.push(LintWarning {
                rule: "suspicious-pattern".to_string(),
                message: format!(
                    "adjacent {}{} is a no-op",
                    bytes[i] as char,
                    bytes[i + 1] as char
                ),
                line: Some(line_for_bf_pos(i)),
                severity: Severity::Warning,
            });
        }
    }
}

fn check_redundant_clear(source: &str, warnings: &mut Vec<LintWarning>) {
    let bf_only: String = source.chars().filter(|c| "+-><.,[]".contains(*c)).collect();

    // [-] at position 0 is redundant (cell starts at 0)
    if bf_only.starts_with("[-]") || bf_only.starts_with("[+]") {
        let line = source
            .chars()
            .take_while(|c| !"[]".contains(*c))
            .filter(|c| *c == '\n')
            .count()
            + 1;
        warnings.push(LintWarning {
            rule: "redundant-clear".to_string(),
            message: "clear on cell that starts at zero".to_string(),
            line: Some(line),
            severity: Severity::Warning,
        });
    }

    // [-][-] is redundant (double clear)
    if bf_only.contains("[-][-]") || bf_only.contains("[+][+]") {
        let pos = bf_only.find("[-][-]").or_else(|| bf_only.find("[+][+]"));
        if pos.is_some() {
            warnings.push(LintWarning {
                rule: "redundant-clear".to_string(),
                message: "consecutive clear operations; second clear is redundant".to_string(),
                line: None,
                severity: Severity::Warning,
            });
        }
    }
}

fn check_unbalanced_pointer(source: &str, warnings: &mut Vec<LintWarning>) {
    // Only check top-level pointer balance (outside loops)
    let mut offset: i64 = 0;
    let mut depth: usize = 0;

    for ch in source.chars() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            '>' if depth == 0 => offset += 1,
            '<' if depth == 0 => offset -= 1,
            _ => {}
        }
    }

    if offset != 0 {
        warnings.push(LintWarning {
            rule: "unbalanced-pointer".to_string(),
            message: format!(
                "data pointer ends at offset {} from start (outside loops)",
                offset
            ),
            line: None,
            severity: Severity::Warning,
        });
    }
}

fn check_missing_doc(source: &str, warnings: &mut Vec<LintWarning>) {
    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("@fn ") {
            // Check if previous non-empty line is @doc
            let has_doc = (0..i)
                .rev()
                .find(|&j| !lines[j].trim().is_empty())
                .is_some_and(|j| lines[j].trim().starts_with("@doc"));

            if !has_doc {
                let fn_name = trimmed
                    .strip_prefix("@fn ")
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or("?");
                warnings.push(LintWarning {
                    rule: "missing-doc".to_string(),
                    message: format!("function '{}' has no @doc comment", fn_name),
                    line: Some(i + 1),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

fn check_long_function(source: &str, warnings: &mut Vec<LintWarning>) {
    let max_len = 500;
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("@fn ") {
            let fn_name = trimmed
                .strip_prefix("@fn ")
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("?");

            // Find the body between { and }
            let mut depth = 0;
            let mut body_len = 0;
            let mut in_body = false;

            for line2 in &lines[i..] {
                for ch in line2.chars() {
                    if ch == '{' {
                        depth += 1;
                        in_body = true;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else if in_body && "+-><.,[]".contains(ch) {
                        body_len += 1;
                    }
                }
                if depth == 0 && in_body {
                    break;
                }
            }

            if body_len > max_len {
                warnings.push(LintWarning {
                    rule: "long-function".to_string(),
                    message: format!(
                        "function '{}' has {} BF instructions (max {})",
                        fn_name, body_len, max_len
                    ),
                    line: Some(i + 1),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

fn check_unused_import(source: &str, warnings: &mut Vec<LintWarning>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut imports: Vec<(usize, String)> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("@import ") {
            let path = trimmed
                .strip_prefix("@import ")
                .map(|s| s.trim().trim_matches('"'))
                .unwrap_or("");
            if !path.is_empty() {
                imports.push((i + 1, path.to_string()));
            }
        }
    }

    // Check if any @call references functions from the import
    // Simple heuristic: if there's no @call at all, imports are unused
    let has_calls = source.contains("@call ");

    if !has_calls && !imports.is_empty() {
        for (line_num, path) in &imports {
            warnings.push(LintWarning {
                rule: "unused-import".to_string(),
                message: format!("imported '{}' but no @call directives found", path),
                line: Some(*line_num),
                severity: Severity::Warning,
            });
        }
    }
}

/// Format lint warnings for display.
pub fn format_warnings(path: &Path, warnings: &[LintWarning]) -> String {
    let mut out = String::new();

    if warnings.is_empty() {
        out.push_str(&format!(
            "{}: {}\n",
            path.display(),
            "no lint warnings".green()
        ));
        return out;
    }

    for w in warnings {
        let location = match w.line {
            Some(l) => format!("{}:{}", path.display(), l),
            None => format!("{}", path.display()),
        };

        let severity_str = match w.severity {
            Severity::Warning => "warning".yellow().to_string(),
            Severity::Error => "error".red().to_string(),
        };

        out.push_str(&format!(
            "{}: {} [{}] {}\n",
            location, severity_str, w.rule, w.message
        ));
    }

    out.push_str(&format!(
        "\n{} warning(s) in {}\n",
        warnings.len(),
        path.display()
    ));

    out
}

/// Lint a file and print results.
pub fn lint_file(path: &Path, preprocess: bool) -> Result<Vec<LintWarning>> {
    let source = fs::read_to_string(path)?;

    let mut warnings = lint_source(&source);

    // If preprocessing, also lint the expanded output
    if preprocess {
        if let Ok(expanded) = Preprocessor::process_file(path) {
            let expanded_warnings = lint_source(&expanded);
            for w in expanded_warnings {
                // Avoid duplicates
                if !warnings
                    .iter()
                    .any(|existing| existing.rule == w.rule && existing.message == w.message)
                {
                    warnings.push(w);
                }
            }
        }
    }

    print!("{}", format_warnings(path, &warnings));
    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_warnings_clean_code() {
        let warnings = lint_source("+++[>+<-]>.");
        let has_suspicious = warnings.iter().any(|w| w.rule == "suspicious-pattern");
        // This code has no suspicious patterns (>+< is not ><)
        assert!(!has_suspicious);
    }

    #[test]
    fn test_deep_nesting() {
        let source = "[[[[[[+]]]]]]"; // depth 6
        let warnings = lint_source(source);
        assert!(warnings.iter().any(|w| w.rule == "deep-nesting"));
    }

    #[test]
    fn test_no_deep_nesting() {
        let source = "[[+]]"; // depth 2
        let warnings = lint_source(source);
        assert!(!warnings.iter().any(|w| w.rule == "deep-nesting"));
    }

    #[test]
    fn test_suspicious_pattern_move() {
        let warnings = lint_source("><");
        assert!(warnings.iter().any(|w| w.rule == "suspicious-pattern"));
    }

    #[test]
    fn test_suspicious_pattern_add_sub() {
        let warnings = lint_source("+-");
        assert!(warnings.iter().any(|w| w.rule == "suspicious-pattern"));
    }

    #[test]
    fn test_redundant_clear_at_start() {
        let warnings = lint_source("[-]+");
        assert!(warnings.iter().any(|w| w.rule == "redundant-clear"));
    }

    #[test]
    fn test_redundant_double_clear() {
        let warnings = lint_source("+++[-][-]");
        assert!(warnings.iter().any(|w| w.rule == "redundant-clear"));
    }

    #[test]
    fn test_unbalanced_pointer() {
        let warnings = lint_source(">>>");
        assert!(warnings.iter().any(|w| w.rule == "unbalanced-pointer"));
    }

    #[test]
    fn test_balanced_pointer() {
        let warnings = lint_source(">>><<<");
        assert!(!warnings.iter().any(|w| w.rule == "unbalanced-pointer"));
    }

    #[test]
    fn test_missing_doc() {
        let source = "@fn my_func {\n+\n}\n";
        let warnings = lint_source(source);
        assert!(warnings.iter().any(|w| w.rule == "missing-doc"));
    }

    #[test]
    fn test_has_doc() {
        let source = "@doc Does something\n@fn my_func {\n+\n}\n";
        let warnings = lint_source(source);
        assert!(!warnings.iter().any(|w| w.rule == "missing-doc"));
    }

    #[test]
    fn test_unused_import() {
        let source = "@import \"std/io.bf\"\n+++\n";
        let warnings = lint_source(source);
        assert!(warnings.iter().any(|w| w.rule == "unused-import"));
    }

    #[test]
    fn test_import_with_call() {
        let source = "@import \"std/io.bf\"\n@call print_newline\n";
        let warnings = lint_source(source);
        assert!(!warnings.iter().any(|w| w.rule == "unused-import"));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Error), "error");
    }

    #[test]
    fn test_format_warnings_empty() {
        let out = format_warnings(Path::new("test.bf"), &[]);
        assert!(out.contains("no lint warnings"));
    }

    #[test]
    fn test_format_warnings_with_warning() {
        let warnings = vec![LintWarning {
            rule: "test-rule".to_string(),
            message: "test message".to_string(),
            line: Some(5),
            severity: Severity::Warning,
        }];
        let out = format_warnings(Path::new("test.bf"), &warnings);
        assert!(out.contains("test-rule"));
        assert!(out.contains("test message"));
        assert!(out.contains("1 warning(s)"));
    }
}
