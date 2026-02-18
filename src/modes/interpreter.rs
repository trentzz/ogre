use anyhow::{bail, Result};

pub struct Interpreter {
    pub tape: Vec<u8>,
    pub data_ptr: usize,
    pub code: Vec<char>,
    pub code_ptr: usize,
    pub jump_table: Vec<Option<usize>>,
    pub output: Vec<u8>,
    pub input: Vec<u8>,
    pub input_ptr: usize,
    /// When true, `,` reads from real stdin after the input buffer is exhausted.
    pub live_stdin: bool,
}

impl Interpreter {
    pub fn new(source: &str) -> Result<Self> {
        Self::with_input(source, "")
    }

    pub fn with_live_stdin(source: &str) -> Result<Self> {
        let code: Vec<char> = source.chars().collect();
        let jump_table = build_jump_table(&code)?;
        Ok(Self {
            tape: vec![0u8; 30_000],
            data_ptr: 0,
            code,
            code_ptr: 0,
            jump_table,
            output: Vec::new(),
            input: Vec::new(),
            input_ptr: 0,
            live_stdin: true,
        })
    }

    pub fn with_input(source: &str, input: &str) -> Result<Self> {
        let code: Vec<char> = source.chars().collect();
        let jump_table = build_jump_table(&code)?;
        Ok(Self {
            tape: vec![0u8; 30_000],
            data_ptr: 0,
            code,
            code_ptr: 0,
            jump_table,
            output: Vec::new(),
            input: input.bytes().collect(),
            input_ptr: 0,
            live_stdin: false,
        })
    }

    pub fn is_done(&self) -> bool {
        self.code_ptr >= self.code.len()
    }

    /// Execute one BF instruction (skipping non-BF characters).
    /// Returns `Ok(true)` if there are more instructions to execute, `Ok(false)` if done.
    pub fn step(&mut self) -> Result<bool> {
        // Skip non-BF characters
        while self.code_ptr < self.code.len() && !is_bf_op(self.code[self.code_ptr]) {
            self.code_ptr += 1;
        }

        if self.is_done() {
            return Ok(false);
        }

        match self.code[self.code_ptr] {
            '>' => {
                if self.data_ptr + 1 >= self.tape.len() {
                    bail!("data pointer out of bounds (right)");
                }
                self.data_ptr += 1;
            }
            '<' => {
                if self.data_ptr == 0 {
                    bail!("data pointer out of bounds (left)");
                }
                self.data_ptr -= 1;
            }
            '+' => {
                self.tape[self.data_ptr] = self.tape[self.data_ptr].wrapping_add(1);
            }
            '-' => {
                self.tape[self.data_ptr] = self.tape[self.data_ptr].wrapping_sub(1);
            }
            '.' => {
                self.output.push(self.tape[self.data_ptr]);
            }
            ',' => {
                if self.input_ptr < self.input.len() {
                    self.tape[self.data_ptr] = self.input[self.input_ptr];
                    self.input_ptr += 1;
                } else if self.live_stdin {
                    use std::io::Read;
                    let mut byte = [0u8; 1];
                    match std::io::stdin().read(&mut byte) {
                        Ok(1) => self.tape[self.data_ptr] = byte[0],
                        _ => self.tape[self.data_ptr] = 0, // EOF
                    }
                } else {
                    self.tape[self.data_ptr] = 0; // EOF
                }
            }
            '[' => {
                if self.tape[self.data_ptr] == 0 {
                    // Jump to matching ] + 1
                    let target = self.jump_table[self.code_ptr]
                        .expect("jump table must have entry for [");
                    self.code_ptr = target + 1;
                    return Ok(!self.is_done());
                }
            }
            ']' => {
                if self.tape[self.data_ptr] != 0 {
                    // Jump back to matching [ + 1
                    let target = self.jump_table[self.code_ptr]
                        .expect("jump table must have entry for ]");
                    self.code_ptr = target + 1;
                    return Ok(!self.is_done());
                }
            }
            _ => {}
        }

        self.code_ptr += 1;
        Ok(!self.is_done())
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.is_done() {
            self.step()?;
        }
        Ok(())
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
}

fn is_bf_op(c: char) -> bool {
    matches!(c, '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']')
}

fn build_jump_table(code: &[char]) -> Result<Vec<Option<usize>>> {
    let mut table = vec![None; code.len()];
    let mut stack: Vec<usize> = Vec::new();

    for (i, &ch) in code.iter().enumerate() {
        match ch {
            '[' => stack.push(i),
            ']' => {
                let open = stack.pop().ok_or_else(|| {
                    anyhow::anyhow!("unmatched `]` at position {}", i)
                })?;
                table[open] = Some(i);
                table[i] = Some(open);
            }
            _ => {}
        }
    }

    if let Some(pos) = stack.pop() {
        bail!("unmatched `[` at position {}", pos);
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut interp = Interpreter::new("+").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape[0], 1);
    }

    #[test]
    fn test_decrement() {
        let mut interp = Interpreter::new("+-").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape[0], 0);
    }

    #[test]
    fn test_move_right() {
        let mut interp = Interpreter::new(">+").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.data_ptr, 1);
        assert_eq!(interp.tape[1], 1);
    }

    #[test]
    fn test_move_left() {
        let mut interp = Interpreter::new(">+<").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.data_ptr, 0);
        assert_eq!(interp.tape[1], 1);
    }

    #[test]
    fn test_output() {
        let mut interp = Interpreter::new("++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output, vec![72]); // 'H' = 72
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
        assert_eq!(interp.tape[0], 0);
    }

    #[test]
    fn test_loop_skip_when_zero() {
        let mut interp = Interpreter::new("[+]").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape[0], 0); // loop body never executed
    }

    #[test]
    fn test_loop_execute_when_nonzero() {
        // +++ loop decrements until 0: tape[0]=0, tape[1]=3
        let mut interp = Interpreter::new("+++[>+<-]").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape[0], 0);
        assert_eq!(interp.tape[1], 3);
    }

    #[test]
    fn test_wrapping_add() {
        // cell starts at 255, +1 should wrap to 0
        let mut code = "-".repeat(1); // start at 0, then we set to 255 via wrapping
        // Use wrapping: 0 - 1 = 255
        let mut interp = Interpreter::new("-").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape[0], 255);
        // Now add 1 to 255 → 0
        let mut interp2 = Interpreter::new("-+").unwrap();
        // actually let's just set tape manually and test
        let mut interp3 = Interpreter::new("+").unwrap();
        interp3.tape[0] = 255;
        interp3.run().unwrap();
        assert_eq!(interp3.tape[0], 0);
        drop(code);
    }

    #[test]
    fn test_wrapping_sub() {
        // 0 - 1 wraps to 255
        let mut interp = Interpreter::new("-").unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape[0], 255);
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
        assert_eq!(interp.tape[0], 2);
    }

    #[test]
    fn test_step_returns_false_when_done() {
        let mut interp = Interpreter::new("+").unwrap();
        let more = interp.step().unwrap();
        assert!(!more); // only one instruction, now done
    }

    #[test]
    fn test_step_returns_true_when_more() {
        let mut interp = Interpreter::new("++").unwrap();
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
}
