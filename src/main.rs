#![allow(dead_code)]

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use colored::control::set_override;
use std::path::Path;
use std::process;

mod modes;
mod project;

use modes::{
    analyse, bench, check, compile, debug, format, generate, init, new, pack, run, start, stdlib,
    test_runner,
};
use project::OgreProject;

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
    #[command(after_help = "Examples:\n  ogre run hello.bf\n  ogre run --tape-size 60000 big.bf")]
    Run(RunArgs),
    /// Compile brainfuck to a native binary via C
    #[command(after_help = "Examples:\n  ogre compile hello.bf -o hello\n  ogre compile hello.bf --keep")]
    Compile(CompileArgs),
    /// Build the current project (requires ogre.toml)
    Build(BuildArgs),
    /// Interactive brainfuck interpreter REPL
    Start(StartArgs),
    /// GDB-style interactive debugger
    Debug(DebugArgs),
    /// Format a brainfuck file in-place (or all project files if omitted)
    Format(FormatArgs),
    /// Static analysis of a brainfuck script (or all project files if omitted)
    Analyse(AnalyseArgs),
    /// Run structured tests from a JSON file (or all project test suites if omitted)
    Test(TestArgs),
    /// Scaffold a new brainfuck project directory
    New(NewArgs),
    /// Generate brainfuck code for common patterns
    #[command(subcommand)]
    Generate(GenerateCommands),
    /// Browse the built-in standard library
    #[command(subcommand)]
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
}

// ---- Per-subcommand arg structs ----

#[derive(Args)]
struct RunArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
    /// Tape size (number of cells, default 30000)
    #[arg(long)]
    tape_size: Option<usize>,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --no-color flag and NO_COLOR env var
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        set_override(false);
    }

    match cli.command {
        Commands::Run(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            match args.file {
                Some(f) => run::run_file_with_tape_size(Path::new(&f), tape_size)?,
                None => {
                    let (proj, base) = require_project()?;
                    let ts = proj
                        .build
                        .as_ref()
                        .and_then(|b| b.tape_size)
                        .unwrap_or(tape_size);
                    let entry = proj.entry_path(&base);
                    run::run_file_with_tape_size(&entry, ts)?;
                }
            }
        }

        Commands::Compile(args) => {
            let file = match args.file {
                Some(f) => std::path::PathBuf::from(f),
                None => {
                    let (proj, base) = require_project()?;
                    proj.entry_path(&base)
                }
            };
            compile::compile(&file, args.output.as_deref(), args.keep)?;
        }

        Commands::Build(args) => {
            let (proj, base) = require_project()?;
            let entry = proj.entry_path(&base);
            let out_name = args
                .output
                .as_deref()
                .unwrap_or(&proj.project.name)
                .to_string();
            compile::compile(&entry, Some(&out_name), args.keep)?;
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

        Commands::Start(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            start::start_repl_with_tape_size(tape_size)?;
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
                    debug::debug_file_with_tape_size(&entry, ts)?;
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
                    if files.is_empty() {
                        println!("No .bf files found in project include paths.");
                    }
                    for f in &files {
                        if !opts.check {
                            println!("Formatting: {}", f.display());
                        }
                        if !format::format_file(f, &opts)? {
                            all_formatted = false;
                        }
                    }
                }
            }
            if opts.check && !all_formatted {
                process::exit(1);
            }
        }

        Commands::Analyse(args) => match args.file {
            Some(f) => analyse::analyse_file(Path::new(&f), args.verbose, args.in_place)?,
            None => {
                let (proj, base) = require_project()?;
                let files = proj.resolve_include_files(&base)?;
                if files.is_empty() {
                    println!("No .bf files found in project include paths.");
                }
                for f in &files {
                    println!("=== {} ===", f.display());
                    analyse::analyse_file(f, args.verbose, args.in_place)?;
                }
            }
        },

        Commands::Test(args) => match args.test_file {
            Some(f) => test_runner::run_tests(Path::new(&f))?,
            None => {
                let (proj, base) = require_project()?;
                test_runner::run_project_tests(&proj, &base)?;
            }
        },

        Commands::New(args) => {
            new::new_project(&args.name, args.with_std)?;
        }

        Commands::Check(args) => {
            let mut all_ok = true;
            match args.file {
                Some(f) => {
                    if !check::check_and_report(Path::new(&f))? {
                        all_ok = false;
                    }
                }
                None => {
                    let (proj, base) = require_project()?;
                    let files = proj.resolve_include_files(&base)?;
                    if files.is_empty() {
                        println!("No .bf files found in project include paths.");
                    }
                    for f in &files {
                        if !check::check_and_report(f)? {
                            all_ok = false;
                        }
                    }
                }
            }
            if !all_ok {
                process::exit(1);
            }
        }

        Commands::Pack(args) => {
            let file = match args.file {
                Some(f) => std::path::PathBuf::from(f),
                None => {
                    let (proj, base) = require_project()?;
                    proj.entry_path(&base)
                }
            };
            pack::pack_and_output(&file, args.output.as_deref(), args.optimize)?;
        }

        Commands::Init => {
            init::init_project()?;
        }

        Commands::Bench(args) => {
            let tape_size = args.tape_size.unwrap_or(30_000);
            let file = match args.file {
                Some(f) => std::path::PathBuf::from(f),
                None => {
                    let (proj, base) = require_project()?;
                    proj.entry_path(&base)
                }
            };
            bench::bench_and_report(&file, tape_size)?;
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
