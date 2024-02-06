"""
(ogre): starting point for the ogre tool.
"""

from ogre.ogre_modes.debug import Debug
from ogre.ogre_modes.base import Base
from ogre.ogre_modes.run import Run
from ogre.ogre_modes.start import Start


class Ogre:
    def __init__(self):
        self.base = Base(self)
        self.debug = Debug(self)
        self.run = Run(self)
        self.start = Start(self)

        self.state = self.base

    def cli_loop(self):
        while True:
            command = input(self.state.get_input_string())
            command = command.split()
            self.parse(command)

    def cli_call(self, command: list):
        # print(f"(cli call) {command}")
        if len(command) > 0:
            self.parse(command)

        self.cli_loop()

    def parse(self, command):
        # print(f"(parse) {command}")
        self.state.parse(command)

    def switch_base(self):
        self.state = self.base

    def switch_debug(self):
        self.state = self.debug

    def switch_run(self):
        self.state = self.run

    def switch_start(self):
        self.state = self.start
