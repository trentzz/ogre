use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ogre::modes::interpreter::Interpreter;
use ogre::modes::ir::Program;
use ogre::modes::preprocess::Preprocessor;
use std::path::Path;

/// Hello World BF program.
const HELLO_WORLD: &str = include_str!("../tests/brainfuck_scripts/hello_world.bf");

/// Simple multiply BF program.
const SIMPLE_MULTIPLY: &str = include_str!("../tests/brainfuck_scripts/simple_multiply.bf");

/// A small but non-trivial BF program that prints "Hello World!" using multiplication loops.
const COMPACT_HELLO: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

fn bench_interpret_hello_world(c: &mut Criterion) {
    c.bench_function("interpret hello_world", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new(black_box(HELLO_WORLD)).unwrap();
            interp.run().unwrap();
        })
    });
}

fn bench_interpret_simple_multiply(c: &mut Criterion) {
    c.bench_function("interpret simple_multiply", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new(black_box(SIMPLE_MULTIPLY)).unwrap();
            interp.run().unwrap();
        })
    });
}

fn bench_interpret_compact_hello(c: &mut Criterion) {
    c.bench_function("interpret compact_hello", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new(black_box(COMPACT_HELLO)).unwrap();
            interp.run().unwrap();
        })
    });
}

fn bench_interpret_optimized(c: &mut Criterion) {
    c.bench_function("interpret compact_hello optimized", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new_optimized(black_box(COMPACT_HELLO)).unwrap();
            interp.run().unwrap();
        })
    });
}

fn bench_ir_parse(c: &mut Criterion) {
    c.bench_function("IR parse hello_world", |b| {
        b.iter(|| {
            Program::from_source(black_box(HELLO_WORLD)).unwrap();
        })
    });
}

fn bench_ir_parse_and_optimize(c: &mut Criterion) {
    c.bench_function("IR parse+optimize hello_world", |b| {
        b.iter(|| {
            let mut prog = Program::from_source(black_box(HELLO_WORLD)).unwrap();
            prog.optimize();
        })
    });
}

fn bench_ir_to_bf_string(c: &mut Criterion) {
    let prog = Program::from_source(HELLO_WORLD).unwrap();
    c.bench_function("IR to_bf_string hello_world", |b| {
        b.iter(|| {
            black_box(prog.to_bf_string());
        })
    });
}

fn bench_preprocess_with_stdlib(c: &mut Criterion) {
    let src = "@import \"std/io\"\n@import \"std/math\"\n@call print_newline\n@call zero";
    c.bench_function("preprocess with stdlib imports", |b| {
        b.iter(|| {
            Preprocessor::process_source(black_box(src), Path::new(".")).unwrap();
        })
    });
}

fn bench_compile_c_codegen(c: &mut Criterion) {
    let mut prog = Program::from_source(HELLO_WORLD).unwrap();
    prog.optimize();
    c.bench_function("C codegen hello_world", |b| {
        b.iter(|| {
            black_box(ogre::modes::compile::generate_c_from_program(&prog, 30000));
        })
    });
}

criterion_group!(
    benches,
    bench_interpret_hello_world,
    bench_interpret_simple_multiply,
    bench_interpret_compact_hello,
    bench_interpret_optimized,
    bench_ir_parse,
    bench_ir_parse_and_optimize,
    bench_ir_to_bf_string,
    bench_preprocess_with_stdlib,
    bench_compile_c_codegen,
);
criterion_main!(benches);
