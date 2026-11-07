#!/usr/bin/env python3
"""Run a stdio MCP process and stop it after an idle period."""

from __future__ import annotations

import argparse
import contextlib
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
    parser.add_argument("--idle-seconds", type=int, default=60)
    parser.add_argument("--grace-seconds", type=int)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not 1 <= args.idle_seconds <= MAX_IDLE_SECONDS:
        parser.error(f"--idle-seconds must be an integer from 1 to {MAX_IDLE_SECONDS} seconds")
    if args.grace_seconds is None:
        args.grace_seconds = min(15, args.idle_seconds)
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
    if process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signum)


def run(command: Iterable[str], idle_seconds: int, grace_seconds: int) -> int:  # noqa: PLR0915
    process = subprocess.Popen(
        list(command),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,
        start_new_session=True,
    )
    if process.stdin is None or process.stdout is None:
        raise RuntimeError("MCP watchdog could not create stdio pipes")

    selector = selectors.DefaultSelector()
    selector.register(sys.stdin.fileno(), selectors.EVENT_READ, "request")
    selector.register(process.stdout, selectors.EVENT_READ, "response")
    stdin_open = True
    timed_out = False
    terminating = False
    last_request = time.monotonic()
    idle_deadline = last_request + idle_seconds
    term_at = idle_deadline - grace_seconds
    shutdown_deadline: float | None = None

    def terminate(signum: signal.Signals) -> None:
        nonlocal terminating, shutdown_deadline
        if not terminating:
            terminating = True
            shutdown_deadline = time.monotonic() + grace_seconds
            signal_process_group(process, signum)

    previous_sigterm = signal.signal(signal.SIGTERM, lambda _signum, _frame: terminate(signal.SIGTERM))
    previous_sigint = signal.signal(signal.SIGINT, lambda _signum, _frame: terminate(signal.SIGINT))
    try:
        while process.poll() is None:
            now = time.monotonic()
            if not terminating and now >= term_at:
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
            events = selector.select(max(0.0, wait_until - now))
            for key, _ in events:
                if key.data == "request":
                    data = os.read(sys.stdin.fileno(), 65536)
                    if not data:
                        selector.unregister(sys.stdin.fileno())
                        stdin_open = False
                        process.stdin.close()
                    elif not terminating:
                        process.stdin.write(data)
                        process.stdin.flush()
                        last_request = time.monotonic()
                        idle_deadline = last_request + idle_seconds
                        term_at = idle_deadline - grace_seconds
                else:
                    data = os.read(process.stdout.fileno(), 65536)
                    if not data:
                        selector.unregister(process.stdout)
                    else:
                        write_all(sys.stdout.fileno(), data)

            if not stdin_open and process.poll() is not None:
                break
    finally:
        signal.signal(signal.SIGTERM, previous_sigterm)
        signal.signal(signal.SIGINT, previous_sigint)
        selector.close()
        if process.poll() is None:
            signal_process_group(process, signal.SIGKILL if timed_out else signal.SIGTERM)
        process.wait()

    return TIMEOUT_EXIT_STATUS if timed_out else process.returncode


def main() -> int:
    args = parse_args()
    return run(args.command, args.idle_seconds, args.grace_seconds)


if __name__ == "__main__":
    raise SystemExit(main())
