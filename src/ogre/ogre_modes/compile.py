from typing import Optional
from textwrap import dedent
import subprocess
import os
import sys
from ogre.interpreter import Interpreter
from ogre.ogre_modes.ogre_mode import OgreMode


class CCodeGenerator(Interpreter):
    """
    Generates c code.
    """

    def __init__(self, code: str):
        super().__init__(code)
        self.generated_c_code = ""

    def add_generated_line(self, gen: str):
        self.generated_c_code += f"    {gen}"

    def generate(self, generated_c_file: str):
        self.generated_c_code = dedent(
            """
        #include <stdio.h>
    
        int main() {
            char array[30000] = {{0}};
            char *ptr = array;
            
            """
        )

        for instruction in self.code:
            match instruction:
                case ">":
                    self.add_generated_line("++ptr;\n")
                case "<":
                    self.add_generated_line("--ptr;\n")
                case "+":
                    self.add_generated_line("++*ptr;\n")
                case "-":
                    self.add_generated_line("--*ptr;\n")
                case ".":
                    self.add_generated_line("putchar(*ptr);\n")
                case ",":
                    self.add_generated_line("*ptr = getchar();\n")
                case "[":
                    self.add_generated_line("while (*ptr) {\n")
                case "]":
                    self.add_generated_line("}\n")

        self.generated_c_code += dedent(
            """
            return 0;
        }
            """
        )
        with open(generated_c_file, "w", encoding="utf-8") as file:
            file.write(self.generated_c_code)


class Compile(OgreMode):
    """
    Compiles brainfuck code by first generating c code, and compiling that
    with gcc.
    """

    def __init__(self, file: str, output: Optional[str], keep: bool = False):
        self.file = file

        self.output = output
        if self.output is None:
            self.output = self.file.replace(".bf", "")

        self.keep = keep
        self.generated_c_file = f"{self.output}.c"
        self.run()

    def run(self):

        self.generate_c()
        self.compile_c()

    def generate_c(self):
        with open(self.file, "r", encoding="utf-8") as file:
            code_generator = CCodeGenerator(file.read())
            if code_generator.validate():
                code_generator.generate(self.generated_c_file)
            else:
                self.ogre_error("invalid brainfuck script.\n" + code_generator.errors)
                sys.exit()

    def compile_c(self):
        captured_output = subprocess.run(
            ["gcc", self.generated_c_file, "-o", self.output],
            check=True,
            capture_output=True,
        )
        captured_output.check_returncode()

        if not self.keep:
            os.remove(self.generated_c_file)
