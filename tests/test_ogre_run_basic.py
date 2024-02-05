import pytest
from click.testing import CliRunner
from ogre.main import run


def test_run():
    runner = CliRunner()
    res = runner.invoke(run, ["example.txt"])
    assert res.exit_code == 1
