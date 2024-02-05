from ogre.ogre_modes.ogre_mode import OgreMode


class Base(OgreMode):
    def __init__(self, ogre):
        self.ogre = ogre

    def parse(self, command):
        pass
