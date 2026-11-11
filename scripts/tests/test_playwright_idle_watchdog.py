"""Behavior tests for Playwright MCP watchdog."""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def image_build_result(
    *, image_present: bool, engine: str = "podman-compose"
) -> subprocess.CompletedProcess[str]:
    script = ROOT / "scripts/playwright-timeout.sh"
    command = (
        f"source {script}; COMPOSE=({engine}); "
        "podman() { [[ \"$*\" == 'image exists localhost/leedevkit-dev_playwright-mcp:latest' ]] && "
        "[[ \"$IMAGE_PRESENT\" == 1 ]]; }; "
        "docker() { [[ \"$*\" == 'image inspect localhost/leedevkit-dev_playwright-mcp:latest' ]] && "
        "[[ \"$IMAGE_PRESENT\" == 1 ]]; }; "
        "playwright_timeout_run() { printf 'build %s\\n' \"$*\"; }; "
        "playwright_build_image_if_missing localhost/leedevkit-dev_playwright-mcp:latest "
        f"{engine} -p project --profile mcp"
    )
    return subprocess.run(
        ["bash", "-c", command],
        cwd=ROOT,
        env={**os.environ, "IMAGE_PRESENT": "1" if image_present else "0"},
        capture_output=True,
        text=True,
        check=False,
    )
WATCHDOG = ROOT / "scripts/playwright-idle-watchdog.py"


def watchdog_command(child: str, *, idle: int = 3, grace: int = 1) -> list[str]:
    return [
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
    ]


def test_idle_timeout_cli_defaults_to_ten_seconds(monkeypatch) -> None:
    import runpy

    parse_args = runpy.run_path(str(WATCHDOG), run_name="watchdog_test")["parse_args"]
    monkeypatch.setattr(
        sys,
        "argv",
        [str(WATCHDOG), "--", sys.executable, "-c", "pass"],
    )

    assert parse_args().idle_seconds == 10


def test_existing_podman_image_skips_build() -> None:
    result = image_build_result(image_present=True)

    assert result.returncode == 0
    assert result.stdout == ""


def test_missing_podman_image_builds_once() -> None:
    result = image_build_result(image_present=False)

    assert result.returncode == 0
    assert result.stderr == "build podman-compose -p project --profile mcp build playwright-mcp\n"

def test_existing_docker_image_skips_build() -> None:
    result = image_build_result(image_present=True, engine="docker compose")

    assert result.returncode == 0
    assert result.stdout == ""


def test_missing_docker_image_builds_once() -> None:
    result = image_build_result(image_present=False, engine="docker compose")

    assert result.returncode == 0
    assert result.stderr == "build docker compose -p project --profile mcp build playwright-mcp\n"



