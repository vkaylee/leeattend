# PTY Safety Quick Reference

Bảng tra nhanh cho AI — dùng trước khi tạo/chạy bất kỳ script nào.

## 🔴 Anti-Patterns → Fix ngay

| Anti-pattern | Vấn đề | Fix |
|---|---|---|
| `cmd &` | bare background, PTY FD leak | `cmd </dev/null &` |
| `cmd \| tee file` | pipe FD leak vào children | `cmd >file 2>&1` |
| `podman exec svc cmd` | inherit PTY + block | `podman exec -T svc cmd </dev/null` |
| `docker exec svc cmd` | inherit PTY + block | `docker exec -T svc cmd </dev/null` |
| `wait` (no PID) | block on background daemons | `wait $SPECIFIC_PID` |
| `fzf`, `vim`, `less` | interactive, hang 100% | ❌ Không chạy trong agentic |

## ✅ Safe Patterns — Copy & Paste Ready

### Single command (không spawn children):
```bash
timeout 300 cargo test >/tmp/out.log 2>&1 </dev/null
```

### Container command:
```bash
podman-compose exec -T SERVICE_NAME CMD ARG >/tmp/out.log 2>&1 </dev/null
```

### Background job:
```bash
./script.sh >/tmp/out.log 2>&1 </dev/null &
PID=$!
# ... do other work ...
wait $PID
```

### Parallel jobs:
```bash
job_a >/tmp/a.log 2>&1 </dev/null & PID_A=$!
job_b >/tmp/b.log 2>&1 </dev/null & PID_B=$!
wait $PID_A; STATUS_A=$?
wait $PID_B; STATUS_B=$?
```

### Thay thế pipe | tee (khi cần live view):
```bash
touch /tmp/out.log
tail -f /tmp/out.log </dev/null &
TAIL_PID=$!
./script.sh >/tmp/out.log 2>&1 </dev/null
kill $TAIL_PID 2>/dev/null
wait $TAIL_PID 2>/dev/null
```

## 🤖 Heartbeat Template (cho scripts > 30 giây)

```bash
echo "🤖 [AI AGENT NOTICE]: Script X đang chạy — TỐI ĐA Y PHÚT."
echo "🤖 [AI AGENT NOTICE]: KHÔNG phải PTY hang. Chờ exit code."
```

## 📝 Script Header Template

```bash
#!/usr/bin/env bash
# PTY-SAFE: Designed for AI agentic terminals
set -euo pipefail
[ -t 0 ] && exec </dev/null  # Defensive stdin detach
```

## Decision Tree

```
Cần chạy command?
├── Có spawn container? → Thêm -T + </dev/null
├── Có background (&)? → Thêm </dev/null, lưu PID
├── Có pipe | tee? → Đổi thành >file redirect  
├── Có interactive? → ❌ KHÔNG chạy trong agentic
├── Chạy > 30s? → Thêm heartbeat + timeout
└── Còn lại → Safe, chạy bình thường
```
