use anyhow::{bail, Result};

use super::preprocess;

pub fn list_modules() {
    println!("Available standard library modules:");
    println!();
    for name in preprocess::stdlib_modules() {
        let desc = match *name {
            "ascii" => "ASCII utilities (to_upper, to_lower, is_digit, is_alpha, is_upper, is_lower, ...)",
            "cli" => "CLI toolkit (skip_dashes, read_flag_char, read_arg, match_char, print_error_prefix, ...)",
            "convert" => "Data conversion (print_decimal, print_hex_digit, print_binary_8, atoi/itoa, ...)",
            "debug" => "Debugging helpers (dump_cell, dump_decimal, dump_hex, dump_range_5, separator, ...)",
            "io" => "I/O utilities (print_newline, print_space, read_char, print_char, flush_input, ...)",
            "logic" => "Boolean/conditional logic (not, bool, and, or, xor, equal, greater_than, ...)",
            "math" => "Arithmetic (zero, inc, dec, double, triple, multiply, square, modulo, clamp, ...)",
            "memory" => "Memory ops (clear, swap, dup, copy_right, rotate3, reverse3, fill_5, ...)",
            "string" => "String/text (skip_char, skip_spaces, read_decimal, read_line, print_string, ...)",
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
