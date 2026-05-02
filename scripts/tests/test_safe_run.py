import os
import subprocess
import sys
import threading
import time
from pathlib import Path
from unittest.mock import MagicMock, patch

import psutil
import pytest

# Add scripts to path so we can import _safe_run
sys.path.append(str(Path(__file__).parent.parent))
import _safe_run


def test_kill_process_tree() -> None:
    """Test killing a process tree."""
    # Start a dummy process that stays alive
    proc = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(10)"])
    pid = proc.pid
    assert psutil.pid_exists(pid)

    _safe_run.kill_process_tree(pid)

    # Wait a bit for it to be killed
    time.sleep(0.5)
    assert not psutil.pid_exists(pid)


def test_read_from_pty_eof() -> None:
    """Test reading from PTY on EOF."""
    master, slave = os.openpty()
    stop_event = threading.Event()

    # Write something to master (from slave side)
    os.write(slave, b"hello")
    os.close(slave)

    # This should read "hello" and then exit because slave is closed
    _safe_run.read_from_pty(master, stop_event)
    os.close(master)


def test_read_from_pty_stop_event() -> None:
    """Test reading from PTY when stop_event is set."""
    master, slave = os.openpty()
    stop_event = threading.Event()
    stop_event.set()

    # Should exit immediately
    _safe_run.read_from_pty(master, stop_event)

    os.close(slave)
    os.close(master)


def test_main_not_enough_args() -> None:
    """Test main with insufficient arguments."""
    with patch.object(sys, "argv", ["safe-run.py"]), pytest.raises(SystemExit) as exc:
        _safe_run.main()
    assert exc.value.code == 1


def test_main_invalid_timeout() -> None:
    """Test main with invalid timeout value."""
    with patch.object(sys, "argv", ["safe-run.py", "abc", "ls"]), pytest.raises(SystemExit) as exc:
        _safe_run.main()
    assert exc.value.code == 1


@patch("subprocess.Popen")
def test_main_command_not_found(mock_popen: MagicMock) -> None:
    """Test main when command is not found."""
    mock_popen.side_effect = FileNotFoundError()
    with (
        patch.object(sys, "argv", ["safe-run.py", "5", "nonexistent-cmd"]),
        pytest.raises(SystemExit) as exc,
    ):
        _safe_run.main()
    assert exc.value.code == 127


def test_main_success() -> None:
    """Test main executing command successfully."""
    with (
        patch.object(sys, "argv", ["safe-run.py", "2", "echo", "hello"]),
        patch("sys.stdout.buffer.write"),  # Avoid printing to real stdout
        pytest.raises(SystemExit) as exc,
    ):
        _safe_run.main()
    assert exc.value.code == 0


def test_main_timeout() -> None:
    """Test main when command times out."""
    with (
        patch.object(sys, "argv", ["safe-run.py", "0.1", "sleep", "1"]),
        pytest.raises(SystemExit) as exc,
    ):
        _safe_run.main()
    assert exc.value.code == 124


def test_validate_command_python_c(capsys: pytest.CaptureFixture[str]) -> None:
    """Test validate_command blocks python -c."""
    with pytest.raises(SystemExit) as exc:
        _safe_run.validate_command(["python3", "-c", "print(1)"])
    assert exc.value.code == 1
    captured = capsys.readouterr()
    assert "PTY SECURITY ERROR" in captured.err


def test_validate_command_valid() -> None:
    """Test validate_command allows valid commands."""
    _safe_run.validate_command(["echo", "hello"])  # Should not raise
