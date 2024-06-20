"""
(ogre format): formats a file
"""

from ogre.ogre_modes.ogre_mode import OgreMode


class Format(OgreMode):
    """
    Formatting for brainfuck code.
    """

    name = "format"

    def __init__(
        self,
        file: str,
        indent: int,
        linewidth: int,
        grouping: int,
        label_functions: bool,
        preserve_comments: bool,
    ):
        self.file = file
        self.indent = indent
        self.linewidth = linewidth
        self.grouping = grouping
        self.label_functions = label_functions
        self.preserve_comments = preserve_comments

        self.run()

    def format(self, code: str):
        indent_level = 0
        formatted = []

        for c in code:
            if indent_level * self.indent > self.linewidth - 10:
                self.ogre_error(
                    "too much nesting, please increase the linewidth or decrease indent size"
                )
                return
            match c:
                case "[":
                    formatted.append(" " * (indent_level * self.indent) + c)
                    indent_level += 1
                case "]":
                    indent_level -= 1
                    formatted.append(" " * (indent_level * self.indent) + c)
                case "+" | "-" | ">" | "<" | "." | ",":
                    if (
                        len(formatted) == 0
                        or "]" in formatted[-1]
                        or "[" in formatted[-1]
                        or len(formatted[-1]) >= self.linewidth
                    ):
                        formatted.append(" " * (indent_level * self.indent))

                    formatted[-1] += c
                case "\n":
                    pass
                case _:
                    if self.preserve_comments:
                        formatted[-1] += c

        formatted = [line.rstrip() for line in formatted]
        return "\n".join(formatted)

    def run(self):
        try:
            with open(self.file, "r", encoding="utf-8") as file:
                code = file.read()
        except FileNotFoundError:
            self.ogre_error(f"file '{self.file}' not found.")
            return
        except IOError as e:
            self.ogre_error(f"unable to read file '{self.file}'\n reason: {e}")
            return

        formatted = self.format(code)
        if formatted is None:
            return

        try:
            with open(self.file, "w", encoding="utf-8") as file:
                file.write(formatted)
        except IOError as e:
            self.ogre_error(f"unable to write file '{self.file}'\n reason: {e}")
            return
