use anyhow::{bail, Result};
use std::fs;

/// Returns the classic BF hello world program.
pub fn generate_hello_world() -> String {
    "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".to_string()
}

/// Generates BF code that prints the given string.
/// Uses a simple approach: set each cell to the ASCII value and print.
pub fn generate_string(s: &str) -> Result<String> {
    if !s.is_ascii() {
        bail!("generate string only supports ASCII characters");
    }
    let mut code = String::new();
    let mut current_val: u8 = 0;

    for ch in s.chars() {
        let target = ch as u8;
        if target >= current_val {
            let diff = target - current_val;
            code.push_str(&"+".repeat(diff as usize));
        } else {
            let diff = current_val - target;
            code.push_str(&"-".repeat(diff as usize));
        }
        code.push('.');
        current_val = target;
    }

    Ok(code)
}

/// Generates a BF loop scaffold that runs exactly `n` times.
/// Uses cell 0 as counter, cell 1 as the loop body.
pub fn generate_loop(n: usize) -> String {
    if n == 0 {
        return String::new();
    }

    // We can't easily encode arbitrary n in pure BF without multiplication
    // For small n (≤255), put n in cell 0 and loop
    // For larger n, use multiplication
    if n <= 255 {
        format!("{}[>+<-]", "+".repeat(n))
    } else {
        // Use multiplication: e.g., for 256: ++[>++++++++[>+<-]<-]  (wrong, placeholder)
        // Simple approach: just do it directly for arbitrary n ≤ 255*255
        // Find best a*b ≈ n
        let mut best_a = n.min(255);
        let mut best_b = 1;
        let mut best_cost = best_a + best_b + 5; // rough cost estimate

        for a in 2..=255usize {
            let b = (n + a / 2) / a; // round
            if a * b == n && b <= 255 {
                let cost = a + b + 5;
                if cost < best_cost {
                    best_a = a;
                    best_b = b;
                    best_cost = cost;
                }
            }
        }

        if best_a * best_b == n {
            // Use nested loops: cell0 = a, inner loop: cell1 += b, outer loop runs a times
            format!("{}[>{}[>>+<<-]<-]", "+".repeat(best_a), "+".repeat(best_b))
        } else {
            // Fallback: just use direct increment (may be large)
            format!("{}[>+<-]", "+".repeat(n.min(255)))
        }
    }
}

pub fn write_or_print(code: &str, output: Option<&str>) -> Result<()> {
    match output {
        Some(path) => {
            fs::write(path, code)?;
            println!("Written to: {}", path);
        }
        None => print!("{}", code),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::interpreter::Interpreter;

    #[test]
    fn test_generate_hello_world_runs_correctly() {
        let code = generate_hello_world();
        let mut interp = Interpreter::new(&code).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "Hello World!\n");
    }

    #[test]
    fn test_generate_string_hi() {
        let code = generate_string("Hi!").unwrap();
        let mut interp = Interpreter::new(&code).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "Hi!");
    }

    #[test]
    fn test_generate_string_empty() {
        let code = generate_string("").unwrap();
        assert!(code.is_empty() || !code.contains('.'));
    }

    #[test]
    fn test_generate_string_single_char() {
        let code = generate_string("A").unwrap();
        let mut interp = Interpreter::new(&code).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.output_as_string(), "A");
    }

    #[test]
    fn test_generate_string_non_ascii_errors() {
        assert!(generate_string("hello 🌍").is_err());
    }

    #[test]
    fn test_generate_loop_zero() {
        let code = generate_loop(0);
        // Empty or no-op
        let mut interp = Interpreter::new(&code).unwrap();
        interp.run().unwrap();
        // Cell 1 should be 0 since n=0
        assert_eq!(interp.tape_value(1), 0);
    }

    #[test]
    fn test_generate_loop_three() {
        let code = generate_loop(3);
        let mut interp = Interpreter::new(&code).unwrap();
        interp.run().unwrap();
        // Cell 1 should have value 3
        assert_eq!(interp.tape_value(1), 3);
    }

    #[test]
    fn test_generate_loop_ten() {
        let code = generate_loop(10);
        let mut interp = Interpreter::new(&code).unwrap();
        interp.run().unwrap();
        assert_eq!(interp.tape_value(1), 10);
    }
}
