use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

/// Standard BF operators in canonical order.
const BF_OPS: &[char] = &['+', '-', '<', '>', '.', ',', '[', ']'];

/// Known dialect definitions.
#[derive(Debug, Clone)]
pub struct Dialect {
    pub name: String,
    pub tokens: Vec<String>,
}

impl Dialect {
    /// Standard Brainfuck dialect.
    pub fn brainfuck() -> Self {
        Dialect {
            name: "brainfuck".to_string(),
            tokens: BF_OPS.iter().map(|c| c.to_string()).collect(),
        }
    }

    /// Ook! dialect.
    pub fn ook() -> Self {
        Dialect {
            name: "ook".to_string(),
            tokens: vec![
                "Ook. Ook. ".to_string(), // +
                "Ook! Ook! ".to_string(), // -
                "Ook. Ook? ".to_string(), // <
                "Ook? Ook. ".to_string(), // >
                "Ook! Ook. ".to_string(), // .
                "Ook. Ook! ".to_string(), // ,
                "Ook! Ook? ".to_string(), // [
                "Ook? Ook! ".to_string(), // ]
            ],
        }
    }

    /// Trollscript dialect.
    pub fn trollscript() -> Self {
        Dialect {
            name: "trollscript".to_string(),
            tokens: vec![
                "ooo".to_string(), // +
                "ool".to_string(), // -
                "olo".to_string(), // <
                "oll".to_string(), // >
                "loo".to_string(), // .
                "lol".to_string(), // ,
                "llo".to_string(), // [
                "lll".to_string(), // ]
            ],
        }
    }

    /// Create a custom dialect from a character mapping string.
    /// The string must have exactly 8 characters mapping to: + - < > . , [ ]
    pub fn custom(name: &str, mapping: &str) -> Result<Self> {
        let chars: Vec<char> = mapping.chars().collect();
        if chars.len() != 8 {
            bail!(
                "custom mapping must have exactly 8 characters (got {}). \
                 Order: + - < > . , [ ]",
                chars.len()
            );
        }
        Ok(Dialect {
            name: name.to_string(),
            tokens: chars.iter().map(|c| c.to_string()).collect(),
        })
    }

    /// Get a named dialect.
    pub fn by_name(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "bf" | "brainfuck" => Ok(Self::brainfuck()),
            "ook" | "ook!" => Ok(Self::ook()),
            "trollscript" | "troll" => Ok(Self::trollscript()),
            other => bail!(
                "unknown dialect '{}'. Available: brainfuck, ook, trollscript",
                other
            ),
        }
    }
}

/// Convert source code from one dialect to another.
pub fn convert(source: &str, from: &Dialect, to: &Dialect) -> String {
    let mut result = String::new();

    if from.name == "brainfuck" {
        // Fast path: single-char tokens
        for ch in source.chars() {
            if let Some(idx) = BF_OPS.iter().position(|&c| c == ch) {
                result.push_str(&to.tokens[idx]);
            }
            // Skip non-BF characters
        }
    } else {
        // Multi-char token matching
        let mut pos = 0;
        let chars: Vec<char> = source.chars().collect();

        while pos < chars.len() {
            let remaining: String = chars[pos..].iter().collect();
            let mut matched = false;

            for (idx, token) in from.tokens.iter().enumerate() {
                if remaining.starts_with(token) {
                    result.push_str(&to.tokens[idx]);
                    pos += token.len();
                    matched = true;
                    break;
                }
            }

            if !matched {
                pos += 1; // skip unknown characters
            }
        }
    }

    result
}

/// Convert from BF to a target dialect string.
pub fn convert_bf_to(source: &str, target_name: &str) -> Result<String> {
    let from = Dialect::brainfuck();
    let to = Dialect::by_name(target_name)?;
    Ok(convert(source, &from, &to))
}

/// Convert from a source dialect to BF.
pub fn convert_to_bf(source: &str, source_name: &str) -> Result<String> {
    let from = Dialect::by_name(source_name)?;
    let to = Dialect::brainfuck();
    Ok(convert(source, &from, &to))
}

