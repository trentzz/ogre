use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::OgreError;

#[derive(Deserialize, Debug)]
pub struct OgreProject {
    pub project: ProjectMeta,
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub tests: Vec<TestFileRef>,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
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
    pub tape_size: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct TestFileRef {
    pub name: Option<String>,
    pub file: String,
}

/// A project dependency, supporting path-based dependencies.
///
/// Example in ogre.toml:
/// ```toml
/// [dependencies]
/// mylib = { path = "../mylib" }
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Dependency {
    /// Path to the dependency directory (relative to this project's ogre.toml).
    pub path: Option<String>,
    /// Reserved for future registry-based versioning.
    pub version: Option<String>,
}

impl OgreProject {
    /// Load an `ogre.toml` from the given path.
    pub fn load(toml_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(toml_path)
            .map_err(|e| anyhow::anyhow!("cannot read {:?}: {}", toml_path, e))?;
        let project: OgreProject =
            toml::from_str(&content).map_err(|e| anyhow::anyhow!("invalid ogre.toml: {}", e))?;
        project.validate()?;
        Ok(project)
    }

    /// Validate the project configuration after parsing.
    pub fn validate(&self) -> Result<()> {
        if self.project.name.trim().is_empty() {
            return Err(OgreError::InvalidProject("project.name must not be empty".to_string()).into());
        }

        if !self.project.entry.ends_with(".bf") {
            return Err(OgreError::InvalidProject(
                format!("project.entry must end with .bf, got {:?}", self.project.entry)
            ).into());
        }

        if self.project.version.trim().is_empty() {
            return Err(OgreError::InvalidProject("project.version must not be empty".to_string()).into());
        }

        for (i, test_ref) in self.tests.iter().enumerate() {
            if !test_ref.file.ends_with(".json") {
                return Err(OgreError::InvalidProject(
                    format!("tests[{}].file must end with .json, got {:?}", i, test_ref.file)
                ).into());
            }
        }

        if let Some(build) = &self.build {
            if let Some(tape_size) = build.tape_size {
                if tape_size == 0 {
                    return Err(OgreError::InvalidProject(
                        "build.tape_size must be greater than 0".to_string()
                    ).into());
                }
            }
        }

        // Validate dependencies
        for (name, dep) in &self.dependencies {
            if dep.path.is_none() && dep.version.is_none() {
                return Err(OgreError::InvalidProject(
                    format!(
                        "dependency {:?} must have a 'path' or 'version' field",
                        name
                    ),
                )
                .into());
            }
        }

        Ok(())
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

    /// Resolve all dependency paths relative to `base`.
    /// Returns a map of dependency name -> resolved directory path.
    pub fn resolve_dependencies(&self, base: &Path) -> Result<HashMap<String, PathBuf>> {
        let mut resolved = HashMap::new();
        for (name, dep) in &self.dependencies {
            if let Some(path_str) = &dep.path {
                let dep_dir = base.join(path_str);
                if !dep_dir.exists() {
                    bail!(
                        "dependency {:?} path does not exist: {}",
                        name,
                        dep_dir.display()
                    );
                }
                // The dependency directory should contain an ogre.toml
                let dep_toml = dep_dir.join("ogre.toml");
                if !dep_toml.exists() {
                    bail!(
                        "dependency {:?} has no ogre.toml at {}",
                        name,
                        dep_dir.display()
                    );
                }
                resolved.insert(name.clone(), dep_dir);
            }
        }
        Ok(resolved)
    }

    /// Collect all @fn definitions from dependencies.
    /// Returns a map of function_name -> function_body.
    pub fn collect_dependency_functions(
        &self,
        base: &Path,
    ) -> Result<HashMap<String, String>> {
        use crate::modes::preprocess::Preprocessor;

        let mut all_functions = HashMap::new();
        let dep_dirs = self.resolve_dependencies(base)?;

        // Track visited dependencies to detect cycles
        let mut visited = std::collections::HashSet::new();

        for (name, dep_dir) in &dep_dirs {
            if !visited.insert(name.clone()) {
                continue;
            }

            let dep_toml_path = dep_dir.join("ogre.toml");
            let dep_project = Self::load(&dep_toml_path)?;

            // Collect include files from the dependency
            let include_files = dep_project.resolve_include_files(dep_dir)?;
            for file_path in &include_files {
                let functions =
                    Preprocessor::collect_functions_from_file(file_path)?;
                all_functions.extend(functions);
            }

            // Also collect from the dependency's entry file
            let entry = dep_project.entry_path(dep_dir);
            if entry.exists() {
                let functions =
                    Preprocessor::collect_functions_from_file(&entry)?;
                all_functions.extend(functions);
            }

            // Recursively collect from the dependency's own dependencies
            let nested = dep_project.collect_dependency_functions(dep_dir)?;
            all_functions.extend(nested);
        }

        Ok(all_functions)
    }

    /// Resolve all included .bf files relative to `base`.
    ///
    /// Each entry in `build.include` is either:
    ///   - a path ending with `/` → collect all `.bf` files directly inside that dir
    ///   - a glob pattern (contains `*` or `?`) → expand using glob
    ///   - a file path → include it directly
    pub fn resolve_include_files(&self, base: &Path) -> Result<Vec<PathBuf>> {
        let config = match &self.build {
            Some(b) => b,
            None => return Ok(vec![]),
        };

        let mut files = Vec::new();
        for entry in &config.include {
            if entry.contains('*') || entry.contains('?') {
                // Glob pattern — expand it
                let pattern = base.join(entry).to_string_lossy().to_string();
                let mut glob_files: Vec<PathBuf> = glob::glob(&pattern)
                    .map_err(|e| anyhow::anyhow!("invalid glob pattern {:?}: {}", entry, e))?
                    .filter_map(|r| r.ok())
                    .filter(|p| p.is_file())
                    .collect();
                glob_files.sort();
                files.extend(glob_files);
            } else if entry.ends_with('/') || entry.ends_with(std::path::MAIN_SEPARATOR) {
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

    // ---- Schema validation tests ----

    #[test]
    fn test_validate_empty_name_fails() {
        let toml = r#"
[project]
name = ""
version = "0.1.0"
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(
            err.to_string().contains("name must not be empty"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_validate_whitespace_name_fails() {
        let toml = r#"
[project]
name = "   "
version = "0.1.0"
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(err.to_string().contains("name must not be empty"));
    }

    #[test]
    fn test_validate_entry_not_bf_fails() {
        let toml = r#"
[project]
name = "myproject"
version = "0.1.0"
entry = "src/main.txt"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(
            err.to_string().contains("must end with .bf"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_validate_empty_version_fails() {
        let toml = r#"
[project]
name = "myproject"
version = ""
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(
            err.to_string().contains("version must not be empty"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_validate_test_file_not_json_fails() {
        let toml = r#"
[project]
name = "myproject"
version = "0.1.0"
entry = "src/main.bf"

[[tests]]
name = "Basic"
file = "tests/basic.txt"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(
            err.to_string().contains("must end with .json"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_validate_tape_size_zero_fails() {
        let toml = r#"
[project]
name = "myproject"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
tape_size = 0
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(
            err.to_string().contains("tape_size must be greater than 0"),
            "got: {}",
            err
        );
    }

    #[test]
    fn test_validate_valid_project_passes() {
        let toml = r#"
[project]
name = "myproject"
version = "0.1.0"
description = "A test project"
author = "Alice"
entry = "src/main.bf"

[build]
include = ["src/"]
tape_size = 30000

[[tests]]
name = "Basic"
file = "tests/basic.json"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        assert!(proj.validate().is_ok());
    }

    #[test]
    fn test_validate_minimal_valid_project_passes() {
        let toml = r#"
[project]
name = "x"
version = "0.1.0"
entry = "main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml).unwrap();
        assert!(proj.validate().is_ok());
    }

    // ---- Glob pattern tests ----

    #[test]
    fn test_glob_star_bf_matches() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a.bf"), "+").unwrap();
        fs::write(src.join("b.bf"), "-").unwrap();
        fs::write(src.join("c.txt"), "x").unwrap(); // should not match

        let toml_str = r#"
[project]
name = "test"
version = "0.1.0"
entry = "src/a.bf"

[build]
include = ["src/*.bf"]
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let files = proj.resolve_include_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("a.bf")));
        assert!(files.iter().any(|p| p.ends_with("b.bf")));
    }

    #[test]
    fn test_glob_recursive_matches() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        let sub = src.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(src.join("main.bf"), "+").unwrap();
        fs::write(sub.join("util.bf"), "-").unwrap();

        let toml_str = r#"
[project]
name = "test"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/**/*.bf"]
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let files = proj.resolve_include_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("main.bf")));
        assert!(files.iter().any(|p| p.ends_with("util.bf")));
    }

    #[test]
    fn test_glob_no_matches_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();

        let toml_str = r#"
[project]
name = "test"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/*.bf"]
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let files = proj.resolve_include_files(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_glob_question_mark_pattern() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a.bf"), "+").unwrap();
        fs::write(src.join("ab.bf"), "-").unwrap(); // won't match single ?

        let toml_str = r#"
[project]
name = "test"
version = "0.1.0"
entry = "src/a.bf"

[build]
include = ["src/?.bf"]
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let files = proj.resolve_include_files(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.bf"));
    }

    #[test]
    fn test_mixed_glob_and_directory_includes() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        let lib = dir.path().join("lib");
        fs::create_dir(&src).unwrap();
        fs::create_dir(&lib).unwrap();
        fs::write(src.join("main.bf"), "+").unwrap();
        fs::write(lib.join("utils.bf"), "-").unwrap();

        let toml_str = r#"
[project]
name = "test"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/", "lib/*.bf"]
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let files = proj.resolve_include_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    // ---- Dependency management tests ----

    #[test]
    fn test_parse_toml_with_dependencies() {
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "../mylib" }
utils = { path = "../utils" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        assert_eq!(proj.dependencies.len(), 2);
        assert!(proj.dependencies.contains_key("mylib"));
        assert!(proj.dependencies.contains_key("utils"));
        assert_eq!(
            proj.dependencies["mylib"].path.as_deref(),
            Some("../mylib")
        );
    }

    #[test]
    fn test_parse_toml_without_dependencies() {
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        assert!(proj.dependencies.is_empty());
    }

    #[test]
    fn test_validate_dependency_no_path_or_version() {
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
broken = {}
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let err = proj.validate().unwrap_err();
        assert!(
            err.to_string().contains("path"),
            "expected 'path' in error: {}",
            err
        );
    }

    #[test]
    fn test_validate_dependency_with_path_ok() {
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "../mylib" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        assert!(proj.validate().is_ok());
    }

    #[test]
    fn test_validate_dependency_with_version_ok() {
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { version = "0.1.0" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        assert!(proj.validate().is_ok());
    }

    #[test]
    fn test_resolve_dependencies_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "nonexistent_lib" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let err = proj.resolve_dependencies(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("does not exist"),
            "expected 'does not exist' in error: {}",
            err
        );
    }

    #[test]
    fn test_resolve_dependencies_no_ogre_toml() {
        let dir = tempfile::tempdir().unwrap();
        // Create the dependency directory but without ogre.toml
        let dep_dir = dir.path().join("mylib");
        fs::create_dir(&dep_dir).unwrap();

        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "mylib" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let err = proj.resolve_dependencies(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("no ogre.toml"),
            "expected 'no ogre.toml' in error: {}",
            err
        );
    }

    #[test]
    fn test_resolve_dependencies_valid_path() {
        let dir = tempfile::tempdir().unwrap();
        // Create a valid dependency
        let dep_dir = dir.path().join("mylib");
        fs::create_dir(&dep_dir).unwrap();
        fs::write(
            dep_dir.join("ogre.toml"),
            r#"
[project]
name = "mylib"
version = "0.1.0"
entry = "src/main.bf"
"#,
        )
        .unwrap();

        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "mylib" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let resolved = proj.resolve_dependencies(dir.path()).unwrap();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("mylib"));
        assert_eq!(resolved["mylib"], dep_dir);
    }

    #[test]
    fn test_collect_dependency_functions() {
        let dir = tempfile::tempdir().unwrap();

        // Create a dependency with @fn definitions
        let dep_dir = dir.path().join("mylib");
        let dep_src = dep_dir.join("src");
        fs::create_dir_all(&dep_src).unwrap();
        fs::write(
            dep_dir.join("ogre.toml"),
            r#"
[project]
name = "mylib"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
"#,
        )
        .unwrap();
        fs::write(dep_src.join("main.bf"), "@fn dep_hello { +++ }").unwrap();
        fs::write(dep_src.join("util.bf"), "@fn dep_util { --- }").unwrap();

        // Create the main project
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "mylib" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let fns = proj.collect_dependency_functions(dir.path()).unwrap();

        assert!(
            fns.contains_key("dep_hello"),
            "should contain dep_hello: {:?}",
            fns.keys().collect::<Vec<_>>()
        );
        assert!(
            fns.contains_key("dep_util"),
            "should contain dep_util: {:?}",
            fns.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_collect_dependency_functions_empty_deps() {
        let dir = tempfile::tempdir().unwrap();

        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let fns = proj.collect_dependency_functions(dir.path()).unwrap();
        assert!(fns.is_empty());
    }

    #[test]
    fn test_collect_dependency_functions_version_only_deps_skipped() {
        let dir = tempfile::tempdir().unwrap();

        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
future_pkg = { version = "1.0" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        // version-only deps should be skipped (no path to resolve)
        let fns = proj.collect_dependency_functions(dir.path()).unwrap();
        assert!(fns.is_empty());
    }

    #[test]
    fn test_nested_dependencies() {
        let dir = tempfile::tempdir().unwrap();

        // Create a nested dependency: myapp -> libA -> libB
        let lib_b = dir.path().join("libB");
        let lib_b_src = lib_b.join("src");
        fs::create_dir_all(&lib_b_src).unwrap();
        fs::write(
            lib_b.join("ogre.toml"),
            r#"
[project]
name = "libB"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
"#,
        )
        .unwrap();
        fs::write(lib_b_src.join("main.bf"), "@fn from_b { >>> }").unwrap();

        let lib_a = dir.path().join("libA");
        let lib_a_src = lib_a.join("src");
        fs::create_dir_all(&lib_a_src).unwrap();
        fs::write(
            lib_a.join("ogre.toml"),
            r#"
[project]
name = "libA"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]

[dependencies]
libB = { path = "../libB" }
"#,
        )
        .unwrap();
        fs::write(lib_a_src.join("main.bf"), "@fn from_a { +++ }").unwrap();

        // Main project depends on libA
        let toml_str = r#"
[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
libA = { path = "libA" }
"#;
        let proj: OgreProject = toml::from_str(toml_str).unwrap();
        let fns = proj.collect_dependency_functions(dir.path()).unwrap();

        assert!(fns.contains_key("from_a"), "should have from_a");
        assert!(fns.contains_key("from_b"), "should have from_b (nested)");
    }
}
