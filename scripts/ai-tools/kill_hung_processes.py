import psutil


def kill_hung_processes() -> None:
    killed_count = 0
    for p in psutil.process_iter(["pid", "name", "cmdline"]):
        try:
            cmdline = p.info["cmdline"]
            if not cmdline:
                continue

            cmd_str = " ".join(cmdline)

            # Be careful not to kill this script itself, or system processes
            if "kill_hung_processes.py" in cmd_str:
                continue

            # Find hung python -c or safe-run processes
            if ("python" in cmd_str or "python3" in cmd_str) and (
                "-c" in cmd_str or "safe-run" in cmd_str
            ):
                print(f"Terminating PID {p.info['pid']}: {cmd_str[:100]}...")
                p.kill()
                killed_count += 1

        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            pass

    print(f"\n✨ Terminated {killed_count} ghost processes.")


if __name__ == "__main__":
    kill_hung_processes()
