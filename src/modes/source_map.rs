use std::path::PathBuf;

/// A source location tracking where a character in expanded output originated from.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// The file this code came from.
    pub file: PathBuf,
    /// 1-based line number in the original file.
    pub line: usize,
    /// 1-based column number in the original file.
    pub column: usize,
    /// The @fn name if this code is inside a function expansion.
    pub function: Option<String>,
}

impl SourceLocation {
    pub fn new(file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
            function: None,
        }
    }

    pub fn with_function(file: PathBuf, line: usize, column: usize, function: String) -> Self {
        Self {
            file,
            line,
            column,
            function: Some(function),
        }
    }

    /// Format as a human-readable location string.
    pub fn display_short(&self) -> String {
        let path = self.file.display();
        match &self.function {
            Some(f) => format!("{}:{}:{} (@fn {})", path, self.line, self.column, f),
            None => format!("{}:{}:{}", path, self.line, self.column),
        }
    }
}

/// Maps each character position in the expanded BF output to its original source location.
///
/// After preprocessing, the expanded output is a flat string of BF characters.
/// The SourceMap tracks where each character came from, which is useful for:
/// - Debugger: showing original file/line/function context
/// - Error messages: pointing to the original source location
pub struct SourceMap {
    /// One entry per character in the expanded output string.
    locations: Vec<SourceLocation>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            locations: Vec::new(),
        }
    }

    /// Push a source location for the next character in the expanded output.
    pub fn push(&mut self, loc: SourceLocation) {
        self.locations.push(loc);
    }

    /// Look up the source location for a character position in the expanded output.
    pub fn lookup(&self, position: usize) -> Option<&SourceLocation> {
        self.locations.get(position)
    }

    /// Look up the source location for an IR op index.
    ///
    /// Since the IR collapses consecutive characters into single ops,
    /// we need a mapping from op index to the first character position
    /// of that op. This is provided by `op_to_char_map`.
    pub fn lookup_op(&self, op_index: usize, op_to_char: &[usize]) -> Option<&SourceLocation> {
        op_to_char
            .get(op_index)
            .and_then(|&char_pos| self.locations.get(char_pos))
    }

    /// Number of entries in the map.
    pub fn len(&self) -> usize {
        self.locations.len()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a mapping from IR op indices to character positions in the expanded source.
///
/// When the IR collapses e.g. `+++` into `Add(3)`, we want to map the op back
/// to the character position of the first `+`. This function builds that mapping.
pub fn build_op_to_char_map(source: &str) -> Vec<usize> {
    let mut map = Vec::new();
    let mut last_bf_char: Option<char> = None;

    for (pos, ch) in source.chars().enumerate() {
        match ch {
            '+' | '-' | '>' | '<' | '.' | ',' | '[' | ']' => {
                // Check if this character would be collapsed with the previous one
                let collapsed = matches!(
                    (last_bf_char, ch),
                    (Some('+'), '+') | (Some('-'), '-') | (Some('>'), '>') | (Some('<'), '<')
                );

                if collapsed {
                    // Same op as previous — don't add a new entry
                } else {
                    // New op — record the character position
                    map.push(pos);
                }

                last_bf_char = Some(ch);
            }
            _ => {
                // Non-BF character — doesn't create an op
                // Reset the last_bf_char so the next BF char starts a new op
                last_bf_char = None;
            }
        }
    }

    map
}

/// Compute a line/column tracker for a source string.
/// Returns (line, column) for each character index (0-based index -> 1-based line/col).
pub fn line_col_map(source: &str) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(source.len());
    let mut line = 1;
    let mut col = 1;
    for ch in source.chars() {
        result.push((line, col));
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display_short() {
        let loc = SourceLocation::new(PathBuf::from("src/main.bf"), 5, 12);
        assert_eq!(loc.display_short(), "src/main.bf:5:12");
    }

    #[test]
    fn test_source_location_display_with_function() {
        let loc =
            SourceLocation::with_function(PathBuf::from("src/greet.bf"), 3, 5, "greet".to_string());
        assert_eq!(loc.display_short(), "src/greet.bf:3:5 (@fn greet)");
    }

    #[test]
    fn test_source_map_lookup() {
        let mut map = SourceMap::new();
        map.push(SourceLocation::new(PathBuf::from("a.bf"), 1, 1));
        map.push(SourceLocation::new(PathBuf::from("a.bf"), 1, 2));
        map.push(SourceLocation::new(PathBuf::from("b.bf"), 3, 1));

        assert_eq!(map.lookup(0).unwrap().file, PathBuf::from("a.bf"));
        assert_eq!(map.lookup(1).unwrap().column, 2);
        assert_eq!(map.lookup(2).unwrap().file, PathBuf::from("b.bf"));
        assert!(map.lookup(3).is_none());
    }

    #[test]
    fn test_source_map_empty() {
        let map = SourceMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert!(map.lookup(0).is_none());
    }

    #[test]
    fn test_build_op_to_char_map_simple() {
        // "+>." = 3 ops at positions 0, 1, 2
        let map = build_op_to_char_map("+>.");
        assert_eq!(map, vec![0, 1, 2]);
    }

    #[test]
    fn test_build_op_to_char_map_collapsed() {
        // "+++" collapses to 1 op; first char at position 0
        let map = build_op_to_char_map("+++");
        assert_eq!(map, vec![0]);
    }

    #[test]
    fn test_build_op_to_char_map_mixed() {
        // "+++>>" = Add(3) at 0, Right(2) at 3
        let map = build_op_to_char_map("+++>>");
        assert_eq!(map, vec![0, 3]);
    }

    #[test]
    fn test_build_op_to_char_map_with_comments() {
        // "+ comment +" = two separate Add(1) ops because comment breaks collapsing
        let map = build_op_to_char_map("+ comment +");
        assert_eq!(map, vec![0, 10]);
    }

    #[test]
    fn test_build_op_to_char_map_brackets() {
        // "[+]" = JumpIfZero, Add(1), JumpIfNonZero
        let map = build_op_to_char_map("[+]");
        assert_eq!(map, vec![0, 1, 2]);
    }

    #[test]
    fn test_lookup_op() {
        let mut source_map = SourceMap::new();
        // Positions 0, 1, 2 for "++>" (3 chars, but 2 ops after collapsing)
        source_map.push(SourceLocation::new(PathBuf::from("a.bf"), 1, 1)); // pos 0: first +
        source_map.push(SourceLocation::new(PathBuf::from("a.bf"), 1, 2)); // pos 1: second +
        source_map.push(SourceLocation::new(PathBuf::from("a.bf"), 1, 3)); // pos 2: >

        let op_to_char = build_op_to_char_map("++>");
        // op 0 = Add(2) at char 0, op 1 = Right(1) at char 2
        assert_eq!(op_to_char, vec![0, 2]);

        let loc = source_map.lookup_op(0, &op_to_char).unwrap();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);

        let loc = source_map.lookup_op(1, &op_to_char).unwrap();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 3);
    }

    #[test]
    fn test_line_col_map() {
        let map = line_col_map("ab\ncd");
        assert_eq!(map[0], (1, 1)); // 'a'
        assert_eq!(map[1], (1, 2)); // 'b'
        assert_eq!(map[2], (1, 3)); // '\n'
        assert_eq!(map[3], (2, 1)); // 'c'
        assert_eq!(map[4], (2, 2)); // 'd'
    }

    #[test]
    fn test_line_col_map_empty() {
        let map = line_col_map("");
        assert!(map.is_empty());
    }
}
