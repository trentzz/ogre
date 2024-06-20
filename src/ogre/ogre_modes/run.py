"""
(ogre run): runs a Brainfuck script.
"""

from ogre.interpreter import Interpreter


class Run:
    """
    Run a Brainfuck script.
    """

    name = "run"

    def __init__(self, file):
        self.file = file
        self.run()

    def run(self):
        with open(self.file, "r", encoding="utf-8") as file:
            code = file.read()
            interpreter = Interpreter(code)
            interpreter.run_code()
