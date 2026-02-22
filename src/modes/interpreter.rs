use anyhow::Result;
use std::io::Write;

use super::ir::{Op, Program};
use super::source_map::{build_op_to_char_map, SourceMap};
use crate::error::OgreError;

pub const DEFAULT_TAPE_SIZE: usize = 30_000;

pub struct Interpreter {
    tape: Vec<u8>,
    data_ptr: usize,
    program: Program,
    ip: usize,
    output: Vec<u8>,
    input: Vec<u8>,
    input_ptr: usize,
    /// When true, `,` reads from real stdin after the input buffer is exhausted.
    live_stdin: bool,
    /// When true, `.` flushes output to stdout immediately.
    streaming: bool,
    /// Total instructions executed (for bench mode).
    pub instruction_count: u64,
    /// Track which cells have been touched.
    cells_touched: Vec<bool>,
    /// Original source chars for display purposes (debugger/REPL).
    source_chars: Vec<char>,
    /// Optional source map for enhanced error messages.
    source_map: Option<SourceMap>,
    /// Maps op indices to character positions in expanded source.
    op_to_char: Vec<usize>,
}

impl Interpreter {
    pub fn new(source: &str) -> Result<Self> {
        Self::with_input(source, "")
    }

    pub fn with_tape_size(source: &str, tape_size: usize) -> Result<Self> {
        Self::with_input_and_tape_size(source, "", tape_size)
    }

    pub fn with_live_stdin(source: &str) -> Result<Self> {
        let program = Program::from_source(source)?;
        let tape_size = DEFAULT_TAPE_SIZE;
        let op_to_char = build_op_to_char_map(source);
        Ok(Self {
            tape: vec![0u8; tape_size],
            data_ptr: 0,
            program,
            ip: 0,
            output: Vec::new(),
            input: Vec::new(),
            input_ptr: 0,
            live_stdin: true,
            streaming: false,
            instruction_count: 0,
            cells_touched: vec![false; tape_size],
            source_chars: source.chars().collect(),
            source_map: None,
            op_to_char,
        })
    }

    pub fn with_live_stdin_and_tape_size(source: &str, tape_size: usize) -> Result<Self> {
        let program = Program::from_source(source)?;
        let op_to_char = build_op_to_char_map(source);
        Ok(Self {
            tape: vec![0u8; tape_size],
            data_ptr: 0,
            program,
            ip: 0,
            output: Vec::new(),
            input: Vec::new(),
            input_ptr: 0,
            live_stdin: true,
            streaming: false,
            instruction_count: 0,
            cells_touched: vec![false; tape_size],
            source_chars: source.chars().collect(),
            source_map: None,
            op_to_char,
        })
    }

    pub fn with_input(source: &str, input: &str) -> Result<Self> {
        Self::with_input_and_tape_size(source, input, DEFAULT_TAPE_SIZE)
    }

    pub fn with_input_and_tape_size(source: &str, input: &str, tape_size: usize) -> Result<Self> {
        let program = Program::from_source(source)?;
        let op_to_char = build_op_to_char_map(source);
        Ok(Self {
            tape: vec![0u8; tape_size],
            data_ptr: 0,
            program,
            ip: 0,
            output: Vec::new(),
            input: input.bytes().collect(),
            input_ptr: 0,
            live_stdin: false,
            streaming: false,
            instruction_count: 0,
            cells_touched: vec![false; tape_size],
            source_chars: source.chars().collect(),
            source_map: None,
            op_to_char,
        })
    }

    /// Create an interpreter with an optimized program.
    pub fn new_optimized(source: &str) -> Result<Self> {
        let mut program = Program::from_source(source)?;
        program.optimize();
        Ok(Self {
            tape: vec![0u8; DEFAULT_TAPE_SIZE],
            data_ptr: 0,
            program,
            ip: 0,
            output: Vec::new(),
            input: Vec::new(),
            input_ptr: 0,
            live_stdin: false,
            streaming: false,
            instruction_count: 0,
            cells_touched: vec![false; DEFAULT_TAPE_SIZE],
            source_chars: source.chars().collect(),
            source_map: None,
            op_to_char: vec![],
        })
    }

    pub fn new_optimized_with_input(source: &str, input: &str) -> Result<Self> {
        let mut program = Program::from_source(source)?;
        program.optimize();
        Ok(Self {
            tape: vec![0u8; DEFAULT_TAPE_SIZE],
            data_ptr: 0,
            program,
            ip: 0,
            output: Vec::new(),
            input: input.bytes().collect(),
            input_ptr: 0,
            live_stdin: false,
            streaming: false,
            instruction_count: 0,
            cells_touched: vec![false; DEFAULT_TAPE_SIZE],
            source_chars: source.chars().collect(),
            source_map: None,
            op_to_char: vec![],
        })
    }

