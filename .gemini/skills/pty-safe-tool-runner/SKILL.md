---
name: pty-safe-tool-runner
description: |
  Hướng dẫn AI tạo và chạy shell scripts/tools PTY-safe, tránh hang terminal
  trong agentic workflows. Dùng khi AI tạo script mới, chạy container
  (Podman/Docker), background job, pipe, git hooks, CI/CD. Trigger khi user
  nói "tạo script", "chạy tool", "run tests", "start container", hoặc khi AI
  tự sinh script helper rồi execute nó.
---

# Goal

Đảm bảo mọi script/tool mà AI tạo ra và chạy đều **không gây PTY hang** —
terminal không bao giờ bị block, AI agentic luôn nhận được exit code và tiếp
tục workflow tự động.

---

# Why This Matters

Khi AI chạy trong môi trường agentic (Antigravity, Claude, Cursor...), AI
giao tiếp qua PTY (Pseudo-Terminal). Nếu một script con **kế thừa PTY file
descriptor (FD)** mà không đóng nó đúng cách, terminal sẽ **treo vô thời
hạn** ngay cả sau khi process chính đã xong.

Root causes phổ biến:
- Background process `&` không redirect stdin (kế thừa PTY FD của parent)
- Pipe `| tee` trong automated scripts (pipe FD leak sang container children)
- Podman/Docker exec không có `-T` flag (allocate TTY không cần thiết)
- `wait` không có PID cụ thể (block mãi nếu có background daemon)
- **Terminate git commit khi hook đang chạy** → tạo zombie processes giữ PTY,
  block tất cả git commands sau đó. Recovery: `pkill -9 -f 'git commit'` + `rm -f .git/index.lock`

**⛔ HARD CONSTRAINT: NEVER terminate/kill a running `git commit`.**
If a commit hangs, wait for timeout or ask the user to kill it manually.
Terminating mid-hook creates zombie processes that hold PTY file descriptors
and block ALL subsequent git operations in the same terminal session.

---

# Instructions

## Step 1: Identify PTY hang risk

Before creating or running any script, check this risk table:

| Situation | PTY Hang Risk | Action |
|---|---|---|
| Command with `&` (background) | 🔴 HIGH | Add `</dev/null` before `&` |
| `podman`/`docker` exec | 🔴 HIGH | Add `-T` + `</dev/null` |
| Pipe with `\| tee` | 🔴 HIGH | Replace with direct file redirect |
| `npm run`/`npx` with watch mode | 🔴 HIGH | Add `CI=true` or `--no-interactive` |
| `python -m` spawning subprocess | 🟡 MEDIUM | Add `</dev/null`, verify `subprocess.DEVNULL` |
| `wait` without specific PID | 🟡 MEDIUM | Use `wait $PID` with explicit PID |
| Interactive commands (fzf, vim, less) | 🔴 HIGH | Never run in agentic context |
| Long-running script (>30s) | 🟡 MEDIUM | Add heartbeat logging |
| Terminate `git commit` mid-hook | 🔴 CRITICAL | NEVER kill — creates zombies holding PTY |

## Step 2: Apply PTY-safe patterns

### Pattern A — Background jobs (mandatory)

```bash
# ❌ UNSAFE: PTY FD leaks into background process
some-long-command &

# ✅ SAFE: Detach stdin from PTY
some-long-command </dev/null &
BG_PID=$!
# ... do other work ...
wait $BG_PID
```

### Pattern B — Container execution (mandatory)

```bash
# ❌ UNSAFE: Inherits PTY + may block
podman-compose exec apiserver cargo test
docker exec mycontainer ./run.sh

# ✅ SAFE: Explicit TTY disable + stdin detach
podman-compose exec -T apiserver cargo test </dev/null
docker exec -T mycontainer ./run.sh </dev/null
```

### Pattern C — Replace tee pipes entirely

```bash
# ❌ UNSAFE: Pipe FD leaks into container children
./run-tests.sh | tee logs.txt

# ✅ SAFE: Direct redirect + optional live tail
touch logs.txt
tail -f logs.txt </dev/null &
TAIL_PID=$!
./run-tests.sh >logs.txt 2>&1
kill $TAIL_PID 2>/dev/null
wait $TAIL_PID 2>/dev/null
```

