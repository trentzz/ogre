class Interpreter:
    def __init__(self):
        self.memory = [0] * 30000  # Initialize 30,000 memory cells
        self.pointer = 0
        self.code = ""
        self.code_ptr = 0

    def interpret_all(self, code):
        self.code = code
        self.code_ptr = 0

        while self.code_ptr < len(self.code):
            command = self.code[self.code_ptr]

            if command == ">":
                self.pointer += 1
            elif command == "<":
                self.pointer -= 1
            elif command == "+":
                self.memory[self.pointer] += 1
            elif command == "-":
                self.memory[self.pointer] -= 1
            elif command == "[":
                if self.memory[self.pointer] == 0:
                    loop_count = 1
                    while loop_count != 0:
                        self.code_ptr += 1
                        if self.code[self.code_ptr] == "[":
                            loop_count += 1
                        elif self.code[self.code_ptr] == "]":
                            loop_count -= 1
                else:
                    pass
            elif command == "]":
                loop_count = 1
                while loop_count != 0:
                    self.code_ptr -= 1
                    if self.code[self.code_ptr] == "]":
                        loop_count += 1
                    elif self.code[self.code_ptr] == "[":
                        loop_count -= 1
                self.code_ptr -= 1
            elif command == ".":
                print(chr(self.memory[self.pointer]), end="")
            elif command == ",":
                self.memory[self.pointer] = ord(input()[0])
            self.code_ptr += 1

    def interpret_line(self, line, pointer):
        pass
