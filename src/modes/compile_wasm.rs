use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::ir::{Op, Program};
use super::preprocess::Preprocessor;
use crate::verbosity::Verbosity;

/// Generate WAT (WebAssembly Text Format) from a brainfuck IR Program.
///
/// The generated WAT module uses WASI for I/O:
/// - `fd_write` for output (`.` operator)
/// - `fd_read` for input (`,` operator)
/// - Linear memory for the BF tape
pub fn generate_wat(program: &Program, tape_size: usize) -> String {
    let mut out = String::new();

    // Module header
    out.push_str("(module\n");

    // Import WASI fd_write: (fd, iovs_ptr, iovs_len, nwritten_ptr) -> errno
    out.push_str("  (import \"wasi_snapshot_preview1\" \"fd_write\"\n");
    out.push_str("    (func $fd_write (param i32 i32 i32 i32) (result i32)))\n");

    // Import WASI fd_read: (fd, iovs_ptr, iovs_len, nread_ptr) -> errno
    out.push_str("  (import \"wasi_snapshot_preview1\" \"fd_read\"\n");
    out.push_str("    (func $fd_read (param i32 i32 i32 i32) (result i32)))\n");

    // Memory: tape + scratch space for I/O
    // Layout:
    //   [0 .. tape_size-1] = BF tape
    //   [tape_size .. tape_size+7] = iov buffer (ptr + len) for fd_write/fd_read
    //   [tape_size+8 .. tape_size+11] = nwritten/nread result
    let total_pages = (tape_size + 16).div_ceil(65536).max(1);
    out.push_str(&format!("  (memory (export \"memory\") {})\n", total_pages));

    // Global: data pointer (index into tape)
    out.push_str("  (global $dp (mut i32) (i32.const 0))\n");

    // Constants for I/O scratch area
    let iov_base = tape_size;
    let nwritten_addr = tape_size + 8;

    // Main function (WASI entry point)
    out.push_str("  (func $main (export \"_start\")\n");

    let mut indent_level: usize = 2;

    for op in &program.ops {
        let indent = "  ".repeat(indent_level);
        match op {
            Op::Add(n) => {
                // tape[dp] = (tape[dp] + n) & 0xFF
                out.push_str(&format!("{}(i32.store8\n", indent));
                out.push_str(&format!("{}  (global.get $dp)\n", indent));
                out.push_str(&format!(
                    "{}  (i32.and (i32.add (i32.load8_u (global.get $dp)) (i32.const {})) (i32.const 255)))\n",
                    indent, n
                ));
            }
            Op::Sub(n) => {
                // tape[dp] = (tape[dp] - n) & 0xFF
                out.push_str(&format!("{}(i32.store8\n", indent));
                out.push_str(&format!("{}  (global.get $dp)\n", indent));
                out.push_str(&format!(
                    "{}  (i32.and (i32.sub (i32.load8_u (global.get $dp)) (i32.const {})) (i32.const 255)))\n",
                    indent, n
                ));
            }
            Op::Right(n) => {
                out.push_str(&format!(
                    "{}(global.set $dp (i32.add (global.get $dp) (i32.const {})))\n",
                    indent, n
                ));
            }
            Op::Left(n) => {
                out.push_str(&format!(
                    "{}(global.set $dp (i32.sub (global.get $dp) (i32.const {})))\n",
                    indent, n
                ));
            }
            Op::Output => {
                // Set up iov: ptr = dp (the byte to print), len = 1
                out.push_str(&format!(
                    "{}(i32.store (i32.const {}) (global.get $dp))\n",
                    indent, iov_base
                ));
                out.push_str(&format!(
                    "{}(i32.store (i32.const {}) (i32.const 1))\n",
                    indent,
                    iov_base + 4
                ));
                // Call fd_write(stdout=1, iov_ptr, iov_count=1, nwritten_ptr)
                out.push_str(&format!(
                    "{}(drop (call $fd_write (i32.const 1) (i32.const {}) (i32.const 1) (i32.const {})))\n",
                    indent, iov_base, nwritten_addr
                ));
            }
            Op::Input => {
                // Set up iov: ptr = dp (where to store the byte), len = 1
                out.push_str(&format!(
                    "{}(i32.store (i32.const {}) (global.get $dp))\n",
                    indent, iov_base
                ));
                out.push_str(&format!(
                    "{}(i32.store (i32.const {}) (i32.const 1))\n",
                    indent,
                    iov_base + 4
                ));
                // Call fd_read(stdin=0, iov_ptr, iov_count=1, nread_ptr)
                out.push_str(&format!(
                    "{}(drop (call $fd_read (i32.const 0) (i32.const {}) (i32.const 1) (i32.const {})))\n",
                    indent, iov_base, nwritten_addr
                ));
            }
            Op::JumpIfZero(_) => {
                // block { loop { br_if (tape[dp] == 0) break_out_of_block; ... } }
                out.push_str(&format!("{}(block $B\n", indent));
                indent_level += 1;
                let inner_indent = "  ".repeat(indent_level);
                out.push_str(&format!("{}(loop $L\n", inner_indent));
                indent_level += 1;
                let inner2_indent = "  ".repeat(indent_level);
                // Break out of block if tape[dp] == 0
                out.push_str(&format!(
                    "{}(br_if $B (i32.eqz (i32.load8_u (global.get $dp))))\n",
                    inner2_indent
                ));
            }
            Op::JumpIfNonZero(_) => {
                // Branch back to loop if tape[dp] != 0
                let inner_indent = "  ".repeat(indent_level);
                out.push_str(&format!(
                    "{}(br_if $L (i32.load8_u (global.get $dp)))\n",
                    inner_indent
                ));
                // Close loop and block
                if indent_level > 2 {
                    indent_level -= 1;
                }
                let loop_indent = "  ".repeat(indent_level);
                out.push_str(&format!("{})\n", loop_indent)); // close loop
                if indent_level > 2 {
                    indent_level -= 1;
                }
                let block_indent = "  ".repeat(indent_level);
                out.push_str(&format!("{})\n", block_indent)); // close block
            }
            Op::Clear => {
                // tape[dp] = 0
                out.push_str(&format!(
                    "{}(i32.store8 (global.get $dp) (i32.const 0))\n",
                    indent
                ));
            }
            Op::MoveAdd(offset) => {
                // tape[dp + offset] += tape[dp]; tape[dp] = 0;
                let target_expr = if *offset >= 0 {
                    format!("(i32.add (global.get $dp) (i32.const {}))", offset)
                } else {
                    format!(
                        "(i32.sub (global.get $dp) (i32.const {}))",
                        offset.unsigned_abs()
                    )
                };
                out.push_str(&format!("{}(i32.store8\n", indent));
                out.push_str(&format!("{}  {}\n", indent, target_expr));
                out.push_str(&format!(
                    "{}  (i32.and (i32.add (i32.load8_u {}) (i32.load8_u (global.get $dp))) (i32.const 255)))\n",
                    indent, target_expr
                ));
                out.push_str(&format!(
                    "{}(i32.store8 (global.get $dp) (i32.const 0))\n",
                    indent
                ));
            }
            Op::MoveSub(offset) => {
                // tape[dp + offset] -= tape[dp]; tape[dp] = 0;
                let target_expr = if *offset >= 0 {
                    format!("(i32.add (global.get $dp) (i32.const {}))", offset)
                } else {
                    format!(
                        "(i32.sub (global.get $dp) (i32.const {}))",
                        offset.unsigned_abs()
                    )
                };
                out.push_str(&format!("{}(i32.store8\n", indent));
                out.push_str(&format!("{}  {}\n", indent, target_expr));
                out.push_str(&format!(
                    "{}  (i32.and (i32.sub (i32.load8_u {}) (i32.load8_u (global.get $dp))) (i32.const 255)))\n",
                    indent, target_expr
                ));
                out.push_str(&format!(
                    "{}(i32.store8 (global.get $dp) (i32.const 0))\n",
                    indent
                ));
            }
        }
    }

    out.push_str("  )\n"); // close func
    out.push_str(")\n"); // close module

    out
}