### Pattern D — PTY-safe script template (use when creating new scripts)

```bash
#!/usr/bin/env bash
# PTY-SAFE: Temporary script — will be deleted after execution
set -euo pipefail

# Defensive stdin detach
[ -t 0 ] && exec </dev/null

# Self-cleanup on exit (deletes this script after success or error)
# fallback if realpath unavailable: SCRIPT_PATH="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
SCRIPT_PATH="$(realpath "$0" 2>/dev/null || echo "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

LOG_FILE="/tmp/$(basename "$0" .sh)-$$.log"

echo "🤖 [AI AGENT NOTICE]: Script $(basename "$0") is running — DO NOT interrupt."
echo "🤖 [AI AGENT NOTICE]: Log at $LOG_FILE"

main() {
    # ... your logic here ...
    echo "✅ Done"
}

main "$@" >"$LOG_FILE" 2>&1
cat "$LOG_FILE"
```

> **`trap 'rm -f "$SCRIPT_PATH"' EXIT`** ensures the script self-deletes on both
> normal exit and errors. The log file is kept in `/tmp/` for debugging if needed.

## Step 3: Cleanup after execution (mandatory for temporary scripts)

Any one-off script created by the AI **must delete itself** after running:

```bash
# Method 1: trap self-cleanup (recommended — built into script template)
SCRIPT_PATH="$(realpath "$0" 2>/dev/null || echo "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

# Method 2: Manual cleanup by AI immediately after running
timeout 300 ./tmp-script.sh </dev/null
rm -f ./tmp-script.sh

# Method 3: Use /tmp/ with PID in filename — OS auto-cleans on reboot
SCRIPT="/tmp/ai-task-$$.sh"
cat >"$SCRIPT" <<'EOF'
  # ... script content ...
EOF
chmod +x "$SCRIPT"
timeout 300 "$SCRIPT" </dev/null
rm -f "$SCRIPT"
```

> **Why this matters:** Leftover scripts accumulate and confuse both AI and users
> in future sessions — the AI may accidentally re-execute stale scripts.

## Step 4: Add AI Agent heartbeat for long-running scripts (>30 seconds)

Long-running scripts must emit heartbeat messages so the AI agent does not
mistake them for PTY hangs:

```bash
echo "🤖 [AI AGENT NOTICE]: Starting integration tests — MAY TAKE UP TO 5 MINUTES."
echo "🤖 [AI AGENT NOTICE]: THIS IS NOT A PTY HANG. Wait for the exit code."
```

## Step 5: Verify command/script before executing

Use `scripts/test-pty-safety.sh` to validate safety BEFORE running:

```bash
# Check a single command string:
./scripts/test-pty-safety.sh --command="podman-compose exec apiserver cargo test"
# Exit 0 = SAFE, Exit 1 = UNSAFE (violations printed to stderr)

# Check an entire script file:
./scripts/test-pty-safety.sh --check-file=./my-script.sh

# Gate pattern — only run if pre-check passes:
if ./scripts/test-pty-safety.sh --command="$CMD" 2>/dev/null; then
    eval "$CMD"
else
    echo "PTY safety violation — fix before running"
fi
```

The checker validates 15 patterns including: bare `&`, container exec without
`-T`, missing `</dev/null`, `| tee` pipes, `nohup`, interactive commands,
`wait` without PID, `disown`, `coproc`, `setsid`, and more.

Run `./scripts/test-pty-safety.sh --help` for full documentation.

## Step 6: Run script with safeguards

```bash
# Always add </dev/null and timeout when running scripts from AI agent
timeout 300 ./my-script.sh </dev/null

# To also capture output:
timeout 300 ./my-script.sh </dev/null 2>&1
```

## Step 7: Detect PTY hang vs normal execution

AI agents cannot distinguish a long-running command from a PTY hang.
Use this **PTY-safe heartbeat loop** to auto-update a marker file:

