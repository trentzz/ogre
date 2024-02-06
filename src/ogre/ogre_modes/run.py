"""
(ogre run): runs a Brainfuck script.
"""

from ogre.interpreter import Interpreter
from ogre.ogre_modes.ogre_mode import OgreMode


class Run(OgreMode):
    """
    Run a Brainfuck script.
    """

    name = "run"

    def __init__(self, ogre):
        self.ogre = ogre
        self.interpreter = Interpreter()
        self.bf_script = None

    def parse(self, command):
        if len(command) == 0:
            return

        if len(command) > 1:
            self.ogre_error("too many run arguments")
            return

        if command[0] == "exit":
            self.ogre.switch_base()
            return

        with open(command[0], encoding="utf-8") as file:
            self.bf_script = file.read()
            self.run()

    def run(self):
        """
        Run the Brainfuck script.
        """
        print("Running...")

        self.interpreter.add_script(self.bf_script)
        self.interpreter.interpret_script()
