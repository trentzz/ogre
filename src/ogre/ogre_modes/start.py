"""
WIP (ogre start): enters a real time interpreter environment.
"""

from ogre.interpreter import Interpreter
from ogre.ogre_modes.ogre_mode import OgreMode


class Start(OgreMode):
    def __init__(self, ogre):
        self.ogre = ogre
        self.interpreter = Interpreter("")

        self.run()

    def run(self):
        pass

    def parse(self, command):
        pass
