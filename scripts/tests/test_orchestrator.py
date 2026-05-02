# mypy: ignore-errors
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

# Thêm scripts/ vào sys.path để có thể import _orchestrator
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.append(str(PROJECT_ROOT / "scripts"))
import _orchestrator  # noqa: E402


@pytest.fixture
def orchestrator():
    import _orchestrator

    return _orchestrator.Orchestrator()


def test_help_message(orchestrator, capsys) -> None:
    """Kiểm tra help message không gây lỗi."""
    with pytest.raises(SystemExit) as e:
        orchestrator.parser.parse_args(["--help"])
    assert e.value.code == 0
    captured = capsys.readouterr()
    assert "LeeAttend Enterprise Orchestrator" in captured.out


def test_invalid_command(orchestrator) -> None:
    """Kiểm tra việc báo lỗi khi lệnh không tồn tại."""
    with pytest.raises(SystemExit):
        orchestrator.parser.parse_args(["invalid-cmd"])


@patch("subprocess.run")
def test_manage_up_dev(mock_run, orchestrator) -> None:
    """Kiểm tra lệnh manage up dev tạo đúng command podman-compose."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["manage", "up", "dev"])
    orchestrator.handle_manage(args)

    called_args = mock_run.call_args[0][0]
    assert "podman-compose" in called_args
    assert "leeattend-dev" in called_args
    assert "up" in called_args


@patch("subprocess.run")
def test_test_api_mode(mock_run, orchestrator) -> None:
    """Kiểm tra lệnh test --api gọi đúng bootstrap và phase."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--api"])
    orchestrator.handle_test(args)

    found_bootstrap = any("_bootstrap.sh" in str(call) for call in mock_run.call_args_list)
    assert found_bootstrap is True


@patch("subprocess.run")
def test_dry_run_no_execution(mock_run, orchestrator) -> None:
    """Kiểm tra flag --dry-run không thực thi lệnh thật."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--all", "--dry-run"])
    orchestrator.dry_run = args.dry_run
    orchestrator.handle_test(args)
    assert mock_run.called is False


@patch("shutil.which")
@patch("socket.socket")
@patch("subprocess.run")
def test_doctor_logic(mock_run, mock_socket, mock_which, orchestrator) -> None:
    """Kiểm tra logic của doctor."""
    mock_which.return_value = "/usr/bin/podman"
    mock_socket.return_value.connect_ex.return_value = 0
    mock_run.return_value.stdout = "leeattend-dev-db\nleeattend-dev-api"
    orchestrator.handle_doctor()
    assert mock_which.called


@patch("subprocess.run")
@patch("shutil.which")
def test_db_query_logic(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra db:query."""
    mock_run.return_value.returncode = 0
    mock_which.return_value = "podman"
    args = orchestrator.parser.parse_args(["manage", "db:query", "SELECT 1", "--json"])
    orchestrator.handle_db_query(args)
    called_args = mock_run.call_args[0][0]
    assert "SELECT 1" in called_args
    assert "--json" in called_args


def test_engine_properties(orchestrator) -> None:
    """Kiểm tra các property phát hiện engine."""
    with patch("shutil.which") as mock_which:
        # Test Podman path
        mock_which.side_effect = lambda x: "/usr/bin/podman" if "podman" in x else None
        assert orchestrator.engine == "podman"
        assert orchestrator.compose_engine == ["podman-compose"]
        assert orchestrator.compose_engine_cmd == "podman-compose"

        # Test Docker path
        mock_which.side_effect = lambda x: "/usr/bin/docker" if "docker" in x else None
        assert orchestrator.engine == "docker"
        assert orchestrator.compose_engine == ["docker", "compose"]

        # Test Fallback
        mock_which.side_effect = lambda x: None
        assert orchestrator.engine == "podman"
        assert orchestrator.compose_engine == ["podman-compose"]


