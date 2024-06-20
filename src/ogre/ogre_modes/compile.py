from typing import Optional


class Compile:
    def __init__(self, file: str, output: Optional[str]):
        self.file = file
        self.output = output
        self.run()

    def run(self):
        print(self.file)
        print(self.output)
