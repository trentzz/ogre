use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::directive_parser::{
    skip_spaces, skip_whitespace, take_brace_body, take_identifier, take_quoted_string,
};
use super::source_map::{SourceLocation, SourceMap};
use crate::error::OgreError;

/// Compute Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_len]
}

/// Find close matches for `name` among `candidates` (Levenshtein distance <= max_dist).
fn find_suggestions(name: &str, candidates: &[&str], max_dist: usize) -> Vec<String> {
    let mut matches: Vec<(usize, String)> = candidates
        .iter()
        .filter_map(|c| {
            let d = levenshtein(name, c);
            if d <= max_dist && d > 0 {
                Some((d, c.to_string()))
            } else {
                None
            }
        })
        .collect();
    matches.sort_by_key(|(d, _)| *d);
    matches.into_iter().map(|(_, s)| s).collect()
}

/// Build a helpful error message for unknown function calls.
fn unknown_function_message(name: &str, known_functions: &HashMap<String, String>) -> String {
    let candidates: Vec<&str> = known_functions.keys().map(|s| s.as_str()).collect();
    let suggestions = find_suggestions(name, &candidates, 3);

    let mut msg = format!("unknown function: '{}'", name);
    if let Some(best) = suggestions.first() {
        msg.push_str(&format!(". Did you mean '{}'?", best));
    }

    // Check if the function exists in an unimported stdlib module
    let mut available_in: Vec<String> = Vec::new();
    for module_name in stdlib_modules() {
        if let Some(source) = get_stdlib_module(module_name) {
            // Quick check: does this module define a function with this name?
            let pattern = format!("@fn {} {{", name);
            if source.contains(&pattern) {
                available_in.push(module_name.to_string());
            }
        }
    }
    if !available_in.is_empty() {
        msg.push_str(&format!(
            "\n  hint: '{}' is defined in {}. Add: @import \"std/{}.bf\"",
            name,
            available_in
                .iter()
                .map(|m| format!("std/{}.bf", m))
                .collect::<Vec<_>>()
                .join(", "),
            available_in[0]
        ));
    }

    msg
}

/// Build a helpful error message for unknown stdlib module imports.
fn unknown_module_message(name: &str) -> String {
    let modules = stdlib_modules();
    let candidates: Vec<&str> = modules.to_vec();
    let suggestions = find_suggestions(name, &candidates, 3);

    let mut msg = format!("unknown standard library module: '{}'", name);
    if let Some(best) = suggestions.first() {
        msg.push_str(&format!(". Did you mean '{}'?", best));
    }
    msg.push_str(&format!("\n  available modules: {}", modules.join(", ")));
    msg
}

// Embedded standard library modules
const STDLIB_IO: &str = include_str!("../../stdlib/io.bf");
const STDLIB_MATH: &str = include_str!("../../stdlib/math.bf");
const STDLIB_MEMORY: &str = include_str!("../../stdlib/memory.bf");
const STDLIB_ASCII: &str = include_str!("../../stdlib/ascii.bf");
const STDLIB_DEBUG: &str = include_str!("../../stdlib/debug.bf");
const STDLIB_STRING: &str = include_str!("../../stdlib/string.bf");
const STDLIB_LOGIC: &str = include_str!("../../stdlib/logic.bf");

/// Get the source code for a standard library module by name.
pub fn get_stdlib_module(name: &str) -> Option<&'static str> {
    match name {
        "io" => Some(STDLIB_IO),
        "math" => Some(STDLIB_MATH),
        "memory" => Some(STDLIB_MEMORY),
        "ascii" => Some(STDLIB_ASCII),
        "debug" => Some(STDLIB_DEBUG),
        "string" => Some(STDLIB_STRING),
        "logic" => Some(STDLIB_LOGIC),
        _ => None,
    }
}

/// List all available standard library module names.
pub fn stdlib_modules() -> &'static [&'static str] {
    &["ascii", "debug", "io", "logic", "math", "memory", "string"]
}

/// Result type for preprocessing with documentation: (expanded_code, functions, fn_docs).
pub type PreprocessResult = (String, HashMap<String, String>, HashMap<String, String>);

