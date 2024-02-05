"""
(ogre debug): enters a debug environment.
"""

from ogre.ogre_modes.ogre_mode import OgreMode
from ogre.interpreter import Interpreter


class Debug(OgreMode):
    """
    Debugging mode for Ogre.
    """

    def __init__(self, ogre):
        self.ogre = ogre
        self.interpreter = Interpreter()
