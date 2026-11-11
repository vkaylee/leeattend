#!/usr/bin/env python3
"""Run a stdio MCP process and stop it after startup or request silence."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import selectors
import signal
import subprocess
import sys
import time
from collections.abc import Iterable

MAX_IDLE_SECONDS = 60
MAX_GRACE_SECONDS = 60
TIMEOUT_EXIT_STATUS = 124


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--idle-seconds", type=int, default=10)
    parser.add_argument("--grace-seconds", type=int)
    parser.add_argument("--initialize-instructions")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not 1 <= args.idle_seconds <= MAX_IDLE_SECONDS:
        parser.error(f"--idle-seconds must be an integer from 1 to {MAX_IDLE_SECONDS} seconds")
    if args.initialize_instructions == "":
        parser.error("--initialize-instructions must not be empty")
    if args.grace_seconds is None:
        args.grace_seconds = min(2, args.idle_seconds)
    if not 1 <= args.grace_seconds <= min(MAX_GRACE_SECONDS, args.idle_seconds):
        parser.error(
            "--grace-seconds must be an integer from 1 to "
            f"{min(MAX_GRACE_SECONDS, args.idle_seconds)} seconds"
        )
    if not args.command:
        parser.error("a command is required after --")
    return args


def write_all(fd: int, data: bytes) -> None:
    view = memoryview(data)
    while view:
        written = os.write(fd, view)
        view = view[written:]


def signal_process_group(process: subprocess.Popen[bytes], signum: signal.Signals) -> None:
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signum)


def messages(line: bytes) -> list[dict[str, object]]:
    try:
        payload = json.loads(line)
    except (json.JSONDecodeError, UnicodeDecodeError):
        return []
    if isinstance(payload, dict):
        return [payload]
    if isinstance(payload, list):
        return [item for item in payload if isinstance(item, dict)]
    return []


def message_id(message: dict[str, object]) -> str | None:
    if "id" not in message:
        return None
    return json.dumps(message["id"], sort_keys=True, separators=(",", ":"))


def run(  # noqa: PLR0915
    command: Iterable[str],
    idle_seconds: int,
    grace_seconds: int,
    initialize_instructions: str | None = None,
) -> int:
    process = subprocess.Popen(
        list(command),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,
        start_new_session=True,
    )
    if process.stdin is None or process.stdout is None:
        raise RuntimeError("MCP watchdog could not create stdio pipes")

    selector = selectors.SelectSelector()
    selector.register(sys.stdin.fileno(), selectors.EVENT_READ, "request")
    selector.register(process.stdout, selectors.EVENT_READ, "response")
    stdin_open = True
    stdout_open = True
    timed_out = False
    terminating = False
    initialized = False
    pending: set[str] = set()
    initialize_ids: set[str] = set()
    request_buffer = bytearray()
    response_buffer = bytearray()
    term_at: float | None = time.monotonic() + idle_seconds - grace_seconds
    shutdown_deadline: float | None = None
    eof_deadline: float | None = None

    def schedule_deadline(now: float) -> None:
        nonlocal term_at
        term_at = now + idle_seconds - grace_seconds

    def terminate(signum: signal.Signals) -> None:
        nonlocal terminating, shutdown_deadline
        if not terminating:
            terminating = True
            shutdown_deadline = time.monotonic() + grace_seconds
            signal_process_group(process, signum)

    previous_sigterm = signal.signal(signal.SIGTERM, lambda _signum, _frame: terminate(signal.SIGTERM))
    previous_sigint = signal.signal(signal.SIGINT, lambda _signum, _frame: terminate(signal.SIGINT))
    try:
        while True:
            now = time.monotonic()
            if process.poll() is not None:
                if stdin_open:
                    selector.unregister(sys.stdin.fileno())
                    process.stdin.close()
                while stdout_open and selector.select(0):
                    data = os.read(process.stdout.fileno(), 65536)
                    if not data:
                        selector.unregister(process.stdout)
                        stdout_open = False
                    else:
                        write_all(sys.stdout.fileno(), data)
                break

            if not terminating and eof_deadline is not None and now >= eof_deadline:
                terminate(signal.SIGTERM)

            if not terminating and term_at is not None and now >= term_at:
                timed_out = True
                terminate(signal.SIGTERM)
                print(
                    f"Playwright MCP idle for {idle_seconds}s; sent TERM "
                    f"with {grace_seconds}s cleanup grace.",
                    file=sys.stderr,
                )

            if terminating and shutdown_deadline is not None and now >= shutdown_deadline:
                signal_process_group(process, signal.SIGKILL)
                break

            wait_until = shutdown_deadline if terminating else term_at
            timeout = 0.1 if wait_until is None else min(0.1, max(0.0, wait_until - now))
            for key, _ in selector.select(timeout):
                if key.data == "request":
                    data = os.read(sys.stdin.fileno(), 65536)
                    if not data:
                        selector.unregister(sys.stdin.fileno())
                        stdin_open = False
                        process.stdin.close()
                        eof_deadline = time.monotonic() + grace_seconds
                    elif not terminating:
                        process.stdin.write(data)
                        process.stdin.flush()
                        request_buffer.extend(data)
                        while b"\n" in request_buffer:
                            line, _, remaining = request_buffer.partition(b"\n")
                            request_buffer = bytearray(remaining)
                            for message in messages(line):
                                identifier = message_id(message)
                                if identifier is None or "method" not in message:
                                    continue
                                if not pending:
                                    schedule_deadline(time.monotonic())
                                if message["method"] == "initialize":
                                    initialize_ids.add(identifier)
                                pending.add(identifier)
                else:
                    data = os.read(process.stdout.fileno(), 65536)
                    if not data:
                        selector.unregister(process.stdout)
                        stdout_open = False
                        if process.poll() is None:
                            terminate(signal.SIGTERM)
                    else:
                        response_buffer.extend(data)
                        while b"\n" in response_buffer:
                            line, _, remaining = response_buffer.partition(b"\n")
                            response_buffer = bytearray(remaining)
                            outgoing = line
                            for message in messages(line):
                                identifier = message_id(message)
                                if identifier is None or identifier not in pending or "method" in message:
                                    continue
                                pending.remove(identifier)
                                if identifier not in initialize_ids:
                                    continue
                                initialize_ids.remove(identifier)
                                if "result" not in message:
                                    continue
                                initialized = True
                                result = message["result"]
                                if initialize_instructions is not None and isinstance(result, dict):
                                    result["instructions"] = initialize_instructions
                                    outgoing = json.dumps(message, separators=(",", ":")).encode()
                            write_all(sys.stdout.fileno(), outgoing + b"\n")
                        if initialized and not pending:
                            term_at = None
                        elif pending:
                            schedule_deadline(time.monotonic())

    finally:
        signal.signal(signal.SIGTERM, previous_sigterm)
        signal.signal(signal.SIGINT, previous_sigint)
        selector.close()
        if process.poll() is None:
            signal_process_group(process, signal.SIGKILL if timed_out or terminating else signal.SIGTERM)
            with contextlib.suppress(subprocess.TimeoutExpired):
                process.wait(timeout=grace_seconds)
            if process.poll() is None:
                signal_process_group(process, signal.SIGKILL)
        elif terminating:
            signal_process_group(process, signal.SIGKILL)
        process.wait()

    return TIMEOUT_EXIT_STATUS if timed_out else process.returncode


def main() -> int:
    args = parse_args()
    return run(
        args.command,
        args.idle_seconds,
        args.grace_seconds,
        args.initialize_instructions,
    )


if __name__ == "__main__":
    raise SystemExit(main())
