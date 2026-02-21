use anyhow::Result;
use std::path::Path;
use std::time::Instant;

use super::interpreter::Interpreter;
use super::preprocess::Preprocessor;

pub struct BenchResult {
    pub instruction_count: u64,
    pub cells_touched: usize,
    pub elapsed_ms: f64,
    pub output_bytes: usize,
}

pub fn bench_file(path: &Path, _tape_size: usize) -> Result<BenchResult> {
    let expanded = Preprocessor::process_file(path)?;
    let mut interp = Interpreter::new_optimized(&expanded)?;

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
    println!("Benchmarking: {}", path.display());
    println!();

    let result = bench_file(path, tape_size)?;

    println!("  Instructions executed: {}", format_number(result.instruction_count));
    println!("  Cells touched:         {}", result.cells_touched);
    println!("  Output bytes:          {}", result.output_bytes);
    println!("  Wall time:             {:.3} ms", result.elapsed_ms);

    if result.elapsed_ms > 0.0 {
        let mips = result.instruction_count as f64 / result.elapsed_ms / 1000.0;
        println!("  Throughput:            {:.1} MIPS", mips);
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
}
