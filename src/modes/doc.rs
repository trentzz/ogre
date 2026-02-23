use anyhow::Result;
use std::path::Path;

use super::preprocess::{get_stdlib_module, stdlib_modules, Preprocessor};

/// Generate documentation for a single brainfuck file.
pub fn generate_docs(path: &Path) -> Result<String> {
    let (_, functions, fn_docs) = Preprocessor::process_file_with_docs(path)?;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", filename));

    if functions.is_empty() {
        out.push_str("No functions defined.\n");
        return Ok(out);
    }

    out.push_str(&format!("{} function(s) defined:\n\n", functions.len()));

    let mut names: Vec<&String> = functions.keys().collect();
    names.sort();

    for name in names {
        out.push_str(&format!("## `@fn {}`\n\n", name));

        if let Some(doc) = fn_docs.get(name) {
            out.push_str(doc);
            out.push_str("\n\n");
        }

        let body = &functions[name];
        let trimmed = body.trim();
        if !trimmed.is_empty() {
            out.push_str("```brainfuck\n");
            out.push_str(trimmed);
            out.push_str("\n```\n\n");
        } else {
            out.push_str("*(empty body)*\n\n");
        }
    }

    Ok(out)
}

/// Generate documentation for all standard library modules.
pub fn generate_stdlib_docs() -> Result<String> {
    let mut out = String::new();
    out.push_str("# ogre Standard Library Reference\n\n");

    for module_name in stdlib_modules() {
        let source = get_stdlib_module(module_name).unwrap();
        out.push_str(&format!("## std/{}\n\n", module_name));

        let (_, functions, fn_docs) =
            Preprocessor::process_source_with_docs(source, Path::new("."))?;

        let mut names: Vec<&String> = functions.keys().collect();
        names.sort();

        for name in &names {
            out.push_str(&format!("### `@fn {}`\n\n", name));

            if let Some(doc) = fn_docs.get(*name) {
                out.push_str(doc);
                out.push_str("\n\n");
            }

            let body = &functions[*name];
            let trimmed = body.trim();
            if !trimmed.is_empty() {
                out.push_str("```brainfuck\n");
                out.push_str(trimmed);
                out.push_str("\n```\n\n");
            }
        }

        if names.is_empty() {
            out.push_str("No functions defined.\n\n");
        }
    }

    Ok(out)
}

/// Generate documentation and print it.
pub fn doc_and_output(path: Option<&Path>, stdlib: bool, output: Option<&str>) -> Result<()> {
    let docs = if stdlib {
        generate_stdlib_docs()?
    } else if let Some(p) = path {
        generate_docs(p)?
    } else {
        anyhow::bail!("provide a file path or use --stdlib");
    };

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &docs)?;
            println!("Documentation written to: {}", out_path);
        }
        None => {
            print!("{}", docs);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_generate_docs_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("empty.bf");
        std::fs::File::create(&file)
            .unwrap()
            .write_all(b"+++")
            .unwrap();

        let docs = generate_docs(&file).unwrap();
        assert!(docs.contains("empty.bf"));
        assert!(docs.contains("No functions defined"));
    }

    #[test]
    fn test_generate_docs_with_functions() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.bf");
        std::fs::File::create(&file)
            .unwrap()
            .write_all(b"@fn hello { +++ }\n@fn world { --- }")
            .unwrap();

        let docs = generate_docs(&file).unwrap();
        assert!(docs.contains("@fn hello"));
        assert!(docs.contains("@fn world"));
        assert!(docs.contains("+++"));
        assert!(docs.contains("---"));
    }

    #[test]
    fn test_generate_docs_with_doc_comments() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("documented.bf");
        std::fs::File::create(&file)
            .unwrap()
            .write_all(b"@doc Adds three to the current cell.\n@fn add3 { +++ }")
            .unwrap();

        let docs = generate_docs(&file).unwrap();
        assert!(docs.contains("@fn add3"));
        assert!(docs.contains("Adds three to the current cell."));
    }

    #[test]
    fn test_generate_docs_multi_line_doc() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("multi_doc.bf");
        std::fs::File::create(&file)
            .unwrap()
            .write_all(b"@doc First line.\n@doc Second line.\n@fn multi { + }")
            .unwrap();

        let docs = generate_docs(&file).unwrap();
        assert!(docs.contains("First line."));
        assert!(docs.contains("Second line."));
    }

    #[test]
    fn test_generate_stdlib_docs() {
        let docs = generate_stdlib_docs().unwrap();
        assert!(docs.contains("std/io"));
        assert!(docs.contains("std/math"));
        assert!(docs.contains("print_newline"));
        assert!(docs.contains("zero"));
    }

    #[test]
    fn test_docs_sorted_alphabetically() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sorted.bf");
        std::fs::File::create(&file)
            .unwrap()
            .write_all(b"@fn zebra { + }\n@fn alpha { - }")
            .unwrap();

        let docs = generate_docs(&file).unwrap();
        let alpha_pos = docs.find("@fn alpha").unwrap();
        let zebra_pos = docs.find("@fn zebra").unwrap();
        assert!(alpha_pos < zebra_pos);
    }
}
