use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn new_project(name: &str) -> Result<()> {
    let dir = Path::new(name);
    if dir.exists() {
        anyhow::bail!("directory '{}' already exists", name);
    }

    fs::create_dir_all(dir)?;

    // Create starter .bf file
    let bf_path = dir.join(format!("{}.bf", name));
    fs::write(
        &bf_path,
        "Hello, World! (replace this with your brainfuck program)\n",
    )?;

    // Create starter tests.json
    let tests_path = dir.join("tests.json");
    let tests_content = format!(
        r#"[
  {{
    "name": "hello world",
    "brainfuck": "{}/{}.bf",
    "input": "",
    "output": ""
  }}
]
"#,
        name, name
    );
    fs::write(&tests_path, tests_content)?;

    println!("Created project '{}':", name);
    println!("  {}", bf_path.display());
    println!("  {}", tests_path.display());

    Ok(())
}
