"""Behavior tests for Playwright MCP idle cleanup."""

from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WATCHDOG = ROOT / "scripts/playwright-idle-watchdog.py"
TIMEOUT_SCRIPT = ROOT / "scripts/playwright-timeout.sh"


def run_watchdog(
    *, child: str, idle: int, grace: int, input_data: bytes = b""
) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        [
            sys.executable,
            str(WATCHDOG),
            "--idle-seconds",
            str(idle),
            "--grace-seconds",
            str(grace),
            "--",
            sys.executable,
            "-c",
            child,
        ],
        input=input_data,
        capture_output=True,
        check=False,
        timeout=10,
    )

def test_idle_mcp_process_is_terminated_with_distinct_status() -> None:
    result = run_watchdog(child="import time; time.sleep(10)", idle=2, grace=1)

    assert result.returncode == 124
    assert b"Playwright MCP idle for 2s" in result.stderr


def test_request_activity_resets_idle_deadline() -> None:
    child = "import sys, time; data = sys.stdin.buffer.read(2); sys.stdout.buffer.write(data); sys.stdout.buffer.flush(); time.sleep(10)"
    process = subprocess.Popen(
        [
            sys.executable,
            str(WATCHDOG),
            "--idle-seconds",
            "2",
            "--grace-seconds",
            "1",
            "--",
            sys.executable,
            "-c",
            child,
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    assert process.stdin is not None
    assert process.stdout is not None
    process.stdin.write(b"x")
    process.stdin.flush()
    time.sleep(0.5)
    process.stdin.write(b"y")
    process.stdin.flush()
    assert process.stdout.read(2) == b"xy"
    time.sleep(0.5)
    assert process.poll() is None
    process.stdin.close()
    process.wait(timeout=5)

    assert process.returncode == 124


def test_idle_timeout_environment_is_validated_before_command() -> None:
    command = f"source {TIMEOUT_SCRIPT}; PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS=61 playwright_idle_run true true"
    result = subprocess.run(
        ["bash", "-c", command],
        cwd=ROOT,
        env={**os.environ, "PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS": "61"},
        capture_output=True,
        check=False,
    )

    assert result.returncode == 2
    assert b"PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS" in result.stderr


def test_playwright_runtime_storage_is_memory_backed() -> None:
    compose = (ROOT / ".compose/docker-compose.dev.yml").read_text()
    service = compose.split("  playwright-mcp:\n", 1)[1].split("\n\n", 1)[0]

    assert "    read_only: true\n" in service
    assert "      - /tmp\n" in service
    assert "      - /root/.cache\n" in service
    assert "      - /root/.config\n" in service
    assert "      - /root/.local\n" in service
    assert "    shm_size: 256m\n" in service
    assert "    volumes:" not in service
