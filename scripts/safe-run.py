#!/usr/bin/env python3
"""
safe-run.py

A bulletproof wrapper to run commands without leaking PTY file descriptors.
Replaces safe-run.sh with robust Python process management.
"""

import sys
import os
import subprocess
import tempfile
import threading
import time
import signal
import psutil
from typing import List, Optional, Any
from types import FrameType

def tail_file(filepath: str, stop_event: threading.Event) -> None:
    """Tails a file to stdout until stop_event is set."""
    # Wait for file to be created
    while not os.path.exists(filepath) and not stop_event.is_set():
        time.sleep(0.05)
        
    if not os.path.exists(filepath):
        return

    with open(filepath, 'r') as f:
        while not stop_event.is_set():
            line: str = f.readline()
            if line:
                sys.stdout.write(line)
                sys.stdout.flush()
            else:
                time.sleep(0.05)
        
        # Read remaining lines
        for line in f.readlines():
            sys.stdout.write(line)
            sys.stdout.flush()

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

    # Create a temporary file that is guaranteed to be deleted when closed/garbage collected
    fd, logpath = tempfile.mkstemp(prefix="safe-run-", suffix=".log")
    
    # We will open it for writing by the subprocess
    log_fp = os.fdopen(fd, 'w')
    
    stop_tail_event = threading.Event()
    tail_thread = threading.Thread(target=tail_file, args=(logpath, stop_tail_event))
    tail_thread.start()

    # Execute directly like a standard unix wrapper (e.g. timeout, nohup)
    # Hybrid execution: if passed as a single string, assume shell string.
    # If passed as an array (e.g. from _cargo.sh), execute directly without shell.
    use_shell = len(cmd_args) == 1
    exec_cmd = cmd_args[0] if use_shell else cmd_args
    
    try:
        proc = subprocess.Popen(
            exec_cmd,
            shell=use_shell,
            stdout=log_fp,
            stderr=subprocess.STDOUT,
            stdin=subprocess.DEVNULL,
            start_new_session=True,
            close_fds=True
        )
    except FileNotFoundError:
        print(f"Error: Command not found: {cmd_args[0]}", file=sys.stderr)
        log_fp.close()
        os.remove(logpath)
        sys.exit(127)
    
    def cleanup_and_exit(signum: int, frame: Optional[FrameType]) -> None:
        print("\n⚠️ Interrupted. Killing process tree...", file=sys.stderr)
        kill_process_tree(proc.pid)
        stop_tail_event.set()
        tail_thread.join()
        log_fp.close()
        os.remove(logpath)
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
        stop_tail_event.set()
        tail_thread.join()
        log_fp.close()
        try:
            os.remove(logpath)
        except OSError:
            pass

    print(f"✅ [Python] Command finished with exit code {exit_code}", file=sys.stderr)
    sys.exit(exit_code)

if __name__ == "__main__":
    main()