/// Compile a brainfuck file to WASM.
///
/// If `wat2wasm` is available on PATH, compiles WAT to binary WASM.
/// Otherwise, outputs the .wat file.
pub fn compile_to_wasm(
    file: &Path,
    output: Option<&str>,
    tape_size: usize,
    verbosity: Verbosity,
) -> Result<()> {
    let expanded = Preprocessor::process_file(file)?;
    let mut program = Program::from_source(&expanded)?;
    program.optimize();

    let wat_code = generate_wat(&program, tape_size);

    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let out_base = output.unwrap_or(stem);
    let wat_path = format!("{}.wat", out_base);
    let wasm_path = format!("{}.wasm", out_base);

    fs::write(&wat_path, &wat_code)?;

    // Try to convert WAT to WASM binary using wat2wasm
    if let Ok(status) = Command::new("wat2wasm")
        .args([&wat_path, "-o", &wasm_path])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
    {
        if status.success() {
            // Remove the .wat intermediate file
            let _ = fs::remove_file(&wat_path);
            if !verbosity.is_quiet() {
                println!("Compiled to: {}", wasm_path);
            }
            return Ok(());
        }
    }

    // wat2wasm not available or failed — keep the .wat file
    if !verbosity.is_quiet() {
        println!("Generated WAT: {}", wat_path);
        println!("  (install wabt's wat2wasm to compile to .wasm binary)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wat_empty() {
        let prog = Program::from_source("").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("(module"));
        assert!(wat.contains("(func $main"));
        assert!(wat.contains("_start"));
    }

    #[test]
    fn test_generate_wat_increment() {
        let prog = Program::from_source("+").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("i32.add"));
        assert!(wat.contains("i32.const 1"));
        assert!(wat.contains("i32.store8"));
    }

    #[test]
    fn test_generate_wat_decrement() {
        let prog = Program::from_source("-").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("i32.sub"));
        assert!(wat.contains("i32.store8"));
    }

    #[test]
    fn test_generate_wat_move_right() {
        let prog = Program::from_source(">").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("global.set $dp"));
        assert!(wat.contains("i32.add"));
    }

    #[test]
    fn test_generate_wat_move_left() {
        let prog = Program::from_source("<").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("global.set $dp"));
        assert!(wat.contains("i32.sub"));
    }

    #[test]
    fn test_generate_wat_output() {
        let prog = Program::from_source(".").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("fd_write"));
    }

    #[test]
    fn test_generate_wat_input() {
        let prog = Program::from_source(",").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("fd_read"));
    }

    #[test]
    fn test_generate_wat_loop() {
        let prog = Program::from_source("[+]").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("block $B"));
        assert!(wat.contains("loop $L"));
        assert!(wat.contains("br_if $B"));
        assert!(wat.contains("br_if $L"));
    }

    #[test]
    fn test_generate_wat_clear() {
        let mut prog = Program::from_source("[-]").unwrap();
        prog.optimize();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("i32.store8"));
        assert!(wat.contains("i32.const 0"));
    }

    #[test]
    fn test_generate_wat_collapsed_ops() {
        let prog = Program::from_source("+++").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("i32.const 3"));
    }

    #[test]
    fn test_generate_wat_memory_pages() {
        // 30000 bytes needs 1 page (64KB)
        let prog = Program::from_source("+").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("(memory (export \"memory\") 1)"));
    }

    #[test]
    fn test_generate_wat_large_tape() {
        // 100000 bytes needs 2 pages
        let prog = Program::from_source("+").unwrap();
        let wat = generate_wat(&prog, 100000);
        assert!(wat.contains("(memory (export \"memory\") 2)"));
    }

    #[test]
    fn test_generate_wat_wasi_imports() {
        let prog = Program::from_source("+").unwrap();
        let wat = generate_wat(&prog, 30000);
        assert!(wat.contains("wasi_snapshot_preview1"));
        assert!(wat.contains("fd_write"));
        assert!(wat.contains("fd_read"));
    }

    #[test]
    fn test_generate_wat_nested_loops() {
        let prog = Program::from_source("[[+]]").unwrap();
        let wat = generate_wat(&prog, 30000);
        // Should have two block/loop pairs
        let block_count = wat.matches("block $B").count();
        let loop_count = wat.matches("loop $L").count();
        assert_eq!(block_count, 2);
        assert_eq!(loop_count, 2);
    }
}
