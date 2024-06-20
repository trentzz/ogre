"""
Brainfuck Interpreter.
"""

from typing import Optional


class Interpreter:
    """
    Brainfuck Interpreter.
    """

    def __init__(self, code):
        self.data = [0] * 30000  # Initialize 30,000 memory cells
        self.data_ptr = 0
        self.code = code
        self.code_ptr = 0

        self.loop_jumps = [None] * len(code)
        self.precompile_loop_jumps()

    def precompile_loop_jumps(self):
        if self.code is None:
            return

        loop_stack = []
        for i, op in enumerate(self.code):
            match op:
                case "[":
                    loop_stack.append(i)
                case "]":
                    self.loop_jumps[i] = loop_stack.pop()
                    self.loop_jumps[self.loop_jumps[i]] = i

    def run_code(self):
        """
        Interpret a full Brainfuck script.
        """
        while self.code_ptr < len(self.code):
            self.step()

    def current_value(self):
        """
        Returns the current value at the data pointer
        """
        return self.data[self.data_ptr]

    def run_instruction(self, instruction: str) -> Optional[str]:
        """
        Interpret a single Brainfuck instruction.
        """
        # print(f"{instruction} ptr:{self.memory_ptr} val:{self.memory[self.memory_ptr]}")
        match instruction:
            case ">":
                self.data_ptr += 1
            case "<":
                self.data_ptr -= 1
            case "+":
                self.data[self.data_ptr] += 1
            case "-":
                self.data[self.data_ptr] -= 1
            case "[":
                if self.current_value() == 0:
                    self.code_ptr = self.loop_jumps[self.code_ptr]
            case "]":
                if self.data[self.data_ptr] != 0:
                    self.code_ptr = self.loop_jumps[self.code_ptr]
            case ".":
                print(chr(self.data[self.data_ptr]), end="")
            case ",":
                self.data[self.data_ptr] = ord(input()[0])

    def step(self):
        instruction = self.code[self.code_ptr]
        self.run_instruction(instruction)
        self.code_ptr += 1
