import psutil
import sys

def kill_hung_processes():
    killed_count = 0
    for p in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            cmdline = p.info['cmdline']
            if not cmdline:
                continue
            
            cmd_str = " ".join(cmdline)
            
            # Cẩn thận không giết chính script này, hoặc các tiến trình hệ thống
            if "kill_hung_processes.py" in cmd_str:
                continue
                
            # Tìm các tiến trình python -c hoặc safe-run bị treo
            if ("python" in cmd_str or "python3" in cmd_str) and ("-c" in cmd_str or "safe-run" in cmd_str):
                print(f"Bóp cổ PID {p.info['pid']}: {cmd_str[:100]}...")
                p.kill()
                killed_count += 1
                
        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            pass
            
    print(f"\n✨ Đã tiêu diệt {killed_count} tiến trình ma.")

if __name__ == "__main__":
    kill_hung_processes()
