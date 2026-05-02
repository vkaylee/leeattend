import importlib.util
import runpy
import sys
import typing
from pathlib import Path
from unittest.mock import MagicMock, patch

import psutil

# Add scripts/ai-tools to sys.path
ai_tools_dir = Path(__file__).parent.parent / "ai-tools"
sys.path.append(str(ai_tools_dir))


def load_kill_hung() -> typing.Any:
    path = ai_tools_dir / "kill_hung_processes.py"
    spec = importlib.util.spec_from_file_location("kill_hung_processes_mod", str(path))
    if spec and spec.loader:
        mod = importlib.util.module_from_spec(spec)
        sys.modules["kill_hung_processes_mod"] = mod
        spec.loader.exec_module(mod)
        return mod
    return None  # pragma: no cover


kill_hung_mod = load_kill_hung()


def test_kill_hung_processes() -> None:
    # Mock processes
    mock_proc1 = MagicMock()
    mock_proc1.info = {"name": "python", "cmdline": ["python", "-c", "print(1)"], "pid": 123}

    mock_proc2 = MagicMock()
    mock_proc2.info = {
        "name": "python",
        "cmdline": ["python", "scripts/_safe_run.py", "safe-run"],
        "pid": 456,
    }

    mock_proc3 = MagicMock()
    mock_proc3.info = {"name": "safe", "cmdline": ["ls"], "pid": 789}

    mock_proc4 = MagicMock()
    mock_proc4.info = {"name": "self", "cmdline": ["python", "kill_hung_processes.py"], "pid": 101}

    mock_proc5 = MagicMock()
    mock_proc5.info = {"name": "empty", "cmdline": [], "pid": 102}

    with patch(
        "psutil.process_iter",
        return_value=[mock_proc1, mock_proc2, mock_proc3, mock_proc4, mock_proc5],
    ):
        kill_hung_mod.kill_hung_processes()

    # Check that kill was called for proc1 and proc2
    assert mock_proc1.kill.called
    assert mock_proc2.kill.called
    assert not mock_proc3.kill.called
    assert not mock_proc4.kill.called
    assert not mock_proc5.kill.called


def test_kill_hung_processes_recursive() -> None:
    # Mock parent process
    mock_parent = MagicMock()
    mock_parent.info = {"name": "python", "cmdline": ["python", "-c", "sleep 100"], "pid": 1001}

    # Mock child process
    mock_child = MagicMock()
    mock_child.pid = 1002
    mock_child.name.return_value = "child_proc"

    mock_parent.children.return_value = [mock_child]

    with patch("psutil.process_iter", return_value=[mock_parent]):
        kill_hung_mod.kill_hung_processes()

    # Check that both parent and child were killed
    assert mock_parent.kill.called
    assert mock_child.kill.called


def test_kill_hung_processes_exceptions() -> None:
    mock_proc = MagicMock()
    mock_proc.info = {"name": "python", "cmdline": ["python", "-c", "sleep 100"], "pid": 123}
    mock_proc.kill.side_effect = psutil.NoSuchProcess(123)

    with patch("psutil.process_iter", return_value=[mock_proc]):
        kill_hung_mod.kill_hung_processes()

    assert mock_proc.kill.called


def test_kill_hung_main() -> None:
    path = ai_tools_dir / "kill_hung_processes.py"
    with patch("psutil.process_iter", return_value=[]):
        runpy.run_path(str(path), run_name="__main__")


def test_kill_hung_processes_recursive_exception() -> None:
    mock_parent = MagicMock()
    mock_parent.info = {"name": "python", "cmdline": ["python", "-c", "sleep 100"], "pid": 1001}
    mock_parent.children.side_effect = psutil.AccessDenied(1001)

    with patch("psutil.process_iter", return_value=[mock_parent]):
        kill_hung_mod.kill_hung_processes()

    assert mock_parent.kill.called
