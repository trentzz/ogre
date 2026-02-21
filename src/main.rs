use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use std::path::Path;
use std::process;

mod modes;
mod project;

use modes::{analyse, compile, debug, format, generate, new, run, start, test_runner};
use project::OgreProject;

#[derive(Parser)]
#[command(
    name = "ogre",
    about = "A Cargo-like all-in-one brainfuck tool",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interpret and execute a brainfuck file (or project entry if omitted)
    Run(RunArgs),
    /// Compile brainfuck to a native binary via C
    Compile(CompileArgs),
    /// Build the current project (requires ogre.toml)
    Build(BuildArgs),
    /// Interactive brainfuck interpreter REPL
    Start,
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
}

// ---- Per-subcommand arg structs ----

#[derive(Args)]
struct RunArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
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
struct DebugArgs {
    /// Path to the brainfuck file (uses project entry if omitted)
    file: Option<String>,
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

    match cli.command {
        Commands::Run(args) => match args.file {
            Some(f) => run::run_file(Path::new(&f))?,
            None => {
                let (proj, base) = require_project()?;
                let entry = proj.entry_path(&base);
                run::run_file(&entry)?;
            }
        },

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

        Commands::Start => {
            start::start_repl()?;
        }

        Commands::Debug(args) => match args.file {
            Some(f) => debug::debug_file(Path::new(&f))?,
            None => {
                let (proj, base) = require_project()?;
                let entry = proj.entry_path(&base);
                debug::debug_file(&entry)?;
            }
        },

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
            new::new_project(&args.name)?;
        }

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
