use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

use super::ir::{Op, Program};
use super::preprocess::Preprocessor;

/// Profiling results for a brainfuck program execution.
#[derive(Debug)]
pub struct ProfileReport {
    pub total_instructions: u64,
    pub cells_accessed: usize,
    pub instruction_mix: InstructionMix,
    pub hot_cells: Vec<(usize, u64)>,
    pub loop_stats: Vec<LoopStat>,
    pub max_cell_used: usize,
}

#[derive(Debug, Default)]
pub struct InstructionMix {
    pub add_sub: u64,
    pub moves: u64,
    pub io: u64,
    pub loops: u64,
    pub other: u64,
}

#[derive(Debug)]
pub struct LoopStat {
    pub op_index: usize,
    pub total_iterations: u64,
    pub entries: u64,
}

/// Profile a brainfuck program by executing it with instrumentation.
pub fn profile_source(code: &str, tape_size: usize) -> Result<ProfileReport> {
    let mut program = Program::from_source(code)?;
    program.optimize();

    let mut tape = vec![0u8; tape_size];
    let mut dp: usize = 0;
    let mut ip: usize = 0;
    let mut total_instructions: u64 = 0;
    let max_instructions: u64 = 100_000_000;

    // Cell access counts
    let mut cell_access: HashMap<usize, u64> = HashMap::new();
    let mut max_cell: usize = 0;

    // Instruction mix
    let mut mix = InstructionMix::default();

    // Loop tracking: op_index -> (total_iterations, entries)
    let mut loop_data: HashMap<usize, (u64, u64)> = HashMap::new();
    // Stack of loop start indices for entry tracking
    let mut loop_entry_stack: Vec<usize> = Vec::new();

    while ip < program.ops.len() && total_instructions < max_instructions {
        total_instructions += 1;

        // Track cell access
        *cell_access.entry(dp).or_insert(0) += 1;
        if dp > max_cell {
            max_cell = dp;
        }

        match &program.ops[ip] {
            Op::Add(n) => {
                tape[dp] = tape[dp].wrapping_add(*n);
                mix.add_sub += 1;
            }
            Op::Sub(n) => {
                tape[dp] = tape[dp].wrapping_sub(*n);
                mix.add_sub += 1;
            }
            Op::Right(n) => {
                dp += n;
                if dp >= tape_size {
                    dp = tape_size - 1;
                }
                mix.moves += 1;
            }
            Op::Left(n) => {
                dp = dp.saturating_sub(*n);
                mix.moves += 1;
            }
            Op::Output => {
                mix.io += 1;
            }
            Op::Input => {
                // No input during profiling
                mix.io += 1;
            }
            Op::JumpIfZero(target) => {
                mix.loops += 1;
                if tape[dp] == 0 {
                    ip = *target;
                } else {
                    loop_entry_stack.push(ip);
                    let entry = loop_data.entry(ip).or_insert((0, 0));
                    entry.1 += 1; // new entry
                }
            }
            Op::JumpIfNonZero(target) => {
                mix.loops += 1;
                if tape[dp] != 0 {
                    // Iteration completed
                    if let Some(&loop_start) = loop_entry_stack.last() {
                        if loop_start == *target {
                            let entry = loop_data.entry(loop_start).or_insert((0, 0));
                            entry.0 += 1; // iteration
                        }
                    }
                    ip = *target;
                } else {
                    // Loop exiting
                    if let Some(&loop_start) = loop_entry_stack.last() {
                        if loop_start == *target {
                            let entry = loop_data.entry(loop_start).or_insert((0, 0));
                            entry.0 += 1; // final iteration
                            loop_entry_stack.pop();
                        }
                    }
                }
            }
            Op::Clear => {
                tape[dp] = 0;
                mix.other += 1;
            }
            Op::Set(n) => {
                tape[dp] = *n;
                mix.other += 1;
            }
            Op::MoveAdd(offset) => {
                let target = (dp as isize + offset) as usize;
                if target < tape_size {
                    tape[target] = tape[target].wrapping_add(tape[dp]);
                }
                tape[dp] = 0;
                mix.other += 1;
            }
            Op::MoveSub(offset) => {
                let target = (dp as isize + offset) as usize;
                if target < tape_size {
                    tape[target] = tape[target].wrapping_sub(tape[dp]);
                }
                tape[dp] = 0;
                mix.other += 1;
            }
            Op::ScanRight => {
                while dp < tape_size && tape[dp] != 0 {
                    dp += 1;
                }
                mix.other += 1;
            }
            Op::ScanLeft => {
                while dp > 0 && tape[dp] != 0 {
                    dp -= 1;
                }
                mix.other += 1;
            }
            Op::MultiplyMove(targets) => {
                let src = tape[dp];
                for (offset, factor) in targets {
                    let target = (dp as isize + offset) as usize;
                    if target < tape_size {
                        tape[target] = tape[target].wrapping_add(src.wrapping_mul(*factor));
                    }
                }
                tape[dp] = 0;
                mix.other += 1;
            }
        }

        ip += 1;
    }

    // Build hot cells list (top 10)
    let mut hot_cells: Vec<(usize, u64)> = cell_access.into_iter().collect();
    hot_cells.sort_by(|a, b| b.1.cmp(&a.1));
    hot_cells.truncate(10);

    // Build loop stats (top 10 by iterations)
    let mut loop_stats: Vec<LoopStat> = loop_data
        .into_iter()
        .map(|(op_index, (total_iterations, entries))| LoopStat {
            op_index,
            total_iterations,
            entries,
        })
        .collect();
    loop_stats.sort_by(|a, b| b.total_iterations.cmp(&a.total_iterations));
    loop_stats.truncate(10);

    let cells_accessed = hot_cells
        .len()
        .max(if max_cell > 0 { max_cell + 1 } else { 0 });

    Ok(ProfileReport {
        total_instructions,
        cells_accessed,
        instruction_mix: mix,
        hot_cells,
        loop_stats,
        max_cell_used: max_cell,
    })
}

