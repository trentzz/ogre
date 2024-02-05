"""
Brainfuck Interpreter.
"""


class Interpreter:
    """
    Brainfuck Interpreter.
    """

    def __init__(self, bf_script: str = None):
        self.memory = [0] * 30000  # Initialize 30,000 memory cells
        self.memory_ptr = 0

        self.bf_script = bf_script
        self.bf_script_ptr = 0

    def add_script(self, bf_script: str):
        """Add Brainfuck script."""
        self.bf_script = bf_script
        self.bf_script_ptr = 0

    def interpret_script(self):
        """
        Interpret a full Brainfuck script.
        """
        while self.bf_script_ptr < len(self.bf_script):
            instruction = self.bf_script[self.bf_script_ptr]
            self.interpret_instruction(instruction)

            self.memory_ptr += 1

    def interpret_instruction(self, instruction: str):
        """
        Interpret a single Brainfuck instruction.
        """
        print(instruction)

        if instruction == ">":
            self.memory_ptr += 1
        elif instruction == "<":
            self.memory_ptr -= 1
        elif instruction == "+":
            self.memory[self.memory_ptr] += 1
        elif instruction == "-":
            self.memory[self.memory_ptr] -= 1
        elif instruction == "[":
            if self.memory[self.memory_ptr] == 0:
                loop_count = 1
                while loop_count != 0:
                    self.bf_script_ptr += 1
                    if self.bf_script[self.bf_script_ptr] == "[":
                        loop_count += 1
                    elif self.bf_script[self.bf_script_ptr] == "]":
                        loop_count -= 1
            else:
                pass
        elif instruction == "]":
            loop_count = 1
            while loop_count != 0:
                self.bf_script_ptr -= 1
                if self.bf_script[self.bf_script_ptr] == "]":
                    loop_count += 1
                elif self.bf_script[self.bf_script_ptr] == "[":
                    loop_count -= 1
            self.bf_script_ptr -= 1
        elif instruction == ".":
            print(chr(self.memory[self.memory_ptr]), end="")
        elif instruction == ",":
            self.memory[self.memory_ptr] = input()[0]