```bash
# 1. Start heartbeat (PTY-safe + self-killing when parent dies)
HB="/tmp/.pty-heartbeat-$$"
PARENT_PID=$$
( while kill -0 $PARENT_PID 2>/dev/null; do date +%s > "$HB"; sleep 2; done; rm -f "$HB" ) </dev/null >/dev/null 2>&1 &
HB_PID=$!

# 2. Run the actual command
git -c commit.gpgsign=false commit --no-verify -m "msg" </dev/null
CMD_EXIT=$?

# 3. Cleanup heartbeat (explicit PID, no orphans)
kill $HB_PID 2>/dev/null; wait $HB_PID 2>/dev/null; rm -f "$HB"
exit $CMD_EXIT
```

Why this is safe:
- `</dev/null >/dev/null 2>&1 &` — all 3 FDs detached, no PTY inheritance
- `kill -0 $PARENT_PID` — loop self-exits when parent process dies (crash, disconnect)
- `rm -f "$HB"` in loop — auto-cleanup even without explicit kill
- `kill $HB_PID` + `wait $HB_PID` — belt-and-suspenders cleanup on normal exit

**AI agent detection** — check heartbeat file from a separate command:

```bash
# Read heartbeat timestamp:
cat /tmp/.pty-heartbeat-*
# Compare with current time:
echo $(( $(date +%s) - $(cat /tmp/.pty-heartbeat-*) ))
```

| Heartbeat age | `PTY_DONE` in file? | Diagnosis | Action |
|---|---|---|---|
| < 5 seconds | — | Running normally | Keep waiting |
| > 10 seconds | No | **PTY hang likely** | Ask user to intervene |
| File gone | — | Command finished | Check exit code normally |

**⛔ CRITICAL: When you suspect a hang, NEVER terminate the process.**
Ask the user to investigate or kill it manually. Terminating may create
zombie processes that cascade into more hangs.

---

# Examples

## Example 1: AI tạo test runner script

**Yêu cầu:** "Tạo script chạy unit tests cho Rust project"

**❌ Script nguy hiểm (AI hay tạo sai):**
```bash
#!/bin/bash
cargo test 2>&1 | tee test-results.log
echo "Done: $?"
```

**✅ Script PTY-safe đúng cách:**
```bash
#!/usr/bin/env bash
# PTY-SAFE: Temporary script — will be deleted after execution
set -euo pipefail
[ -t 0 ] && exec </dev/null

SCRIPT_PATH="$(realpath "$0" 2>/dev/null || echo "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

LOG="/tmp/rust-tests-$$.log"
echo "🤖 [AI AGENT NOTICE]: Running Rust tests — may take 2-3 minutes."

cargo test >"$LOG" 2>&1
EXIT_CODE=$?

cat "$LOG"
echo "Exit: $EXIT_CODE"
exit $EXIT_CODE
```

**Tại sao:** `cargo test` là process đơn, không spawn container, nên direct
redirect (`>`) thay vì pipe là safe và đủ. `trap` tự dọn script sau khi xong.

---

## Example 2: AI chạy integration tests qua Podman

**Yêu cầu:** "Chạy integration tests trong container"

**❌ Nguy hiểm:**
```bash
podman-compose exec apiserver cargo test -- --test-threads=1 | tee out.log
```

**✅ PTY-safe:**
```bash
LOG="integration-test.log"
echo "🤖 [AI AGENT NOTICE]: Integration tests đang chạy trong container."
echo "🤖 [AI AGENT NOTICE]: CÓ THỂ MẤT ĐẾN 5 PHÚT. Đừng interrupt."

podman-compose exec -T apiserver cargo test -- --test-threads=1 \
    >"$LOG" 2>&1 </dev/null
EXIT=$?

cat "$LOG"
exit $EXIT
```

---

## Example 3: AI cần chạy job song song

**❌ Nguy hiểm:**
```bash
./build.sh &
./lint.sh &
wait
```

**✅ PTY-safe:**
```bash
./build.sh </dev/null >"build.log" 2>&1 &
BUILD_PID=$!

./lint.sh </dev/null >"lint.log" 2>&1 &
LINT_PID=$!

wait $BUILD_PID; BUILD_EXIT=$?
wait $LINT_PID;  LINT_EXIT=$?

cat build.log
cat lint.log

[ $BUILD_EXIT -eq 0 ] && [ $LINT_EXIT -eq 0 ]
```

