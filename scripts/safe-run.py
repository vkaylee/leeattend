#!/usr/bin/env python3
"""
safe-run.py

A bulletproof wrapper to run commands without leaking PTY file descriptors.
Replaces safe-run.sh with robust Python process management.
Optimized to use PTY for real-time line buffering while preventing background hang.
"""

import sys
import os
import subprocess
import threading
import signal
import psutil
import pty
import select
from typing import List, Optional
from types import FrameType

def read_from_pty(master_fd: int, stop_event: threading.Event) -> None:
    """Reads from the PTY master until stop_event is set and no more data is available."""
    while not stop_event.is_set():
        # Wait up to 0.1s for data to become available
        r, _, _ = select.select([master_fd], [], [], 0.1)
        if master_fd in r:
            try:
                data = os.read(master_fd, 4096)
                if not data:
                    break # EOF
                # Write directly to sys.stdout's underlying buffer to avoid encoding issues
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
            except OSError:
                break # EIO (Input/output error) means slave is closed
                
    # One last non-blocking read to drain buffer
    while True:
        r, _, _ = select.select([master_fd], [], [], 0.0)
        if master_fd in r:
            try:
                data = os.read(master_fd, 4096)
                if not data:
                    break
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
            except OSError:
                break
        else:
            break

def kill_process_tree(pid: int) -> None:
    """Kills a process and all its descendants using psutil."""
    try:
        parent: psutil.Process = psutil.Process(pid)
        children: List[psutil.Process] = parent.children(recursive=True)
        # Send SIGTERM to children
        for child in children:
            try:
                child.terminate()
            except psutil.NoSuchProcess:
                pass
        
        # Send SIGTERM to parent
        try:
            parent.terminate()
        except psutil.NoSuchProcess:
            pass
            
        # Wait for them to die
        gone, alive = psutil.wait_procs(children + [parent], timeout=3.0)
        
        # Force kill if still alive
        for p in alive:
            try:
                p.kill()
            except psutil.NoSuchProcess:
                pass
    except psutil.NoSuchProcess:
        pass

def main() -> None:
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} [timeout_seconds] 'command to run' [args...]", file=sys.stderr)
        sys.exit(1)

    try:
        timeout_sec = float(sys.argv[1])
    except ValueError:
        print("Error: timeout must be a number", file=sys.stderr)
        sys.exit(1)

    cmd_args = sys.argv[2:]

    print(f"🛡️ [Python] Running detached command with {timeout_sec}s timeout: {cmd_args}", file=sys.stderr)

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
            close_fds=True
        )
    except FileNotFoundError:
        print(f"Error: Command not found: {cmd_args[0]}", file=sys.stderr)
        os.close(slave_fd)
        os.close(master_fd)
        sys.exit(127)
    
    # Close slave fd in parent, so only child has it
    os.close(slave_fd)
    
    def cleanup_and_exit(signum: int, frame: Optional[FrameType]) -> None:
        print("\n⚠️ Interrupted. Killing process tree...", file=sys.stderr)
        kill_process_tree(proc.pid)
        stop_read_event.set()
        read_thread.join()
        os.close(master_fd)
        sys.exit(130)

    signal.signal(signal.SIGINT, cleanup_and_exit)
    signal.signal(signal.SIGTERM, cleanup_and_exit)

    exit_code = 1
    try:
        proc.wait(timeout=timeout_sec)
        exit_code = proc.returncode
    except subprocess.TimeoutExpired:
        print(f"\n⏳ Timeout ({timeout_sec}s) reached. Killing process tree...", file=sys.stderr)
        kill_process_tree(proc.pid)
        exit_code = 124 # standard timeout exit code
    finally:
        # Cleanup
        stop_read_event.set()
        read_thread.join()
        try:
            os.close(master_fd)
        except OSError:
            pass

    print(f"✅ [Python] Command finished with exit code {exit_code}", file=sys.stderr)
    sys.exit(exit_code)

if __name__ == "__main__":
    main()
