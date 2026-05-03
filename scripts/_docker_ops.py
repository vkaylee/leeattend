"""Podman/Docker Compose wrapper — Python equivalent of _docker-ops.sh.

Provides subprocess wrappers for docker-compose commands with PTY safety
(</dev/null redirection) and detached execution with timeout protection.
"""

import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

from _bootstrap import PROJECT_ROOT, detect_engine, detect_compose_cmd, bootstrap_env

ENGINE = detect_engine()
COMPOSE_CMD = detect_compose_cmd()
DEFAULT_TIMEOUT = int(os.environ.get("DOCKER_COMPOSE_TIMEOUT", "60"))


def _build_compose_cmd(
    extra_args: list[str],
    compose_file: Optional[Path] = None,
    project_name: Optional[str] = None,
) -> list[str]:
    """Build a full compose command list."""
    if project_name is None:
        project_name = os.environ.get("COMPOSE_PROJECT_NAME", "leeattend-test")
    if compose_file is None:
        compose_file = PROJECT_ROOT / ".compose" / "docker-compose.test.yml"
    return COMPOSE_CMD + ["-p", project_name, "-f", str(compose_file)] + extra_args


def dc(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    """Run a docker-compose command with stdin detached (PTY safety).

    Equivalent to bash: dc() { $DOCKER_COMPOSE_CMD "$@" </dev/null; }

    Returns CompletedProcess. Raises CalledProcessError if check=True and command fails.
    """
    env = os.environ.copy()
    env.update(bootstrap_env())

    return subprocess.run(
        _build_compose_cmd(list(args)),
        cwd=PROJECT_ROOT,
        env=env,
        stdin=subprocess.DEVNULL,
        check=check,
        text=True,
        capture_output=False,
    )


def dc_detached(
    *args: str,
    timeout: int = DEFAULT_TIMEOUT,
) -> int:
    """Run compose command in background with full FD isolation and timeout.

    Equivalent to bash dc_detached() with setsid, FD redirect, and watchdog.

    Returns exit code of the compose command.
    """
    env = os.environ.copy()
    env.update(bootstrap_env())

    proc = subprocess.Popen(
        _build_compose_cmd(list(args)),
        cwd=PROJECT_ROOT,
        env=env,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,  # setsid equivalent
    )

    try:
        proc.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        # Kill the process group (equivalent to bash watchdog kill)
        try:
            os.killpg(proc.pid, 9)
        except OSError:
            pass
        proc.wait()
        return 1

    return proc.returncode
