from pathlib import Path

import psutil


def kill_hung_processes() -> None:
    killed_count = 0

    project_root = str(Path(__file__).resolve().parent.parent.parent)

    for p in psutil.process_iter(["pid", "name", "cmdline"]):
        try:
            cmdline = p.info["cmdline"]
            if not cmdline:
                continue

            cmd_str = " ".join(cmdline)

            # Be careful not to kill this script itself, or system processes
            if "kill_hung_processes.py" in cmd_str:
                continue

            # Target patterns for ghost processes
            targets = [
                "-c" in cmd_str and ("python" in cmd_str or "python3" in cmd_str),
                "safe-run" in cmd_str or "_safe_run.py" in cmd_str,
                "orchestrator" in cmd_str,
                "podman" in cmd_str and project_root in cmd_str,
                "conmon" in cmd_str and project_root in cmd_str,
                "bun" in cmd_str and project_root in cmd_str,
                "node" in cmd_str and project_root in cmd_str,
                "git" in cmd_str and project_root in cmd_str,
            ]

            if any(targets):
                print(f"Terminating PID {p.info['pid']} and its children: {cmd_str[:100]}...")

                # Recursively kill children
                try:
                    children = p.children(recursive=True)
                    for child in children:
                        print(f"  └─ Killing child PID {child.pid}: {child.name()}")
                        child.kill()
                        killed_count += 1
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass

                p.kill()
                killed_count += 1

        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            pass

    print(f"\n✨ Terminated {killed_count} ghost processes (including descendants).")


if __name__ == "__main__":
    kill_hung_processes()
