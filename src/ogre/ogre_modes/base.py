import sys
from ogre.ogre_modes.ogre_mode import OgreMode


class Base(OgreMode):
    def __init__(self, ogre):
        self.ogre = ogre

    def parse(self, command):
        if len(command) == 0:
            return

        if command[0] == "run":
            self.ogre.switch_run()
        elif command[0] == "debug":
            self.ogre.switch_debug()
        elif command[0] == "start":
            self.ogre.switch_start()
        elif command[0] == "exit":
            sys.exit()

        if len(command) > 1:
            self.ogre.parse(command[1:])

    def get_input_string(self) -> str:
        return "(ogre) "