    pub fn set_streaming(&mut self, streaming: bool) {
        self.streaming = streaming;
    }

    /// Attach a source map for enhanced error messages.
    pub fn set_source_map(&mut self, map: SourceMap) {
        self.source_map = Some(map);
    }

    /// Get the source location for the current instruction pointer.
    fn current_source_location(&self) -> Option<String> {
        self.source_map
            .as_ref()
            .and_then(|sm| sm.lookup_op(self.ip, &self.op_to_char))
            .map(|loc| loc.display_short())
    }

    // ---- Accessors ----

    pub fn tape_value(&self, addr: usize) -> u8 {
        self.tape[addr]
    }

    pub fn data_pointer(&self) -> usize {
        self.data_ptr
    }

    pub fn code_pointer(&self) -> usize {
        self.ip
    }

    pub fn set_code_pointer(&mut self, val: usize) {
        self.ip = val;
    }

    /// Number of ops in the program.
    pub fn code_len(&self) -> usize {
        self.program.ops.len()
    }

    /// Get a display character for the op at the given index.
    /// Used by debugger for showing instruction context.
    pub fn code_char(&self, idx: usize) -> char {
        match &self.program.ops[idx] {
            Op::Add(_) => '+',
            Op::Sub(_) => '-',
            Op::Right(_) => '>',
            Op::Left(_) => '<',
            Op::Output => '.',
            Op::Input => ',',
            Op::JumpIfZero(_) => '[',
            Op::JumpIfNonZero(_) => ']',
            Op::Clear => '0',
            Op::MoveAdd(_) => 'M',
            Op::MoveSub(_) => 'm',
        }
    }

    /// Get a descriptive string for the op at the given index.
    pub fn op_description(&self, idx: usize) -> String {
        if idx >= self.program.ops.len() {
            return "END".to_string();
        }
        match &self.program.ops[idx] {
            Op::Add(n) => format!("Add({})", n),
            Op::Sub(n) => format!("Sub({})", n),
            Op::Right(n) => format!("Right({})", n),
            Op::Left(n) => format!("Left({})", n),
            Op::Output => "Output".to_string(),
            Op::Input => "Input".to_string(),
            Op::JumpIfZero(t) => format!("JumpIfZero({})", t),
            Op::JumpIfNonZero(t) => format!("JumpIfNonZero({})", t),
            Op::Clear => "Clear".to_string(),
            Op::MoveAdd(o) => format!("MoveAdd({})", o),
            Op::MoveSub(o) => format!("MoveSub({})", o),
        }
    }

