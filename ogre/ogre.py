"""
Ogre. A brainfuck interpreter and debugger.
"""

import click
from .start import Start


@click.group()
def cli():
    pass


@cli.command()
def start():
    Start()
