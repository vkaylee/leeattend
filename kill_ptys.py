import os
import signal
import subprocess

def kill_hanging_ptys():
    try:
        output = subprocess.check_output(['ps', 'ax', '-o', 'pid,command'], universal_newlines=True)
        for line in output.split('\n'):
            if not line.strip() or 'PID' in line: continue
            parts = line.strip().split(maxsplit=1)
            if len(parts) < 2: continue
            pid_str, cmd = parts
            
            # Avoid killing ourselves
            if 'kill_ptys.py' in cmd or 'ps ax' in cmd:
                continue
                
            # Targets based on user prompt
            targets = ['python3 -c', 'git.sh', 'cat <<', 'echo "refactor(apiserver)', 'EOF']
            for t in targets:
                if t in cmd:
                    try:
                        pid = int(pid_str)
                        print(f"Killing PID {pid}: {cmd}")
                        os.kill(pid, signal.SIGKILL)
                    except Exception as e:
                        print(f"Failed to kill {pid_str}: {e}")
    except Exception as e:
        print(f"Error listing processes: {e}")

if __name__ == '__main__':
    kill_hanging_ptys()
