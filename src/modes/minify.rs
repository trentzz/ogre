use anyhow::Result;
use std::fs;
use std::path::Path;

use super::preprocess::Preprocessor;

/// Strip all non-BF characters from source, producing the smallest valid BF output.
pub fn minify_source(source: &str) -> String {
    source.chars().filter(|c| "+-<>.,[]".contains(*c)).collect()
}

/// Minify a file, optionally preprocessing it first.
pub fn minify_file(path: &Path, preprocess: bool) -> Result<String> {
    let source = if preprocess {
        Preprocessor::process_file(path)?
    } else {
        fs::read_to_string(path)?
    };
    Ok(minify_source(&source))
}

/// Write minified output to file or stdout.
pub fn minify_and_output(path: &Path, output: Option<&str>, preprocess: bool) -> Result<()> {
    let minified = minify_file(path, preprocess)?;
    match output {
        Some(out_path) => {
            fs::write(out_path, &minified)?;
            println!("Minified to: {} ({} bytes)", out_path, minified.len());
        }
        None => {
            println!("{}", minified);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_strips_comments() {
        assert_eq!(minify_source("+ this is a comment +"), "++");
    }

    #[test]
    fn test_minify_preserves_bf() {
        assert_eq!(minify_source("+-<>.,[]"), "+-<>.,[]");
    }

    #[test]
    fn test_minify_empty() {
        assert_eq!(minify_source("hello world"), "");
    }

    #[test]
    fn test_minify_strips_whitespace() {
        assert_eq!(minify_source("+ + + \n > > <"), "+++>><");
    }

    #[test]
    fn test_minify_preserves_complex_program() {
        let src = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.";
        assert_eq!(minify_source(src), src);
    }

    #[test]
    fn test_minify_strips_at_directives() {
        assert_eq!(minify_source("@fn foo { + } @call foo"), "+");
    }
}