/// Format a profile report for display.
pub fn format_report(report: &ProfileReport) -> String {
    let mut out = String::new();

    out.push_str(&format!("{}\n", "=== Execution Profile ===".bold()));
    out.push_str(&format!(
        "Total instructions: {}\n",
        format_number(report.total_instructions)
    ));
    out.push_str(&format!(
        "Unique cells accessed: {}\n",
        report.cells_accessed
    ));
    out.push_str(&format!(
        "Max cell index used: {}\n\n",
        report.max_cell_used
    ));

    // Instruction mix
    let total = report.total_instructions.max(1) as f64;
    let mix = &report.instruction_mix;
    out.push_str(&format!("{}\n", "Instruction Mix:".bold()));
    out.push_str(&format!(
        "  Add/Sub:    {:5.1}%  ({})\n",
        mix.add_sub as f64 / total * 100.0,
        format_number(mix.add_sub)
    ));
    out.push_str(&format!(
        "  Move:       {:5.1}%  ({})\n",
        mix.moves as f64 / total * 100.0,
        format_number(mix.moves)
    ));
    out.push_str(&format!(
        "  I/O:        {:5.1}%  ({})\n",
        mix.io as f64 / total * 100.0,
        format_number(mix.io)
    ));
    out.push_str(&format!(
        "  Loops:      {:5.1}%  ({})\n",
        mix.loops as f64 / total * 100.0,
        format_number(mix.loops)
    ));
    out.push_str(&format!(
        "  Optimized:  {:5.1}%  ({})\n\n",
        mix.other as f64 / total * 100.0,
        format_number(mix.other)
    ));

    // Hot cells
    if !report.hot_cells.is_empty() {
        out.push_str(&format!("{}\n", "Hot Cells (top 10):".bold()));
        for (cell, count) in &report.hot_cells {
            let pct = *count as f64 / total * 100.0;
            let bar_len = (pct * 0.4) as usize;
            let bar = "█".repeat(bar_len);
            out.push_str(&format!(
                "  Cell {:4}: {:>10} accesses ({:5.1}%) {}\n",
                cell,
                format_number(*count),
                pct,
                bar.green()
            ));
        }
        out.push('\n');
    }

    // Loop stats
    if !report.loop_stats.is_empty() {
        out.push_str(&format!("{}\n", "Loop Analysis (top 10):".bold()));
        for stat in &report.loop_stats {
            let avg_iter = if stat.entries > 0 {
                stat.total_iterations as f64 / stat.entries as f64
            } else {
                0.0
            };
            out.push_str(&format!(
                "  Loop at op {:4}: {:>10} iterations ({} entries, avg {:.1})\n",
                stat.op_index,
                format_number(stat.total_iterations),
                format_number(stat.entries),
                avg_iter
            ));
        }
        out.push('\n');
    }

    // Memory heatmap
    let max_cell_display = report.max_cell_used.min(49);
    if max_cell_display > 0 {
        out.push_str(&format!(
            "{}\n  ",
            format!("Memory Heatmap (first {} cells):", max_cell_display + 1).bold()
        ));
        let max_access = report
            .hot_cells
            .first()
            .map(|(_, c)| *c)
            .unwrap_or(1)
            .max(1);
        // Build full access map
        let mut access_map: HashMap<usize, u64> = HashMap::new();
        for (cell, count) in &report.hot_cells {
            access_map.insert(*cell, *count);
        }
        for i in 0..=max_cell_display {
            let count = access_map.get(&i).copied().unwrap_or(0);
            let intensity = (count as f64 / max_access as f64 * 7.0) as usize;
            let ch = match intensity {
                0 => '░',
                1 => '▒',
                2 => '▓',
                3..=7 => '█',
                _ => '█',
            };
            out.push(ch);
        }
        out.push('\n');
    }

    out
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!(
            "{},{:03},{:03}",
            n / 1_000_000,
            (n / 1_000) % 1_000,
            n % 1_000
        )
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1_000, n % 1_000)
    } else {
        n.to_string()
    }
}

