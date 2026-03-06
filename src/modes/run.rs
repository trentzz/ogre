use anyhow::Result;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

use std::collections::HashMap;

use super::interpreter::{Interpreter, DEFAULT_TAPE_SIZE};
use super::preprocess::Preprocessor;

pub fn run_file(path: &Path) -> Result<()> {
    run_file_with_tape_size(path, DEFAULT_TAPE_SIZE)
}

pub fn run_file_with_tape_size(path: &Path, tape_size: usize) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let mut interp = Interpreter::with_live_stdin_and_tape_size(&expanded, tape_size)?;
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}

/// Run a file with pre-loaded dependency functions available.
pub fn run_file_with_deps(
    path: &Path,
    tape_size: usize,
    dep_functions: &HashMap<String, String>,
) -> Result<()> {
    let expanded = Preprocessor::process_file_with_deps(path, dep_functions)?;
    let mut interp = Interpreter::with_live_stdin_and_tape_size(&expanded, tape_size)?;
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}

/// Run a file with CLI arguments passed via `--`.
/// Arguments are joined with spaces, terminated with newline, and fed as
/// input prefix to the BF program. After those bytes are consumed, further
/// `,` reads come from real stdin.
pub fn run_file_with_args(path: &Path, tape_size: usize, program_args: &[String]) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let arg_input = format!("{}\n", program_args.join(" "));
    let mut interp = Interpreter::with_input_and_tape_size(&expanded, &arg_input, tape_size)?;
    interp.set_live_stdin_fallback();
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}

/// Run a file with CLI arguments and pre-loaded dependency functions.
pub fn run_file_with_args_and_deps(
    path: &Path,
    tape_size: usize,
    program_args: &[String],
    dep_functions: &HashMap<String, String>,
) -> Result<()> {
    let expanded = Preprocessor::process_file_with_deps(path, dep_functions)?;
    let arg_input = format!("{}\n", program_args.join(" "));
    let mut interp = Interpreter::with_input_and_tape_size(&expanded, &arg_input, tape_size)?;
    interp.set_live_stdin_fallback();
    interp.set_streaming(true);
    interp.run()?;
    Ok(())
}

/// Run a file in watch mode — re-run whenever the file changes.
pub fn run_file_watch(path: &Path, tape_size: usize) -> Result<()> {
    use colored::Colorize;

    let canonical = std::fs::canonicalize(path)?;

    // Initial run
    println!("{} {}", "Watching".green().bold(), path.display());
    run_once(path, tape_size);

    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if event.kind.is_modify() || event.kind.is_create() {
                let _ = tx.send(());
            }
        }
    })?;

    // Watch the file's parent directory (more reliable than watching the file directly)
    let watch_dir = canonical.parent().unwrap_or_else(|| Path::new("."));
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;

    println!("{}", "Press Ctrl+C to stop watching.".dimmed());

    // Debounce: wait for events, then re-run
    while let Ok(()) = rx.recv() {
        // Debounce: drain any additional events that arrived quickly
        std::thread::sleep(Duration::from_millis(100));
        while rx.try_recv().is_ok() {}

        // Clear terminal
        print!("\x1B[2J\x1B[H");
        let now = chrono_timestamp();
        println!(
            "{} {} at {}",
            "Re-running".green().bold(),
            path.display(),
            now.dimmed()
        );
        run_once(path, tape_size);
    }

    Ok(())
}

/// Run a file once, printing errors instead of propagating them.
fn run_once(path: &Path, tape_size: usize) {
    use colored::Colorize;

    match Preprocessor::process_file(path) {
        Err(e) => {
            eprintln!("{} {}", "Preprocess error:".red(), e);
        }
        Ok(expanded) => {
            match Interpreter::with_tape_size(&expanded, tape_size) {
                Err(e) => {
                    eprintln!("{} {}", "Parse error:".red(), e);
                }
                Ok(mut interp) => {
                    interp.set_streaming(true);
                    if let Err(e) = interp.run() {
                        eprintln!("{} {}", "Runtime error:".red(), e);
                    }
                    // Print any buffered output
                    if !interp.output().is_empty() {
                        print!("{}", String::from_utf8_lossy(interp.output()));
                    }
                    println!();
                }
            }
        }
    }
}

/// Get a simple timestamp string.
fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}
