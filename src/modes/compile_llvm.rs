use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::ir::{Op, Program};
use super::preprocess::Preprocessor;
use crate::error::OgreError;
use crate::verbosity::Verbosity;

/// Optimization level for LLVM compilation.
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    pub fn as_flag(&self) -> &str {
        match self {
            OptLevel::O0 => "-O0",
            OptLevel::O1 => "-O1",
            OptLevel::O2 => "-O2",
            OptLevel::O3 => "-O3",
        }
    }

    pub fn parse_level(s: &str) -> Option<OptLevel> {
        match s {
            "0" => Some(OptLevel::O0),
            "1" => Some(OptLevel::O1),
            "2" => Some(OptLevel::O2),
            "3" => Some(OptLevel::O3),
            _ => None,
        }
    }
}

/// Generate LLVM IR text from a brainfuck IR Program.
pub fn generate_llvm_ir(program: &Program, tape_size: usize) -> String {
    let mut out = String::new();

    // Module header
    out.push_str("; ModuleID = 'brainfuck'\n");
    out.push_str("source_filename = \"brainfuck\"\n\n");

    // Tape as global zero-initialized array
    out.push_str(&format!(
        "@tape = global [{} x i8] zeroinitializer\n\n",
        tape_size
    ));

    // External function declarations
    out.push_str("declare i32 @putchar(i32)\n");
    out.push_str("declare i32 @getchar()\n");
    out.push_str("declare ptr @memset(ptr, i32, i64)\n\n");

    // Main function
    out.push_str("define i32 @main() {\n");
    out.push_str("entry:\n");
    out.push_str("  %ptr = alloca ptr\n");
    out.push_str("  store ptr @tape, ptr %ptr\n");

    let mut reg_counter: usize = 0;
    let mut loop_stack: Vec<usize> = Vec::new();
    let mut loop_counter: usize = 0;

    for op in &program.ops {
        match op {
            Op::Add(n) => {
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", r1, r0));
                out.push_str(&format!("  %t{} = add i8 %t{}, {}\n", r2, r1, n));
                let r3 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r3));
                out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", r2, r3));
            }
            Op::Sub(n) => {
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", r1, r0));
                out.push_str(&format!("  %t{} = sub i8 %t{}, {}\n", r2, r1, n));
                let r3 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r3));
                out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", r2, r3));
            }
            Op::Right(n) => {
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!(
                    "  %t{} = getelementptr i8, ptr %t{}, i64 {}\n",
                    r1, r0, n
                ));
                out.push_str(&format!("  store ptr %t{}, ptr %ptr\n", r1));
            }
            Op::Left(n) => {
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!(
                    "  %t{} = getelementptr i8, ptr %t{}, i64 -{}\n",
                    r1, r0, n
                ));
                out.push_str(&format!("  store ptr %t{}, ptr %ptr\n", r1));
            }
            Op::Output => {
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", r1, r0));
                out.push_str(&format!("  %t{} = zext i8 %t{} to i32\n", r2, r1));
                let r3 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = call i32 @putchar(i32 %t{})\n", r3, r2));
            }
            Op::Input => {
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = call i32 @getchar()\n", r0));
                out.push_str(&format!("  %t{} = trunc i32 %t{} to i8\n", r1, r0));
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r2));
                out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", r1, r2));
            }
            Op::JumpIfZero(_) => {
                let loop_id = loop_counter;
                loop_counter += 1;
                loop_stack.push(loop_id);

                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  br label %loop_cond_{}\n", loop_id));
                out.push_str(&format!("loop_cond_{}:\n", loop_id));
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", r1, r0));
                out.push_str(&format!("  %t{} = icmp ne i8 %t{}, 0\n", r2, r1));
                out.push_str(&format!(
                    "  br i1 %t{}, label %loop_body_{}, label %loop_end_{}\n",
                    r2, loop_id, loop_id
                ));
                out.push_str(&format!("loop_body_{}:\n", loop_id));
            }
            Op::JumpIfNonZero(_) => {
                if let Some(loop_id) = loop_stack.pop() {
                    out.push_str(&format!("  br label %loop_cond_{}\n", loop_id));
                    out.push_str(&format!("loop_end_{}:\n", loop_id));
                }
            }
            Op::Clear => {
                let r0 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  store i8 0, ptr %t{}\n", r0));
            }
            Op::Set(n) => {
                let r0 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  store i8 {}, ptr %t{}\n", n, r0));
            }
            Op::MoveAdd(offset) => {
                // src = *ptr; dst_ptr = ptr + offset; *dst_ptr += src; *ptr = 0
                let rs = reg_counter;
                reg_counter += 1;
                let rv = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", rs));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", rv, rs));
                let rd = reg_counter;
                reg_counter += 1;
                out.push_str(&format!(
                    "  %t{} = getelementptr i8, ptr %t{}, i64 {}\n",
                    rd, rs, offset
                ));
                let rdv = reg_counter;
                reg_counter += 1;
                let rsum = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", rdv, rd));
                out.push_str(&format!("  %t{} = add i8 %t{}, %t{}\n", rsum, rdv, rv));
                out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", rsum, rd));
                out.push_str(&format!("  store i8 0, ptr %t{}\n", rs));
            }
            Op::MoveSub(offset) => {
                let rs = reg_counter;
                reg_counter += 1;
                let rv = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", rs));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", rv, rs));
                let rd = reg_counter;
                reg_counter += 1;
                out.push_str(&format!(
                    "  %t{} = getelementptr i8, ptr %t{}, i64 {}\n",
                    rd, rs, offset
                ));
                let rdv = reg_counter;
                reg_counter += 1;
                let rsum = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", rdv, rd));
                out.push_str(&format!("  %t{} = sub i8 %t{}, %t{}\n", rsum, rdv, rv));
                out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", rsum, rd));
                out.push_str(&format!("  store i8 0, ptr %t{}\n", rs));
            }
            Op::ScanRight => {
                let loop_id = loop_counter;
                loop_counter += 1;
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  br label %scan_cond_{}\n", loop_id));
                out.push_str(&format!("scan_cond_{}:\n", loop_id));
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", r1, r0));
                out.push_str(&format!("  %t{} = icmp ne i8 %t{}, 0\n", r2, r1));
                out.push_str(&format!(
                    "  br i1 %t{}, label %scan_body_{}, label %scan_end_{}\n",
                    r2, loop_id, loop_id
                ));
                out.push_str(&format!("scan_body_{}:\n", loop_id));
                let r3 = reg_counter;
                reg_counter += 1;
                let r4 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r3));
                out.push_str(&format!(
                    "  %t{} = getelementptr i8, ptr %t{}, i64 1\n",
                    r4, r3
                ));
                out.push_str(&format!("  store ptr %t{}, ptr %ptr\n", r4));
                out.push_str(&format!("  br label %scan_cond_{}\n", loop_id));
                out.push_str(&format!("scan_end_{}:\n", loop_id));
            }
            Op::ScanLeft => {
                let loop_id = loop_counter;
                loop_counter += 1;
                let r0 = reg_counter;
                reg_counter += 1;
                let r1 = reg_counter;
                reg_counter += 1;
                let r2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  br label %scanl_cond_{}\n", loop_id));
                out.push_str(&format!("scanl_cond_{}:\n", loop_id));
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r0));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", r1, r0));
                out.push_str(&format!("  %t{} = icmp ne i8 %t{}, 0\n", r2, r1));
                out.push_str(&format!(
                    "  br i1 %t{}, label %scanl_body_{}, label %scanl_end_{}\n",
                    r2, loop_id, loop_id
                ));
                out.push_str(&format!("scanl_body_{}:\n", loop_id));
                let r3 = reg_counter;
                reg_counter += 1;
                let r4 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", r3));
                out.push_str(&format!(
                    "  %t{} = getelementptr i8, ptr %t{}, i64 -1\n",
                    r4, r3
                ));
                out.push_str(&format!("  store ptr %t{}, ptr %ptr\n", r4));
                out.push_str(&format!("  br label %scanl_cond_{}\n", loop_id));
                out.push_str(&format!("scanl_end_{}:\n", loop_id));
            }
            Op::MultiplyMove(targets) => {
                let rs = reg_counter;
                reg_counter += 1;
                let rv = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", rs));
                out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", rv, rs));
                for (offset, factor) in targets {
                    let rd = reg_counter;
                    reg_counter += 1;
                    out.push_str(&format!(
                        "  %t{} = getelementptr i8, ptr %t{}, i64 {}\n",
                        rd, rs, offset
                    ));
                    let rdv = reg_counter;
                    reg_counter += 1;
                    out.push_str(&format!("  %t{} = load i8, ptr %t{}\n", rdv, rd));
                    if *factor == 1 {
                        let rsum = reg_counter;
                        reg_counter += 1;
                        out.push_str(&format!("  %t{} = add i8 %t{}, %t{}\n", rsum, rdv, rv));
                        out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", rsum, rd));
                    } else {
                        let rmul = reg_counter;
                        reg_counter += 1;
                        let rsum = reg_counter;
                        reg_counter += 1;
                        out.push_str(&format!("  %t{} = mul i8 %t{}, {}\n", rmul, rv, factor));
                        out.push_str(&format!("  %t{} = add i8 %t{}, %t{}\n", rsum, rdv, rmul));
                        out.push_str(&format!("  store i8 %t{}, ptr %t{}\n", rsum, rd));
                    }
                }
                let rs2 = reg_counter;
                reg_counter += 1;
                out.push_str(&format!("  %t{} = load ptr, ptr %ptr\n", rs2));
                out.push_str(&format!("  store i8 0, ptr %t{}\n", rs2));
            }
        }
    }

    // Suppress unused variable warning
    let _ = reg_counter;

    out.push_str("  ret i32 0\n");
    out.push_str("}\n");

    out
}

