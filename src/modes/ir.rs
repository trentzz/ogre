use anyhow::Result;

use crate::error::OgreError;

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
    /// Move current cell's value to cell at `data_ptr + offset`, zeroing current cell.
    /// Recognized from patterns like `[->+<]` (MoveAdd(1)) or `[->>+<<]` (MoveAdd(2)).
    MoveAdd(isize),
    /// Move current cell's value by subtracting it from cell at `data_ptr + offset`.
    /// Recognized from patterns like `[->-<]` (MoveSub(1)).
    MoveSub(isize),
    /// Set current cell to a specific value (replaces Clear + Add sequences).
    Set(u8),
    /// Scan right until a zero cell is found: `[>]`.
    ScanRight,
    /// Scan left until a zero cell is found: `[<]`.
    ScanLeft,
    /// Multiply-move: multiply current cell by factor and add to cell at offset.
    /// `[->+++<]` becomes MultiplyMove(1, 3). Zeroes current cell.
    /// Vec contains (offset, factor) pairs for multi-target multiply moves.
    MultiplyMove(Vec<(isize, u8)>),
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
                        .ok_or(OgreError::UnmatchedCloseBracket)?;
                    let close = ops.len();
                    ops.push(Op::JumpIfNonZero(open));
                    // Patch the opening bracket to point past the closing
                    ops[open] = Op::JumpIfZero(close);
                }
                _ => {} // comments ignored
            }
        }

        if let Some(pos) = bracket_stack.pop() {
            return Err(OgreError::UnmatchedOpenBracket(pos).into());
        }

        Ok(Program { ops })
    }

    /// Apply optimization passes to the program.
    pub fn optimize(&mut self) {
        self.optimize_clear_idiom();
        self.optimize_move_idiom();
        self.optimize_scan_idiom();
        self.optimize_multiply_move();
        self.optimize_cancellation();
        self.optimize_set_idiom();
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
                Op::MoveAdd(offset) => {
                    // [->+<] for positive, [-<+>] for negative
                    out.push_str("[-");
                    if *offset > 0 {
                        for _ in 0..*offset {
                            out.push('>');
                        }
                        out.push('+');
                        for _ in 0..*offset {
                            out.push('<');
                        }
                    } else {
                        for _ in 0..offset.unsigned_abs() {
                            out.push('<');
                        }
                        out.push('+');
                        for _ in 0..offset.unsigned_abs() {
                            out.push('>');
                        }
                    }
                    out.push(']');
                }
                Op::MoveSub(offset) => {
                    out.push_str("[-");
                    if *offset > 0 {
                        for _ in 0..*offset {
                            out.push('>');
                        }
                        out.push('-');
                        for _ in 0..*offset {
                            out.push('<');
                        }
                    } else {
                        for _ in 0..offset.unsigned_abs() {
                            out.push('<');
                        }
                        out.push('-');
                        for _ in 0..offset.unsigned_abs() {
                            out.push('>');
                        }
                    }
                    out.push(']');
                }
                Op::Set(n) => {
                    out.push_str("[-]");
                    for _ in 0..*n {
                        out.push('+');
                    }
                }
                Op::ScanRight => {
                    out.push_str("[>]");
                }
                Op::ScanLeft => {
                    out.push_str("[<]");
                }
                Op::MultiplyMove(targets) => {
                    out.push_str("[-");
                    for (offset, factor) in targets {
                        if *offset > 0 {
                            for _ in 0..*offset {
                                out.push('>');
                            }
                        } else {
                            for _ in 0..offset.unsigned_abs() {
                                out.push('<');
                            }
                        }
                        for _ in 0..*factor {
                            out.push('+');
                        }
                        if *offset > 0 {
                            for _ in 0..*offset {
                                out.push('<');
                            }
                        } else {
                            for _ in 0..offset.unsigned_abs() {
                                out.push('>');
                            }
                        }
                    }
                    out.push(']');
                }
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

    /// Detect `[->+<]` and `[-<+>]` move patterns and replace with `MoveAdd(offset)`.
    /// Also detects `[->-<]` for `MoveSub(offset)`.
    /// Pattern: JumpIfZero, Sub(1), Right(n)/Left(n), Add(1)/Sub(1), Left(n)/Right(n), JumpIfNonZero
    fn optimize_move_idiom(&mut self) {
        let mut i = 0;
        while i + 5 < self.ops.len() {
            if matches!(self.ops[i], Op::JumpIfZero(_))
                && self.ops[i + 1] == Op::Sub(1)
                && matches!(self.ops[i + 5], Op::JumpIfNonZero(_))
            {
                // Check for [- >n + <n] pattern (MoveAdd forward)
                if let Op::Right(n) = self.ops[i + 2] {
                    if self.ops[i + 3] == Op::Add(1) {
                        if let Op::Left(m) = self.ops[i + 4] {
                            if n == m {
                                self.ops
                                    .splice(i..i + 6, std::iter::once(Op::MoveAdd(n as isize)));
                                continue;
                            }
                        }
                    }
                    // Check for [- >n - <n] pattern (MoveSub forward)
                    if self.ops[i + 3] == Op::Sub(1) {
                        if let Op::Left(m) = self.ops[i + 4] {
                            if n == m {
                                self.ops
                                    .splice(i..i + 6, std::iter::once(Op::MoveSub(n as isize)));
                                continue;
                            }
                        }
                    }
                }
                // Check for [- <n + >n] pattern (MoveAdd backward)
                if let Op::Left(n) = self.ops[i + 2] {
                    if self.ops[i + 3] == Op::Add(1) {
                        if let Op::Right(m) = self.ops[i + 4] {
                            if n == m {
                                self.ops
                                    .splice(i..i + 6, std::iter::once(Op::MoveAdd(-(n as isize))));
                                continue;
                            }
                        }
                    }
                    // Check for [- <n - >n] pattern (MoveSub backward)
                    if self.ops[i + 3] == Op::Sub(1) {
                        if let Op::Right(m) = self.ops[i + 4] {
                            if n == m {
                                self.ops
                                    .splice(i..i + 6, std::iter::once(Op::MoveSub(-(n as isize))));
                                continue;
                            }
                        }
                    }
                }
            }
            i += 1;
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

    /// Replace Clear + Add(n) with Set(n), and standalone Clear with Set(0).
    fn optimize_set_idiom(&mut self) {
        let mut i = 0;
        while i < self.ops.len() {
            if self.ops[i] == Op::Clear {
                if i + 1 < self.ops.len() {
                    if let Op::Add(n) = self.ops[i + 1] {
                        self.ops.splice(i..i + 2, std::iter::once(Op::Set(n)));
                        continue;
                    }
                }
                self.ops[i] = Op::Set(0);
            }
            i += 1;
        }
    }

    /// Detect `[>]` and `[<]` scan patterns.
    fn optimize_scan_idiom(&mut self) {
        let mut i = 0;
        while i + 2 < self.ops.len() {
            if matches!(self.ops[i], Op::JumpIfZero(_))
                && matches!(self.ops[i + 2], Op::JumpIfNonZero(_))
            {
                if self.ops[i + 1] == Op::Right(1) {
                    self.ops.splice(i..i + 3, std::iter::once(Op::ScanRight));
                    continue;
                }
                if self.ops[i + 1] == Op::Left(1) {
                    self.ops.splice(i..i + 3, std::iter::once(Op::ScanLeft));
                    continue;
                }
            }
            i += 1;
        }
    }

    /// Detect multiplication loops like `[->+++>++<<]` and replace with MultiplyMove.
    /// Pattern: JumpIfZero, Sub(1), then pairs of (Right/Left(n), Add(m), Left/Right(n)),
    /// ending with JumpIfNonZero. The net pointer movement must be zero.
    fn optimize_multiply_move(&mut self) {
        let mut i = 0;
        while i < self.ops.len() {
            if !matches!(self.ops[i], Op::JumpIfZero(_)) {
                i += 1;
                continue;
            }
            // Find matching JumpIfNonZero
            let close = match self.ops[i] {
                Op::JumpIfZero(t) => t,
                _ => {
                    i += 1;
                    continue;
                }
            };
            if close >= self.ops.len() || !matches!(self.ops[close], Op::JumpIfNonZero(_)) {
                i += 1;
                continue;
            }
            // Must start with Sub(1)
            if i + 1 >= close || self.ops[i + 1] != Op::Sub(1) {
                i += 1;
                continue;
            }
            // Already handled by optimize_move_idiom for simple cases,
            // only handle multi-target patterns here (more than one target)
            let body = &self.ops[i + 2..close];
            if let Some(targets) = Self::parse_multiply_body(body) {
                // Skip single-target with factor=1, already handled by move_idiom as MoveAdd
                let dominated_by_move = targets.len() == 1 && targets[0].1 == 1;
                if !dominated_by_move {
                    self.ops
                        .splice(i..close + 1, std::iter::once(Op::MultiplyMove(targets)));
                    continue;
                }
            }
            i += 1;
        }
    }

    /// Parse the body of a potential multiply loop.
    /// Returns Some(vec of (offset, factor)) if the body is a valid multiply pattern.
    /// Handles patterns like: Right(1) Add(2) Right(1) Add(3) Left(2)
    fn parse_multiply_body(body: &[Op]) -> Option<Vec<(isize, u8)>> {
        let mut targets = Vec::new();
        let mut current_offset: isize = 0;
        let mut j = 0;

        while j < body.len() {
            match &body[j] {
                Op::Right(n) => {
                    current_offset += *n as isize;
                    j += 1;
                }
                Op::Left(n) => {
                    current_offset -= *n as isize;
                    j += 1;
                }
                Op::Add(n) => {
                    if current_offset == 0 {
                        return None; // Adding to the loop counter cell is invalid
                    }
                    targets.push((current_offset, *n));
                    j += 1;
                }
                _ => return None, // Any other op makes this not a multiply loop
            }
        }

        // Net movement must return to origin
        if current_offset != 0 {
            return None;
        }

        if targets.is_empty() {
            return None;
        }

        Some(targets)
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
                Op::JumpIfZero(4), // outer [ -> outer ]
                Op::JumpIfZero(3), // inner [ -> inner ]
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
        assert_eq!(prog.ops, vec![Op::Set(0)]);
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
        // Clear + Add(3) -> Set(3)
        assert_eq!(prog.ops, vec![Op::Set(3)]);
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
        // [-] becomes Set(0), so we get [Set(0)]
        assert_eq!(
            prog.ops,
            vec![Op::JumpIfZero(2), Op::Set(0), Op::JumpIfNonZero(0)]
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

    #[test]
    fn test_move_add_forward() {
        // [->+<] should optimize to MoveAdd(1)
        let mut prog = Program::from_source("[->+<]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::MoveAdd(1)]);
    }

    #[test]
    fn test_move_add_forward_2() {
        // [->>+<<] should optimize to MoveAdd(2)
        let mut prog = Program::from_source("[->>+<<]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::MoveAdd(2)]);
    }

    #[test]
    fn test_move_add_backward() {
        // [-<+>] should optimize to MoveAdd(-1)
        let mut prog = Program::from_source("[-<+>]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::MoveAdd(-1)]);
    }

    #[test]
    fn test_move_sub_forward() {
        // [->-<] should optimize to MoveSub(1)
        let mut prog = Program::from_source("[->-<]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::MoveSub(1)]);
    }

    #[test]
    fn test_move_add_to_bf_string() {
        let prog = Program {
            ops: vec![Op::MoveAdd(1)],
        };
        assert_eq!(prog.to_bf_string(), "[->+<]");
    }

    #[test]
    fn test_move_add_backward_to_bf_string() {
        let prog = Program {
            ops: vec![Op::MoveAdd(-2)],
        };
        assert_eq!(prog.to_bf_string(), "[-<<+>>]");
    }

    #[test]
    fn test_move_preserves_semantics() {
        // [->+<] with cell 0 = 5 should: cell 1 += 5, cell 0 = 0
        let src = "+++++[->+<]";
        let mut prog_opt = Program::from_source(src).unwrap();
        prog_opt.optimize();
        assert!(prog_opt.ops.contains(&Op::MoveAdd(1)));
    }

    // ---- Set optimization tests ----

    #[test]
    fn test_set_idiom() {
        let mut prog = Program::from_source("[-]+++++").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::Set(5)]);
    }

    #[test]
    fn test_set_zero() {
        let mut prog = Program::from_source("[-]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::Set(0)]);
    }

    // ---- Scan optimization tests ----

    #[test]
    fn test_scan_right() {
        let mut prog = Program::from_source("[>]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::ScanRight]);
    }

    #[test]
    fn test_scan_left() {
        let mut prog = Program::from_source("[<]").unwrap();
        prog.optimize();
        assert_eq!(prog.ops, vec![Op::ScanLeft]);
    }

    #[test]
    fn test_scan_right_to_bf() {
        let prog = Program {
            ops: vec![Op::ScanRight],
        };
        assert_eq!(prog.to_bf_string(), "[>]");
    }

    #[test]
    fn test_scan_left_to_bf() {
        let prog = Program {
            ops: vec![Op::ScanLeft],
        };
        assert_eq!(prog.to_bf_string(), "[<]");
    }

    // ---- MultiplyMove tests ----

    #[test]
    fn test_multiply_move_double() {
        // [->++>+++<<] should be detected as MultiplyMove
        let mut prog = Program::from_source("[->++>+++<<]").unwrap();
        prog.optimize();
        assert!(prog.ops.iter().any(|op| matches!(op, Op::MultiplyMove(_))));
    }

    #[test]
    fn test_set_to_bf() {
        let prog = Program {
            ops: vec![Op::Set(3)],
        };
        assert_eq!(prog.to_bf_string(), "[-]+++");
    }

    #[test]
    fn test_set_zero_to_bf() {
        let prog = Program {
            ops: vec![Op::Set(0)],
        };
        assert_eq!(prog.to_bf_string(), "[-]");
    }
}