/// Profile a brainfuck file and print the report.
pub fn profile_file(path: &Path, tape_size: usize) -> Result<()> {
    let expanded = Preprocessor::process_file(path)?;
    let report = profile_source(&expanded, tape_size)?;
    print!("{}", format_report(&report));
    Ok(())
}

/// Profile with pre-loaded dependency functions.
pub fn profile_file_with_deps(
    path: &Path,
    tape_size: usize,
    dep_functions: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let expanded = Preprocessor::process_file_with_deps(path, dep_functions)?;
    let report = profile_source(&expanded, tape_size)?;
    print!("{}", format_report(&report));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_empty() {
        let report = profile_source("", 30000).unwrap();
        assert_eq!(report.total_instructions, 0);
    }

    #[test]
    fn test_profile_simple_add() {
        let report = profile_source("+++", 30000).unwrap();
        // After optimization: Add(3), so 1 instruction
        assert_eq!(report.total_instructions, 1);
        assert_eq!(report.instruction_mix.add_sub, 1);
    }

    #[test]
    fn test_profile_moves() {
        let report = profile_source(">>><<<", 30000).unwrap();
        // After optimization: Right(3), Left(3) → cancels to nothing
        // Just verify it ran without error
        let _ = report.instruction_mix.moves;
    }

    #[test]
    fn test_profile_loop() {
        let report = profile_source("+++[-]", 30000).unwrap();
        // Add(3), then Clear (optimized from [-])
        assert!(report.total_instructions >= 1);
    }

    #[test]
    fn test_profile_hot_cells() {
        let report = profile_source("+++>++>+", 30000).unwrap();
        assert!(!report.hot_cells.is_empty());
    }

    #[test]
    fn test_profile_cells_tracked() {
        let report = profile_source(">>>>>+", 30000).unwrap();
        assert!(report.max_cell_used >= 5);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_report_produces_output() {
        let report = profile_source("+++[-].", 30000).unwrap();
        let formatted = format_report(&report);
        assert!(formatted.contains("Execution Profile"));
        assert!(formatted.contains("Instruction Mix"));
    }
}
