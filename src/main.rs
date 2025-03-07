use clap::{Arg, ArgAction, Command};

fn cli_commands() {
    let matches = Command::new("ogre")
        .about("A brainfu(ck/nct) tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Run a brainfuck file")
                .arg(Arg::new("file").help("File to run").required(true).index(1)),
        )
        .subcommand(
            Command::new("debug")
                .about("Debug brainfuck with a gdb like interface")
                .arg(
                    Arg::new("file")
                        .help("File to debug")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("start").about("Start a brainfuck interpreter"))
        .subcommand(
            Command::new("format")
                .about("Format brainfuck code")
                .arg(
                    Arg::new("file")
                        .help("File to format")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("indent")
                        .long("indent")
                        .help("Indent size in spaces (default: 4)")
                        .default_value("4")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("linewidth")
                        .long("linewidth")
                        .help("Max linewidth in spaces (default: 80)")
                        .default_value("80")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("grouping")
                        .long("grouping")
                        .help("How to group consecutive operators e.g. +++++ +++++ (default: 5)")
                        .default_value("5")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("label-functions")
                        .long("label-functions")
                        .help("Label functions with comments")
                        .takes_value(false),
                )
                .arg(
                    Arg::new("preserve-comments")
                        .long("preserve-comments")
                        .short('p')
                        .help("Preserve comments (at risk of lower quality formatting)")
                        .takes_value(false),
                ),
        )
        .subcommand(
            Command::new("compile")
                .about("Compile brainfuck code")
                .arg(
                    Arg::new("file")
                        .help("File to compile")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output file")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("keep")
                        .long("keep")
                        .short('k')
                        .help("Keep generated c file")
                        .action(ArgAction::SetTrue),
                ),
        )
        .get_matches();
}

fn main() {
    cli_commands();
}