/// Compile a brainfuck file to native binary via LLVM IR.
pub fn compile_to_llvm(
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
    opt_level: OptLevel,
    verbosity: Verbosity,
) -> Result<()> {
    let expanded = Preprocessor::process_file(file)?;
    compile_llvm_expanded(
        &expanded, file, output, keep, tape_size, opt_level, verbosity,
    )
}

/// Compile with pre-loaded dependency functions.
pub fn compile_to_llvm_with_deps(
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
    opt_level: OptLevel,
    verbosity: Verbosity,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let expanded = Preprocessor::process_file_with_deps(file, dep_functions)?;
    compile_llvm_expanded(
        &expanded, file, output, keep, tape_size, opt_level, verbosity,
    )
}

fn compile_llvm_expanded(
    expanded: &str,
    file: &Path,
    output: Option<&str>,
    keep: bool,
    tape_size: usize,
    opt_level: OptLevel,
    verbosity: Verbosity,
) -> Result<()> {
    let mut program = Program::from_source(expanded)?;
    program.optimize();

    let llvm_ir = generate_llvm_ir(&program, tape_size);

    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let out_path = output.unwrap_or(stem).to_string();

    // Write .ll file
    let ll_path = if keep {
        format!("{}.ll", stem)
    } else {
        let tmp = std::env::temp_dir().join(format!("ogre_{}.ll", stem));
        tmp.to_string_lossy().into_owned()
    };

    fs::write(&ll_path, &llvm_ir)?;

    // Try clang first (simplest), then fall back to llc + cc
    let compiled = try_compile_with_clang(&ll_path, &out_path, opt_level)
        .or_else(|_| try_compile_with_llc(&ll_path, &out_path, opt_level));

    match compiled {
        Ok(()) => {
            if !keep {
                let _ = fs::remove_file(&ll_path);
            }
            if !verbosity.is_quiet() {
                println!("Compiled to: {} (via LLVM)", out_path);
            }
            Ok(())
        }
        Err(_) => {
            if !keep {
                let _ = fs::remove_file(&ll_path);
            }
            Err(OgreError::CompilationFailed(
                "no LLVM toolchain found. Install clang or llc.".to_string(),
            )
            .into())
        }
    }
}

