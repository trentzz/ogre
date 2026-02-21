use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn new_project(name: &str, with_std: bool) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        anyhow::bail!("directory '{}' already exists", name);
    }

    // Create directory structure
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    // ogre.toml
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
    fs::write(dir.join("ogre.toml"), toml_content)?;

    // src/main.bf — starter file
    let main_bf = if with_std {
        "@import \"std/io.bf\"\n\n@fn main {\n    @call print_newline\n}\n\n@call main\n"
    } else {
        "@fn main {\n    \n}\n\n@call main\n"
    };
    fs::write(dir.join("src/main.bf"), main_bf)?;

    // tests/basic.json — starter test pointing at src/main.bf
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

    println!("Created project '{}':", name);
    println!("  {}/ogre.toml", name);
    println!("  {}/src/main.bf", name);
    println!("  {}/tests/basic.json", name);
    if with_std {
        println!("  (with standard library imports)");
    }

    Ok(())
}
