use anyhow::{bail, Result};

use super::preprocess;

pub fn list_modules() {
    println!("Available standard library modules:");
    println!();
    for name in preprocess::stdlib_modules() {
        let desc = match *name {
            "io" => "I/O utilities (print_newline, print_space, read_char, print_char, print_zero)",
            "math" => "Arithmetic (zero, inc, dec, inc10, double, add_to_next, move_right, move_left, copy_right)",
            "memory" => "Memory operations (clear, clear2, clear3, swap, push_right, pull_left)",
            "ascii" => "ASCII character output (print_A, print_B, print_exclaim, print_dash, print_colon)",
            "debug" => "Debugging helpers (dump_cell, dump_and_newline, marker_start, marker_end)",
            _ => "",
        };
        println!("  std/{}.bf — {}", name, desc);
    }
    println!();
    println!("Usage: @import \"std/io.bf\"");
}

pub fn show_module(name: &str) -> Result<()> {
    match preprocess::get_stdlib_module(name) {
        Some(source) => {
            println!("=== std/{}.bf ===", name);
            println!();
            print!("{}", source);
            Ok(())
        }
        None => {
            bail!(
                "unknown module: '{}'. Run `ogre stdlib list` to see available modules.",
                name
            );
        }
    }
}
