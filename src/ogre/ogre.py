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

    def cli(self):
        while True:
            command = input("self.state.get_input_string()")
            self.state.parse(command)
