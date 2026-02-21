use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::directive_parser::{
    skip_spaces, skip_whitespace, take_brace_body, take_identifier, take_quoted_string,
};

/// Two-pass macro preprocessor for @fn/@call/@import directives.
///
/// Pass 1 (collect): resolve @import recursively, accumulate @fn bodies,
/// return top-level BF code (with @call markers preserved).
///
/// Pass 2 (expand): replace every @call with the recursively-expanded body,
/// using a call-stack Vec for cycle detection.
pub struct Preprocessor {
    functions: HashMap<String, String>,
    imported: HashSet<PathBuf>,
}

impl Preprocessor {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            imported: HashSet::new(),
        }
    }

    /// Process a file on disk, resolving @import paths relative to the file's directory.
    pub fn process_file(path: &Path) -> Result<String> {
        let source = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", path, e))?;
        let base_dir = path.parent().unwrap_or(Path::new("."));
        Self::process_source(&source, base_dir)
    }

    /// Process a source string with @import paths resolved relative to `base_dir`.
    pub fn process_source(source: &str, base_dir: &Path) -> Result<String> {
        let mut pp = Self::new();
        let top_level = pp.collect(source, base_dir)?;
        let mut stack = Vec::new();
        pp.expand(&top_level, &mut stack)
    }

    // ---- Pass 1: collect ----

    fn collect(&mut self, source: &str, base_dir: &Path) -> Result<String> {
        let mut top_level = String::new();
        let chars: Vec<char> = source.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '@' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                i += 1; // skip '@'
                let keyword = take_identifier(&chars, &mut i);

                match keyword.as_str() {
                    "import" => {
                        skip_spaces(&chars, &mut i);
                        let path_str = take_quoted_string(&chars, &mut i)
                            .map_err(|e| anyhow::anyhow!("@import: {}", e))?;

                        let import_path = base_dir.join(&path_str);
                        // Use canonical path for cycle detection when possible
                        let canonical = import_path
                            .canonicalize()
                            .unwrap_or_else(|_| import_path.clone());

                        if self.imported.contains(&canonical) {
                            bail!("import cycle detected: {}", import_path.display());
                        }
                        self.imported.insert(canonical);

                        let imported_source = fs::read_to_string(&import_path)
                            .map_err(|e| anyhow::anyhow!("@import \"{}\": {}", path_str, e))?;
                        let import_base = import_path
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| PathBuf::from("."));

                        // Collect @fn definitions; discard top-level code from imports
                        self.collect(&imported_source, &import_base)?;
                    }

                    "fn" => {
                        skip_spaces(&chars, &mut i);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@fn: missing function name");
                        }
                        skip_whitespace(&chars, &mut i);
                        if i >= chars.len() || chars[i] != '{' {
                            bail!("@fn {}: expected '{{', found {:?}", name, chars.get(i));
                        }
                        i += 1; // skip '{'
                        let body = take_brace_body(&chars, &mut i)
                            .map_err(|e| anyhow::anyhow!("@fn {}: {}", name, e))?;
                        self.functions.insert(name, body);
                    }

                    "call" => {
                        skip_spaces(&chars, &mut i);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@call: missing function name");
                        }
                        // Preserve @call marker for expand pass
                        top_level.push_str("@call ");
                        top_level.push_str(&name);
                    }

                    other => {
                        bail!("unknown directive: @{}", other);
                    }
                }
            } else {
                top_level.push(chars[i]);
                i += 1;
            }
        }

        Ok(top_level)
    }

    // ---- Pass 2: expand ----

    fn expand(&self, code: &str, stack: &mut Vec<String>) -> Result<String> {
        let mut result = String::new();
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '@' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                i += 1; // skip '@'
                let keyword = take_identifier(&chars, &mut i);

                if keyword == "call" {
                    skip_spaces(&chars, &mut i);
                    let name = take_identifier(&chars, &mut i);

                    if stack.contains(&name) {
                        let mut cycle = stack.clone();
                        cycle.push(name.clone());
                        bail!("cycle detected: {}", cycle.join(" → "));
                    }

                    let body = self
                        .functions
                        .get(&name)
                        .ok_or_else(|| anyhow::anyhow!("unknown function: @call {}", name))?;

                    stack.push(name.clone());
                    let expanded = self.expand(body, stack)?;
                    stack.pop();

                    result.push_str(&expanded);
                } else {
                    // Other @directives in the expand phase — pass through (shouldn't occur)
                    result.push('@');
                    result.push_str(&keyword);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(src: &str) -> Result<String> {
        Preprocessor::process_source(src, Path::new("."))
    }

    #[test]
    fn test_plain_bf_unchanged() {
        let out = process("++[>+<-]>.").unwrap();
        assert_eq!(out, "++[>+<-]>.");
    }

    #[test]
    fn test_fn_call_inline() {
        let src = "@fn inc { + } @call inc @call inc";
        let out = process(src).unwrap();
        // top-level code between directives is whitespace/spaces
        // expand should replace @call inc with " + "
        assert!(out.contains('+'));
        assert!(!out.contains("@call"));
        assert!(!out.contains("@fn"));
    }

    #[test]
    fn test_fn_call_twice() {
        let out = process("@fn add2 { ++ } @call add2 @call add2").unwrap();
        assert_eq!(out.trim().replace(' ', ""), "++++");
    }

    #[test]
    fn test_fn_can_call_another_fn() {
        let src = "@fn inner { + } @fn outer { @call inner @call inner } @call outer";
        let out = process(src).unwrap();
        assert_eq!(out.trim().replace(' ', ""), "++");
    }

    #[test]
    fn test_cycle_detection_direct() {
        let src = "@fn a { @call b } @fn b { @call a } @call a";
        assert!(process(src).is_err());
        let err = process(src).unwrap_err().to_string();
        assert!(err.contains("cycle"));
    }

    #[test]
    fn test_cycle_detection_self() {
        let src = "@fn a { @call a } @call a";
        assert!(process(src).is_err());
    }

    #[test]
    fn test_unknown_call_errors() {
        let src = "@call nonexistent";
        assert!(process(src).is_err());
        let err = process(src).unwrap_err().to_string();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn test_fn_not_in_output() {
        // @fn definitions should not produce output themselves
        let src = "@fn noop { } +++";
        let out = process(src).unwrap();
        assert!(!out.contains("@fn"));
        assert!(!out.contains("noop"));
        // But the +++ at top-level should remain
        assert!(out.contains("+++"));
    }

    #[test]
    fn test_bf_around_calls() {
        // BF before and after @call should be preserved
        let src = "@fn bump { + } >[->+<] @call bump <";
        let out = process(src).unwrap();
        assert!(out.contains('>'));
        assert!(out.contains('<'));
        assert!(out.contains('+'));
        assert!(!out.contains("@call"));
    }

    #[test]
    fn test_empty_fn_body() {
        let out = process("@fn noop {} @call noop +++").unwrap();
        // @call noop should expand to empty string
        let trimmed = out.replace(' ', "").replace('\n', "");
        assert_eq!(trimmed, "+++");
    }

    #[test]
    fn test_unknown_directive_errors() {
        assert!(process("@unknown hello").is_err());
    }

    #[test]
    fn test_fn_missing_brace_errors() {
        assert!(process("@fn foo +").is_err());
    }

    #[test]
    fn test_import_nonexistent_file_errors() {
        assert!(process("@import \"nonexistent_xyz.bf\"").is_err());
    }
}
