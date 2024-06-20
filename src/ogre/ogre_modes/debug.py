"""
(ogre debug): enters a debug environment.
"""

import sys

from ogre.interpreter import Interpreter


class Debug:
    """
    Debugging mode for Ogre.
    """

    name = "debug"

    def __init__(self):
        self.interpreter = Interpreter()
        self.log = ""

    def cli_loop(self):
        while True:
            command = input("(ogre debug) ").split()
            self.parse(command)

    def parse(self, command):
        if len(command) == 0:
            return

        match command[0]:
            case "exit":
                sys.exit()
            case "peek":
                self.parse_peek(command[1:])
            case "step":
                self.parse_step(command[1:])
            case "save":
                self.parse_save(command[1:])
            case _:
                print(f'option "{command[0]}" not recognised')

    def parse_peek(self, command):
        pass

    def parse_step(self, command):
        pass

    def parse_save(self, command):
        pass
