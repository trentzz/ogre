#![allow(dead_code)]

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use colored::control::set_override;
use std::path::Path;
use std::process;

pub mod error;
mod modes;
mod project;
pub mod verbosity;

use modes::{
    analyse, bench, check, compile, compile_wasm, debug, doc, format, generate, init, new, pack,
    run, start, stdlib, test_runner, trace,
};
use project::OgreProject;
use verbosity::Verbosity;

#[derive(Parser)]
#[command(
    name = "ogre",
    about = "A Cargo-like all-in-one brainfuck tool",
    version,
    after_help = "Examples:\n  ogre run hello.bf\n  ogre compile hello.bf -o hello\n  ogre new myproject\n  ogre test tests/basic.json\n  ogre generate string \"Hello!\" -o hello.bf"
)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress non-essential output
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Enable verbose output
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interpret and execute a brainfuck file (or project entry if omitted)
    #[command(after_help = "Examples:\n  ogre run hello.bf\n  ogre run --tape-size 60000 big.bf\n  ogre run --watch hello.bf")]
    Run(RunArgs),
    /// Compile brainfuck to a native binary via C (or to WASM)
    #[command(after_help = "Examples:\n  ogre compile hello.bf -o hello\n  ogre compile hello.bf --keep\n  ogre compile hello.bf --target wasm")]
    Compile(CompileArgs),
    /// Build the current project (requires ogre.toml)
    #[command(after_help = "Examples:\n  ogre build\n  ogre build -o myapp\n  ogre build --keep")]
    Build(BuildArgs),
    /// Interactive brainfuck interpreter REPL
    #[command(after_help = "Examples:\n  ogre start\n  ogre start --tape-size 60000")]
    Start(StartArgs),
    /// GDB-style interactive debugger
    #[command(after_help = "Examples:\n  ogre debug hello.bf\n  ogre debug --tape-size 60000 big.bf")]
    Debug(DebugArgs),
    /// Format a brainfuck file in-place (or all project files if omitted)
    #[command(after_help = "Examples:\n  ogre format hello.bf\n  ogre format --check hello.bf\n  ogre format --diff hello.bf\n  ogre format --indent 2 --grouping 10")]
    Format(FormatArgs),
    /// Static analysis of a brainfuck script (or all project files if omitted)
    #[command(after_help = "Examples:\n  ogre analyse hello.bf\n  ogre analyse --verbose hello.bf\n  ogre analyse --in-place hello.bf")]
    Analyse(AnalyseArgs),
    /// Run structured tests from a JSON file (or all project test suites if omitted)
    #[command(after_help = "Examples:\n  ogre test tests/basic.json\n  ogre test  # runs all project test suites")]
    Test(TestArgs),
    /// Scaffold a new brainfuck project directory
    #[command(after_help = "Examples:\n  ogre new myproject\n  ogre new myproject --with-std")]
    New(NewArgs),
    /// Generate brainfuck code for common patterns
    #[command(subcommand, after_help = "Examples:\n  ogre generate helloworld\n  ogre generate string \"Hello!\" -o hello.bf\n  ogre generate loop 10")]
    Generate(GenerateCommands),
    /// Browse the built-in standard library
    #[command(subcommand, after_help = "Examples:\n  ogre stdlib list\n  ogre stdlib show io\n  ogre stdlib show math")]
    Stdlib(StdlibCommands),
    /// Validate brackets, imports, and calls (exit 0 if OK, 1 if errors)
    #[command(after_help = "Examples:\n  ogre check hello.bf\n  ogre check  # checks all project files")]
    Check(CheckArgs),
    /// Output fully preprocessed and expanded brainfuck
    #[command(after_help = "Examples:\n  ogre pack hello.bf\n  ogre pack hello.bf --optimize -o packed.bf")]
    Pack(PackArgs),
    /// Initialize ogre.toml in the current directory
    #[command(after_help = "Example:\n  cd myproject && ogre init")]
    Init,
    /// Benchmark a brainfuck program (instruction count, wall time, cells touched)
    #[command(after_help = "Examples:\n  ogre bench hello.bf\n  ogre bench --tape-size 60000 big.bf")]
    Bench(BenchArgs),
    /// Generate documentation from @doc comments and @fn definitions
    #[command(after_help = "Examples:\n  ogre doc hello.bf\n  ogre doc --stdlib\n  ogre doc hello.bf -o docs.md")]
    Doc(DocArgs),
    /// Trace execution of a brainfuck program (print tape state per instruction)
    #[command(after_help = "Examples:\n  ogre trace hello.bf\n  ogre trace --every 100 hello.bf\n  ogre trace --tape-size 1000 hello.bf")]
    Trace(TraceArgs),
}