    /// Get the underlying program.
    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn output(&self) -> &[u8] {
        &self.output
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    pub fn tape(&self) -> &[u8] {
        &self.tape
    }

    pub fn is_done(&self) -> bool {
        self.ip >= self.program.ops.len()
    }

    /// Count of unique cells that have been written to.
    pub fn cells_touched_count(&self) -> usize {
        self.cells_touched.iter().filter(|&&b| b).count()
    }

    /// Execute one IR instruction.
    /// Returns `Ok(true)` if there are more instructions, `Ok(false)` if done.
    pub fn step(&mut self) -> Result<bool> {
        if self.is_done() {
            return Ok(false);
        }

        self.instruction_count += 1;

        match &self.program.ops[self.ip] {
            Op::Add(n) => {
                let n = *n;
                self.tape[self.data_ptr] = self.tape[self.data_ptr].wrapping_add(n);
                self.cells_touched[self.data_ptr] = true;
            }
            Op::Sub(n) => {
                let n = *n;
                self.tape[self.data_ptr] = self.tape[self.data_ptr].wrapping_sub(n);
                self.cells_touched[self.data_ptr] = true;
            }
            Op::Right(n) => {
                let n = *n;
                if self.data_ptr + n >= self.tape.len() {
                    let msg = match self.current_source_location() {
                        Some(loc) => format!("right at {}", loc),
                        None => "right".to_string(),
                    };
                    return Err(OgreError::TapeOverflow(msg).into());
                }
                self.data_ptr += n;
            }
            Op::Left(n) => {
                let n = *n;
                if self.data_ptr < n {
                    let msg = match self.current_source_location() {
                        Some(loc) => format!("left at {}", loc),
                        None => "left".to_string(),
                    };
                    return Err(OgreError::TapeOverflow(msg).into());
                }
                self.data_ptr -= n;
            }
            Op::Output => {
                let byte = self.tape[self.data_ptr];
                if self.streaming {
                    let stdout = std::io::stdout();
                    let mut handle = stdout.lock();
                    let _ = handle.write_all(&[byte]);
                    let _ = handle.flush();
                } else {
                    self.output.push(byte);
                }
            }
            Op::Input => {
                if self.input_ptr < self.input.len() {
                    self.tape[self.data_ptr] = self.input[self.input_ptr];
                    self.input_ptr += 1;
                } else if self.live_stdin {
                    use std::io::Read;
                    let mut byte = [0u8; 1];
                    match std::io::stdin().read(&mut byte) {
                        Ok(1) => self.tape[self.data_ptr] = byte[0],
                        _ => self.tape[self.data_ptr] = 0,
                    }
                } else {
                    self.tape[self.data_ptr] = 0;
                }
                self.cells_touched[self.data_ptr] = true;
            }
            Op::JumpIfZero(target) => {
                let target = *target;
                if self.tape[self.data_ptr] == 0 {
                    self.ip = target + 1;
                    return Ok(!self.is_done());
                }
            }
            Op::JumpIfNonZero(target) => {
                let target = *target;
                if self.tape[self.data_ptr] != 0 {
                    self.ip = target + 1;
                    return Ok(!self.is_done());
                }
            }
            Op::Clear => {
                self.tape[self.data_ptr] = 0;
                self.cells_touched[self.data_ptr] = true;
            }
            Op::MoveAdd(offset) => {
                let offset = *offset;
                let target = (self.data_ptr as isize + offset) as usize;
                if target >= self.tape.len() {
                    let msg = match self.current_source_location() {
                        Some(loc) => format!("move target out of bounds at {}", loc),
                        None => "move target out of bounds".to_string(),
                    };
                    return Err(OgreError::TapeOverflow(msg).into());
                }
                self.tape[target] = self.tape[target].wrapping_add(self.tape[self.data_ptr]);
                self.tape[self.data_ptr] = 0;
                self.cells_touched[self.data_ptr] = true;
                self.cells_touched[target] = true;
            }
            Op::MoveSub(offset) => {
                let offset = *offset;
                let target = (self.data_ptr as isize + offset) as usize;
                if target >= self.tape.len() {
                    let msg = match self.current_source_location() {
                        Some(loc) => format!("move target out of bounds at {}", loc),
                        None => "move target out of bounds".to_string(),
                    };
                    return Err(OgreError::TapeOverflow(msg).into());
                }
                self.tape[target] = self.tape[target].wrapping_sub(self.tape[self.data_ptr]);
                self.tape[self.data_ptr] = 0;
                self.cells_touched[self.data_ptr] = true;
                self.cells_touched[target] = true;
            }
        }

        self.ip += 1;
        Ok(!self.is_done())
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.is_done() {
            self.step()?;
        }
        Ok(())
    }

    /// Run with an instruction limit. Returns Ok(true) if completed,
    /// Ok(false) if the limit was reached.
    pub fn run_with_limit(&mut self, max_instructions: u64) -> Result<bool> {
        while !self.is_done() {
            if self.instruction_count >= max_instructions {
                return Ok(false);
            }
            self.step()?;
        }
        Ok(true)
    }

    pub fn output_as_string(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }

    /// Returns a window of cells around `center`: (address, value, is_current_ptr)
    pub fn peek_window(&self, center: usize, radius: usize) -> Vec<(usize, u8, bool)> {
        let start = center.saturating_sub(radius);
        let end = (center + radius + 1).min(self.tape.len());
        (start..end)
            .map(|i| (i, self.tape[i], i == self.data_ptr))
            .collect()
    }

    /// Feed new code into the interpreter, appending to existing program.
    /// Used by the REPL to add code incrementally.
    pub fn feed(&mut self, source: &str) -> Result<()> {
        // Rebuild the entire program from concatenated source characters
        self.source_chars.extend(source.chars());
        let full_source: String = self.source_chars.iter().collect();
        self.program = Program::from_source(&full_source)?;
        self.op_to_char = build_op_to_char_map(&full_source);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut interp = Interpreter::new("+").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 1);
    }

