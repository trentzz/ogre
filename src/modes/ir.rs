use anyhow::{bail, Result};

/// A single bytecode operation in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add(u8),
    Sub(u8),
    Right(usize),
    Left(usize),
    Output,
    Input,
    JumpIfZero(usize),
    JumpIfNonZero(usize),
    Clear,
}

/// A compiled brainfuck program represented as a sequence of IR operations.
#[derive(Debug, Clone)]
pub struct Program {
    pub ops: Vec<Op>,
}

impl Program {
    /// Parse brainfuck source into an IR program with run-length collapsing
    /// and bracket pairing. Non-BF characters are ignored.
    pub fn from_source(source: &str) -> Result<Self> {
        let mut ops = Vec::new();
        let mut bracket_stack: Vec<usize> = Vec::new();

        for ch in source.chars() {
            match ch {
                '+' => {
                    if let Some(Op::Add(n)) = ops.last_mut() {
                        *n = n.wrapping_add(1);
                    } else {
                        ops.push(Op::Add(1));
                    }
                }
                '-' => {
                    if let Some(Op::Sub(n)) = ops.last_mut() {
                        *n = n.wrapping_add(1);
                    } else {
                        ops.push(Op::Sub(1));
                    }
                }
                '>' => {
                    if let Some(Op::Right(n)) = ops.last_mut() {
                        *n += 1;
                    } else {
                        ops.push(Op::Right(1));
                    }
                }
                '<' => {
                    if let Some(Op::Left(n)) = ops.last_mut() {
                        *n += 1;
                    } else {
                        ops.push(Op::Left(1));
                    }
                }
                '.' => ops.push(Op::Output),
                ',' => ops.push(Op::Input),
                '[' => {
                    let pos = ops.len();
                    ops.push(Op::JumpIfZero(0)); // placeholder
                    bracket_stack.push(pos);
                }
                ']' => {
                    let open = bracket_stack
                        .pop()
                        .ok_or_else(|| anyhow::anyhow!("unmatched `]`"))?;
                    let close = ops.len();
                    ops.push(Op::JumpIfNonZero(open));
                    // Patch the opening bracket to point past the closing
                    ops[open] = Op::JumpIfZero(close);
                }
                _ => {} // comments ignored
            }
        }

        if let Some(pos) = bracket_stack.pop() {
            bail!("unmatched `[` at op index {}", pos);
        }

        Ok(Program { ops })
    }

    /// Apply optimization passes to the program.
    pub fn optimize(&mut self) {
        self.optimize_clear_idiom();
        self.optimize_cancellation();
        self.optimize_dead_store();
        self.reindex_jumps();
    }

    /// Convert the IR back to a brainfuck source string.
    pub fn to_bf_string(&self) -> String {
        let mut out = String::new();
        for op in &self.ops {
            match op {
                Op::Add(n) => {
                    for _ in 0..*n {
                        out.push('+');
                    }
                }
                Op::Sub(n) => {
                    for _ in 0..*n {
                        out.push('-');
                    }
                }
                Op::Right(n) => {
                    for _ in 0..*n {
                        out.push('>');
                    }
                }
                Op::Left(n) => {
                    for _ in 0..*n {
                        out.push('<');
                    }
                }
                Op::Output => out.push('.'),
                Op::Input => out.push(','),
                Op::JumpIfZero(_) => out.push('['),
                Op::JumpIfNonZero(_) => out.push(']'),
                Op::Clear => out.push_str("[-]"),
            }
        }
        out
    }

    /// Replace `[Sub(1)]` (i.e., `[-]`) with `Clear`.
    fn optimize_clear_idiom(&mut self) {
        let mut i = 0;
        while i + 2 < self.ops.len() {
            if matches!(self.ops[i], Op::JumpIfZero(_))
                && self.ops[i + 1] == Op::Sub(1)
                && matches!(self.ops[i + 2], Op::JumpIfNonZero(_))
            {
                self.ops.splice(i..i + 3, std::iter::once(Op::Clear));
                // Don't advance i — check the new position
            } else {
                i += 1;
            }
        }
    }

