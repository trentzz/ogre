use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn new_project(name: &str) -> Result<()> {
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

    // src/main.bf — starter file with an empty @fn main
    fs::write(
        dir.join("src/main.bf"),
        "@fn main {\n    \n}\n\n@call main\n",
    )?;

    // tests/basic.json — starter test pointing at src/main.bf
    let test_content = r#"[
  {
    "name": "basic",
    "brainfuck": "src/main.bf",
    "input": "",
    "output": ""
  }
]
"#;
    fs::write(dir.join("tests/basic.json"), test_content)?;

    println!("Created project '{}':", name);
    println!("  {}/ogre.toml", name);
    println!("  {}/src/main.bf", name);
    println!("  {}/tests/basic.json", name);

    Ok(())
}
