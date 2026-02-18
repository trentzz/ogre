use anyhow::Result;
use clap::{Parser, Subcommand, Args};

mod modes;
use modes::{analyse, compile, debug, format, generate, new, run, start, test_runner};

#[derive(Parser)]
#[command(name = "ogre", about = "A Cargo-like all-in-one brainfuck tool", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interpret and execute a brainfuck file
    Run(RunArgs),
    /// Compile brainfuck to a native binary via C
    Compile(CompileArgs),
    /// Interactive brainfuck interpreter REPL
    Start,
    /// GDB-style interactive debugger
    Debug(DebugArgs),
    /// Format a brainfuck file in-place
    Format(FormatArgs),
    /// Static analysis of a brainfuck script
    Analyse(AnalyseArgs),
    /// Run structured tests from a JSON test file
    Test(TestArgs),
    /// Scaffold a new brainfuck project
    New(NewArgs),
    /// Generate brainfuck code for common patterns
    #[command(subcommand)]
    Generate(GenerateCommands),
}

#[derive(Args)]
struct RunArgs {
    /// Path to the brainfuck file
    file: String,
}

#[derive(Args)]
struct CompileArgs {
    /// Path to the brainfuck file
    file: String,
    /// Output binary name
    #[arg(short = 'o', long)]
    output: Option<String>,
    /// Keep the intermediate .c file
    #[arg(short = 'k', long)]
    keep: bool,
}

#[derive(Args)]
struct DebugArgs {
    /// Path to the brainfuck file
    file: String,
}

#[derive(Args)]
struct FormatArgs {
    /// Path to the brainfuck file
    file: String,
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
}

#[derive(Args)]
struct AnalyseArgs {
    /// Path to the brainfuck file
    file: String,
    /// Embed the analysis as comments in the source file
    #[arg(long)]
    in_place: bool,
    /// Extra detail per section
    #[arg(long)]
    verbose: bool,
}

#[derive(Args)]
struct TestArgs {
    /// Path to the JSON test file
    test_file: String,
}

#[derive(Args)]
struct NewArgs {
    /// Project name
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
    /// Output file (stdout if not specified)
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(Args)]
struct GenerateStringArgs {
    /// The string to print
    string: String,
    /// Output file (stdout if not specified)
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(Args)]
struct GenerateLoopArgs {
    /// Number of loop iterations
    n: usize,
    /// Output file (stdout if not specified)
    #[arg(short = 'o', long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            run::run_file(&args.file)?;
        }
        Commands::Compile(args) => {
            compile::compile(&args.file, args.output.as_deref(), args.keep)?;
        }
        Commands::Start => {
            start::start_repl()?;
        }
        Commands::Debug(args) => {
            debug::debug_file(&args.file)?;
        }
        Commands::Format(args) => {
            let opts = format::FormatOptions {
                indent: args.indent,
                linewidth: args.linewidth,
                grouping: args.grouping,
                label_functions: args.label_functions,
                preserve_comments: args.preserve_comments,
            };
            format::format_file(&args.file, &opts)?;
        }
        Commands::Analyse(args) => {
            analyse::analyse_file(&args.file, args.verbose, args.in_place)?;
        }
        Commands::Test(args) => {
            test_runner::run_tests(&args.test_file)?;
        }
        Commands::New(args) => {
            new::new_project(&args.name)?;
        }
        Commands::Generate(gen) => match gen {
            GenerateCommands::Helloworld(args) => {
                let code = generate::generate_hello_world();
                generate::write_or_print(&code, args.output.as_deref())?;
            }
            GenerateCommands::String(args) => {
                let code = generate::generate_string(&args.string);
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
