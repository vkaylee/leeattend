"""Real-container MCP checks; requires the existing dev image and public login page."""

from __future__ import annotations

import contextlib
import json
import os
import selectors
import shlex
import shutil
import signal
import subprocess
import sys
import time
import uuid
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
ENGINE = shutil.which("podman") or shutil.which("docker")
COMPOSE = shutil.which("podman-compose")


class Session:
    def __init__(self, tmp_path: Path, *, session_idle: int = 300, extra: tuple[str, ...] = ()) -> None:
        assert ENGINE and COMPOSE, "Container engine and project Compose engine required"
        self.name = f"playwright-lifecycle-{uuid.uuid4().hex}"
        # Inject a test-only exact name, never infer ownership from a ps difference.
        shim = tmp_path / self.name
        shim.write_text(
            '#!/bin/bash\nargs=(); for arg in "$@"; do args+=("$arg"); '
            f'if [[ "$arg" == run ]]; then args+=(--name {self.name}); fi; done\n'
            f'exec {shlex.quote(COMPOSE)} "${{args[@]}}"\n'
        )
        shim.chmod(0o755)
        self.errors = (tmp_path / f"{self.name}.stderr").open("wb")
        command = ["bash", str(ROOT / "scripts/playwright-mcp-role.sh")]
        if session_idle != 300:
            command = [sys.executable, str(ROOT / "scripts/playwright-idle-watchdog.py"),
                       "--idle-seconds", os.environ.get("PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS", "10"),
                       "--session-idle-seconds", str(session_idle), "--", *command, "--supervised"]
        self.process = subprocess.Popen(
            [*command, "employee", "--endpoint", "http://caddy", *extra],
            cwd=ROOT, env={**os.environ, "DOCKER_COMPOSE_CMD": str(shim)},
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.errors, bufsize=0,
        )
        self.buffer = bytearray()
        self.selector = selectors.DefaultSelector()
        assert self.process.stdout is not None
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.request_id = 0

    def engine(self, *args: str) -> subprocess.CompletedProcess[str]:
        assert ENGINE
        return subprocess.run([ENGINE, *args], capture_output=True, text=True, timeout=10, check=False)

    def send(self, message: dict) -> None:
        assert self.process.stdin is not None
        data = memoryview(json.dumps(message).encode() + b"\n")
        while data:
            written = self.process.stdin.write(data)
            assert written
            data = data[written:]

    def call(self, method: str, params: dict | None = None) -> dict:
        self.request_id += 1
        self.send({"jsonrpc": "2.0", "id": self.request_id, "method": method, "params": params or {}})
        deadline = time.monotonic() + 40
        while time.monotonic() < deadline:
            while b"\n" in self.buffer:
                line, _, rest = self.buffer.partition(b"\n")
                self.buffer = bytearray(rest)
                message = json.loads(line)
                if message.get("id") == self.request_id:
                    assert "error" not in message, "JSON-RPC request failed"
                    result = message["result"]
                    assert not result.get("isError"), "MCP tool failed"
                    return result
            if self.selector.select(0.1):
                assert self.process.stdout is not None
                data = os.read(self.process.stdout.fileno(), 65536)
                assert data, f"MCP stdout closed (exit={self.process.poll()})"
                self.buffer.extend(data)
        pytest.fail("MCP response deadline exceeded")

    def initialize(self) -> None:
        self.call("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                 "clientInfo": {"name": "lifecycle-test", "version": "1"}})
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def browser_running(self) -> bool:
        # comm excludes CLI arguments such as --browser=chrome from the match.
        args = ("top", self.name, "comm") if Path(ENGINE or "").name == "podman" else ("top", self.name, "-eo", "comm")
        result = self.engine(*args)
        assert result.returncode == 0, "Test container process inspection failed"
        return any("chrome" in line.lower() for line in result.stdout.splitlines()[1:])

    def navigate(self) -> None:
        self.call("tools/call", {"name": "browser_navigate", "arguments": {"url": "http://caddy/login"}})

    def close(self) -> None:
        try:
            if self.process.poll() is None:
                self.process.terminate()
                try:
                    self.process.wait(timeout=8)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=5)
            self.engine("rm", "-f", self.name)
        finally:
            self.selector.close()
            for pipe in (self.process.stdin, self.process.stdout):
                if pipe is not None:
                    pipe.close()
            self.errors.close()


def wait_for(check, timeout: float = 8) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if check():
            return
        # Bounded polling of daemon state, not a fixed wait for success.
        time.sleep(0.1)
    pytest.fail("Container lifecycle deadline exceeded")


def test_session_idle_removes_container_and_fresh_reconnect_works(tmp_path: Path) -> None:
    with contextlib.ExitStack() as cleanup:
        session = Session(tmp_path, session_idle=2)
        cleanup.callback(session.close)
        session.initialize()
        session.navigate()
        assert session.browser_running()
        # In-flight tool outlasts session idle without native browser interruption.
        session.call("tools/call", {"name": "browser_wait_for", "arguments": {"time": 3}})
        assert session.process.wait(timeout=8) == 124
        wait_for(lambda: session.engine("container", "inspect", session.name).returncode != 0)
        fresh = Session(tmp_path)
        cleanup.callback(fresh.close)
        fresh.initialize()
        fresh.navigate()
        assert fresh.browser_running()


def test_idle_session_does_not_stop_parallel_active_session(tmp_path: Path) -> None:
    with contextlib.ExitStack() as cleanup:
        active = Session(tmp_path)
        cleanup.callback(active.close)
        active.initialize()
        idle = Session(tmp_path, session_idle=2)
        cleanup.callback(idle.close)
        idle.initialize()
        assert idle.process.wait(timeout=8) == 124
        wait_for(lambda: idle.engine("container", "inspect", idle.name).returncode != 0)
        assert active.process.poll() is None
        assert "tools" in active.call("tools/list")


def test_startup_failure_removes_own_container(tmp_path: Path) -> None:
    session = Session(tmp_path, extra=("--unsupported-lifecycle-test-option",))
    try:
        assert session.process.wait(timeout=10) != 0
        wait_for(lambda: session.engine("container", "inspect", session.name).returncode != 0)
    finally:
        session.close()


def test_pending_request_timeout_removes_own_container(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setenv("PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS", "10")
    session = Session(tmp_path)
    try:
        session.initialize()
        session.navigate()
        session.send({"jsonrpc": "2.0", "id": 99, "method": "tools/call", "params": {
            "name": "browser_wait_for", "arguments": {"time": 15},
        }})
        assert session.process.wait(timeout=15) == 124
        wait_for(lambda: session.engine("container", "inspect", session.name).returncode != 0)
    finally:
        session.close()


@pytest.mark.parametrize("shutdown", [None, signal.SIGTERM, signal.SIGINT])
def test_session_shutdown_isolated_and_container_removed(tmp_path: Path, shutdown: int | None) -> None:
    with contextlib.ExitStack() as cleanup:
        first = Session(tmp_path)
        cleanup.callback(first.close)
        second = Session(tmp_path)
        cleanup.callback(second.close)
        first.initialize()
        second.initialize()
        if shutdown is None:
            assert first.process.stdin is not None
            first.process.stdin.close()
        else:
            first.process.send_signal(shutdown)
        first.process.wait(timeout=8)
        wait_for(lambda: first.engine("container", "inspect", first.name).returncode != 0)
        assert second.process.poll() is None
        assert "tools" in second.call("tools/list")
