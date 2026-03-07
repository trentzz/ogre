use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::verbosity::Verbosity;

/// Available project templates.
pub fn list_templates() -> Vec<&'static str> {
    vec!["basic", "game", "library", "converter"]
}

pub fn new_project(name: &str, with_std: bool) -> Result<()> {
    new_project_ex(name, with_std, None, Verbosity::Normal)
}

pub fn new_project_ex(
    name: &str,
    with_std: bool,
    template: Option<&str>,
    verbosity: Verbosity,
) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        anyhow::bail!("directory '{}' already exists", name);
    }

    let tmpl = template.unwrap_or("basic");

    match tmpl {
        "basic" => create_basic(dir, name, with_std)?,
        "game" => create_game(dir, name)?,
        "library" => create_library(dir, name)?,
        "converter" => create_converter(dir, name)?,
        other => anyhow::bail!(
            "unknown template '{}'. Available: {}",
            other,
            list_templates().join(", ")
        ),
    }

    if !verbosity.is_quiet() {
        println!("Created project '{}' (template: {}):", name, tmpl);
        print_tree(dir);
    }

    Ok(())
}

fn print_tree(dir: &Path) {
    for entry in walkdir(dir) {
        let rel = entry
            .strip_prefix(dir.parent().unwrap_or(Path::new(".")))
            .unwrap_or(&entry);
        println!("  {}", rel.display());
    }
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn write_toml(dir: &Path, name: &str, description: &str, includes: &[&str]) -> Result<()> {
    let include_str = includes
        .iter()
        .map(|i| format!("\"{}\"", i))
        .collect::<Vec<_>>()
        .join(", ");
    let content = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
description = "{description}"
author = ""
entry = "src/main.bf"

[build]
include = [{includes}]

[[tests]]
name = "Basic"
file = "tests/basic.json"
"#,
        name = name,
        description = description,
        includes = include_str,
    );
    fs::write(dir.join("ogre.toml"), content)?;
    Ok(())
}

// ---- Templates ----

fn create_basic(dir: &Path, name: &str, with_std: bool) -> Result<()> {
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    write_toml(dir, name, "", &["src/"])?;

    let main_bf = if with_std {
        "@import \"std/io.bf\"\n\n@fn main {\n    @call print_newline\n}\n\n@call main\n"
    } else {
        "@fn main {\n    \n}\n\n@call main\n"
    };
    fs::write(dir.join("src/main.bf"), main_bf)?;

    let test_output = if with_std { "\\n" } else { "" };
    let test_content = format!(
        r#"[
  {{
    "name": "basic",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "{}"
  }}
]
"#,
        test_output
    );
    fs::write(dir.join("tests/basic.json"), test_content)?;
    Ok(())
}

fn create_game(dir: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    write_toml(dir, name, "An interactive brainfuck game", &["src/"])?;

    let main_bf = r#"@import "std/io.bf"
@import "std/ascii.bf"
@import "std/math.bf"

@doc Main game loop: reads input and responds
@fn game_loop {
    @call print_newline
    ,                    read a character
    [                    loop while input is not zero
        .                echo the character
        @call print_newline
        ,                read next character
    ]
}

@call game_loop
"#;
    fs::write(dir.join("src/main.bf"), main_bf)?;

    let test_content = r#"[
  {
    "name": "echo single char",
    "brainfuck": "src/main.bf",
    "input": "A\0",
    "output": "\nA\n"
  }
]
"#;
    fs::write(dir.join("tests/basic.json"), test_content)?;
    Ok(())
}

fn create_library(dir: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    write_toml(dir, name, "A reusable brainfunct library", &["src/"])?;

    let main_bf = format!(
        r#"@doc Add two to the current cell.
@fn {name}_add2 {{
    ++
}}

@doc Double the current cell value using a temp cell.
@fn {name}_double {{
    [->++<]>[-<+>]<
}}

Test the library functions:
+++++          set cell 0 to 5
@call {name}_double   cell 0 is now 10
@call {name}_add2     cell 0 is now 12
"#,
        name = name
    );
    fs::write(dir.join("src/main.bf"), main_bf)?;

    let test_content = r#"[
  {
    "name": "double and add",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": "",
    "output_regex": ".*"
  }
]
"#
    .to_string();
    fs::write(dir.join("tests/basic.json"), test_content)?;
    Ok(())
}

fn create_converter(dir: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("lib"))?;
    fs::create_dir_all(dir.join("tests"))?;

    write_toml(dir, name, "A CLI text converter tool", &["src/", "lib/"])?;

    let main_bf = r#"@import "std/io.bf"
@import "std/ascii.bf"
@import "std/math.bf"
@import "lib/transform.bf"

@doc Read characters and transform each one, printing the result.
@fn process {
    ,
    [
        @call transform
        .
        [-]
        ,
    ]
}

@call process
"#;
    fs::write(dir.join("src/main.bf"), main_bf)?;

    let lib_bf = r#"@import "std/ascii.bf"

@doc Transform a character (default: convert to uppercase).
@fn transform {
    @call to_upper
}
"#;
    fs::write(dir.join("lib/transform.bf"), lib_bf)?;

    let test_content = r#"[
  {
    "name": "uppercase conversion",
    "brainfuck": "src/main.bf",
    "input": "hello",
    "output": "HELLO"
  }
]
"#;
    fs::write(dir.join("tests/basic.json"), test_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates() {
        let templates = list_templates();
        assert!(templates.contains(&"basic"));
        assert!(templates.contains(&"game"));
        assert!(templates.contains(&"library"));
        assert!(templates.contains(&"converter"));
    }

    #[test]
    fn test_create_basic() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_basic");
        create_basic(&project_dir, "test_basic", false).unwrap();
        assert!(project_dir.join("ogre.toml").exists());
        assert!(project_dir.join("src/main.bf").exists());
        assert!(project_dir.join("tests/basic.json").exists());
    }

    #[test]
    fn test_create_basic_with_std() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_std");
        create_basic(&project_dir, "test_std", true).unwrap();
        let main = fs::read_to_string(project_dir.join("src/main.bf")).unwrap();
        assert!(main.contains("@import"));
    }

    #[test]
    fn test_create_game() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_game");
        create_game(&project_dir, "test_game").unwrap();
        assert!(project_dir.join("ogre.toml").exists());
        let main = fs::read_to_string(project_dir.join("src/main.bf")).unwrap();
        assert!(main.contains("game_loop"));
    }

    #[test]
    fn test_create_library() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_lib");
        create_library(&project_dir, "test_lib").unwrap();
        assert!(project_dir.join("ogre.toml").exists());
        let main = fs::read_to_string(project_dir.join("src/main.bf")).unwrap();
        assert!(main.contains("@doc"));
        assert!(main.contains("@fn"));
    }

    #[test]
    fn test_create_converter() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_conv");
        create_converter(&project_dir, "test_conv").unwrap();
        assert!(project_dir.join("ogre.toml").exists());
        assert!(project_dir.join("lib/transform.bf").exists());
        let main = fs::read_to_string(project_dir.join("src/main.bf")).unwrap();
        assert!(main.contains("@import"));
    }

    #[test]
    fn test_unknown_template() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_unknown");
        let result = new_project_ex(
            project_dir.to_str().unwrap(),
            false,
            Some("nonexistent"),
            Verbosity::Quiet,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_project() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_dup");
        fs::create_dir_all(&project_dir).unwrap();
        let result = new_project_ex(project_dir.to_str().unwrap(), false, None, Verbosity::Quiet);
        assert!(result.is_err());
    }
}
