# Antigravity Instructions - LeeAttend Project

> [!IMPORTANT]
> **Single Source of Truth (SSOT):** [PROJECT_OS.md](../PROJECT_OS.md)
> You MUST read and follow `PROJECT_OS.md` before executing any command or making any implementation decision.

## ⚖️ Rules of Engagement
1. **No Host Commands:** NEVER run `cargo`, `npm`, `pytest`, or `sqlx` on the host.
2. **Docker Protocol:** ALWAYS use `./run-tests.sh --docker=ENV CMD` for Docker operations. NEVER run scripts in `scripts/` directly.
3. **RAM-First:** Ensure all build/test flows use the `tmpfs` mounts defined in the Docker workflow to avoid WSL2 freezes.
4. **Modularity:** Keep files < 200 lines. Refactor immediately if this limit is reached.
5. **Naming:** Use `kebab-case` for all files and directories.

## ⚠️ CRITICAL: PTY-Safe Testing

> [!CAUTION]
> **NEVER run `./run-tests.sh` directly from AI agent terminal.** It will hang due to PTY file descriptor inheritance from Docker.
> **ALWAYS use `./ai-test.sh` instead.** It is a drop-in replacement that wraps `run-tests.sh` with `setsid` to prevent PTY hangs.

```bash
# ✅ CORRECT — always use ai-test.sh
./ai-test.sh --api --smoke
./ai-test.sh --web --smoke
./ai-test.sh --api --test=my_test_name
./ai-test.sh --all

# ❌ WRONG — will hang in AI terminal
./run-tests.sh --api --smoke
```

## 🛠️ Common Workflows
- **Run Tests:** `./ai-test.sh --all` (or `--api`, `--web`, `--smoke`)
- **Run One Test:** `./ai-test.sh --api --test=TEST_NAME`
- **Check Logs:** `./run-tests.sh --docker=dev logs -f [service]`
- **Build Service:** `./run-tests.sh --docker=dev build [service]`
- **Interactive Shell:** `./run-tests.sh --docker=dev exec [service] bash`
- **AI Toolkit:** Use `./scripts/ai-tools/*.sh` for direct DB/API/Redis access.

---
