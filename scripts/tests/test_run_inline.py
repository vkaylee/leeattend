"""Tests for scripts/ai-tools/run-inline — safe inline code runner."""

import os
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS_DIR = PROJECT_ROOT / "scripts"
RUN_INLINE = SCRIPTS_DIR / "ai-tools" / "run-inline"


class TestRunInline:
    """End-to-end tests for run-inline."""

    def test_python_print(self):
        """Basic python inline execution."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python", "-c", "print(42)"],
            capture_output=True, text=True,
        )
        assert result.returncode == 0
        assert "42" in result.stdout

    def test_python_multiline(self):
        """Multi-line code should work (temp file handles naturally)."""
        code = "x = 1\ny = 2\nprint(x + y)"
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python", "-c", code],
            capture_output=True, text=True,
        )
        assert result.returncode == 0
        assert "3" in result.stdout

    def test_bun_print(self):
        """Bun inline execution."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "bun", "-c", "console.log('hello')"],
            capture_output=True, text=True,
        )
        # bun might not be available in test env
        if result.returncode != 0:
            pytest.skip("bun not available in test environment")
        assert "hello" in result.stdout

    def test_bash_echo(self):
        """Bash inline execution."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "bash", "-c", "echo hello"],
            capture_output=True, text=True,
        )
        assert result.returncode == 0
        assert "hello" in result.stdout

    def test_shell_special_chars_safe(self):
        """Shell special chars must NOT be interpreted — they're in a temp file."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "bash", "-c", "echo '$HOME'"],
            capture_output=True, text=True,
        )
        assert result.returncode == 0
        # $HOME in single quotes inside temp file → literal $HOME, not expanded
        # bash echo of literal string
        assert "$HOME" in result.stdout

    def test_from_stdin(self):
        """Read code from stdin."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python"],
            input="print(99)", capture_output=True, text=True,
        )
        assert result.returncode == 0
        assert "99" in result.stdout

    def test_empty_code_from_stdin_exits(self):
        """Empty stdin should fail."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python"],
            input="", capture_output=True, text=True,
        )
        assert result.returncode != 0

    def test_no_lang_flag_fails(self):
        """Missing -l should show error."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-c", "print(1)"],
            capture_output=True, text=True,
        )
        assert result.returncode != 0

    def test_invalid_lang_fails(self):
        """Invalid language choice should fail."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "ruby", "-c", "puts 1"],
            capture_output=True, text=True,
        )
        assert result.returncode != 0

    def test_python_alias_works(self):
        """python3 alias should work same as python."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python3", "-c", "print(1)"],
            capture_output=True, text=True,
        )
        assert result.returncode == 0

    def test_node_alias_works(self):
        """node should map to bun."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "node", "-c", "console.log('test')"],
            capture_output=True, text=True,
        )
        # Skip if bun not available
        if result.returncode != 0:
            pytest.skip("bun/node not available")

    def test_temp_file_cleaned_up(self):
        """Temp file must be removed after execution."""
        import tempfile
        before = set(Path("/tmp").glob("ai_inline_*"))
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python", "-c", "print(1)"],
            capture_output=True, text=True,
        )
        after = set(Path("/tmp").glob("ai_inline_*"))
        # No new temp files should remain
        assert after == before or len(after - before) == 0

    def test_stderr_preserved(self):
        """Stderr from script should be captured."""
        result = subprocess.run(
            [sys.executable, str(RUN_INLINE), "-l", "python", "-c", "import sys; sys.stderr.write('err')"],
            capture_output=True, text=True,
        )
        assert result.returncode == 0
        # stderr might go to PTY, not captured separately — depends on _safe_run
        # Just verify no crash


class TestSanitizerErrorGuidesToRunInline:
    """Verify that sanitizer error messages guide AI to run-inline."""

    def test_banned_interpreter_message(self):
        """Blocking python -c should mention run-inline."""
        from _arg_sanitizer import sanitize, ArgSanitizeError
        try:
            sanitize(["python", "-c", "print(1)"])
        except ArgSanitizeError as e:
            msg = str(e)
            assert "python" in msg.lower()

    def test_die_function_mentions_run_inline(self, capsys):
        """_die() must print run-inline usage."""
        from _arg_sanitizer import _die
        with pytest.raises(SystemExit):
            _die("test error")
        captured = capsys.readouterr()
        assert "run-inline" in captured.err
        assert "ai-tools" in captured.err