---

## Example 4: AI chạy Node.js / npm scripts

**Yêu cầu:** "Chạy tests cho frontend app"

**❌ Nguy hiểm:**
```bash
npm test
npm run build 2>&1 | tee build.log
```

**✅ PTY-safe:**
```bash
#!/usr/bin/env bash
# PTY-SAFE: Temporary script — will be deleted after execution
set -euo pipefail
[ -t 0 ] && exec </dev/null

SCRIPT_PATH="$(realpath "$0" 2>/dev/null || echo "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

LOG="/tmp/npm-test-$$.log"
echo "🤖 [AI AGENT NOTICE]: Running npm tests — may take 2-5 minutes."

# CI=true: tắt watch mode + interactive prompts của đa số npm tools
CI=true npm test >"$LOG" 2>&1
EXIT_CODE=$?

cat "$LOG"
echo "Exit: $EXIT_CODE"
exit $EXIT_CODE
```

**Tại sao `CI=true`:** Các tool như Jest, Create React App, Vitest tự detect
env var `CI` để tắt interactive/watch mode — tự động PTY-safe mà không cần flag.

---

## Example 5: AI chạy Python script có subprocess

**Yêu cầu:** "Chạy migration script Python"

**❌ Nguy hiểm:**
```bash
python -m scripts.migrate
# Hoặc nếu Python script nội bộ dùng subprocess.Popen() không có stdin=DEVNULL
```

**✅ PTY-safe (gọi từ shell):**
```bash
#!/usr/bin/env bash
# PTY-SAFE: Temporary script — will be deleted after execution
set -euo pipefail
[ -t 0 ] && exec </dev/null

SCRIPT_PATH="$(realpath "$0" 2>/dev/null || echo "$0")"
trap 'rm -f "$SCRIPT_PATH"' EXIT

LOG="/tmp/py-migrate-$$.log"
echo "🤖 [AI AGENT NOTICE]: Running Python migration script."

# Chạy với stdin từ /dev/null — ngăn Python script đọc input tự PTY
python -m scripts.migrate >"$LOG" 2>&1 </dev/null
EXIT_CODE=$?

cat "$LOG"
echo "Exit: $EXIT_CODE"
exit $EXIT_CODE
```

**Lưu ý nếu Python script tự tạo subprocess:** Đảm bảo dùng
`subprocess.Popen(..., stdin=subprocess.DEVNULL)` trong code Python
— shell-level `</dev/null` chỉ protect tầng này, không propagate xuống.

---

# Constraints

## Hard constraints — no exceptions

- **NEVER** create a script with bare `cmd &` (without `</dev/null`) — this is
  the #1 cause of PTY hangs in agentic terminals
- **NEVER** use `nohup` — it detaches stdout/stderr and masks exit codes, making
  it impossible for the AI agent to detect failures or completion
- **NEVER** run interactive commands in agentic context: `fzf`, `vim`, `nano`,
  `less`, `more`, `interactive python/node/ruby REPL`
- **NEVER** use `wait` without a specific PID when background daemons exist
- **ALWAYS** add `-T` to `podman exec` and `docker exec` inside scripts
- **ALWAYS** self-delete temporary scripts using `trap 'rm -f "$SCRIPT_PATH"' EXIT`

## Defensive defaults — apply by default

- Any script running ≥30 seconds → add AI Agent heartbeat messages
- Any generated script → start with `# PTY-SAFE: ...` header
- When unsure → `timeout 300 cmd </dev/null` is always the safest option
- Prefer direct file redirect (`>file`) over pipe (`| tee`) for container calls
- Use `CI=true` for npm/node tools to suppress watch mode and interactive prompts

## When interactive commands cannot be avoided

Some tools inherently require a TTY (e.g. `ssh -t`, `gpg`, `sudo` with password).
When encountered:
1. Look for a non-interactive flag: `--no-tty`, `-y`, `--batch`, `--yes`, `--non-interactive`
2. If none exists → tell the user to run it manually; never run it inside an agentic workflow

<!-- Generated by Skill Creator Ultra v1.0 -->

