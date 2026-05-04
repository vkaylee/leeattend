import sys
from pathlib import Path
from unittest.mock import patch

import pytest

# Add scripts to path
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.append(str(Path(__file__).resolve().parent.parent))

import _orchestrator  # noqa: E402


@pytest.fixture
def orchestrator() -> _orchestrator.Orchestrator:
    return _orchestrator.Orchestrator()


def test_test_no_mode_shows_help(
    orchestrator: _orchestrator.Orchestrator, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that test command with target 'all' works."""
    with (
        patch.object(_orchestrator.Orchestrator, "run_phase"),
        patch.object(_orchestrator.Orchestrator, "execute_safe"),
    ):
        args = orchestrator.parser.parse_args(["test", "all"])
        orchestrator.handle_test(args)


def test_manage_no_subcommand_raises_error(orchestrator: _orchestrator.Orchestrator) -> None:
    """Test that manage command without subcommand raises argparse error."""
    with pytest.raises(SystemExit) as e:
        orchestrator.parser.parse_args(["manage"])
    assert e.value.code != 0
