# Antigravity Instructions - LeeAttend Project

> [!IMPORTANT]
> **Single Source of Truth (SSOT):** [PROJECT_OS.md](../PROJECT_OS.md)
> You MUST read and follow `PROJECT_OS.md` before executing any command or making any implementation decision.

## ⚖️ Rules of Engagement
1. **No Host Commands:** NEVER run `cargo`, `npm`, `pytest`, or `sqlx` on the host.
2. **Container Protocol:** ALWAYS use `./run-tests.sh --docker=ENV CMD` for container operations. NEVER run scripts in `scripts/` directly.
3. **RAM-First:** Ensure all build/test flows use the `tmpfs` mounts defined in the Docker workflow to avoid WSL2 freezes.
4. **Modularity:** Keep files < 200 lines. Refactor immediately if this limit is reached.
5. **Naming:** Use `kebab-case` for all files and directories.

## ⚠️ CRITICAL: PTY-Safe Testing

> [!CAUTION]
> Always append `</dev/null` to `./run-tests.sh` commands when running from AI agent terminals.
> This prevents PTY file descriptor inheritance from child processes.

```bash
# ✅ CORRECT — append </dev/null
./run-tests.sh --api --smoke </dev/null
./run-tests.sh --web --smoke </dev/null
./run-tests.sh --api --test=my_test_name </dev/null
./run-tests.sh --all </dev/null

# ❌ WRONG — may hang in AI terminal
./run-tests.sh --api --smoke
```

## 🛠️ Common Workflows
- **Run Tests:** `./run-tests.sh --all </dev/null` (or `--api`, `--web`, `--smoke`)
- **Run One Test:** `./run-tests.sh --api --test=TEST_NAME </dev/null`
- **Check Logs:** `./run-tests.sh --docker=dev logs -f [service]`
- **Build Service:** `./run-tests.sh --docker=dev build [service]`
- **Interactive Shell:** `./run-tests.sh --docker=dev exec [service] bash`
- **AI Toolkit:** Use `./scripts/ai-tools/*.sh` for direct DB/API/Redis access.

---
