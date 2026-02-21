use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct OgreProject {
    pub project: ProjectMeta,
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub tests: Vec<TestFileRef>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub entry: String,
}

#[derive(Deserialize, Debug)]
pub struct BuildConfig {
    pub include: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TestFileRef {
    pub name: Option<String>,
    pub file: String,
}

impl OgreProject {
    /// Load an `ogre.toml` from the given path.
    pub fn load(toml_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(toml_path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", toml_path, e))?;
        let project: OgreProject =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("invalid ogre.toml: {}", e))?;
        Ok(project)
    }

    /// Walk the current directory upward looking for `ogre.toml`.
    /// Returns `(project, directory_containing_ogre_toml)` or `None`.
    pub fn find() -> Result<Option<(Self, PathBuf)>> {
        let mut dir = std::env::current_dir()?;
        loop {
            let candidate = dir.join("ogre.toml");
            if candidate.exists() {
                let project = Self::load(&candidate)?;
                return Ok(Some((project, dir)));
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => return Ok(None),
            }
        }
    }

    /// Resolve the entry file path relative to the project's base directory.
    pub fn entry_path(&self, base: &Path) -> PathBuf {
        base.join(&self.project.entry)
    }

    /// Resolve all included .bf files relative to `base`.
    ///
    /// Each entry in `build.include` is either:
    ///   - a path ending with `/` → collect all `.bf` files directly inside that dir
    ///   - a file path → include it directly
    pub fn resolve_include_files(&self, base: &Path) -> Result<Vec<PathBuf>> {
        let config = match &self.build {
            Some(b) => b,
            None => return Ok(vec![]),
        };

        let mut files = Vec::new();
        for entry in &config.include {
            if entry.ends_with('/') || entry.ends_with(std::path::MAIN_SEPARATOR) {
                // Directory — collect all .bf files directly inside (non-recursive)
                let dir = base.join(entry);
                if !dir.is_dir() {
                    bail!("include directory not found: {}", dir.display());
                }
                let mut dir_files: Vec<PathBuf> = fs::read_dir(&dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("bf"))
                    .collect();
                dir_files.sort();
                files.extend(dir_files);
            } else {
                let path = base.join(entry);
                if !path.exists() {
                    bail!("include file not found: {}", path.display());
                }
                files.push(path);
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_toml() {
        let toml = r#"
[project]
name = "myproject"
version = "0.1.0"
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        assert_eq!(proj.project.name, "myproject");
        assert_eq!(proj.project.entry, "src/main.bf");
        assert!(proj.build.is_none());
        assert!(proj.tests.is_empty());
    }

    #[test]
    fn test_parse_full_toml() {
        let toml = r#"
[project]
name = "myproject"
version = "0.1.0"
description = "My BF project"
author = "Alice"
entry = "src/main.bf"

[build]
include = ["src/", "lib/utils.bf"]

[[tests]]
name = "Basic"
file = "tests/basic.json"

[[tests]]
file = "tests/advanced.json"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        assert_eq!(proj.project.author.as_deref(), Some("Alice"));
        let build = proj.build.unwrap();
        assert_eq!(build.include.len(), 2);
        assert_eq!(proj.tests.len(), 2);
        assert_eq!(proj.tests[0].name.as_deref(), Some("Basic"));
        assert!(proj.tests[1].name.is_none());
    }

    #[test]
    fn test_entry_path() {
        let toml = r#"
[project]
name = "x"
version = "0.1.0"
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let base = Path::new("/home/user/myproject");
        let entry = proj.entry_path(base);
        assert_eq!(entry, Path::new("/home/user/myproject/src/main.bf"));
    }
}
