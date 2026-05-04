"""Internal watchdog spawned by _lifecycle.start_watchdog().

Kills a given parent PID after a timeout. All FDs severed
so the watchdog itself never causes a PTY hang.
"""

import contextlib
import os
import signal
import sys
import time


def main() -> None:
    if len(sys.argv) < 3:
        sys.exit(1)
    timeout_sec = int(sys.argv[1])
    parent_pid = int(sys.argv[2])

    time.sleep(timeout_sec)

    for sig in (signal.SIGKILL, signal.SIGKILL):
        with contextlib.suppress(ProcessLookupError):
            os.kill(parent_pid, sig)
        time.sleep(2)


if __name__ == "__main__":
    main()
