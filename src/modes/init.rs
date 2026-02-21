use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

pub fn init_project() -> Result<()> {
    let toml_path = Path::new("ogre.toml");
    if toml_path.exists() {
        bail!("ogre.toml already exists in current directory");
    }

    // Derive project name from current directory
    let cwd = std::env::current_dir()?;
    let name = cwd
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("myproject");

    // Create directories if they don't exist
    let src_dir = Path::new("src");
    let tests_dir = Path::new("tests");

    if !src_dir.exists() {
        fs::create_dir_all(src_dir)?;
    }
    if !tests_dir.exists() {
        fs::create_dir_all(tests_dir)?;
    }

    // Write ogre.toml
    let toml_content = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
description = ""
author = ""
entry = "src/main.bf"

[build]
include = ["src/"]

[[tests]]
name = "Basic"
file = "tests/basic.json"
"#,
        name = name
    );
    fs::write(toml_path, toml_content)?;

    // Create src/main.bf if it doesn't exist
    let main_bf = src_dir.join("main.bf");
    if !main_bf.exists() {
        fs::write(&main_bf, "@fn main {\n    \n}\n\n@call main\n")?;
    }

    // Create tests/basic.json if it doesn't exist
    let test_file = tests_dir.join("basic.json");
    if !test_file.exists() {
        let test_content = r#"[
  {
    "name": "basic",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": ""
  }
]
"#;
        fs::write(&test_file, test_content)?;
    }

    println!("Initialized ogre project '{}' in current directory.", name);
    println!("  ogre.toml");
    if !main_bf.exists() {
        println!("  src/main.bf");
    }
    if !test_file.exists() {
        println!("  tests/basic.json");
    }

    Ok(())
}
