"""Tests for _lifecycle.py — lifecycle_up, lifecycle_down, health checks."""

import subprocess
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.append(str(PROJECT_ROOT / "scripts"))

from _lifecycle import (  # noqa: E402
    _container_healthy,
    lifecycle_down,
    lifecycle_up,
    start_watchdog,
)


class TestContainerHealthy:
    def test_healthy_container(self) -> None:
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="healthy", returncode=0)
            assert _container_healthy("test_apiserver_1") is True

    def test_unhealthy_container(self) -> None:
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="unhealthy\n", returncode=0)
            assert _container_healthy("test_apiserver_1") is False

    def test_empty_output(self) -> None:
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="", returncode=1)
            assert _container_healthy("test_apiserver_1") is False


class TestLifecycleUp:
    def test_api_mode_starts_backend(self) -> None:
        with (
            patch("_lifecycle._run") as mock_run,
            patch("_lifecycle._container_healthy", return_value=True),
            patch("_lifecycle.time.sleep"),
        ):
            result = lifecycle_up("api")
            assert result is True
            all_call_args = str(mock_run.call_args_list)
            assert "up" in all_call_args

    def test_web_mode_checks_bun_init(self) -> None:
        with (
            patch("_lifecycle._run"),
            patch("_lifecycle._container_healthy", return_value=True),
            patch("_lifecycle.time.sleep"),
        ):
            result = lifecycle_up("web")
            assert result is True

    def test_health_check_timeout_returns_false(self) -> None:
        with (
            patch("_lifecycle._run"),
            patch("_lifecycle._container_healthy", return_value=False),
            patch("_lifecycle.time.sleep"),
        ):
            result = lifecycle_up("api")
            assert result is False

    def test_no_pre_cleanup_on_up(self) -> None:
        """lifecycle_up must NOT do a pre-down — concurrent agents share containers."""
        with (
            patch("_lifecycle._run") as mock_run,
            patch("_lifecycle._container_healthy", return_value=True),
            patch("_lifecycle.time.sleep"),
        ):
            lifecycle_up("api")
            all_call_str = str(mock_run.call_args_list)
            assert "down" not in all_call_str

    def test_stops_containers(self) -> None:
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_down("all")
            all_call_str = str(mock_run.call_args_list)
            assert "down" in all_call_str

    def test_uses_remove_orphans(self) -> None:
        """With per-run isolation, lifecycle_down MUST use --remove-orphans to completely clean up."""
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_down("all")
            all_call_str = str(mock_run.call_args_list)
            assert "--remove-orphans" in all_call_str

    def test_force_kill_pod(self) -> None:
        """Force-kill aggressively removes the pod since it is isolated per run."""
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_down("web")
            all_call_str = str(mock_run.call_args_list)
            # Must remove the pod
            assert "pod" in all_call_str and "rm" in all_call_str

    def test_no_volume_cleanup(self) -> None:
        """lifecycle_down must NOT remove shared cache volumes (used by other agents).
        Specifically ensures -v and --volumes flags are NOT passed to compose down.
        """
        with patch("_lifecycle._run") as mock_run:
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_down("all")
            all_call_str = str(mock_run.call_args_list)
            assert "volume" not in all_call_str
            assert "'-v'" not in all_call_str
            assert "'--volumes'" not in all_call_str


class TestStartWatchdog:
    def test_spawns_detached_process(self) -> None:
        with patch("subprocess.Popen") as mock_popen:
            mock_proc = MagicMock()
            mock_proc.pid = 99999
            mock_popen.return_value = mock_proc
            result = start_watchdog(60, 12345)
            assert result == 99999
            kwargs = mock_popen.call_args.kwargs
            assert kwargs["start_new_session"] is True
            assert kwargs["stdin"] == subprocess.DEVNULL
            assert kwargs["stdout"] == subprocess.DEVNULL
            assert kwargs["stderr"] == subprocess.DEVNULL

    def test_uses_correct_parent_pid(self) -> None:
        with patch("subprocess.Popen") as mock_popen:
            mock_proc = MagicMock()
            mock_proc.pid = 1
            mock_popen.return_value = mock_proc
            start_watchdog(60, 54321)
            cmd_str = " ".join(mock_popen.call_args.args[0])
            assert "54321" in cmd_str


class TestLifecycleComposeCommand:
    """Verify lifecycle_up/down always use correct compose base (bug: bun_init leak)."""

    def test_lifecycle_down_uses_base_not_env_override(self) -> None:
        """lifecycle_down must use DOCKER_COMPOSE_BASE with -p and -f in its down command."""
        with (
            patch("_lifecycle._run") as mock_run,
            patch.dict("os.environ", {"DOCKER_COMPOSE_CMD": "podman-compose"}),
        ):
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_down("web")
            # The down command is no longer the very last call (due to sweep_stale_environments)
            down_cmd = ""
            for call in mock_run.call_args_list:
                cmd = " ".join(call[0][0])
                if "down" in cmd:
                    down_cmd = cmd
                    break
            assert "-p" in down_cmd
            assert "leeattend-test" in down_cmd
            assert "-f" in down_cmd
            assert "docker-compose.test.yml" in down_cmd

    def test_lifecycle_up_web_uses_task_profiles_not_frontend(self) -> None:
        """lifecycle_up(web) must use lint-web, unit-web, e2e-web (not frontend)."""
        with (
            patch("_lifecycle._run") as mock_run,
            patch("_lifecycle._container_healthy", return_value=True),
            patch("_lifecycle.time.sleep"),
        ):
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_up("web")
            up_cmd = " ".join(mock_run.call_args_list[0][0][0])  # only up -d, no pre-down
            assert "lint-web" in up_cmd
            assert "e2e-web" in up_cmd

    def test_lifecycle_up_has_correct_compose_base(self) -> None:
        """lifecycle_up must use DOCKER_COMPOSE_BASE with -p and -f."""
        with (
            patch("_lifecycle._run") as mock_run,
            patch("_lifecycle._container_healthy", return_value=True),
            patch("_lifecycle.time.sleep"),
            patch.dict("os.environ", {"DOCKER_COMPOSE_CMD": "wrong-value-only"}),
        ):
            mock_run.return_value = MagicMock(stdout="", returncode=0)
            lifecycle_up("api")
            up_cmd = " ".join(mock_run.call_args_list[0][0][0])
            # Must use correct compose file, not env override
            assert "docker-compose.test.yml" in up_cmd
            assert "leeattend-test" in up_cmd
