# AGENTS.md — Rules for AI Coding Assistants

> This file is read by AI coding tools (Claude Code, Cursor, Copilot, Windsurf,
> Gemini, etc.) to provide project-specific instructions.

## ⚠️ PTY Compatibility — CRITICAL for terminal commands

> [!CAUTION]
> When running commands via terminal tools, child processes can inherit PTY file
> descriptors and prevent command completion detection. The agent will hang
> waiting for a "done" status that never arrives.

**Root cause:** fd 0 (stdin), fd 1 (stdout), fd 2 (stderr) all point to the PTY slave
device. Child processes inherit these FDs. If any child outlives the parent with
PTY FDs open, the PTY master never sees EOF → infinite hang.

**Defense layers in this project:**

| Layer | Where | What it does |
|-------|-------|--------------|
| 0. `safe-run.py` | `scripts/safe-run.py` | **ULTIMATE WORKAROUND**: Isolates FDs to a file and uses python threading to detach from PTY. |
| 1. `dc()` wrapper | `scripts/docker-ops.sh` | All podman-compose via `dc`/`dc_detached` gets `</dev/null` automatically |
| 2. `dc_detached()` + setsid | `scripts/docker-ops.sh` | Cleanup ops: new session + full FD redirect + 60s hard timeout |
| 3. `cleanup()` 3-layer kill | `scripts/lifecycle.sh` | Process group kill + `/proc` FD sweep + **cleanup deadline watchdog** (120s) |
| 4. Global watchdog | `scripts/lifecycle.sh` (via `start_watchdog`) | Last-resort safety net: kills after `GLOBAL_TIMEOUT` |
| 5. PTY safety scanner | `scripts/test-pty-safety.sh` | Static scan (20 patterns) + runtime PTY tests |
| 6. Tracked background tails | `scripts/test-utils.sh` | All `tail -f` PIDs tracked and killed on task completion |

**ALWAYS append `</dev/null` to these commands:**

| Command | Why |
|---------|-----|
| `./git.sh commit`, `./git.sh push` | Replaces `git` directly. GPG agent and multi-line strings hold PTY |
| `podman-compose up/down/exec/build` | Podman CLI spawns children |
| Any script with `&` (background jobs) | Background process inherits PTY |
| Any `tail -f` in background | Pipe buffer + `grep` keeps PTY fd open |
| `git log`, `less`, `more` | Pager blocks waiting for user input. Use `--no-pager`. |
| `rg | awk`, `grep | sed` | Piped commands inherit PTY and hang without `</dev/null` |

**🚨 STRICTLY BANNED INLINE COMMANDS & PATTERNS (Deadlock Triggers):**
> **Do NOT use these via AI terminal tools.** They cause infinite PTY hangs due to unclosed quotes or waiting for stdin.
- **Banned Patterns**: Heredocs (`cat <<EOF` or `cat >> file <<EOF`)

```bash
# ✅ Correct — Use the git wrapper to avoid PTY hangs with multi-line messages
AGENT_MODE=1 ./git.sh commit -m "message"
podman-compose down </dev/null
./test.sh api </dev/null
git --no-pager log -n 5
./scripts/_python.sh my_script.py </dev/null

# ❌ Wrong — will hang
git commit -m "message"
podman-compose down
git log
python3 my_script.py
rg "pattern" | awk '{print $1}'
```

## 🧪 Testing — MANDATORY RULES

> [!WARNING]
> **NEVER run `cargo test`, `npm test`, `npx playwright test`, or any test command directly.** 
> This project uses Podman-based test isolation. The ONLY way to run tests is through `./test.sh`.
> 
> **NEVER run `cargo`, `npm`, `sqlx`, `python`, or `python3` directly.**
> ALWAYS use the hermetic toolbox wrappers:
> - `./scripts/_cargo.sh` for Rust
> - `./scripts/_npm.sh` for Bun/npm
> - `./scripts/_sqlx.sh` for SQLx
> - `./scripts/_python.sh` for Python (ensures `.venv`)
> 
> **AI Agents**: Always append `</dev/null` to prevent PTY hangs.

| Scope | Command | Purpose |
|---------------|---------|---------|
| **Frontend** | `./test.sh web </dev/null` | Lint, Unit (Vitest), E2E (Playwright) |
| **Backend** | `./test.sh api </dev/null` | Lint, Unit (Rust), Integration (SQLx) |
| **Full Suite** | `./test.sh all </dev/null` | Comprehensive verification |

**After fixing code, ALWAYS verify with the appropriate domain flag above before claiming done.**

## 🤖 AI Tools — Safe Inline Code Runner

> [!IMPORTANT]
> **NEVER use `python -c`, `python3 -c`, `node -e`, `bash -c` for inline code.**
> These cause PTY hangs through shell interpretation of quotes and escape sequences.
> Use `run-inline` instead — writes code to a temp file, executes via `_safe_run.py`.
> 
> **For Python, `run-inline` auto-uses `./scripts/_python.sh` to ensure `.venv` access.**

```bash
# ✅ Correct — inline code via run-inline
echo 'print(1)' | ./scripts/ai-tools/run-inline -l python
./scripts/ai-tools/run-inline -l python -c 'import os; print(os.getcwd())'
./scripts/ai-tools/run-inline -l bun -c 'console.log(process.cwd())'
./scripts/ai-tools/run-inline -l bash -c 'echo $HOME'

# ✅ Correct — run a Python script file (venv active)
./scripts/_python.sh my_script.py </dev/null

# ❌ Wrong — will hang PTY
python3 -c "print(1)"
python3 my_script.py
node -e "console.log(1)"
bash -c "echo hello"
```

Supported languages: `python`, `python3`, `bun`, `node`, `js`, `bash`, `sh`

## 🏗️ Project Structure

| Component | Language | Path |
|-----------|----------|------|
| API Server | Rust (Axum) | `apiserver/` |
| Agent (ZK Worker) | Rust | `agent-main/` |
| Web Dashboard | React + TypeScript | `webdashboard/` |
| Containers | Compose | `docker-compose*.yml` |
| Tests | Rust + Playwright | `./test.sh` |

## 📝 Commit Convention

Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
