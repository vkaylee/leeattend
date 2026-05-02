#!/usr/bin/env python3
# ruff: noqa: N999
"""
safe-run.py

A bulletproof wrapper to run commands without leaking PTY file descriptors.
Replaces safe-run.sh with robust Python process management.
Optimized to use PTY for real-time line buffering while preventing background hang.
"""

import contextlib
import os
import pty
import select
import signal
import subprocess
import sys
import threading
from types import FrameType

import psutil


def read_from_pty(master_fd: int, stop_event: threading.Event) -> None:
    """Reads from the PTY master until stop_event is set and no more data is available."""
    while not stop_event.is_set():
        try:
            # Wait up to 0.1s for data to become available
            r, _, _ = select.select([master_fd], [], [], 0.1)
            if master_fd in r:
                data = os.read(master_fd, 4096)
                if not data:
                    break  # EOF  # pragma: no cover
                # Write directly to sys.stdout's underlying buffer to avoid encoding issues
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
        except (OSError, ValueError):
            break  # EIO or closed FD

    # One last non-blocking read to drain buffer
    while True:  # pragma: no cover
        try:
            r, _, _ = select.select([master_fd], [], [], 0.0)
            if master_fd in r:
                data = os.read(master_fd, 4096)
                if not data:
                    break
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
            else:
                break
        except (OSError, ValueError):
            break


def kill_process_tree(pid: int) -> None:
    """Kills a process and all its descendants using psutil."""
    try:
        parent: psutil.Process = psutil.Process(pid)
        children: list[psutil.Process] = parent.children(recursive=True)
        # Send SIGTERM to children
        for child in children:
            with contextlib.suppress(psutil.NoSuchProcess):  # pragma: no cover
                child.terminate()

        # Send SIGTERM to parent
        with contextlib.suppress(psutil.NoSuchProcess):
            parent.terminate()

        # Wait for them to die
        gone, alive = psutil.wait_procs(children + [parent], timeout=3.0)

        # Force kill if still alive
        for p in alive:
            with contextlib.suppress(psutil.NoSuchProcess):  # pragma: no cover
                p.kill()
    except psutil.NoSuchProcess:  # pragma: no cover
        pass

def validate_command(cmd_args: list[str]) -> None:
    """Ngăn chặn TUYỆT ĐỐI logic inline để tránh treo PTY và vi phạm chính sách dự án."""
    cmd_str = " ".join(cmd_args).lower()
    
    # Chính sách Zero Tolerance: Cấm tuyệt đối python -c
    if "python" in cmd_str and " -c " in cmd_str:
        print("\n" + "!"*80, file=sys.stderr)
        print("🚨 LỖI NGHIÊM TRỌNG: Cấm sử dụng logic Python inline (-c)!", file=sys.stderr)
        print("⚠️  Chính sách dự án: 100% logic phải nằm trong file script vật lý.", file=sys.stderr)
        print("💡 Giải pháp: Hãy tạo file trong scripts/ai-tools/ và chạy file đó.", file=sys.stderr)
        print("!"*80 + "\n", file=sys.stderr)
        sys.exit(1)


def main() -> None:
    if len(sys.argv) < 3:
        print(
            f"Usage: {sys.argv[0]} [timeout_seconds] 'command to run' [args...]",
            file=sys.stderr,
        )
        sys.exit(1)

    try:
        timeout_sec = float(sys.argv[1])
    except ValueError:
        print("Error: timeout must be a number", file=sys.stderr)
        sys.exit(1)

    cmd_args = sys.argv[2:]
    validate_command(cmd_args)

    print(
        f"🛡️ [Python] Running detached command with {timeout_sec}s timeout: {cmd_args}",
        file=sys.stderr,
    )

    # Use a PTY to force child processes to use line-buffering
    master_fd, slave_fd = pty.openpty()

    stop_read_event = threading.Event()
    read_thread = threading.Thread(target=read_from_pty, args=(master_fd, stop_read_event))
    read_thread.start()

    # Hybrid execution: if passed as a single string, assume shell string.
    use_shell = len(cmd_args) == 1
    exec_cmd = cmd_args[0] if use_shell else cmd_args

    try:
        proc = subprocess.Popen(
            exec_cmd,
            shell=use_shell,
            stdout=slave_fd,
            stderr=subprocess.STDOUT,
            stdin=subprocess.DEVNULL,
            start_new_session=True,
            close_fds=True,
        )  # nosec
    except FileNotFoundError:
        print(f"Error: Command not found: {cmd_args[0]}", file=sys.stderr)
        stop_read_event.set()
        read_thread.join()
        with contextlib.suppress(OSError):
            os.close(slave_fd)
            os.close(master_fd)
        sys.exit(127)

    # Close slave fd in parent, so only child has it
    os.close(slave_fd)

    def cleanup_and_exit(signum: int, frame: FrameType | None) -> None:  # pragma: no cover
        print("\n⚠️ Interrupted. Killing process tree...", file=sys.stderr)
        kill_process_tree(proc.pid)
        stop_read_event.set()
        read_thread.join()
        os.close(master_fd)
        sys.exit(130)

    signal.signal(signal.SIGINT, cleanup_and_exit)  # pragma: no cover
    signal.signal(signal.SIGTERM, cleanup_and_exit)  # pragma: no cover

    exit_code = 1
    try:
        proc.wait(timeout=timeout_sec)
        exit_code = proc.returncode
    except subprocess.TimeoutExpired:
        print(
            f"\n⏳ Timeout ({timeout_sec}s) reached. Killing process tree...",
            file=sys.stderr,
        )
        kill_process_tree(proc.pid)
        exit_code = 124  # standard timeout exit code
    finally:
        # Cleanup
        stop_read_event.set()
        read_thread.join()
        with contextlib.suppress(OSError):
            os.close(master_fd)

    print(f"✅ [Python] Command finished with exit code {exit_code}", file=sys.stderr)
    sys.exit(exit_code)


if __name__ == "__main__":
    main()  # pragma: no cover
