import sys
from pathlib import Path

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
    """Kiểm tra lệnh test không có mode hiển thị help."""
    args = orchestrator.parser.parse_args(["test"])
    with pytest.raises(SystemExit) as e:
        orchestrator.handle_test(args)
    assert e.value.code == 2
    captured = capsys.readouterr()
    assert "No test mode specified" in captured.err
    assert "usage:" in captured.err


def test_manage_no_subcommand_raises_error(orchestrator: _orchestrator.Orchestrator) -> None:
    """Kiểm tra lệnh manage không có subcommand báo lỗi argparse."""
    with pytest.raises(SystemExit) as e:
        orchestrator.parser.parse_args(["manage"])
    assert e.value.code != 0
