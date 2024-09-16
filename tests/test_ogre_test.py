import pytest
from ogre.ogre_modes.test import Test

def test_ogre_test_basic_test():
    t = Test(test_file="tests/ogre_test_input/basic_test.json")
    t.run()