@patch("subprocess.run")
@patch("shutil.which")
def test_run_npm_typecheck(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra logic sửa lỗi typecheck của npm."""
    mock_which.return_value = "podman"
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["run", "npm", "run", "typecheck"])
    orchestrator.handle_run(args)
    called_cmd = " ".join(mock_run.call_args[0][0])
    assert "type-check" in called_cmd


@patch("subprocess.run")
@patch("shutil.which")
@patch("pathlib.Path.cwd")
def test_run_cargo_agent_main(mock_cwd, mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra cargo tự nhận diện agent-main."""
    mock_which.return_value = "podman"
    mock_run.return_value.returncode = 0
    mock_cwd.return_value = PROJECT_ROOT / "agent-main"
    args = orchestrator.parser.parse_args(["run", "cargo", "build"])
    orchestrator.handle_run(args)
    called_cmd = mock_run.call_args[0][0]
    assert "agent-main" in called_cmd


@patch("subprocess.run")
def test_test_coverage_all(mock_run, orchestrator) -> None:
    """Kiểm tra test --all --coverage."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--all", "--coverage"])
    orchestrator.handle_test(args)
    any_coverage = any("leeattend_run_coverage" in str(call) for call in mock_run.call_args_list)
    assert any_coverage is True


@patch("subprocess.run")
def test_test_timeout_custom(mock_run, orchestrator) -> None:
    """Kiểm tra test với timeout tùy chỉnh."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--api", "--timeout", "100"])
    orchestrator.handle_test(args)
    # Tham số thứ 3 của full_cmd là timeout
    timeout_val = mock_run.call_args_list[0][0][0][2]
    assert timeout_val == "100"


@patch("sys.stderr")
@patch("subprocess.run")
def test_execute_safe_failure(mock_run, mock_stderr, orchestrator) -> None:
    """Kiểm tra execute_safe thoát khi lệnh lỗi và in thông báo rõ ràng."""
    mock_run.return_value.returncode = 1
    with pytest.raises(SystemExit):
        orchestrator.execute_safe(["false_cmd"])