/// Convert a file and output the result.
pub fn convert_file(
    path: &Path,
    from_name: Option<&str>,
    to_name: &str,
    output: Option<&str>,
) -> Result<()> {
    let source = fs::read_to_string(path)?;
    let from = match from_name {
        Some(name) => Dialect::by_name(name)?,
        None => Dialect::brainfuck(),
    };
    let to = Dialect::by_name(to_name)?;
    let result = convert(&source, &from, &to);

    match output {
        Some(out_path) => {
            fs::write(out_path, &result)?;
            println!("Converted to {}: {}", to.name, out_path);
        }
        None => {
            print!("{}", result);
        }
    }

    Ok(())
}

/// List available dialect names.
pub fn list_dialects() -> Vec<&'static str> {
    vec!["brainfuck", "ook", "trollscript"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bf_to_ook() {
        let result = convert_bf_to("+", "ook").unwrap();
        assert_eq!(result, "Ook. Ook. ");
    }

    #[test]
    fn test_bf_to_ook_all_ops() {
        let result = convert_bf_to("+-<>.,[]", "ook").unwrap();
        assert!(result.contains("Ook. Ook. ")); // +
        assert!(result.contains("Ook! Ook! ")); // -
        assert!(result.contains("Ook. Ook? ")); // <
        assert!(result.contains("Ook? Ook. ")); // >
        assert!(result.contains("Ook! Ook. ")); // .
        assert!(result.contains("Ook. Ook! ")); // ,
        assert!(result.contains("Ook! Ook? ")); // [
        assert!(result.contains("Ook? Ook! ")); // ]
    }

    #[test]
    fn test_ook_to_bf() {
        let result = convert_to_bf("Ook. Ook. Ook! Ook! ", "ook").unwrap();
        assert_eq!(result, "+-");
    }

    #[test]
    fn test_bf_to_trollscript() {
        let result = convert_bf_to("+-", "trollscript").unwrap();
        assert_eq!(result, "oooool");
    }

    #[test]
    fn test_roundtrip_bf_ook_bf() {
        let original = "+++[>+<-]>.";
        let ook = convert_bf_to(original, "ook").unwrap();
        let back = convert_to_bf(&ook, "ook").unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn test_roundtrip_bf_troll_bf() {
        let original = "+++[>+<-]>.";
        let troll = convert_bf_to(original, "trollscript").unwrap();
        let back = convert_to_bf(&troll, "trollscript").unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn test_custom_dialect() {
        let d = Dialect::custom("test", "abcdefgh").unwrap();
        assert_eq!(d.tokens.len(), 8);
        assert_eq!(d.tokens[0], "a"); // +
        assert_eq!(d.tokens[7], "h"); // ]
    }

    #[test]
    fn test_custom_dialect_wrong_length() {
        assert!(Dialect::custom("bad", "abc").is_err());
    }

    #[test]
    fn test_unknown_dialect() {
        assert!(Dialect::by_name("nonexistent").is_err());
    }

    #[test]
    fn test_known_dialects() {
        assert!(Dialect::by_name("brainfuck").is_ok());
        assert!(Dialect::by_name("bf").is_ok());
        assert!(Dialect::by_name("ook").is_ok());
        assert!(Dialect::by_name("trollscript").is_ok());
    }

    #[test]
    fn test_strips_non_bf() {
        let result = convert_bf_to("+ hello +", "ook").unwrap();
        // Only + + should be converted, "hello" stripped
        assert_eq!(result, "Ook. Ook. Ook. Ook. ");
    }

    #[test]
    fn test_empty_input() {
        let result = convert_bf_to("", "ook").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_list_dialects() {
        let dialects = list_dialects();
        assert!(dialects.contains(&"brainfuck"));
        assert!(dialects.contains(&"ook"));
        assert!(dialects.contains(&"trollscript"));
    }

    #[test]
    fn test_hello_world_roundtrip() {
        let hello = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let ook = convert_bf_to(hello, "ook").unwrap();
        let back = convert_to_bf(&ook, "ook").unwrap();
        assert_eq!(back, hello);
    }
}
