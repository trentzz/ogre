"""
(ogre debug): enters a debug environment.
"""

from ogre.ogre_modes.ogre_mode import OgreMode
from ogre.interpreter import Interpreter


class Debug(OgreMode):
    """
    Debugging mode for Ogre.
    """

    name = "debug"

    def __init__(self, ogre):
        self.ogre = ogre
        self.interpreter = Interpreter()

    def parse(self, command):
        if len(command) == 0:
            return

        if command[0] == "exit":
            self.ogre.switch_base()
        elif command[0] == "peek":
            self.parse_peek(command[1:])

    def parse_peek(self, command):
        pass

    def parse_step(self, command):
        pass

    def parse_save(self, command):
        pass
