use ogre::modes::format::{format_source, FormatOptions};
use ogre::modes::interpreter::Interpreter;

fn default_opts() -> FormatOptions {
    FormatOptions::default()
}

#[test]
fn test_format_preserves_semantics() {
    // Run original, then format and run again — same output
    let original = "++++[>+++<-]>.";
    let opts = default_opts();
    let formatted = format_source(original, &opts).unwrap();

    let mut interp1 = Interpreter::new(original).unwrap();
    interp1.run().unwrap();

    let mut interp2 = Interpreter::new(&formatted).unwrap();
    interp2.run().unwrap();

    assert_eq!(interp1.output_as_string(), interp2.output_as_string());
}

#[test]
fn test_format_nested_loops_indentation() {
    let opts = FormatOptions {
        indent: 4,
        ..Default::default()
    };
    let result = format_source("[[+]]", &opts).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    // Find the line with '+'
    let inner_line = lines.iter().find(|l| l.contains('+')).unwrap();
    // Should be indented 8 spaces (depth 2 × 4)
    assert!(
        inner_line.starts_with("        "),
        "expected 8-space indent, got: {:?}",
        inner_line
    );
}

#[test]
fn test_roundtrip_idempotent() {
    let opts = default_opts();
    let code = "++++[>+++<-]";
    let once = format_source(code, &opts).unwrap();
    let twice = format_source(&once, &opts).unwrap();
    assert_eq!(once, twice, "format should be idempotent");
}

#[test]
fn test_format_hello_world_preserves_output() {
    let hw = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    let opts = default_opts();
    let formatted = format_source(hw, &opts).unwrap();

    let mut interp = Interpreter::new(&formatted).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.output_as_string(), "Hello World!\n");
}

#[test]
fn test_format_grouping_with_semantics() {
    // 10 increments grouped as "+++++ +++++" still produce value 10
    let opts = FormatOptions {
        grouping: 5,
        ..Default::default()
    };
    let code = "++++++++++";
    let formatted = format_source(code, &opts).unwrap();
    assert!(formatted.contains("+++++ +++++"), "got: {:?}", formatted);

    let mut interp = Interpreter::new(&formatted).unwrap();
    interp.run().unwrap();
    assert_eq!(interp.tape_value(0), 10);
}

#[test]
fn test_format_strips_comments_by_default() {
    let opts = default_opts();
    let code = "+ increment the cell - decrement it";
    let result = format_source(code, &opts).unwrap();
    assert!(!result.contains("increment"));
    assert!(!result.contains("decrement"));
}

#[test]
fn test_format_preserve_comments_flag() {
    let opts = FormatOptions {
        preserve_comments: true,
        ..Default::default()
    };
    let code = "+ this is a comment";
    let result = format_source(code, &opts).unwrap();
    assert!(result.contains("this is a comment"));
}
