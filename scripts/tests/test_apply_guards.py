import importlib.util
import runpy
import sys
import typing
from pathlib import Path
from unittest.mock import MagicMock, patch

# Add scripts/ai-tools to sys.path
ai_tools_dir = Path(__file__).parent.parent / "ai-tools"
sys.path.append(str(ai_tools_dir))


# Load apply-guards as a module
def load_apply_guards() -> typing.Any:
    path = ai_tools_dir / "apply-guards.py"
    spec = importlib.util.spec_from_file_location("apply_guards_mod", str(path))
    if spec and spec.loader:
        mod = importlib.util.module_from_spec(spec)
        sys.modules["apply_guards_mod"] = mod
        spec.loader.exec_module(mod)
        return mod
    return None  # pragma: no cover


apply_guards_mod = load_apply_guards()


def test_apply_guards_with_shebang() -> None:
    mock_script = MagicMock(spec=Path)
    mock_script.name = "_test.sh"
    mock_script.read_text.return_value = "#!/bin/bash\necho hello"

    with patch("pathlib.Path.glob", return_value=[mock_script]):
        apply_guards_mod.apply_guards()

    # Check that write_text was called
    args, kwargs = mock_script.write_text.call_args
    content = args[0]
    assert content.startswith("#!/bin/bash\n\n# Guard:")
    assert "BASH_SOURCE" in content


def test_apply_guards_without_shebang() -> None:
    mock_script = MagicMock(spec=Path)
    mock_script.name = "_test.sh"
    mock_script.read_text.return_value = "echo hello"

    with patch("pathlib.Path.glob", return_value=[mock_script]):
        apply_guards_mod.apply_guards()

    # Check that write_text was called
    args, kwargs = mock_script.write_text.call_args
    content = args[0]
    assert content.startswith("# Guard:")
    assert "BASH_SOURCE" in content


def test_apply_guards_already_has_guard() -> None:
    mock_script = MagicMock(spec=Path)
    mock_script.name = "_test.sh"
    mock_script.read_text.return_value = 'if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then\nexit 1\nfi'

    with patch("pathlib.Path.glob", return_value=[mock_script]):
        apply_guards_mod.apply_guards()

    # Check that write_text was NOT called
    mock_script.write_text.assert_not_called()


def test_apply_guards_excluded() -> None:
    mock_script = MagicMock(spec=Path)
    mock_script.name = "_cargo.sh"

    with (
        patch("pathlib.Path.glob", return_value=[Path("scripts/_cargo.sh")]),
        patch.object(Path, "read_text") as mock_read,
    ):
        apply_guards_mod.apply_guards()
        mock_read.assert_not_called()


def test_apply_guards_main() -> None:
    path = ai_tools_dir / "apply-guards.py"
    # Mock Path.glob to return empty so apply_guards does nothing
    with patch("pathlib.Path.glob", return_value=[]):
        # runpy.run_path will hit the if __name__ == "__main__" block
        runpy.run_path(str(path), run_name="__main__")
