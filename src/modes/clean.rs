use anyhow::Result;
use std::fs;
use std::path::Path;

/// Remove build artifacts and cached files from the current project.
pub fn clean(base: &Path, verbose: bool) -> Result<usize> {
    let mut removed = 0;

    // Patterns to clean
    let patterns = ["*.o", "*.c", "*.wat", "*.wasm"];

    for pattern in &patterns {
        let full_pattern = base.join(pattern).to_string_lossy().to_string();
        for path in glob::glob(&full_pattern)
            .unwrap_or_else(|_| glob::glob("").unwrap())
            .flatten()
        {
            // Don't delete source files in lib/ or src/
            if let Some(parent) = path.parent() {
                let parent_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if parent_name == "lib" || parent_name == "src" {
                    continue;
                }
            }
            if verbose {
                println!("  removing: {}", path.display());
            }
            let _ = fs::remove_file(&path);
            removed += 1;
        }
    }

    // Clean .ogre-cache directory if it exists
    let cache_dir = base.join(".ogre-cache");
    if cache_dir.exists() {
        if verbose {
            println!("  removing: {}", cache_dir.display());
        }
        let _ = fs::remove_dir_all(&cache_dir);
        removed += 1;
    }

    // Clean target directory if it exists (for ogre projects, not cargo)
    let ogre_out = base.join("ogre-out");
    if ogre_out.exists() {
        if verbose {
            println!("  removing: {}", ogre_out.display());
        }
        let _ = fs::remove_dir_all(&ogre_out);
        removed += 1;
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_clean_empty_dir() {
        let dir = tempdir().unwrap();
        let removed = clean(dir.path(), false).unwrap();
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_clean_removes_artifacts() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("output.o"), "").unwrap();
        fs::write(dir.path().join("output.wat"), "").unwrap();
        let removed = clean(dir.path(), false).unwrap();
        assert_eq!(removed, 2);
        assert!(!dir.path().join("output.o").exists());
        assert!(!dir.path().join("output.wat").exists());
    }

    #[test]
    fn test_clean_preserves_source() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.c"), "// source").unwrap();
        let removed = clean(dir.path(), false).unwrap();
        assert_eq!(removed, 0);
        assert!(src_dir.join("main.c").exists());
    }

    #[test]
    fn test_clean_removes_cache_dir() {
        let dir = tempdir().unwrap();
        let cache = dir.path().join(".ogre-cache");
        fs::create_dir(&cache).unwrap();
        fs::write(cache.join("data"), "cached").unwrap();
        let removed = clean(dir.path(), false).unwrap();
        assert_eq!(removed, 1);
        assert!(!cache.exists());
    }
}
