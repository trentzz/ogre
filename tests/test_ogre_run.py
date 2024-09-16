import pytest
from ogre.ogre_modes.run import Run

def test_ogre_run_hello_world():
    r = Run(file="tests/brainfuck_scripts/hello_world.bf")
    r.run()
    