/// Two-pass macro preprocessor for @fn/@call/@import directives.
///
/// Pass 1 (collect): resolve @import recursively, accumulate @fn bodies,
/// return top-level BF code (with @call markers preserved).
///
/// Pass 2 (expand): replace every @call with the recursively-expanded body,
/// using a call-stack Vec for cycle detection.
pub struct Preprocessor {
    functions: HashMap<String, String>,
    fn_docs: HashMap<String, String>,
    constants: HashMap<String, usize>,
    imported: HashSet<PathBuf>,
    /// Track which file each @fn body came from.
    fn_origins: HashMap<String, PathBuf>,
    /// Source map being built (only when map mode is enabled).
    source_map: Option<SourceMap>,
    /// Whether to build a source map during processing.
    build_map: bool,
}

impl Preprocessor {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            fn_docs: HashMap::new(),
            constants: HashMap::new(),
            imported: HashSet::new(),
            fn_origins: HashMap::new(),
            source_map: None,
            build_map: false,
        }
    }

    fn new_with_map() -> Self {
        Self {
            functions: HashMap::new(),
            fn_docs: HashMap::new(),
            constants: HashMap::new(),
            imported: HashSet::new(),
            fn_origins: HashMap::new(),
            source_map: Some(SourceMap::new()),
            build_map: true,
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

    /// Process a file and return the preprocessor state (for doc generation).
    /// Returns (expanded_code, functions, fn_docs).
    pub fn process_file_with_docs(path: &Path) -> Result<PreprocessResult> {
        let source = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", path, e))?;
        let base_dir = path.parent().unwrap_or(Path::new("."));
        Self::process_source_with_docs(&source, base_dir)
    }

    /// Process source and return the preprocessor state (for doc generation).
    /// Returns (expanded_code, functions, fn_docs).
    pub fn process_source_with_docs(source: &str, base_dir: &Path) -> Result<PreprocessResult> {
        let mut pp = Self::new();
        let top_level = pp.collect(source, base_dir)?;
        let mut stack = Vec::new();
        let expanded = pp.expand(&top_level, &mut stack)?;
        Ok((expanded, pp.functions, pp.fn_docs))
    }

    /// Process a file and return the expanded code along with a SourceMap.
    /// The SourceMap maps each character in the expanded output to its origin.
    pub fn process_file_with_map(path: &Path) -> Result<(String, SourceMap)> {
        let source = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", path, e))?;
        let base_dir = path.parent().unwrap_or(Path::new("."));
        Self::process_source_with_map(&source, base_dir, path)
    }

    /// Process source and return expanded code with a SourceMap.
    pub fn process_source_with_map(
        source: &str,
        base_dir: &Path,
        file_path: &Path,
    ) -> Result<(String, SourceMap)> {
        let mut pp = Self::new_with_map();
        let top_level = pp.collect_with_tracking(source, base_dir, file_path)?;
        let mut stack = Vec::new();
        let expanded = pp.expand_with_tracking(&top_level, &mut stack)?;
        let source_map = pp.source_map.take().unwrap_or_default();
        Ok((expanded, source_map))
    }

    /// Process a file with pre-loaded dependency functions available.
    /// These functions are available for @call expansion as if they were imported.
    pub fn process_file_with_deps(
        path: &Path,
        dep_functions: &HashMap<String, String>,
    ) -> Result<String> {
        let source = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", path, e))?;
        let base_dir = path.parent().unwrap_or(Path::new("."));
        let mut pp = Self::new();
        // Pre-load dependency functions
        pp.functions.extend(dep_functions.clone());
        let top_level = pp.collect(&source, base_dir)?;
        let mut stack = Vec::new();
        pp.expand(&top_level, &mut stack)
    }

    /// Collect all @fn definitions from a file (for REPL preloading).
    /// Returns the function names and bodies without expanding top-level code.
    pub fn collect_functions_from_file(path: &Path) -> Result<HashMap<String, String>> {
        let source = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", path, e))?;
        let base_dir = path.parent().unwrap_or(Path::new("."));
        let mut pp = Self::new();
        let _ = pp.collect(&source, base_dir)?;
        Ok(pp.functions)
    }

    /// Collect all @fn definitions from a source string (for REPL preloading).
    pub fn collect_functions_from_source(
        source: &str,
        base_dir: &Path,
    ) -> Result<HashMap<String, String>> {
        let mut pp = Self::new();
        let _ = pp.collect(source, base_dir)?;
        Ok(pp.functions)
    }

    /// Expand @call/@use directives in code given a set of known functions and constants.
    /// Used by the REPL to preprocess user input against preloaded definitions.
    pub fn expand_with_functions(
        code: &str,
        functions: &HashMap<String, String>,
    ) -> Result<String> {
        let mut pp = Self::new();
        pp.functions = functions.clone();
        let top_level = pp.collect(code, Path::new("."))?;
        let mut stack = Vec::new();
        pp.expand(&top_level, &mut stack)
    }

    // ---- Pass 1: collect (with source tracking) ----

    /// Collect pass that also tracks source locations for each character in top_level output.
    /// `current_file` is the file being processed (for source map entries).
    #[allow(unused_assignments)]
    fn collect_with_tracking(
        &mut self,
        source: &str,
        base_dir: &Path,
        current_file: &Path,
    ) -> Result<String> {
        let mut top_level = String::new();
        let chars: Vec<char> = source.chars().collect();
        let mut i = 0;
        let mut pending_doc = String::new();

        // Track line/column in the current source
        let mut line: usize = 1;
        let mut col: usize = 1;

        while i < chars.len() {
            if chars[i] == '@' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                i += 1; // skip '@'
                col += 1;
                let keyword_start = i;
                let keyword = take_identifier(&chars, &mut i);
                // Advance col past the keyword
                col += i - keyword_start;

                match keyword.as_str() {
                    "import" => {
                        skip_spaces(&chars, &mut i);
                        col = Self::update_col_after_skip(source, i, line);
                        let path_str = take_quoted_string(&chars, &mut i)
                            .map_err(|e| anyhow::anyhow!("@import: {}", e))?;
                        col = Self::update_col_after_skip(source, i, line);

                        if path_str.starts_with("std/") {
                            let module_name = path_str
                                .strip_prefix("std/")
                                .unwrap()
                                .strip_suffix(".bf")
                                .unwrap_or(path_str.strip_prefix("std/").unwrap());

                            let stdlib_source =
                                get_stdlib_module(module_name).ok_or_else(|| {
                                    OgreError::UnknownStdModule(unknown_module_message(module_name))
                                })?;

                            let sentinel = PathBuf::from(format!("<stdlib:{}>", module_name));
                            if !self.imported.contains(&sentinel) {
                                self.imported.insert(sentinel.clone());
                                let stdlib_path = PathBuf::from(format!("std/{}.bf", module_name));
                                self.collect_with_tracking(stdlib_source, base_dir, &stdlib_path)?;
                            }
                        } else {
                            let import_path = base_dir.join(&path_str);
                            let canonical = import_path
                                .canonicalize()
                                .unwrap_or_else(|_| import_path.clone());

                            if self.imported.contains(&canonical) {
                                return Err(OgreError::ImportCycle(
                                    import_path.display().to_string(),
                                )
                                .into());
                            }
                            self.imported.insert(canonical);

                            let imported_source = fs::read_to_string(&import_path)
                                .map_err(|e| anyhow::anyhow!("@import \"{}\": {}", path_str, e))?;
                            let import_base = import_path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| PathBuf::from("."));

                            self.collect_with_tracking(
                                &imported_source,
                                &import_base,
                                &import_path,
                            )?;
                        }
                    }

                    "doc" => {
                        skip_spaces(&chars, &mut i);
                        col = Self::update_col_after_skip(source, i, line);
                        let mut doc_line = String::new();
                        while i < chars.len() && chars[i] != '\n' {
                            doc_line.push(chars[i]);
                            i += 1;
                            col += 1;
                        }
                        if !pending_doc.is_empty() {
                            pending_doc.push('\n');
                        }
                        pending_doc.push_str(doc_line.trim());
                    }

                    "fn" => {
                        skip_spaces(&chars, &mut i);
                        col = Self::update_col_after_skip(source, i, line);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@fn: missing function name");
                        }
                        col = Self::update_col_after_skip(source, i, line);
                        skip_whitespace(&chars, &mut i);
                        // Recalculate line/col after whitespace skip
                        let (new_line, new_col) = Self::compute_line_col(source, i);
                        line = new_line;
                        col = new_col;
                        if i >= chars.len() || chars[i] != '{' {
                            bail!("@fn {}: expected '{{', found {:?}", name, chars.get(i));
                        }
                        i += 1; // skip '{'
                        col += 1;
                        let body = take_brace_body(&chars, &mut i)
                            .map_err(|e| anyhow::anyhow!("@fn {}: {}", name, e))?;
                        // Recalculate after brace body
                        let (new_line, new_col) = Self::compute_line_col(source, i);
                        line = new_line;
                        col = new_col;
                        if !pending_doc.is_empty() {
                            self.fn_docs.insert(name.clone(), pending_doc.clone());
                            pending_doc.clear();
                        }
                        self.fn_origins
                            .insert(name.clone(), current_file.to_path_buf());
                        self.functions.insert(name, body);
                    }

                    "call" => {
                        skip_spaces(&chars, &mut i);
                        col = Self::update_col_after_skip(source, i, line);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@call: missing function name");
                        }
                        // Preserve @call marker with source location annotation
                        top_level.push_str("@call ");
                        top_level.push_str(&name);
                        // Push placeholder source map entries for the @call marker
                        if self.build_map {
                            // The @call marker itself doesn't produce BF,
                            // it's handled in expand. We tag its position.
                        }
                    }

                    "const" => {
                        skip_spaces(&chars, &mut i);
                        col = Self::update_col_after_skip(source, i, line);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@const: missing constant name");
                        }
                        skip_spaces(&chars, &mut i);
                        col = Self::update_col_after_skip(source, i, line);
                        let mut num_str = String::new();
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            num_str.push(chars[i]);
                            i += 1;
                            col += 1;
                        }
                        if num_str.is_empty() {
                            bail!("@const {}: expected numeric value", name);
                        }
                        let value: usize = num_str.parse().map_err(|_| {
                            anyhow::anyhow!("@const {}: invalid value {:?}", name, num_str)
                        })?;
                        self.constants.insert(name, value);
                    }

                    "use" => {
                        skip_spaces(&chars, &mut i);
                        let use_line = line;
                        let use_col = col;
                        col = Self::update_col_after_skip(source, i, line);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@use: missing constant name");
                        }
                        let value = self.constants.get(&name).ok_or_else(|| {
                            OgreError::Other(format!("undefined constant: @use {}", name))
                        })?;
                        for j in 0..*value {
                            top_level.push('+');
                            if self.build_map {
                                if let Some(ref mut sm) = self.source_map {
                                    sm.push(SourceLocation::new(
                                        current_file.to_path_buf(),
                                        use_line,
                                        use_col + j,
                                    ));
                                }
                            }
                        }
                    }

                    other => {
                        return Err(OgreError::UnknownDirective(other.to_string()).into());
                    }
                }
            } else {
                // Regular character — add to top_level with source tracking
                if self.build_map {
                    if let Some(ref mut sm) = self.source_map {
                        sm.push(SourceLocation::new(current_file.to_path_buf(), line, col));
                    }
                }
                top_level.push(chars[i]);

                if chars[i] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }
        }

        Ok(top_level)
    }

    /// Expand pass that also builds source map entries for expanded @call bodies.
    fn expand_with_tracking(&mut self, code: &str, stack: &mut Vec<String>) -> Result<String> {
        let mut result = String::new();
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        // Track position in the source_map: the collect pass already pushed
        // entries for non-directive characters. We need to filter them through
        // as we encounter @call expansions.
        let mut source_char_idx = 0;

        // We need to rebuild the source map during expand, replacing @call
        // markers with the expanded function body locations.
        let collect_map = if self.build_map {
            self.source_map.take()
        } else {
            None
        };
        let collect_locations = collect_map.as_ref().map(|m| {
            // Extract the locations vec for indexed access
            let mut locs = Vec::new();
            let mut idx = 0;
            while let Some(loc) = m.lookup(idx) {
                locs.push(loc.clone());
                idx += 1;
            }
            locs
        });

        if self.build_map {
            self.source_map = Some(SourceMap::new());
        }

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
                        return Err(OgreError::CycleDetected(cycle.join(" → ")).into());
                    }

                    let body = self
                        .functions
                        .get(&name)
                        .ok_or_else(|| {
                            OgreError::UnknownFunction(unknown_function_message(
                                &name,
                                &self.functions,
                            ))
                        })?
                        .clone();

                    let fn_file = self
                        .fn_origins
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| PathBuf::from("<unknown>"));

                    stack.push(name.clone());
                    // Recursively expand the body
                    let expanded_body = self.expand(&body, stack)?;
                    stack.pop();

                    // Push source map entries for each character in the expanded body
                    if self.build_map {
                        if let Some(ref mut sm) = self.source_map {
                            // Track line/col within the function body
                            let mut fn_line: usize = 1;
                            let mut fn_col: usize = 1;
                            for ch in expanded_body.chars() {
                                sm.push(SourceLocation::with_function(
                                    fn_file.clone(),
                                    fn_line,
                                    fn_col,
                                    name.clone(),
                                ));
                                if ch == '\n' {
                                    fn_line += 1;
                                    fn_col = 1;
                                } else {
                                    fn_col += 1;
                                }
                            }
                        }
                    }

                    result.push_str(&expanded_body);
                } else if keyword == "use" {
                    skip_spaces(&chars, &mut i);
                    let name = take_identifier(&chars, &mut i);
                    let value = self.constants.get(&name).ok_or_else(|| {
                        OgreError::Other(format!("undefined constant: @use {}", name))
                    })?;
                    for _ in 0..*value {
                        result.push('+');
                        if self.build_map {
                            if let Some(ref mut sm) = self.source_map {
                                sm.push(SourceLocation::new(PathBuf::from("<const>"), 1, 1));
                            }
                        }
                    }
                } else {
                    result.push('@');
                    result.push_str(&keyword);
                    // These characters aren't tracked in source map
                }
            } else {
                result.push(chars[i]);
                // Copy the source location from the collect pass
                if self.build_map {
                    if let Some(ref mut sm) = self.source_map {
                        if let Some(ref locs) = collect_locations {
                            if source_char_idx < locs.len() {
                                sm.push(locs[source_char_idx].clone());
                            }
                        }
                    }
                    source_char_idx += 1;
                }
                i += 1;
            }
        }

        Ok(result)
    }

    /// Helper: compute line/col from byte position in source.
    fn compute_line_col(source: &str, char_idx: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (idx, ch) in source.chars().enumerate() {
            if idx >= char_idx {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// Helper: recompute column after a skip_spaces call.
    fn update_col_after_skip(source: &str, char_idx: usize, _current_line: usize) -> usize {
        let (_, col) = Self::compute_line_col(source, char_idx);
        col
    }

    // ---- Pass 1: collect ----

    fn collect(&mut self, source: &str, base_dir: &Path) -> Result<String> {
        let mut top_level = String::new();
        let chars: Vec<char> = source.chars().collect();
        let mut i = 0;
        let mut pending_doc = String::new();

        while i < chars.len() {
            if chars[i] == '@' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                i += 1; // skip '@'
                let keyword = take_identifier(&chars, &mut i);

                match keyword.as_str() {
                    "import" => {
                        skip_spaces(&chars, &mut i);
                        let path_str = take_quoted_string(&chars, &mut i)
                            .map_err(|e| anyhow::anyhow!("@import: {}", e))?;

                        // Check for standard library imports (std/module.bf)
                        if path_str.starts_with("std/") {
                            let module_name = path_str
                                .strip_prefix("std/")
                                .unwrap()
                                .strip_suffix(".bf")
                                .unwrap_or(path_str.strip_prefix("std/").unwrap());

                            let stdlib_source =
                                get_stdlib_module(module_name).ok_or_else(|| {
                                    OgreError::UnknownStdModule(unknown_module_message(module_name))
                                })?;

                            let sentinel = PathBuf::from(format!("<stdlib:{}>", module_name));
                            if !self.imported.contains(&sentinel) {
                                self.imported.insert(sentinel);
                                self.collect(stdlib_source, base_dir)?;
                            }
                        } else {
                            let import_path = base_dir.join(&path_str);
                            // Use canonical path for cycle detection when possible
                            let canonical = import_path
                                .canonicalize()
                                .unwrap_or_else(|_| import_path.clone());

                            if self.imported.contains(&canonical) {
                                return Err(OgreError::ImportCycle(
                                    import_path.display().to_string(),
                                )
                                .into());
                            }
                            self.imported.insert(canonical);

                            let imported_source = fs::read_to_string(&import_path)
                                .map_err(|e| anyhow::anyhow!("@import \"{}\": {}", path_str, e))?;
                            let import_base = import_path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| PathBuf::from("."));

                            // Collect @fn definitions; discard top-level code from imports
                            let import_top_level = self.collect(&imported_source, &import_base)?;
                            // Warn if the imported file had top-level BF code that's being dropped
                            let has_bf_code =
                                import_top_level.chars().any(|c| "+-><.,[]".contains(c));
                            if has_bf_code {
                                eprintln!(
                                    "warning: top-level code in imported file '{}' is discarded",
                                    path_str
                                );
                            }
                        }
                    }

                    "doc" => {
                        skip_spaces(&chars, &mut i);
                        // Read rest of line as doc text
                        let mut doc_line = String::new();
                        while i < chars.len() && chars[i] != '\n' {
                            doc_line.push(chars[i]);
                            i += 1;
                        }
                        if !pending_doc.is_empty() {
                            pending_doc.push('\n');
                        }
                        pending_doc.push_str(doc_line.trim());
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
                        if !pending_doc.is_empty() {
                            self.fn_docs.insert(name.clone(), pending_doc.clone());
                            pending_doc.clear();
                        }
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

                    "const" => {
                        skip_spaces(&chars, &mut i);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@const: missing constant name");
                        }
                        skip_spaces(&chars, &mut i);
                        // Parse the numeric value
                        let mut num_str = String::new();
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            num_str.push(chars[i]);
                            i += 1;
                        }
                        if num_str.is_empty() {
                            bail!("@const {}: expected numeric value", name);
                        }
                        let value: usize = num_str.parse().map_err(|_| {
                            anyhow::anyhow!("@const {}: invalid value {:?}", name, num_str)
                        })?;
                        self.constants.insert(name, value);
                    }

                    "use" => {
                        skip_spaces(&chars, &mut i);
                        let name = take_identifier(&chars, &mut i);
                        if name.is_empty() {
                            bail!("@use: missing constant name");
                        }
                        let value = self.constants.get(&name).ok_or_else(|| {
                            OgreError::Other(format!("undefined constant: @use {}", name))
                        })?;
                        // Expand to N '+' characters
                        for _ in 0..*value {
                            top_level.push('+');
                        }
                    }

                    other => {
                        return Err(OgreError::UnknownDirective(other.to_string()).into());
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
                        return Err(OgreError::CycleDetected(cycle.join(" → ")).into());
                    }

                    let body = self.functions.get(&name).ok_or_else(|| {
                        OgreError::UnknownFunction(unknown_function_message(&name, &self.functions))
                    })?;

                    stack.push(name.clone());
                    let expanded = self.expand(body, stack)?;
                    stack.pop();

                    result.push_str(&expanded);
                } else if keyword == "use" {
                    skip_spaces(&chars, &mut i);
                    let name = take_identifier(&chars, &mut i);
                    let value = self.constants.get(&name).ok_or_else(|| {
                        OgreError::Other(format!("undefined constant: @use {}", name))
                    })?;
                    for _ in 0..*value {
                        result.push('+');
                    }
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

    #[test]
    fn test_stdlib_import_io() {
        let out = process("@import \"std/io.bf\" @call print_newline").unwrap();
        assert!(!out.contains("@call"));
        assert!(!out.contains("@fn"));
        assert!(!out.contains("@import"));
        // Should contain the BF for printing newline
        assert!(out.contains('.'));
    }

    #[test]
    fn test_stdlib_import_math() {
        let out = process("@import \"std/math.bf\" @call zero").unwrap();
        assert!(!out.contains("@call"));
        assert!(out.contains("[-]"));
    }

    #[test]
    fn test_stdlib_unknown_module_errors() {
        assert!(process("@import \"std/nonexistent.bf\"").is_err());
    }

    #[test]
    fn test_stdlib_duplicate_import_ok() {
        let out =
            process("@import \"std/io.bf\" @import \"std/io.bf\" @call print_newline").unwrap();
        assert!(!out.contains("@call"));
    }

    // ---- @const / @use tests ----

    #[test]
    fn test_const_use_basic() {
        let out = process("@const X 5\n@use X").unwrap();
        assert_eq!(
            out.replace(|c: char| !c.is_ascii() && c != '+', "")
                .matches('+')
                .count(),
            5
        );
    }

    #[test]
    fn test_const_use_zero() {
        let out = process("@const Z 0\n@use Z").unwrap();
        // Zero expansion means no + chars from the @use
        let plus_count = out.chars().filter(|c| *c == '+').count();
        assert_eq!(plus_count, 0);
    }

    #[test]
    fn test_const_use_large() {
        let out = process("@const BIG 255\n@use BIG").unwrap();
        let plus_count = out.chars().filter(|c| *c == '+').count();
        assert_eq!(plus_count, 255);
    }

    #[test]
    fn test_const_use_in_fn_body() {
        let src = "@const N 3\n@fn add_n { @use N }\n@call add_n";
        let out = process(src).unwrap();
        let plus_count = out.chars().filter(|c| *c == '+').count();
        assert_eq!(plus_count, 3);
    }

    #[test]
    fn test_const_undefined_use_errors() {
        assert!(process("@use UNDEFINED").is_err());
        let err = process("@use UNDEFINED").unwrap_err().to_string();
        assert!(err.contains("UNDEFINED"));
    }

    #[test]
    fn test_const_missing_value_errors() {
        assert!(process("@const X").is_err());
    }

    #[test]
    fn test_const_multiple() {
        let src = "@const A 2\n@const B 3\n@use A @use B";
        let out = process(src).unwrap();
        let plus_count = out.chars().filter(|c| *c == '+').count();
        assert_eq!(plus_count, 5);
    }

    // ---- @doc tests ----

    #[test]
    fn test_doc_directive_ignored_in_output() {
        let src = "@doc This is a doc comment\n@fn foo { + }\n@call foo";
        let out = process(src).unwrap();
        assert!(!out.contains("@doc"));
        assert!(!out.contains("This is a doc"));
        assert!(out.contains('+'));
    }

    #[test]
    fn test_doc_attaches_to_fn() {
        let src = "@doc My docs\n@fn bar { + }";
        let (_, _, fn_docs) = Preprocessor::process_source_with_docs(src, Path::new(".")).unwrap();
        assert_eq!(fn_docs.get("bar").map(|s| s.as_str()), Some("My docs"));
    }

    #[test]
    fn test_doc_multi_line() {
        let src = "@doc Line one\n@doc Line two\n@fn baz { + }";
        let (_, _, fn_docs) = Preprocessor::process_source_with_docs(src, Path::new(".")).unwrap();
        let doc = fn_docs.get("baz").unwrap();
        assert!(doc.contains("Line one"));
        assert!(doc.contains("Line two"));
    }

    #[test]
    fn test_doc_without_fn_is_discarded() {
        let src = "@doc Orphaned doc\n+++";
        let out = process(src).unwrap();
        assert!(!out.contains("doc"));
        assert!(out.contains("+++"));
    }

    #[test]
    fn test_collect_functions_from_source() {
        let src = "@fn hello { +++ }\n@fn world { --- }";
        let fns = Preprocessor::collect_functions_from_source(src, Path::new(".")).unwrap();
        assert_eq!(fns.len(), 2);
        assert!(fns.contains_key("hello"));
        assert!(fns.contains_key("world"));
        assert!(fns["hello"].contains("+++"));
        assert!(fns["world"].contains("---"));
    }

    #[test]
    fn test_expand_with_functions() {
        let mut fns = HashMap::new();
        fns.insert("greet".to_string(), "+++.".to_string());
        let result = Preprocessor::expand_with_functions("@call greet", &fns).unwrap();
        assert!(result.contains("+++."));
    }

    #[test]
    fn test_expand_with_functions_unknown_call_errors() {
        let fns = HashMap::new();
        let result = Preprocessor::expand_with_functions("@call unknown", &fns);
        assert!(result.is_err());
    }

    #[test]
    fn test_collect_functions_includes_stdlib() {
        let src = "@import \"std/io\"\n@fn myfn { + }";
        let fns = Preprocessor::collect_functions_from_source(src, Path::new(".")).unwrap();
        assert!(fns.contains_key("myfn"));
        assert!(fns.contains_key("print_newline"));
    }

    #[test]
    fn test_stdlib_memory_import() {
        let src = "@import \"std/memory\"\n@fn test { @call copy_right }";
        let out = process(src).unwrap();
        // copy_right body should be expanded
        assert!(!out.contains("@call"));
    }

    #[test]
    fn test_stdlib_ascii_import() {
        let src = "@import \"std/ascii\"\n@call print_A";
        let out = process(src).unwrap();
        // print_A generates ASCII 65 with + characters
        assert!(out.contains('+'));
        assert!(out.contains('.'));
    }

    #[test]
    fn test_mixing_std_and_file_imports() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let lib_file = dir.path().join("mylib.bf");
        std::fs::File::create(&lib_file)
            .unwrap()
            .write_all(b"@fn myfunc { +++ }")
            .unwrap();

        let src =
            format!("@import \"std/io\"\n@import \"mylib.bf\"\n@call print_newline\n@call myfunc");
        let out = Preprocessor::process_source(&src, dir.path()).unwrap();
        assert!(out.contains("+++"));
        assert!(!out.contains("@call"));
    }

    // ---- Source map tests ----

    #[test]
    fn test_source_map_plain_bf() {
        let src = "+>.";
        let (expanded, map) =
            Preprocessor::process_source_with_map(src, Path::new("."), Path::new("test.bf"))
                .unwrap();
        assert_eq!(expanded, "+>.");
        assert_eq!(map.len(), 3);
        // Each character should map to test.bf
        let loc0 = map.lookup(0).unwrap();
        assert_eq!(loc0.file, PathBuf::from("test.bf"));
        assert_eq!(loc0.line, 1);
        assert_eq!(loc0.column, 1);
        assert!(loc0.function.is_none());

        let loc2 = map.lookup(2).unwrap();
        assert_eq!(loc2.column, 3);
    }

    #[test]
    fn test_source_map_with_fn_call() {
        let src = "@fn add { +++ }\n@call add";
        let (expanded, map) =
            Preprocessor::process_source_with_map(src, Path::new("."), Path::new("main.bf"))
                .unwrap();
        // The expanded output should contain the +++ from the function body
        assert!(expanded.contains("+++"));
        // The source map entries for the function body should have function="add"
        let mut found_fn = false;
        let mut idx = 0;
        while let Some(loc) = map.lookup(idx) {
            if loc.function.as_deref() == Some("add") {
                found_fn = true;
                break;
            }
            idx += 1;
        }
        assert!(found_fn, "source map should contain entries for @fn add");
    }

    #[test]
    fn test_source_map_multiline() {
        let src = "+\n+\n+";
        let (expanded, map) =
            Preprocessor::process_source_with_map(src, Path::new("."), Path::new("test.bf"))
                .unwrap();
        assert_eq!(expanded, "+\n+\n+");
        // First + is line 1, col 1
        let loc0 = map.lookup(0).unwrap();
        assert_eq!(loc0.line, 1);
        assert_eq!(loc0.column, 1);
        // Second + (position 2, after "\n") is line 2, col 1
        let loc2 = map.lookup(2).unwrap();
        assert_eq!(loc2.line, 2);
        assert_eq!(loc2.column, 1);
        // Third + (position 4) is line 3, col 1
        let loc4 = map.lookup(4).unwrap();
        assert_eq!(loc4.line, 3);
        assert_eq!(loc4.column, 1);
    }

    #[test]
    fn test_source_map_preserves_output() {
        // Processing with source map should produce the same expanded output
        let src = "@fn inc { + }\n@fn dec { - }\n@call inc @call dec";
        let without_map = Preprocessor::process_source(src, Path::new(".")).unwrap();
        let (with_map, _) =
            Preprocessor::process_source_with_map(src, Path::new("."), Path::new("test.bf"))
                .unwrap();
        assert_eq!(without_map, with_map);
    }

    #[test]
    fn test_source_map_empty_produces_empty_map() {
        let (expanded, map) =
            Preprocessor::process_source_with_map("", Path::new("."), Path::new("empty.bf"))
                .unwrap();
        assert_eq!(expanded, "");
        assert!(map.is_empty());
    }

    #[test]
    fn test_source_map_with_const_use() {
        let src = "@const N 3\n@use N";
        let (expanded, map) =
            Preprocessor::process_source_with_map(src, Path::new("."), Path::new("test.bf"))
                .unwrap();
        // @use N expands to "+++"
        let plus_count = expanded.chars().filter(|c| *c == '+').count();
        assert_eq!(plus_count, 3);
        // Source map should have entries for the + characters
        assert!(map.len() >= 3);
    }
}
