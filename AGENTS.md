# AGENTS.md — Rules for AI Coding Assistants

> This file is read by AI coding tools (Claude Code, Cursor, Copilot, Windsurf,
> Gemini, etc.) to provide project-specific instructions.

## ⚠️ PTY Compatibility — CRITICAL for terminal commands

> [!CAUTION]
> When running commands via terminal tools, child processes can inherit PTY file
> descriptors and prevent command completion detection. The agent will hang
> waiting for a "done" status that never arrives.

**ALWAYS append `</dev/null` to these commands:**

| Command | Why |
|---------|-----|
| `git commit`, `git push`, `git pull` | GPG agent holds PTY |
| `podman-compose up/down/exec/build` | Podman CLI spawns children |
| Any script with `&` (background jobs) | Background process inherits PTY |

```bash
# ✅ Correct — AGENT_MODE skips Podman tests in pre-commit hook
AGENT_MODE=1 git commit --no-gpg-sign -m "message" </dev/null
podman-compose down </dev/null
./run-tests.sh --api </dev/null

# ❌ Wrong — will hang
git commit -m "message"
podman-compose down
```

## 🧪 Testing — MANDATORY RULES

> [!WARNING]
> **NEVER run `cargo test`, `npm test`, `npx playwright test`, or any test command directly.** 
> This project uses Podman-based test isolation. The ONLY way to run tests is through `./run-tests.sh`.
> 
> **NEVER run `cargo`, `npm`, or `sqlx` directly for development tasks.**
> ALWAYS use the hermetic toolbox wrappers: `./scripts/_cargo.sh`, `./scripts/_npm.sh`, `./scripts/_sqlx.sh`.
> 
> **AI Agents**: Always append `</dev/null` to prevent PTY hangs.

| What you want | Command |
|---------------|---------|
| Quick API check (lint + unit + integration) | `./run-tests.sh --api </dev/null` |
| Full test suite | `./run-tests.sh --all </dev/null` |
| Single test | `./run-tests.sh --api --test-only --test=test_name </dev/null` |
| See what would run | `./run-tests.sh --api --dry-run` |
| Read failure logs | `cat .test_logs/latest-failure.log` |
| Full help | `./run-tests.sh --help` |

**After fixing code, ALWAYS verify with `./run-tests.sh` before claiming done.**

## 🏗️ Project Structure

| Component | Language | Path |
|-----------|----------|------|
| API Server | Rust (Axum) | `apiserver/` |
| Agent (ZK Worker) | Rust | `agent-main/` |
| Web Dashboard | React + TypeScript | `webdashboard/` |
| Containers | Compose | `docker-compose*.yml` |
| Tests | Rust + Playwright | `./run-tests.sh` |

## 📝 Commit Convention

Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
