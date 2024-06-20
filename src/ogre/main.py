"""
Ogre. A brainfuck interpreter and debugger.
"""

import argparse

from ogre.ogre_modes.run import Run

# from ogre.ogre_modes.start import Start
from ogre.ogre_modes.compile import Compile

# from ogre.ogre_modes.debug import Debug
from ogre.ogre_modes.format import Format


def ogre_run(args):
    Run(args.file)


def ogre_debug(args):
    print("(ogre debug) WIP")


def ogre_start(args):
    print("(ogre start) WIP")


def ogre_format(args):
    Format(
        file=args.file,
        indent=args.indent,
        linewidth=args.linewidth,
        grouping=args.grouping,
        label_functions=args.label_functions,
        preserve_comments=args.preserve_comments,
    )


def ogre_compile(args):
    Compile(args.file, args.output)


def main():
    parser = argparse.ArgumentParser(description="A brainfu(ck/nct) tool")
    subparsers = parser.add_subparsers(
        title="subcommands", description="valid subcommands", help="subcommand help"
    )

    # run
    parser_run = subparsers.add_parser("run", help="Run a brainfuck file")
    parser_run.add_argument("file", type=str, help="File to run")
    parser_run.set_defaults(func=ogre_run)

    # debug
    parser_debug = subparsers.add_parser(
        "debug", help="Debug brainfuck with a gdb like interface"
    )
    parser_debug.add_argument("file", type=str, help="File to debug")
    parser_debug.set_defaults(func=ogre_debug)

    # start
    parser_start = subparsers.add_parser("start", help="Start a brainfuck interpreter")
    parser_start.set_defaults(func=ogre_start)

    # format
    parser_format = subparsers.add_parser("format", help="Format brainfuck code")
    parser_format.add_argument("file", type=str, help="file to format")
    parser_format.add_argument(
        "--indent", type=int, default=4, help="Indent size in spaces (default: 4)"
    )
    parser_format.add_argument(
        "--linewidth",
        type=int,
        default=80,
        help="Max linewidth in spaces (default: 80)",
    )
    parser_format.add_argument(
        "--grouping",
        type=int,
        default=5,
        help="How to group consecutive operators e.g. +++++ +++++ (default: 5)",
    )
    parser_format.add_argument(
        "--label-functions", action="store_true", help="Label functions with comments"
    )
    parser_format.add_argument(
        "-p",
        "--preserve-comments",
        action="store_true",
        help="Preserve comments (at risk of lower quality formatting)",
    )
    parser_format.set_defaults(func=ogre_format)

    # compile
    parser_compile = subparsers.add_parser("compile", help="Compile brainfuck code")
    parser_compile.add_argument("file", type=str, help="File to compile")
    parser_compile.add_argument("-o", "--output", help="Output file")
    parser_compile.set_defaults(func=ogre_compile)

    # parsing
    args = parser.parse_args()
    if "func" in args:
        args.func(args)
    else:
        parser.print_help()
