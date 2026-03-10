# Example: Podman Container Tests (PTY-Safe)

## Scenario
AI chạy integration tests bên trong Podman container qua `docker-compose exec`.

## ❌ UNSAFE Version

```bash
#!/bin/bash
podman-compose exec apiserver cargo test -- --test-threads=1 2>&1 | tee out.log
echo "Done"
```

**Vấn đề kép:**
1. `podman exec` không có `-T` → allocate TTY không cần, AI agent đã có PTY rồi
2. `| tee` → pipe FD leak sang container process
3. Container exec giữ PTY open → terminal **hang vĩnh viễn** sau khi xong

## ✅ SAFE Version

```bash
#!/usr/bin/env bash
# PTY-SAFE: Temporary script — will be deleted after execution
set -euo pipefail
[ -t 0 ] && exec </dev/null

# Self-cleanup: xóa script này sau khi chạy xong hoặc gặp lỗi
SCRIPT_PATH="$(realpath "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

LOG="/tmp/integration-tests-$$.log"

echo "🤖 [AI AGENT NOTICE]: Integration tests starting in Podman container."
echo "🤖 [AI AGENT NOTICE]: EXPECTED DURATION: up to 5 minutes."
echo "🤖 [AI AGENT NOTICE]: NOT a PTY hang. Wait patiently for exit code."

podman-compose exec -T apiserver \
    cargo test -- --test-threads=1 \
    >"$LOG" 2>&1 </dev/null

EXIT_CODE=$?

echo "--- Integration Test Results ---"
cat "$LOG"
echo "--- Exit: $EXIT_CODE ---"

exit $EXIT_CODE
```

## Key differences
- `-T` flag: Disable pseudo-TTY allocation
- `</dev/null`: Detach stdin từ parent PTY  
- Direct redirect `>$LOG`: Không dùng pipe

## Cách chạy từ AI agent

```bash
timeout 600 ./integration-tests.sh </dev/null
```