    /// Merge/cancel adjacent Add/Sub and Right/Left operations.
    fn optimize_cancellation(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_ops = Vec::with_capacity(self.ops.len());
            let mut i = 0;
            while i < self.ops.len() {
                if i + 1 < self.ops.len() {
                    match (&self.ops[i], &self.ops[i + 1]) {
                        (Op::Add(a), Op::Sub(b)) => {
                            changed = true;
                            let a = *a;
                            let b = *b;
                            if a > b {
                                new_ops.push(Op::Add(a - b));
                            } else if b > a {
                                new_ops.push(Op::Sub(b - a));
                            }
                            // if equal, both cancel out
                            i += 2;
                            continue;
                        }
                        (Op::Sub(a), Op::Add(b)) => {
                            changed = true;
                            let a = *a;
                            let b = *b;
                            if a > b {
                                new_ops.push(Op::Sub(a - b));
                            } else if b > a {
                                new_ops.push(Op::Add(b - a));
                            }
                            i += 2;
                            continue;
                        }
                        (Op::Right(a), Op::Left(b)) => {
                            changed = true;
                            let a = *a;
                            let b = *b;
                            if a > b {
                                new_ops.push(Op::Right(a - b));
                            } else if b > a {
                                new_ops.push(Op::Left(b - a));
                            }
                            i += 2;
                            continue;
                        }
                        (Op::Left(a), Op::Right(b)) => {
                            changed = true;
                            let a = *a;
                            let b = *b;
                            if a > b {
                                new_ops.push(Op::Left(a - b));
                            } else if b > a {
                                new_ops.push(Op::Right(b - a));
                            }
                            i += 2;
                            continue;
                        }
                        // Merge adjacent same-type ops
                        (Op::Add(a), Op::Add(b)) => {
                            changed = true;
                            new_ops.push(Op::Add(a.wrapping_add(*b)));
                            i += 2;
                            continue;
                        }
                        (Op::Sub(a), Op::Sub(b)) => {
                            changed = true;
                            new_ops.push(Op::Sub(a.wrapping_add(*b)));
                            i += 2;
                            continue;
                        }
                        (Op::Right(a), Op::Right(b)) => {
                            changed = true;
                            new_ops.push(Op::Right(a + b));
                            i += 2;
                            continue;
                        }
                        (Op::Left(a), Op::Left(b)) => {
                            changed = true;
                            new_ops.push(Op::Left(a + b));
                            i += 2;
                            continue;
                        }
                        _ => {}
                    }
                }
                new_ops.push(self.ops[i].clone());
                i += 1;
            }
            self.ops = new_ops;
        }
    }

    /// Remove Clear before Add(n) — the Clear is redundant if immediately
    /// followed by an Add that sets the value.
    fn optimize_dead_store(&mut self) {
        let mut i = 0;
        while i + 1 < self.ops.len() {
            if self.ops[i] == Op::Clear && matches!(self.ops[i + 1], Op::Add(_)) {
                self.ops.remove(i);
                // Don't advance — check from same position
            } else {
                i += 1;
            }
        }
    }

    /// Reindex all jump targets after ops have been inserted/removed.
    fn reindex_jumps(&mut self) {
        let mut bracket_stack: Vec<usize> = Vec::new();
        for i in 0..self.ops.len() {
            match self.ops[i] {
                Op::JumpIfZero(_) => {
                    bracket_stack.push(i);
                }
                Op::JumpIfNonZero(_) => {
                    if let Some(open) = bracket_stack.pop() {
                        self.ops[i] = Op::JumpIfNonZero(open);
                        self.ops[open] = Op::JumpIfZero(i);
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let prog = Program::from_source("").unwrap();
        assert!(prog.ops.is_empty());
    }

    #[test]
    fn test_comments_stripped() {
        let prog = Program::from_source("hello world this is not bf").unwrap();
        assert!(prog.ops.is_empty());
    }

    #[test]
    fn test_run_length_collapsing() {
        let prog = Program::from_source("+++").unwrap();
        assert_eq!(prog.ops, vec![Op::Add(3)]);
    }

    #[test]
    fn test_move_collapsing() {
        let prog = Program::from_source(">>>").unwrap();
        assert_eq!(prog.ops, vec![Op::Right(3)]);

        let prog = Program::from_source("<<<").unwrap();
        assert_eq!(prog.ops, vec![Op::Left(3)]);
    }

    #[test]
    fn test_mixed_ops_no_collapse() {
        let prog = Program::from_source("+-").unwrap();
        assert_eq!(prog.ops, vec![Op::Add(1), Op::Sub(1)]);
    }

    #[test]
    fn test_bracket_pairing() {
        let prog = Program::from_source("[+]").unwrap();
        assert_eq!(
            prog.ops,
            vec![Op::JumpIfZero(2), Op::Add(1), Op::JumpIfNonZero(0)]
        );
    }

    #[test]
    fn test_nested_brackets() {
        let prog = Program::from_source("[[+]]").unwrap();
        assert_eq!(
            prog.ops,
            vec![
                Op::JumpIfZero(4),    // outer [ -> outer ]
                Op::JumpIfZero(3),    // inner [ -> inner ]
                Op::Add(1),
                Op::JumpIfNonZero(1), // inner ] -> inner [
                Op::JumpIfNonZero(0), // outer ] -> outer [
            ]
        );
    }

    #[test]
    fn test_unmatched_open() {
        assert!(Program::from_source("[+").is_err());
    }

    #[test]
    fn test_unmatched_close() {
        assert!(Program::from_source("+]").is_err());
    }

    #[test]
    fn test_clear_idiom() {
        let mut prog = Program::from_source("[-]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::Clear]);
    }

    #[test]
    fn test_cancellation_add_sub() {
        let mut prog = Program::from_source("+++--").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::Add(1)]);
    }

    #[test]
    fn test_cancellation_partial() {
        let mut prog = Program::from_source("++---").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::Sub(1)]);
    }

    #[test]
    fn test_cancellation_moves() {
        let mut prog = Program::from_source(">>><<").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::Right(1)]);
    }

    #[test]
    fn test_clear_then_add() {
        let mut prog = Program::from_source("[-]+++").unwrap();
        prog.optimize();
        // Clear + Add(3) -> just Add(3) (dead store optimization)
        assert_eq!(prog.ops, vec![Op::Add(3)]);
    }

    #[test]
    fn test_to_bf_string() {
        let prog = Program::from_source("+++>>.<[-]").unwrap();
        let bf = prog.to_bf_string();
        assert_eq!(bf, "+++>>.<[-]");
    }

    #[test]
    fn test_to_bf_string_roundtrip() {
        let original = "+++++[>++++<-]>.";
        let prog = Program::from_source(original).unwrap();
        let bf = prog.to_bf_string();
        assert_eq!(bf, original);
    }

    #[test]
    fn test_optimize_preserves_semantics() {
        // After optimization, converting back should produce valid BF that does the same thing.
        let mut prog = Program::from_source("+++-->><<<").unwrap();
        prog.optimize();
        // +++ -- → Add(1), >>> << → Left(1) after cancellation
        // Actually: Add(3), Sub(2) → Add(1); Right(2), Left(3) → Left(1)
        // Wait: ">>>" is Right(3), "<<" is Left(2) — they're separate tokens after parsing
        // Actually the source "+++-->><<<" parses as Add(3), Sub(2), Right(2), Left(3)
        assert_eq!(prog.ops, vec![Op::Add(1), Op::Left(1)]);
    }

    #[test]
    fn test_nested_bracket_jump_indices_after_optimize() {
        let mut prog = Program::from_source("[[-]]").unwrap();
        prog.optimize();
        // [-] becomes Clear, so we get [Clear]
        // Which is JumpIfZero(?), Clear, JumpIfNonZero(?)
        assert_eq!(
            prog.ops,
            vec![Op::JumpIfZero(2), Op::Clear, Op::JumpIfNonZero(0)]
        );
    }

    #[test]
    fn test_sub_collapsing() {
        let prog = Program::from_source("---").unwrap();
        assert_eq!(prog.ops, vec![Op::Sub(3)]);
    }

    #[test]
    fn test_io_ops() {
        let prog = Program::from_source(",.").unwrap();
        assert_eq!(prog.ops, vec![Op::Input, Op::Output]);
    }

    #[test]
    fn test_wrapping_add_256() {
        // 256 +'s should wrap to Add(0) with u8
        let src: String = std::iter::repeat('+').take(256).collect();
        let prog = Program::from_source(&src).unwrap();
        assert_eq!(prog.ops, vec![Op::Add(0)]);
    }
}
