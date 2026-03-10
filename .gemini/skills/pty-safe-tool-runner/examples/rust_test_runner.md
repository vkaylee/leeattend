# Example: Rust Test Runner (PTY-Safe)

## Scenario
AI được yêu cầu tạo script chạy Rust unit tests và capture output.

## ❌ UNSAFE Version (AI hay sinh ra)

```bash
#!/bin/bash
echo "Running tests..."
cargo test 2>&1 | tee test-results.log
echo "Exit: $?"
```

**Vấn đề:** `| tee` tạo pipe FD. Với một số build tools, children có thể
kế thừa và giữ pipe open. Còn `$?` sau pipe captures exit code của `tee`,
không phải `cargo test`.

## ✅ SAFE Version

```bash
#!/usr/bin/env bash
# PTY-SAFE: Temporary script — will be deleted after execution
set -euo pipefail
[ -t 0 ] && exec </dev/null

# Self-cleanup: xóa script này sau khi chạy xong hoặc gặp lỗi
SCRIPT_PATH="$(realpath "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

LOG="/tmp/rust-tests-$$.log"

echo "🤖 [AI AGENT NOTICE]: Rust unit tests starting — may take 2-3 minutes."
echo "🤖 [AI AGENT NOTICE]: NOT a PTY hang. Wait for exit code."

cargo test >"$LOG" 2>&1
EXIT_CODE=$?

echo "--- Test Results ---"
cat "$LOG"
echo "--- Exit: $EXIT_CODE ---"

exit $EXIT_CODE
```

## Cách chạy từ AI agent

```bash
timeout 300 ./run-tests.sh </dev/null
```