def test_silent_startup_is_terminated() -> None:
    started = time.monotonic()
    with subprocess.Popen(
        watchdog_command("import time; time.sleep(10)"),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process:
        process.wait(timeout=5)
        assert process.returncode == 124
        assert process.stderr is not None
        assert b"Playwright MCP idle for 3s" in process.stderr.read()
    assert 2 <= time.monotonic() - started < 4


def test_client_notifications_do_not_extend_pending_request_deadline() -> None:
    process = subprocess.Popen(
        watchdog_command("import time; time.sleep(10)"),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    assert process.stdin is not None
    process.stdin.write(b'{"jsonrpc":"2.0","id":1,"method":"initialize"}\n')
    process.stdin.flush()
    started = time.monotonic()
    while time.monotonic() - started < 1.5:
        process.stdin.write(b'{"jsonrpc":"2.0","method":"notifications/progress"}\n')
        process.stdin.flush()
        time.sleep(0.2)
    process.stdin.close()
    process.wait(timeout=2)

    assert process.returncode == 124
    assert time.monotonic() - started < 3


def test_server_output_progress_resets_pending_request_deadline() -> None:
    child = (
        "import sys, time; sys.stdin.buffer.readline(); "
        "notification=b'{\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\\n'; "
        "time.sleep(1.3); sys.stdout.buffer.write(notification); sys.stdout.buffer.flush(); "
        "time.sleep(1.3); sys.stdout.buffer.write(notification); sys.stdout.buffer.flush(); "
        "time.sleep(0.7)"
    )
    with subprocess.Popen(
        watchdog_command(child, idle=3, grace=1),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process:
        assert process.stdin is not None
        process.stdin.write(b'{"jsonrpc":"2.0","id":1,"method":"initialize"}\n')
        process.stdin.flush()
        process.wait(timeout=5)
        assert process.returncode == 0
        assert process.stdout is not None
        assert process.stdout.read().count(b'"method":"notifications/progress"') == 2


def test_replied_initialization_leaves_healthy_idle_process_alive() -> None:
    child = (
        "import sys, time; sys.stdin.buffer.readline(); "
        "sys.stdout.buffer.write(b'{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\\n'); "
        "sys.stdout.buffer.flush(); time.sleep(2.5)"
    )
    with subprocess.Popen(
        watchdog_command(child, idle=2, grace=1),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process:
        assert process.stdin is not None
        process.stdin.write(b'{"jsonrpc":"2.0","id":1,"method":"initialize"}\n')
        process.stdin.flush()
        process.wait(timeout=5)
        assert process.returncode == 0
        assert process.stdout is not None
        assert process.stdout.read() == b'{"jsonrpc":"2.0","id":1,"result":{}}\n'


def test_process_exit_is_reaped_without_waiting_for_idle_deadline() -> None:
    child = (
        "import sys; sys.stdin.buffer.readline(); "
        "sys.stdout.buffer.write(b'{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\\n'); "
        "sys.stdout.buffer.flush()"
    )
    started = time.monotonic()
    result = subprocess.run(
        watchdog_command(child, idle=4, grace=1),
        input=b'{"jsonrpc":"2.0","id":1,"method":"initialize"}\n',
        capture_output=True,
        check=False,
        timeout=3,
    )

    assert result.returncode == 0
    assert time.monotonic() - started < 2.5


def test_stdio_forwards_json_rpc_bytes_both_directions() -> None:
    request = b'{"jsonrpc":"2.0","id":1,"method":"ping"}\n'
    response = b'{"jsonrpc":"2.0","id":1,"result":{}}\n'
    child = (
        f"import sys; data=sys.stdin.buffer.readline(); assert data == {request!r}; "
        f"sys.stdout.buffer.write({response!r}); sys.stdout.buffer.flush()"
    )
    result = subprocess.run(
        watchdog_command(child),
        input=request,
        capture_output=True,
        check=False,
        timeout=3,
    )

    assert result.returncode == 0

def test_initialize_response_includes_server_instructions() -> None:
    request = b'{"jsonrpc":"2.0","id":1,"method":"initialize"}\n'
    response = b'{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"test","version":"1"}}}\n'
    child = (
        f"import sys; data=sys.stdin.buffer.readline(); assert data == {request!r}; "
        f"sys.stdout.buffer.write({response!r}); sys.stdout.buffer.flush()"
    )
    result = subprocess.run(
        [
            sys.executable,
            str(WATCHDOG),
            "--initialize-instructions",
            "Use employee role access only.",
            "--",
            sys.executable,
            "-c",
            child,
        ],
        input=request,
        capture_output=True,
        check=False,
        timeout=3,
    )

    assert result.returncode == 0
    message = json.loads(result.stdout)
    assert message["result"]["instructions"] == "Use employee role access only."
    assert message["result"]["protocolVersion"] == "2025-11-25"


def test_process_ignoring_term_is_force_killed_at_deadline() -> None:
    child = "import signal, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(10)"
    started = time.monotonic()
    with subprocess.Popen(
        watchdog_command(child),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process:
        process.wait(timeout=5)
        assert process.returncode == 124
    assert time.monotonic() - started < 4


@pytest.mark.parametrize("shutdown", [None, signal.SIGTERM, signal.SIGINT])
def test_client_shutdown_bounds_initialized_child_lifetime(shutdown: int | None) -> None:
    child = (
        "import signal, sys, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); "
        "signal.signal(signal.SIGINT, signal.SIG_IGN); sys.stdin.buffer.readline(); "
        "print('{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}', flush=True); time.sleep(30)"
    )
    process = subprocess.Popen(
        watchdog_command(child), stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    try:
        assert process.stdin is not None
        assert process.stdout is not None
        process.stdin.write(b'{"jsonrpc":"2.0","id":1,"method":"initialize"}\n')
        process.stdin.flush()
        assert json.loads(process.stdout.readline())["id"] == 1
        started = time.monotonic()
        if shutdown is None:
            process.stdin.close()
        else:
            process.send_signal(shutdown)
        process.wait(timeout=4)
        assert process.returncode == 256 - signal.SIGKILL
        assert time.monotonic() - started < 3.5
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=3)
        if process.stdin is not None and not process.stdin.closed:
            process.stdin.close()
        process.stdout.close()
        assert process.stderr is not None
        process.stderr.close()


def test_client_eof_allows_final_response_to_flush() -> None:
    response = b'{"jsonrpc":"2.0","id":1,"result":{}}\n'
    child = (
        "import sys; sys.stdin.buffer.read(); "
        f"sys.stdout.buffer.write({response!r}); sys.stdout.buffer.flush()"
    )
    result = subprocess.run(
        watchdog_command(child), input=b'{"jsonrpc":"2.0","id":1,"method":"ping"}\n',
        capture_output=True, timeout=4, check=False,
    )
    assert result.returncode == 0
    assert result.stdout == response


def test_idle_timeout_cli_rejects_out_of_range_value() -> None:
    result = subprocess.run(
        [sys.executable, str(WATCHDOG), "--idle-seconds", "61", "--", sys.executable, "-c", "pass"],
        capture_output=True,
        check=False,
        timeout=3,
    )

    assert result.returncode == 2
    assert b"--idle-seconds must be an integer from 1 to 60 seconds" in result.stderr