    #[test]
    fn test_decrement() {
        let mut interp = Interpreter::new("+-").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0);
    }

    #[test]
    fn test_move_right() {
        let mut interp = Interpreter::new(">+").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.data_pointer(), 1);
        assert_eq!(interp.tape_value(1), 1);
    }

    #[test]
    fn test_move_left() {
        let mut interp = Interpreter::new(">+<").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.data_pointer(), 0);
        assert_eq!(interp.tape_value(1), 1);
    }

    #[test]
    fn test_output() {
        let mut interp = Interpreter::new(
            "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.",
        )
        .unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output(), vec![72]); // 'H' = 72
    }

    #[test]
    fn test_input() {
        let mut interp = Interpreter::with_input(",.", "A").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "A");
    }

    #[test]
    fn test_input_eof_gives_zero() {
        let mut interp = Interpreter::with_input(",", "").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0);
    }

    #[test]
    fn test_loop_skip_when_zero() {
        let mut interp = Interpreter::new("[+]").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0); // loop body never executed
    }

    #[test]
    fn test_loop_execute_when_nonzero() {
        // +++ loop decrements until 0: tape[0]=0, tape[1]=3
        let mut interp = Interpreter::new("+++[>+<-]").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0);
        assert_eq!(interp.tape_value(1), 3);
    }

    #[test]
    fn test_wrapping_add() {
        // 0 - 1 wraps to 255, then + 1 wraps back to 0
        let mut interp = Interpreter::new("-+").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0);
    }

    #[test]
    fn test_wrapping_sub() {
        // 0 - 1 wraps to 255
        let mut interp = Interpreter::new("-").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 255);
    }

    #[test]
    fn test_unmatched_open_bracket() {
        assert!(Interpreter::new("[+").is_err());
    }

    #[test]
    fn test_unmatched_close_bracket() {
        assert!(Interpreter::new("+]").is_err());
    }

    #[test]
    fn test_comments_ignored() {
        let mut interp = Interpreter::new("+ this is a comment +").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 2);
    }

    #[test]
    fn test_step_returns_false_when_done() {
        let mut interp = Interpreter::new("+").unwrap();
        let more = interp.step().unwrap();
        assert!(!more); // only one instruction, now done
    }

    #[test]
    fn test_step_returns_true_when_more() {
        // Use two different ops so they aren't collapsed into one
        let mut interp = Interpreter::new("+>").unwrap();
        let more = interp.step().unwrap();
        assert!(more);
    }

    #[test]
    fn test_peek_window() {
        let mut interp = Interpreter::new(">>+++").unwrap();
        interp.run().unwrap();
        let window = interp.peek_window(2, 2);
        assert_eq!(window.len(), 5); // cells 0..=4
        assert_eq!(window[2], (2, 3, true)); // dp=2, val=3, is_ptr=true
        assert_eq!(window[0], (0, 0, false));
    }

    #[test]
    fn test_hello_world() {
        // Classic hello world BF
        let hw = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let mut interp = Interpreter::new(hw).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "Hello World!\n");
    }

    #[test]
    fn test_cat_program() {
        let mut interp = Interpreter::with_input(",[.,]", "Hello").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "Hello");
    }

    #[test]
    fn test_instruction_count() {
        let mut interp = Interpreter::new("+++").unwrap();
        interp.run().unwrap();
        // +++ is compiled to Add(3), which is 1 instruction
        assert_eq!(interp.instruction_count, 1);
    }

    #[test]
    fn test_run_with_limit() {
        let mut interp = Interpreter::new("+>+>+>+").unwrap();
        // 4 ops: Add(1), Right(1), Add(1), Right(1), Add(1), Right(1), Add(1)
        let completed = interp.run_with_limit(3).unwrap();
        assert!(!completed); // should not complete in 3 instructions
    }

    #[test]
    fn test_cells_touched() {
        let mut interp = Interpreter::new("+>++>+++").unwrap();
        interp.run().unwrap();
        assert!(interp.cells_touched_count() >= 3);
    }

    #[test]
    fn test_custom_tape_size() {
        let mut interp = Interpreter::with_tape_size("+", 100).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape().len(), 100);
        assert_eq!(interp.tape_value(0), 1);
    }

    #[test]
    fn test_move_add_optimized() {
        // [->+<] moves cell 0 value to cell 1
        let mut interp = Interpreter::new_optimized("+++++[->+<]").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0);
        assert_eq!(interp.tape_value(1), 5);
    }

    #[test]
    fn test_move_sub_optimized() {
        // Set cell 1 to 10, then move-subtract cell 0 from cell 1
        let mut interp = Interpreter::new_optimized(">++++++++++<+++++[->-<]").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(0), 0);
        assert_eq!(interp.tape_value(1), 5); // 10 - 5
    }

    #[test]
    fn test_large_tape_size() {
        // Verify the interpreter handles a 100,000-cell tape correctly
        let mut interp = Interpreter::with_tape_size("+>++>+++", 100_000).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape().len(), 100_000);
        assert_eq!(interp.tape_value(0), 1);
        assert_eq!(interp.tape_value(1), 2);
        assert_eq!(interp.tape_value(2), 3);
    }
}