// ---- Per-subcommand arg structs ----

#[derive(Args)]
struct RunArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Tape size (number of cells, default 30000)
    #[arg(long)]
    tape_size: Option<usize>,
    /// Watch the file for changes and re-run automatically
    #[arg(short = 'w', long)]
    watch: bool,
    /// Arguments to pass to the brainfuck program (after --)
    #[arg(last = true)]
    program_args: Vec<String>,
}

#[derive(Args)]
struct CompileArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Output binary name
    #[arg(short = 'o', long)]
    output: Option<String>,
    /// Keep the intermediate .c file
    #[arg(short = 'k', long)]
    keep: bool,
    /// Compilation target: "native" (default) or "wasm"
    #[arg(long, default_value = "native")]
    target: String,
}

#[derive(Args)]
struct BuildArgs {
    /// Output binary name (defaults to project name)
    #[arg(short = 'o', long)]
    output: Option<String>,
    /// Keep the intermediate .c file
    #[arg(short = 'k', long)]
    keep: bool,
}

#[derive(Args)]
struct StartArgs {
    /// Tape size (number of cells, default 30000)
    #[arg(long)]
    tape_size: Option<usize>,
}

#[derive(Args)]
struct DebugArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Tape size (number of cells, default 30000)
    #[arg(long)]
    tape_size: Option<usize>,
}

#[derive(Args)]
struct FormatArgs {
    /// Path to the brainfuck file (formats all project files if omitted)
    file: Option<String>,
    /// Indentation per loop level in spaces
    #[arg(long, default_value = "4")]
    indent: usize,
    /// Maximum line width
    #[arg(long, default_value = "80")]
    linewidth: usize,
    /// Group consecutive identical operators
    #[arg(long, default_value = "5")]
    grouping: usize,
    /// (brainfunct) Insert comment labels above each function
    #[arg(long)]
    label_functions: bool,
    /// Keep non-BF characters in place as comments
    #[arg(short = 'p', long)]
    preserve_comments: bool,
    /// Check formatting without modifying files (exit 1 if unformatted)
    #[arg(long)]
    check: bool,
    /// Show a diff of what the formatter would change (without modifying files)
    #[arg(long)]
    diff: bool,
}

#[derive(Args)]
struct AnalyseArgs {
    /// Path to the brainfuck file (analyses all project files if omitted)
    file: Option<String>,
    /// Embed the analysis as comments in the source file
    #[arg(long)]
    in_place: bool,
    /// Extra detail per section
    #[arg(long)]
    verbose: bool,
}

#[derive(Args)]
struct TestArgs {
    /// Path to the JSON test file (runs all project test suites if omitted)
    test_file: Option<String>,
    /// Show verbose per-test output
    #[arg(long)]
    verbose: bool,
}

#[derive(Args)]
struct NewArgs {
    /// Project name / directory to create
    name: String,
    /// Include standard library imports in the starter file
    #[arg(long)]
    with_std: bool,
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate a Hello World program
    Helloworld(GenerateOutputArgs),
    /// Generate code to print a string
    String(GenerateStringArgs),
    /// Generate a loop scaffold that runs n times
    Loop(GenerateLoopArgs),
}

#[derive(Args)]
struct CheckArgs {
    /// Path to the brainfuck file (checks all project files if omitted)
    file: Option<String>,
}

#[derive(Args)]
struct PackArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Output file (prints to stdout if omitted)
    #[arg(short = 'o', long)]
    output: Option<String>,
    /// Apply IR optimizations to the output
    #[arg(long)]
    optimize: bool,
}

#[derive(Args)]
struct BenchArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Tape size (number of cells, default 30000)
    #[arg(long)]
    tape_size: Option<usize>,
}

