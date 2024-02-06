"""
Ogre. A brainfuck interpreter and debugger.
"""

import sys

from ogre.ogre import Ogre


def main():
    ogre = Ogre()
    ogre.cli_call(sys.argv[1:])
