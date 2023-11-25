"""
Ogre start.
"""
from .interpreter import Interpreter


class Start:
    def __init__(self):
        self.interpreter = Interpreter()

        self.run()

    def run(self):
        pass
