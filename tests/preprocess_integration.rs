use ogre::modes::interpreter::Interpreter;
use ogre::modes::preprocess::Preprocessor;
use std::path::Path;

fn script_path(name: &str) -> std::path::PathBuf {
    Path::new("tests/brainfuck_scripts").join(name)
}

/// Process a file and run it through the interpreter.
fn run_file(name: &str) -> String {
    let path = script_path(name);
    let expanded = Preprocessor::process_file(&path)
        .unwrap_or_else(|e| panic!("preprocess error for {}: {}", name, e));
    let mut interp = Interpreter::with_input(&expanded, "")
        .unwrap_or_else(|e| panic!("parse error for {}: {}", name, e));
    interp
        .run()
        .unwrap_or_else(|e| panic!("runtime error for {}: {}", name, e));
    interp.output_as_string()
}

#[test]
fn test_uses_import_outputs_a_and_newline() {
    // uses_import.bf @imports utils.bf which defines print_A and print_newline
    let output = run_file("uses_import.bf");
    // print_A outputs 'A' (ASCII 65), print_newline outputs '\n' (ASCII 10)
    assert_eq!(output, "A\n", "expected 'A\\n', got {:?}", output);
}

#[test]
fn test_plain_bf_file_unaffected() {
    // hello_world.bf has no directives — should work exactly as before
    let path = script_path("hello_world.bf");
    let expanded = Preprocessor::process_file(&path).unwrap();
    let mut interp = Interpreter::new(&expanded).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "Hello World!\n");
}

#[test]
fn test_inline_fn_call_expansion() {
    let src = "@fn double { ++ } @call double @call double";
    let out = Preprocessor::process_source(src, Path::new(".")).unwrap();
    // Four '+' in total
    assert_eq!(out.chars().filter(|&c| c == '+').count(), 4);
    assert!(!out.contains("@call"));
    assert!(!out.contains("@fn"));
}

#[test]
fn test_cycle_detection_gives_error() {
    let src = "@fn a { @call b } @fn b { @call a } @call a";
    let result = Preprocessor::process_source(src, Path::new("."));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cycle"), "expected 'cycle' in: {}", msg);
}

#[test]
fn test_self_cycle_detection() {
    let src = "@fn recurse { @call recurse } @call recurse";
    let result = Preprocessor::process_source(src, Path::new("."));
    assert!(result.is_err());
}

#[test]
fn test_unknown_call_is_error() {
    let src = "@call undefined_fn";
    let result = Preprocessor::process_source(src, Path::new("."));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("undefined_fn"));
}

#[test]
fn test_fn_not_emitted_without_call() {
    // Defining a function without calling it should produce no BF output
    let src = "@fn unused { +++ }";
    let out = Preprocessor::process_source(src, Path::new(".")).unwrap();
    // No '+' should appear (function never called)
    assert!(!out.contains('+'), "got: {:?}", out);
}

#[test]
fn test_nested_calls_expand_correctly() {
    let src = "@fn inner { + } @fn outer { @call inner @call inner @call inner } @call outer";
    let out = Preprocessor::process_source(src, Path::new(".")).unwrap();
    assert_eq!(out.chars().filter(|&c| c == '+').count(), 3);
}

#[test]
fn test_import_nonexistent_file_is_error() {
    let src = "@import \"definitely_does_not_exist_xyzzy.bf\"";
    let result = Preprocessor::process_source(src, Path::new("."));
    assert!(result.is_err());
}

#[test]
fn test_process_source_empty() {
    let out = Preprocessor::process_source("", Path::new(".")).unwrap();
    assert_eq!(out, "");
}

#[test]
fn test_expanded_output_is_valid_bf() {
    // uses_import.bf produces BF that can be parsed by the interpreter
    let path = script_path("uses_import.bf");
    let expanded = Preprocessor::process_file(&path).unwrap();
    // Must not contain any '@' characters after expansion
    assert!(
        !expanded.contains('@'),
        "expanded output contains '@': {:?}",
        expanded
    );
    // Must be parseable by interpreter
    assert!(Interpreter::new(&expanded).is_ok());
}
