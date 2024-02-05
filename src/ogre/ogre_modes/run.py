"""
(ogre run): runs a Brainfuck script.
"""

from ogre.interpreter import Interpreter
from ogre.ogre_modes.ogre_mode import OgreMode


class Run(OgreMode):
    """
    Run a Brainfuck script.
    """

    def __init__(self, ogre):
        self.ogre = ogre
        self.interpreter = Interpreter()
        self.bf_script = None

    def parse(self, command):
        pass

    def run(self):
        """
        Run the Brainfuck script.
        """
        print("Running...")

        self.interpreter.add_script(self.bf_script)
        self.interpreter.interpret_script()
