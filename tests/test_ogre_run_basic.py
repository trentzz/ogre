import pytest
from click.testing import CliRunner
from ogre.main import ogre_run


def test_run():
    runner = CliRunner()
    res = runner.invoke(ogre_run, ["example.txt"])
    assert res.exit_code == 1
