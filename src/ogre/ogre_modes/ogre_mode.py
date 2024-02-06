class OgreMode:
    name = ""

    def parse(self, command):
        pass

    def get_input_string(self) -> str:
        return f"(ogre {self.name}) "

    def ogre_error(self, message):
        print(f"(ogre {self.name}) error: {message}")
