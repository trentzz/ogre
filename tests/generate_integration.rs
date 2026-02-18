use ogre::modes::generate::{generate_hello_world, generate_loop, generate_string};
use ogre::modes::interpreter::Interpreter;

#[test]
fn test_generate_hello_world() {
    let code = generate_hello_world();
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "Hello World!\n");
}

#[test]
fn test_generate_string_hi() {
    let code = generate_string("Hi!");
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "Hi!");
}

#[test]
fn test_generate_string_empty() {
    let code = generate_string("");
    // Empty string → no output ops
    assert!(!code.contains('.'));
}

#[test]
fn test_generate_string_single_char() {
    let code = generate_string("Z");
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "Z");
}

#[test]
fn test_generate_string_decreasing_ascii() {
    // 'C' (67) followed by 'A' (65) — requires decrements
    let code = generate_string("CA");
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "CA");
}

#[test]
fn test_generate_loop_counter_reaches_zero() {
    // generate_loop(n) moves value n to cell 1, leaving cell 0 at 0
    let code = generate_loop(5);
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    // Cell 0 should be 0 (loop counter exhausted)
    assert_eq!(interp.tape[0], 0);
    // Cell 1 should be 5 (incremented 5 times in loop body)
    assert_eq!(interp.tape[1], 5);
}

#[test]
fn test_generate_loop_zero() {
    let code = generate_loop(0);
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.tape[0], 0);
    assert_eq!(interp.tape[1], 0);
}

#[test]
fn test_generate_loop_one() {
    let code = generate_loop(1);
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.tape[1], 1);
}

#[test]
fn test_generate_string_newline() {
    let code = generate_string("\n");
    let mut interp = Interpreter::new(&code).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "\n");
}
