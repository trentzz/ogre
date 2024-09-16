"""
WIP (ogre test): test bf files
"""

import json
import sys
from ogre.interpreter import Interpreter
from ogre.ogre_modes.ogre_mode import OgreMode


class Test(OgreMode):
    __test__ = False

    def __init__(self, test_file: str):
        self.interpreter = Interpreter()
        self.test_file = test_file
        self.test_data = self.load_test_data()

        self.validate_test_json()

    def load_test_data(self):
        try:
            with open(self.test_file, "r", encoding="utf-8") as file:
                data = json.load(file)
                if not data:
                    self.ogre_error(f"test file {self.test_file} is empty or invalid.")
                    sys.exit(1)
                return data
        except FileNotFoundError:
            self.ogre_error(f"test file {self.test_file} not found.")
            sys.exit(1)
        except json.JSONDecodeError:
            self.ogre_error(f"test file {self.test_file} contains invalid JSON.")
            sys.exit(1)

    def validate_test_json(self):
        required_keys = {"name", "brainfuck", "input", "output"}
        invalid_tests = []
        for i, obj in enumerate(self.test_data):
            missing_keys = required_keys - obj.keys()
            if missing_keys:
                invalid_tests.append((i, missing_keys))

        if invalid_tests:
            self.ogre_error(f"error validating test file {self.test_file}")
            for i, missing_keys in invalid_tests:
                self.ogre_error(f"error at test {i} due to missing json keys {missing_keys}")
            sys.exit(1)

    def run(self):
        pass
