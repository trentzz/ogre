use anyhow::{bail, Result};

use super::preprocess;

pub fn list_modules() {
    println!("Available standard library modules:");
    println!();
    for name in preprocess::stdlib_modules() {
        let desc = match *name {
            "io" => "I/O utilities (print_newline, print_space, print_tab, print_bang, read_char, ...)",
            "math" => "Arithmetic (zero, inc, dec, double, triple, multiply_by_10, divmod_10, copy_right, ...)",
            "memory" => "Memory operations (clear, clear2-5, swap, copy_right, copy_left, dup, rotate3, ...)",
            "ascii" => "ASCII utilities (print_A, print_B, to_upper, to_lower, is_digit, digit_to_char, ...)",
            "debug" => "Debugging helpers (dump_cell, dump_and_newline, marker_start, marker_end)",
            "string" => "String/text operations (skip_char, skip_spaces, skip_line, read_decimal)",
            "logic" => "Boolean/conditional logic (not, bool, and, or, equal)",
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
