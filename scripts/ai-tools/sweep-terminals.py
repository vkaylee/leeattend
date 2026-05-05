#!/usr/bin/env python3
"""
sweep-terminals.py

A utility to detect and clean up orphaned interactive PTY sessions (User Terminals)
that may be hanging due to incomplete shell commands.
"""

import argparse
import os
import signal
import sys
import time

try:
    import psutil
except ImportError:
    print("Error: psutil is required. Run: pip install psutil", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(description="Sweep orphaned User Terminals (PTYs)")
    parser.add_argument("--kill", nargs="+", type=int, help="Kill specific PIDs")
    parser.add_argument(
        "--kill-all", action="store_true", help="Kill ALL bash PTYs (except current)"
    )
    parser.add_argument(
        "--older-than", type=int, default=0, help="Only list/kill sessions older than X minutes"
    )
    args = parser.parse_args()

    current_pid = os.getpid()
    # Find the bash process that is the parent of this script
    parent_bash_pids = set()
    try:
        p = psutil.Process(current_pid)
        while p:
            if p.name() == "bash":
                parent_bash_pids.add(p.pid)
            p = p.parent()
    except (psutil.NoSuchProcess, psutil.AccessDenied):
        pass

    now = time.time()
    terminals = []

    for p in psutil.process_iter(["pid", "terminal", "status", "cmdline", "name", "create_time"]):
        try:
            tty = p.info["terminal"]
            if tty and "pts/" in tty and "bash" in p.info["name"]:
                age_minutes = (now - p.info["create_time"]) / 60.0

                if args.older_than > 0 and age_minutes < args.older_than:
                    continue

                terminals.append(
                    {
                        "pid": p.info["pid"],
                        "tty": tty,
                        "status": p.info["status"],
                        "age": age_minutes,
                        "cmd": " ".join(p.info.get("cmdline", [])),
                        "is_current": p.info["pid"] in parent_bash_pids,
                    }
                )
        except (psutil.NoSuchProcess, psutil.AccessDenied, TypeError):
            pass

    if not terminals:
        print("No interactive terminals found matching the criteria.")
        return

    # Handle Kill actions
    if args.kill or args.kill_all:
        to_kill = []
        if args.kill:
            to_kill = [t for t in terminals if t["pid"] in args.kill]
        elif args.kill_all:
            to_kill = [t for t in terminals if not t["is_current"]]

        if not to_kill:
            print("No matching processes found to kill (current terminal is protected).")
            return

        print(f"🧹 Sweeping {len(to_kill)} terminal(s)...")
        for t in to_kill:
            try:
                os.kill(t["pid"], signal.SIGKILL)
                print(f"  [OK] Killed PID {t['pid']} ({t['tty']})")
            except Exception as e:
                print(f"  [FAIL] Could not kill PID {t['pid']}: {e}")
        return

    # List actions
    print(f"{'PID':<8} {'TTY':<12} {'STATUS':<10} {'AGE (min)':<10} {'COMMAND'}")
    print("-" * 80)
    for t in sorted(terminals, key=lambda x: x["age"], reverse=True):
        marker = "⭐ (You)" if t["is_current"] else ""
        print(
            f"{t['pid']:<8} {t['tty']:<12} {t['status']:<10} {t['age']:<10.1f} {t['cmd'][:30]} {marker}"
        )

    print("-" * 80)
    print("💡 To clean up hanging terminals, run:")
    print("   ./scripts/ai-tools/sweep-terminals.py --kill <PID1> <PID2>")
    print("   ./scripts/ai-tools/sweep-terminals.py --kill-all --older-than 60")


if __name__ == "__main__":
    main()