#[derive(Args)]
struct DocArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Generate documentation for the standard library
    #[arg(long)]
    stdlib: bool,
    /// Output file (prints to stdout if omitted)
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(Args)]
struct TraceArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Tape size (number of cells, default 30000)
    #[arg(long)]
    tape_size: Option<usize>,
    /// Print trace every N instructions (default: every instruction)
    #[arg(long, default_value = "1")]
    every: usize,
}

#[derive(Subcommand)]
enum StdlibCommands {
    /// List all available standard library modules
    List,
    /// Show the source of a standard library module
    Show(StdlibShowArgs),
}

#[derive(Args)]
struct StdlibShowArgs {
    /// Module name (e.g., io, math, memory, ascii, debug)
    module: String,
}

#[derive(Args)]
struct GenerateOutputArgs {
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(Args)]
struct GenerateStringArgs {
    string: String,
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(Args)]
struct GenerateLoopArgs {
    n: usize,
    #[arg(short = 'o', long)]
    output: Option<String>,
}

// ---- main ----

fn main() {
    let cli = Cli::parse();

    // Handle --no-color flag and NO_COLOR env var
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        set_override(false);
    }

    if let Err(e) = run(cli) {
        eprintln!(
            "{} {}",
            colored::Colorize::red("error:"),
            e
        );
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let verbosity = if cli.quiet {
        Verbosity::Quiet
    } else if cli.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };

    match cli.command {
        Commands::Run(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            let has_args = !args.program_args.is_empty();
            let file = match args.file {
                Some(f) => std::path::PathBuf::from(f),
                None => {
                    let (proj, base) = require_project()?;
                    let ts = proj
                        .build
                        .as_ref()
                        .and_then(|b| b.tape_size)
                        .unwrap_or(tape_size);
                    let dep_fns = proj.collect_dependency_functions(&base)?;
                    let entry = proj.entry_path(&base);
                    if args.watch {
                        run::run_file_watch(&entry, ts)?;
                    } else if has_args && !dep_fns.is_empty() {
                        run::run_file_with_args_and_deps(&entry, ts, &args.program_args, &dep_fns)?;
                    } else if has_args {
                        run::run_file_with_args(&entry, ts, &args.program_args)?;
                    } else if dep_fns.is_empty() {
                        run::run_file_with_tape_size(&entry, ts)?;
                    } else {
                        run::run_file_with_deps(&entry, ts, &dep_fns)?;
                    }
                    return Ok(());
                }
            };
            if args.watch {
                run::run_file_watch(&file, tape_size)?;
            } else if has_args {
                run::run_file_with_args(&file, tape_size, &args.program_args)?;
            } else {
                run::run_file_with_tape_size(&file, tape_size)?;
            }
        }

        Commands::Compile(args) => {
            let (file, dep_fns) = match args.file {
                Some(f) => (std::path::PathBuf::from(f), std::collections::HashMap::new()),
                None => {
                    let (proj, base) = require_project()?;
                    let deps = proj.collect_dependency_functions(&base)?;
                    (proj.entry_path(&base), deps)
                }
            };
            match args.target.as_str() {
                "wasm" => {
                    compile_wasm::compile_to_wasm(
                        &file,
                        args.output.as_deref(),
                        30_000,
                        verbosity,
                    )?;
                }
                "native" | "" => {
                    if dep_fns.is_empty() {
                        compile::compile_ex(
                            &file,
                            args.output.as_deref(),
                            args.keep,
                            verbosity,
                        )?;
                    } else {
                        compile::compile_with_deps_ex(
                            &file,
                            args.output.as_deref(),
                            args.keep,
                            30_000,
                            verbosity,
                            &dep_fns,
                        )?;
                    }
                }
                other => {
                    bail!("unknown target {:?}. Use \"native\" or \"wasm\".", other);
                }
            }
        }

        Commands::Build(args) => {
            let (proj, base) = require_project()?;
            let entry = proj.entry_path(&base);
            let dep_fns = proj.collect_dependency_functions(&base)?;
            let out_name = args
                .output
                .as_deref()
                .unwrap_or(&proj.project.name)
                .to_string();
            if dep_fns.is_empty() {
                compile::compile_ex(&entry, Some(&out_name), args.keep, verbosity)?;
            } else {
                compile::compile_with_deps_ex(&entry, Some(&out_name), args.keep, 30_000, verbosity, &dep_fns)?;
            }
            if !verbosity.is_quiet() {
                let desc = proj
                    .project
                    .description
                    .as_deref()
                    .filter(|d| !d.is_empty());
                let author = proj.project.author.as_deref().filter(|a| !a.is_empty());
                match (desc, author) {
                    (Some(d), Some(a)) => println!(
                        "Built {} v{} — {} (by {})",
                        proj.project.name, proj.project.version, d, a
                    ),
                    (Some(d), None) => println!(
                        "Built {} v{} — {}",
                        proj.project.name, proj.project.version, d
                    ),
                    _ => println!("Built {} v{}", proj.project.name, proj.project.version),
                }
            }
        }

        Commands::Start(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            // If there's a project, preload its @fn definitions
            match OgreProject::find()? {
                Some((proj, base)) => {
                    let ts = proj
                        .build
                        .as_ref()
                        .and_then(|b| b.tape_size)
                        .unwrap_or(tape_size);
                    start::start_repl_project(ts, &proj, &base)?;
                }
                None => {
                    start::start_repl_with_tape_size(tape_size)?;
                }
            }
        }

        Commands::Debug(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            match args.file {
                Some(f) => debug::debug_file_with_tape_size(Path::new(&f), tape_size)?,
                None => {
                    let (proj, base) = require_project()?;
                    let ts = proj
                        .build
                        .as_ref()
                        .and_then(|b| b.tape_size)
                        .unwrap_or(tape_size);
                    let entry = proj.entry_path(&base);
                    let dep_fns = proj.collect_dependency_functions(&base)?;
                    if dep_fns.is_empty() {
                        debug::debug_file_with_tape_size(&entry, ts)?;
                    } else {
                        debug::debug_file_with_deps(&entry, ts, &dep_fns)?;
                    }
                }
            }
        }

        Commands::Format(args) => {
            let opts = format::FormatOptions {
                indent: args.indent,
                linewidth: args.linewidth,
                grouping: args.grouping,
                label_functions: args.label_functions,
                preserve_comments: args.preserve_comments,
                check: args.check,
                diff: args.diff,
            };
            let mut all_formatted = true;
            match args.file {
                Some(f) => {
                    if !format::format_file(Path::new(&f), &opts)? {
                        all_formatted = false;
                    }
                }
                None => {
                    let (proj, base) = require_project()?;
                    let files = proj.resolve_include_files(&base)?;
                    if files.is_empty() && !verbosity.is_quiet() {
                        println!("No .bf files found in project include paths.");
                    }
                    for f in &files {
                        if !opts.check && !verbosity.is_quiet() {
                            println!("Formatting: {}", f.display());
                        }
                        if !format::format_file(f, &opts)? {
                            all_formatted = false;
                        }
                    }
                }
            }
            if (opts.check || opts.diff) && !all_formatted {
                process::exit(1);
            }
        }

        Commands::Analyse(args) => {
            let analyse_verbose = args.verbose || verbosity.is_verbose();
            match args.file {
                Some(f) => analyse::analyse_file(Path::new(&f), analyse_verbose, args.in_place)?,
                None => {
                    let (proj, base) = require_project()?;
                    let files = proj.resolve_include_files(&base)?;
                    if files.is_empty() && !verbosity.is_quiet() {
                        println!("No .bf files found in project include paths.");
                    }
                    for f in &files {
                        if !verbosity.is_quiet() {
                            println!("=== {} ===", f.display());
                        }
                        analyse::analyse_file(f, analyse_verbose, args.in_place)?;
                    }
                }
            }
        }

        Commands::Test(args) => {
            let test_verbosity = if args.verbose {
                Verbosity::Verbose
            } else {
                verbosity
            };
            match args.test_file {
                Some(f) => test_runner::run_tests_ex(Path::new(&f), test_verbosity)?,
                None => {
                    let (proj, base) = require_project()?;
                    test_runner::run_project_tests_ex(&proj, &base, test_verbosity)?;
                }
            }
        }

        Commands::New(args) => {
            new::new_project_ex(&args.name, args.with_std, verbosity)?;
        }

        Commands::Check(args) => {
            let mut all_ok = true;
            match args.file {
                Some(f) => {
                    if !check::check_and_report_ex(Path::new(&f), verbosity)? {
                        all_ok = false;
                    }
                }
                None => {
                    let (proj, base) = require_project()?;
                    let dep_fns = proj.collect_dependency_functions(&base)?;
                    let files = proj.resolve_include_files(&base)?;
                    if files.is_empty() && !verbosity.is_quiet() {
                        println!("No .bf files found in project include paths.");
                    }
                    for f in &files {
                        if dep_fns.is_empty() {
                            if !check::check_and_report_ex(f, verbosity)? {
                                all_ok = false;
                            }
                        } else {
                            let result = check::check_file_with_deps(f, &dep_fns)?;
                            if result.errors.is_empty() {
                                if !verbosity.is_quiet() {
                                    println!("{}: {}", f.display(), colored::Colorize::green("OK"));
                                }
                            } else {
                                for err in &result.errors {
                                    println!("{}: {} {}", f.display(), colored::Colorize::red("ERROR"), err);
                                }
                                all_ok = false;
                            }
                        }
                    }
                }
            }
            if !all_ok {
                process::exit(1);
            }
        }

        Commands::Pack(args) => {
            let (file, dep_fns) = match args.file {
                Some(f) => (std::path::PathBuf::from(f), std::collections::HashMap::new()),
                None => {
                    let (proj, base) = require_project()?;
                    let deps = proj.collect_dependency_functions(&base)?;
                    (proj.entry_path(&base), deps)
                }
            };
            if dep_fns.is_empty() {
                pack::pack_and_output_ex(&file, args.output.as_deref(), args.optimize, verbosity)?;
            } else {
                pack::pack_and_output_with_deps(&file, args.output.as_deref(), args.optimize, verbosity, &dep_fns)?;
            }
        }

        Commands::Init => {
            init::init_project_ex(verbosity)?;
        }

        Commands::Bench(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            let (file, dep_fns) = match args.file {
                Some(f) => (std::path::PathBuf::from(f), std::collections::HashMap::new()),
                None => {
                    let (proj, base) = require_project()?;
                    let deps = proj.collect_dependency_functions(&base)?;
                    (proj.entry_path(&base), deps)
                }
            };
            if dep_fns.is_empty() {
                bench::bench_and_report_ex(&file, tape_size, verbosity)?;
            } else {
                bench::bench_and_report_with_deps(&file, tape_size, verbosity, &dep_fns)?;
            }
        }

        Commands::Doc(args) => {
            let path = match &args.file {
                Some(f) => Some(std::path::PathBuf::from(f)),
                None if !args.stdlib => {
                    let (proj, base) = require_project()?;
                    Some(proj.entry_path(&base))
                }
                None => None,
            };
            doc::doc_and_output(
                path.as_deref(),
                args.stdlib,
                args.output.as_deref(),
            )?;
        }

        Commands::Trace(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            let file = match args.file {
                Some(f) => std::path::PathBuf::from(f),
                None => {
                    let (proj, base) = require_project()?;
                    proj.entry_path(&base)
                }
            };
            trace::trace_file(&file, tape_size, args.every)?;
        }

        Commands::Stdlib(cmd) => match cmd {
            StdlibCommands::List => {
                stdlib::list_modules();
            }
            StdlibCommands::Show(args) => {
                stdlib::show_module(&args.module)?;
            }
        },

        Commands::Generate(gen) => match gen {
            GenerateCommands::Helloworld(args) => {
                let code = generate::generate_hello_world();
                generate::write_or_print(&code, args.output.as_deref())?;
            }
            GenerateCommands::String(args) => {
                let code = generate::generate_string(&args.string)?;
                generate::write_or_print(&code, args.output.as_deref())?;
            }
            GenerateCommands::Loop(args) => {
                let code = generate::generate_loop(args.n);
                generate::write_or_print(&code, args.output.as_deref())?;
            }
        },
    }

    Ok(())
}

/// Find an ogre.toml by walking upward from CWD, or bail with a helpful error.
fn require_project() -> Result<(OgreProject, std::path::PathBuf)> {
    match OgreProject::find()? {
        Some(pair) => Ok(pair),
        None => bail!(
            "no ogre.toml found. Run `ogre new <name>` to create a project, \
             or supply a file argument."
        ),
    }
}
