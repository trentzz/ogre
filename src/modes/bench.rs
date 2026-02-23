use anyhow::Result;
use std::path::Path;
use std::time::Instant;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;
use crate::verbosity::Verbosity;

pub struct BenchResult {
    pub instruction_count: u64,
    pub cells_touched: usize,
    pub elapsed_ms: f64,
    pub output_bytes: usize,
}

/// Benchmark a file with pre-loaded dependency functions.
pub fn bench_file_with_deps(
    path: &Path,
    _tape_size: usize,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<BenchResult> {
    let expanded = Preprocessor::process_file_with_deps(path, dep_functions)?;
    bench_expanded(&expanded)
}

pub fn bench_file(path: &Path, _tape_size: usize) -> Result<BenchResult> {
    let expanded = Preprocessor::process_file(path)?;
    bench_expanded(&expanded)
}

fn bench_expanded(expanded: &str) -> Result<BenchResult> {
    let mut interp = Interpreter::new_optimized(expanded)?;

    let start = Instant::now();
    interp.run()?;
    let elapsed = start.elapsed();

    Ok(BenchResult {
        instruction_count: interp.instruction_count,
        cells_touched: interp.cells_touched_count(),
        elapsed_ms: elapsed.as_secs_f64() * 1000.0,
        output_bytes: interp.output().len(),
    })
}

pub fn bench_and_report(path: &Path, tape_size: usize) -> Result<()> {
    bench_and_report_ex(path, tape_size, Verbosity::Normal)
}

/// Benchmark with pre-loaded dependency functions.
pub fn bench_and_report_with_deps(
    path: &Path,
    tape_size: usize,
    verbosity: Verbosity,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<()> {
    if !verbosity.is_quiet() {
        println!("Benchmarking: {}", path.display());
        println!();
    }

    let result = bench_file_with_deps(path, tape_size, dep_functions)?;
    print_bench_result(&result, tape_size, verbosity)
}

pub fn bench_and_report_ex(path: &Path, tape_size: usize, verbosity: Verbosity) -> Result<()> {
    if !verbosity.is_quiet() {
        println!("Benchmarking: {}", path.display());
        println!();
    }

    let result = bench_file(path, tape_size)?;
    print_bench_result(&result, tape_size, verbosity)
}

fn print_bench_result(result: &BenchResult, tape_size: usize, verbosity: Verbosity) -> Result<()> {
    println!(
        "  Instructions executed: {}",
        format_number(result.instruction_count)
    );
    println!("  Cells touched:         {}", result.cells_touched);
    println!("  Output bytes:          {}", result.output_bytes);
    println!("  Wall time:             {:.3} ms", result.elapsed_ms);

    if result.elapsed_ms > 0.0 {
        let mips = result.instruction_count as f64 / result.elapsed_ms / 1000.0;
        println!("  Throughput:            {:.1} MIPS", mips);
    }

    if verbosity.is_verbose() {
        println!("  Tape size:             {}", tape_size);
    }

    Ok(())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_bench_hello_world() {
        let path = Path::new("tests/brainfuck_scripts/hello_world.bf");
        let result = bench_file(path, 30_000).unwrap();
        assert!(result.instruction_count > 0);
        assert!(result.cells_touched > 0);
        assert!(result.output_bytes > 0);
        assert!(result.elapsed_ms >= 0.0);
    }

    #[test]
    fn test_bench_cells_touched_correct() {
        let path = Path::new("tests/brainfuck_scripts/simple_multiply.bf");
        let result = bench_file(path, 30_000).unwrap();
        // ++++[>+++<-] touches cells 0 and 1
        assert!(result.cells_touched >= 2);
    }

    #[test]
    fn test_bench_instruction_count_reasonable() {
        let path = Path::new("tests/brainfuck_scripts/simple_multiply.bf");
        let result = bench_file(path, 30_000).unwrap();
        // ++++[>+++<-] = 4 inits + 4 loops * (>+++<-) = 4 + 4*5 = 24 ops approx
        assert!(result.instruction_count > 10);
        assert!(result.instruction_count < 1000);
    }
}
