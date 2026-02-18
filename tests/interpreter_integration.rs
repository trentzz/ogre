use ogre::modes::interpreter::Interpreter;
use std::fs;

fn script_path(name: &str) -> String {
    format!("tests/brainfuck_scripts/{}", name)
}

#[test]
fn test_hello_world_from_file() {
    let source = fs::read_to_string(script_path("hello_world.bf")).unwrap();
    let mut interp = Interpreter::new(&source).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "Hello World!\n");
}

#[test]
fn test_cat_program() {
    let source = fs::read_to_string(script_path("cat.bf")).unwrap();
    let input = "Hello, World!";
    let mut interp = Interpreter::with_input(&source, input).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), input);
}

#[test]
fn test_cell_multiplication() {
    // ++++[>+++<-] → cell 0: 0, cell 1: 12 (4×3)
    let mut interp = Interpreter::new("++++[>+++<-]").unwrap();
    interp.run().unwrap();
    assert_eq!(interp.tape[0], 0);
    assert_eq!(interp.tape[1], 12);
}

#[test]
fn test_multiply_from_file() {
    let source = fs::read_to_string(script_path("simple_multiply.bf")).unwrap();
    let mut interp = Interpreter::new(&source).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.tape[0], 0);
    assert_eq!(interp.tape[1], 12);
}

#[test]
fn test_multiple_outputs() {
    // Print ASCII 65 'A' and 66 'B' (65 + signs = 'A', then +1 = 'B')
    let mut interp = Interpreter::new(
        "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.+.",
    )
    .unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "AB");
}

#[test]
fn test_data_ptr_bounds_left() {
    let mut interp = Interpreter::new("<").unwrap();
    assert!(interp.run().is_err());
}

#[test]
fn test_is_done_after_run() {
    let mut interp = Interpreter::new("+++").unwrap();
    interp.run().unwrap();
    assert!(interp.is_done());
}

#[test]
fn test_step_by_step() {
    let mut interp = Interpreter::new("+++").unwrap();
    assert_eq!(interp.tape[0], 0);
    interp.step().unwrap();
    assert_eq!(interp.tape[0], 1);
    interp.step().unwrap();
    assert_eq!(interp.tape[0], 2);
    interp.step().unwrap();
    assert_eq!(interp.tape[0], 3);
    assert!(interp.is_done());
}