fn try_compile_with_clang(ll_path: &str, out_path: &str, opt_level: OptLevel) -> Result<()> {
    let status = Command::new("clang")
        .args([ll_path, "-o", out_path, opt_level.as_flag()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("clang compilation failed")
    }
}

fn try_compile_with_llc(ll_path: &str, out_path: &str, opt_level: OptLevel) -> Result<()> {
    let obj_path = format!("{}.o", out_path);

    // llc: .ll -> .o
    let status = Command::new("llc")
        .args([
            ll_path,
            "-filetype=obj",
            "-o",
            &obj_path,
            opt_level.as_flag(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()?;

    if !status.success() {
        let _ = fs::remove_file(&obj_path);
        anyhow::bail!("llc compilation failed");
    }

    // Link with cc
    let status = Command::new("cc")
        .args([&obj_path, "-o", out_path])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()?;

    let _ = fs::remove_file(&obj_path);

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("linking failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_llvm_ir_structure() {
        let prog = Program::from_source("+").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("ModuleID"));
        assert!(ir.contains("@tape = global [30000 x i8] zeroinitializer"));
        assert!(ir.contains("declare i32 @putchar(i32)"));
        assert!(ir.contains("declare i32 @getchar()"));
        assert!(ir.contains("define i32 @main()"));
        assert!(ir.contains("ret i32 0"));
    }

    #[test]
    fn test_generate_llvm_ir_add() {
        let prog = Program::from_source("+").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("add i8"));
    }

    #[test]
    fn test_generate_llvm_ir_sub() {
        let prog = Program::from_source("-").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("sub i8"));
    }

    #[test]
    fn test_generate_llvm_ir_right() {
        let prog = Program::from_source(">").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("getelementptr i8"));
        assert!(ir.contains("i64 1"));
    }

    #[test]
    fn test_generate_llvm_ir_left() {
        let prog = Program::from_source("<").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("getelementptr i8"));
        assert!(ir.contains("i64 -1"));
    }

    #[test]
    fn test_generate_llvm_ir_output() {
        let prog = Program::from_source(".").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("call i32 @putchar"));
        assert!(ir.contains("zext i8"));
    }

    #[test]
    fn test_generate_llvm_ir_input() {
        let prog = Program::from_source(",").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("call i32 @getchar()"));
        assert!(ir.contains("trunc i32"));
    }

    #[test]
    fn test_generate_llvm_ir_loop() {
        let prog = Program::from_source("[+]").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("loop_cond_0"));
        assert!(ir.contains("loop_body_0"));
        assert!(ir.contains("loop_end_0"));
        assert!(ir.contains("icmp ne i8"));
    }

    #[test]
    fn test_generate_llvm_ir_clear() {
        let mut prog = Program::from_source("[-]").unwrap();
        prog.optimize();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("store i8 0"));
    }

    #[test]
    fn test_generate_llvm_ir_set() {
        let mut prog = Program::from_source("[-]+++++").unwrap();
        prog.optimize();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("store i8 5"));
    }

    #[test]
    fn test_generate_llvm_ir_collapsed() {
        let prog = Program::from_source("+++").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("add i8") && ir.contains(", 3"));
    }

    #[test]
    fn test_generate_llvm_ir_tape_size() {
        let prog = Program::from_source("+").unwrap();
        let ir = generate_llvm_ir(&prog, 65536);
        assert!(ir.contains("[65536 x i8]"));
    }

    #[test]
    fn test_generate_llvm_ir_scan_right() {
        let mut prog = Program::from_source("[>]").unwrap();
        prog.optimize();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("scan_cond_"));
        assert!(ir.contains("scan_body_"));
        assert!(ir.contains("scan_end_"));
    }

    #[test]
    fn test_generate_llvm_ir_move_add() {
        let mut prog = Program::from_source("[->+<]").unwrap();
        prog.optimize();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("getelementptr i8"));
        assert!(ir.contains("store i8 0"));
    }

    #[test]
    fn test_generate_llvm_ir_nested_loops() {
        let prog = Program::from_source("[[+]]").unwrap();
        let ir = generate_llvm_ir(&prog, 30000);
        assert!(ir.contains("loop_cond_0"));
        assert!(ir.contains("loop_cond_1"));
    }

    #[test]
    fn test_opt_level_flags() {
        assert_eq!(OptLevel::O0.as_flag(), "-O0");
        assert_eq!(OptLevel::O1.as_flag(), "-O1");
        assert_eq!(OptLevel::O2.as_flag(), "-O2");
        assert_eq!(OptLevel::O3.as_flag(), "-O3");
    }

    #[test]
    fn test_opt_level_from_str() {
        assert!(OptLevel::parse_level("0").is_some());
        assert!(OptLevel::parse_level("3").is_some());
        assert!(OptLevel::parse_level("4").is_none());
        assert!(OptLevel::parse_level("x").is_none());
    }
}