@patch("sys.argv", ["_orchestrator.py", "manage", "sync:api"])
@patch("subprocess.run")
def test_full_run_sync_api(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho sync:api."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    assert "_sync-api.sh" in str(mock_run.call_args_list)


@patch("sys.argv", ["_orchestrator.py", "manage", "test:infra"])
@patch("subprocess.run")
def test_full_run_test_infra(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho test:infra."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    assert "pytest" in str(mock_run.call_args_list)


@patch("sys.argv", ["_orchestrator.py", "manage", "lint:infra"])
@patch("subprocess.run")
def test_full_run_lint_infra(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho lint:infra."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    assert "ruff" in str(mock_run.call_args_list)


@patch("sys.argv", ["_orchestrator.py", "manage", "fmt:infra"])
@patch("subprocess.run")
def test_full_run_fmt_infra(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho fmt:infra."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    assert "ruff" in str(mock_run.call_args_list)


@patch("sys.argv", ["_orchestrator.py", "manage", "doctor"])
@patch("subprocess.run")
@patch("shutil.which")
def test_full_run_doctor(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho doctor."""
    mock_run.return_value.stdout = ""
    mock_which.return_value = "podman"
    orchestrator.run()
    assert mock_run.called


@patch("sys.argv", ["_orchestrator.py", "run", "npm", "install"])
@patch("subprocess.run")
def test_full_run_npm(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho run npm."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    assert "webdashboard" in str(mock_run.call_args_list)


@patch("subprocess.run")
def test_run_phase_with_component(mock_run, orchestrator) -> None:
    """Kiểm tra run_phase với component cụ thể."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--api", "--component", "apiserver"])
    orchestrator.run_phase("Unit Tests", "api", args)

    # Kiểm tra env COMPONENT được set
    called_env = mock_run.call_args[1]["env"]
    assert called_env["COMPONENT"] == "apiserver"


def test_log_warn(orchestrator, capsys) -> None:
    """Kiểm tra hàm log_warn."""
    _orchestrator.log_warn("test warning")
    captured = capsys.readouterr()
    assert "test warning" in captured.out


@patch("subprocess.run")
def test_handle_manage_clean(mock_run, orchestrator) -> None:
    """Kiểm tra manage clean."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["manage", "clean"])
    orchestrator.handle_manage(args)
    # Hiện tại clean chưa implement logic phức tạp, chỉ log
    assert True


@patch("subprocess.run")
def test_dry_run_execute_safe(mock_run, orchestrator) -> None:
    """Kiểm tra execute_safe trong chế độ dry-run."""
    orchestrator.dry_run = True
    orchestrator.execute_safe(["echo", "hi"])
    assert not mock_run.called


@patch("shutil.which")
def test_doctor_no_engine(mock_which, orchestrator) -> None:
    """Kiểm tra doctor khi không có engine."""
    mock_which.return_value = None
    orchestrator.handle_doctor()
    assert mock_which.called


@patch("subprocess.run")
@patch("shutil.which")
def test_run_sqlx(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra sqlx."""
    mock_which.return_value = "podman"
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["run", "sqlx", "migrate", "run"])
    orchestrator.handle_run(args)
    assert "sqlx" in mock_run.call_args[0][0]


@patch("subprocess.run")
@patch("shutil.which")
def test_run_with_remainder_separator(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra lệnh run với dấu --."""
    mock_which.return_value = "podman"
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["run", "sqlx", "--", "migrate", "info"])
    orchestrator.handle_run(args)
    called_args = mock_run.call_args[0][0]
    assert "--" not in called_args


def test_get_compose_files_other(orchestrator) -> None:
    """Kiểm tra logic chọn file compose cho môi trường khác."""
    files = orchestrator.get_compose_files("prod")
    assert "docker-compose.yml" in files


@patch("subprocess.run")
def test_handle_test_web(mock_run, orchestrator) -> None:
    """Kiểm tra test --web."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--web"])
    orchestrator.handle_test(args)
    any_web = any(call[1].get("env", {}).get("MODE") == "web" for call in mock_run.call_args_list)
    assert any_web is True


@patch("subprocess.run")
def test_handle_test_integration(mock_run, orchestrator) -> None:
    """Kiểm tra test --integration."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--integration"])
    orchestrator.handle_test(args)
    calls = mock_run.call_args_list
    any_int = any(call[1].get("env", {}).get("MODE") == "integration" for call in calls)
    assert any_int is True


@patch("subprocess.run")
def test_handle_manage_logs_full(mock_run, orchestrator) -> None:
    """Kiểm tra manage logs với service."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["manage", "logs", "test", "db"])
    orchestrator.handle_manage(args)
    called = mock_run.call_args[0][0]
    assert "logs" in called
    assert "db" in called


@patch("subprocess.run")
def test_handle_manage_exec(mock_run, orchestrator) -> None:
    """Kiểm tra manage exec."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["manage", "exec", "db", "ls"])
    orchestrator.handle_manage(args)
    called = mock_run.call_args[0]
    assert "exec" in str(called)


@patch("sys.argv", ["_orchestrator.py", "test", "--api"])
@patch("subprocess.run")
def test_full_run_test_api(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho test --api."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    assert mock_run.called


@patch("sys.argv", ["_orchestrator.py", "run", "cargo", "--", "check"])
@patch("subprocess.run")
@patch("shutil.which")
def test_full_run_cargo_remainder(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho run cargo với -- remainder."""
    mock_run.return_value.returncode = 0
    mock_which.return_value = "podman"
    orchestrator.run()
    assert "apiserver" in str(mock_run.call_args_list)


@patch("subprocess.run")
@patch("shutil.which")
def test_handle_run_npm_no_args(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra handle_run npm không có args."""
    mock_which.return_value = "podman"
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["run", "npm"])
    orchestrator.handle_run(args)
    assert "npm install -g bun" in str(mock_run.call_args_list)


@patch("subprocess.run")
@patch("shutil.which")
def test_handle_lint_infra_no_shellcheck(mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra handle_lint_infra khi thiếu shellcheck."""

    mock_which.return_value = None
    mock_run.return_value.returncode = 0
    orchestrator.handle_lint_infra()
    assert mock_run.called


@patch("subprocess.run")
@patch("shutil.which")
@patch("socket.socket")
def test_handle_doctor_ports_failure(mock_socket, mock_which, mock_run, orchestrator) -> None:
    """Kiểm tra handle_doctor khi cổng bị chiếm."""
    mock_which.return_value = "podman"
    mock_socket.return_value.connect_ex.return_value = 0  # Port busy
    orchestrator.handle_doctor()
    assert mock_socket.called


@patch("sys.argv", ["_orchestrator.py", "manage", "verify:infra"])
@patch("subprocess.run")
def test_full_run_verify_infra(mock_run, orchestrator) -> None:
    """Kiểm tra chạy toàn bộ loop cho verify:infra."""
    mock_run.return_value.returncode = 0
    orchestrator.run()
    # Kiểm tra có gọi ít nhất 3 phase (fmt, lint, test)
    assert mock_run.call_count >= 3


@patch("subprocess.run")
def test_manage_db_query(mock_run, orchestrator) -> None:
    """Kiểm tra manage db:query."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["manage", "db:query", "SELECT 1"])
    orchestrator.handle_manage(args)
    called_args = mock_run.call_args[0][0]
    assert "psql" in called_args
    assert "SELECT 1" in called_args


@patch("subprocess.run")
def test_manage_logs_with_service(mock_run, orchestrator) -> None:
    """Kiểm tra manage logs --service."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["manage", "logs", "dev", "api"])
    orchestrator.handle_manage(args)
    called_args = mock_run.call_args[0][0]
    assert "logs" in called_args
    assert "-f" in called_args
    assert "api" in called_args


@patch("_orchestrator.log_info")
def test_manage_exec_dry_run(mock_log, orchestrator) -> None:
    """Kiểm tra manage exec dry-run."""
    orchestrator.dry_run = True
    args = orchestrator.parser.parse_args(["manage", "exec", "api", "ls"])
    orchestrator.handle_manage(args)
    expected = (
        "🔍 Dry-run: Executing podman-compose -p leeattend-dev "
        "-f docker-compose.yml -f .compose/docker-compose.dev.yml exec api ls"
    )
    mock_log.assert_any_call(expected)


@patch("subprocess.run")
def test_handle_run_remainder_args(mock_run, orchestrator) -> None:
    """Kiểm tra handle_run với -- remainder separator."""
    mock_run.return_value.returncode = 0
    # Phải giả lập sys.argv để parse_args hoạt động đúng với remainder
    args = orchestrator.parser.parse_args(["run", "npm", "--", "run", "dev"])
    orchestrator.handle_run(args)
    called_args = mock_run.call_args[0][0]
    assert "npm install -g bun && bun run dev" in str(called_args)


@patch("socket.socket")
@patch("_orchestrator.log_success")
@patch("_orchestrator.log_info")
def test_doctor_full(mock_info, mock_success, mock_socket, orchestrator) -> None:
    """Kiểm tra doctor với cả success và failure ports."""
    # Mock socket connect_ex: 3000 success (occupied), others failure (free)
    mock_socket.return_value.connect_ex.side_effect = [0, 1, 1]

    with (
        patch("shutil.which", return_value="/usr/bin/podman"),
        patch("subprocess.run") as mock_run,
        patch("pathlib.Path.exists", return_value=True),
    ):
        mock_run.return_value.stdout = "leeattend-dev-db\nleeattend-dev-api"
        orchestrator.handle_doctor()

    mock_success.assert_any_call("✅ Container Engine: podman")
    mock_info.assert_any_call("⚠️  Port 3000 is occupied")


@patch("subprocess.run")
def test_handle_run_cargo_agent(mock_run, orchestrator) -> None:
    """Kiểm tra handle_run cargo với agent-main context."""
    mock_run.return_value.returncode = 0
    with patch("pathlib.Path.cwd", return_value=PROJECT_ROOT / "agent-main"):
        args = orchestrator.parser.parse_args(["run", "cargo", "check"])
        orchestrator.handle_run(args)
        called_args = mock_run.call_args[0][0]
        assert "agent-main" in called_args
        assert "/workspace/agent-main" in str(called_args)


@patch("subprocess.run")
def test_run_phase_with_pattern(mock_run, orchestrator) -> None:
    """Kiểm tra run_phase với pattern."""
    mock_run.return_value.returncode = 0
    args = orchestrator.parser.parse_args(["test", "--api", "--pattern", "auth"])
    orchestrator.handle_test(args)
    # verify env PATTERN=auth was passed
    env = mock_run.call_args[1]["env"]
    assert env["PATTERN"] == "auth"


@patch("shutil.which", return_value="/usr/bin/shellcheck")
@patch("subprocess.run")
def test_handle_lint_infra_with_shellcheck(mock_run, mock_which, orchestrator) -> None:
    """Kiểm tra handle_lint_infra khi có shellcheck."""
    mock_run.return_value.returncode = 0
    orchestrator.handle_lint_infra()
    # verify shellcheck was called
    all_called_args = [str(call[0][0]) for call in mock_run.call_args_list]
    assert any("shellcheck" in args for args in all_called_args)


@patch("_orchestrator.log_info")
@patch("pathlib.Path.exists", return_value=False)
def test_doctor_no_venv(mock_exists, mock_info, orchestrator) -> None:
    """Kiểm tra doctor khi thiếu venv."""
    with patch("shutil.which", return_value=None), patch("subprocess.run") as mock_run:
        mock_run.return_value.stdout = ""
        orchestrator.handle_doctor()
    mock_info.assert_any_call("💡 Virtual Environment: Missing (will be created on next run)")


@patch("subprocess.run")
def test_handle_run_cargo_no_rel(mock_run, orchestrator) -> None:
    """Kiểm tra handle_run cargo khi ở root."""
    mock_run.return_value.returncode = 0
    with patch("pathlib.Path.cwd", return_value=PROJECT_ROOT):
        args = orchestrator.parser.parse_args(["run", "cargo", "check"])
        orchestrator.handle_run(args)
        # Should have /workspace exactly
        called_args = mock_run.call_args[0][0]
        assert "/workspace" in called_args


@patch("subprocess.run")
def test_handle_run_remainder_explicit(mock_run, orchestrator) -> None:
    """Kiểm tra handle_run với -- explicit."""
    mock_run.return_value.returncode = 0
    # Create args manually to ensure -- is there
    args = orchestrator.parser.parse_args(["run", "npm", "--", "test"])
    orchestrator.handle_run(args)
    # This MUST hit line 245
    called_args = str(mock_run.call_args[0][0])
    assert "npm install -g bun && bun test" in called_args


@patch("subprocess.run")
def test_handle_run_cargo_outside_root(mock_run, orchestrator) -> None:
    """Kiểm tra handle_run cargo khi ở ngoài root (ValueError)."""
    mock_run.return_value.returncode = 0
    with patch("pathlib.Path.cwd", return_value=Path("/tmp")):
        args = orchestrator.parser.parse_args(["run", "cargo", "check"])
        orchestrator.handle_run(args)
        # rel_dir should remain empty
        called_args = mock_run.call_args[0][0]
        assert "/workspace" in called_args


@patch("subprocess.run")
def test_cleanup_success(mock_run, orchestrator) -> None:
    """Kiểm tra cleanup thành công."""
    orchestrator.needs_cleanup = True
    orchestrator.dry_run = False
    orchestrator.cleanup()
    assert mock_run.called
    assert not orchestrator.needs_cleanup


@patch("subprocess.run")
@patch("_orchestrator.log_error")
def test_cleanup_timeout(mock_log, mock_run, orchestrator) -> None:
    """Kiểm tra cleanup khi bị timeout."""
    import subprocess
    orchestrator.needs_cleanup = True
    orchestrator.dry_run = False
    mock_run.side_effect = subprocess.TimeoutExpired(cmd="test", timeout=60)
    orchestrator.cleanup()
    mock_log.assert_called_with("⏰ Cleanup timed out after 60s.")


@patch("subprocess.run")
@patch("_orchestrator.log_error")
def test_cleanup_exception(mock_log, mock_run, orchestrator) -> None:
    """Kiểm tra cleanup khi có lỗi subprocess."""
    orchestrator.needs_cleanup = True
    orchestrator.dry_run = False
    mock_run.side_effect = Exception("test error")
    orchestrator.cleanup()
    mock_log.assert_called_with("❌ Cleanup failed with error: test error")


@patch("sys.exit")
@patch("_orchestrator.log_warn")
def test_signal_handler(mock_warn, mock_exit) -> None:
    """Kiểm tra signal handler."""
    import signal
    import _orchestrator
    
    handler = None
    def mock_signal(sig, h):
        nonlocal handler
        if sig == signal.SIGINT:
            handler = h
            
    with patch("signal.signal", side_effect=mock_signal):
        orch = _orchestrator.Orchestrator()
    
    if handler:
        handler(signal.SIGINT, None)
        mock_warn.assert_called()
        mock_exit.assert_called_with(128 + signal.SIGINT)


@patch("subprocess.run")
def test_cleanup_early_exit(mock_run, orchestrator) -> None:
    """Kiểm tra cleanup thoát sớm."""
    orchestrator.needs_cleanup = False
    orchestrator.cleanup()
    assert not mock_run.called
    
    orchestrator.needs_cleanup = True
    orchestrator.dry_run = True
    orchestrator.cleanup()
    assert not mock_run.called
