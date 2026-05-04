"""Lifecycle management — Python equivalent of _lifecycle.sh.

Provides lifecycle_up() and lifecycle_down() for test infrastructure
container orchestration. Handles profile selection, health checks,
and force-kill fallbacks for stuck containers.
"""

import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

from _bootstrap import (
    PROJECT_ROOT,
    SCRIPTS_DIR,
    detect_engine,
    resolve_profiles,
    resolve_lifecycle_profiles,
    bootstrap_env,
)


def _get_engine() -> str:
    """Return the container engine (podman or docker) at call time."""
    return os.environ.get("CONTAINER_ENGINE") or detect_engine()


def _run(
    cmd: list[str],
    timeout: Optional[int] = None,
    capture: bool = True,
    check: bool = False,
    silent: bool = False,
) -> subprocess.CompletedProcess[str]:
    """Run a command with project env and PTY-safe stdin."""
    env = os.environ.copy()
    env.update(bootstrap_env())

    kwargs: dict = {
        "cwd": PROJECT_ROOT,
        "env": env,
        "stdin": subprocess.DEVNULL,
        "text": True,
        "check": check,
    }
    if silent:
        kwargs["stdout"] = subprocess.DEVNULL
        kwargs["stderr"] = subprocess.DEVNULL
    elif capture:
        kwargs["stdout"] = subprocess.PIPE
        kwargs["stderr"] = subprocess.PIPE

    if timeout is not None:
        kwargs["timeout"] = timeout

    return subprocess.run(cmd, **kwargs)


def _container_healthy(container_name: str) -> bool:
    """Check if a container reports healthy status."""
    result = _run(
        [_get_engine(), "inspect", "--format={{.State.Health.Status}}", container_name],
        silent=False,
        capture=True,
    )
    return result.stdout.strip() == "healthy"


def lifecycle_up(mode: str = "all") -> bool:
    """Bring up test infrastructure containers for the given mode.

    Returns True if all health checks pass, False otherwise.
    Idempotent — safe for concurrent agents sharing the same project.
    """
    profiles = resolve_profiles(mode)
    env = bootstrap_env(mode)
    compose_base = env["DOCKER_COMPOSE_BASE"].split()

    # Bring services up (idempotent — no pre-cleanup needed)
    _run(compose_base + profiles + ["up", "-d"], silent=False, capture=False)

    # Health check
    project_name = os.environ.get("COMPOSE_PROJECT_NAME", "leeattend-test")
    if mode != "web":
        container = f"{project_name}_apiserver_1"
        for _ in range(60):
            if _container_healthy(container):
                return True
            time.sleep(1)
        # Diagnostic
        _run([_get_engine(), "ps", "-a", "--filter", f"name={project_name}"], capture=False)
        return False
    else:
        container = f"{project_name}_bun_init_1"
        for _ in range(120):
            if _container_healthy(container):
                return True
            time.sleep(1)
        _run([_get_engine(), "ps", "-a", "--filter", f"name={project_name}"], capture=False)
        return False


# Containers per lifecycle profile — used for force-kill fallback
# when compose down fails to stop PID 1 (e.g. tail -f /dev/null ignoring SIGTERM)
_PROFILE_CONTAINERS: dict[str, list[str]] = {
    "frontend": ["bun_init", "webdashboard"],
    "backend": ["apiserver", "agent-main", "db", "db_init", "redis"],
}


def lifecycle_down(mode: str = "all") -> None:
    """Tear down test infrastructure for the given mode.

    Stops containers via compose for the specified profiles only.
    Safe for concurrent agents — never touches containers from other profiles.

    After graceful down, force-removes profile-specific containers that
    compose could not stop (e.g. PID 1 tail -f /dev/null ignoring SIGTERM).
    """
    profiles = resolve_lifecycle_profiles(mode)
    env = bootstrap_env(mode)
    compose_base = env["DOCKER_COMPOSE_BASE"].split()
    project_name = os.environ.get("COMPOSE_PROJECT_NAME", "leeattend-test")

    # Graceful compose down first
    _run(
        compose_base + profiles + ["down", "--timeout", "5"],
        silent=True,
    )

    # Force-kill profile-specific containers that compose couldn't stop
    # PID 1 in containers ignores SIGTERM by default (Linux kernel behavior)
    # — only SIGKILL works reliably on processes like tail -f /dev/null
    profile_names = profiles[1::2]  # extract names from ["--profile", "frontend", ...]
    for pname in profile_names:
        for svc in _PROFILE_CONTAINERS.get(pname, []):
            cname = f"{project_name}_{svc}_1"
            _run([_get_engine(), "rm", "-f", cname], silent=True)


def start_watchdog(timeout_sec: int, parent_pid: int) -> int:
    """Spawn a fully detached watchdog that kills parent_pid after timeout.

    Returns the watchdog's own PID so it can be cancelled later.
    All FDs severed from PTY so the watchdog itself never causes a PTY hang.
    """
    watchdog_script = SCRIPTS_DIR / "_watchdog.py"
    proc = subprocess.Popen(
        [
            sys.executable,
            str(watchdog_script),
            str(timeout_sec),
            str(parent_pid),
        ],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    return proc.pid
