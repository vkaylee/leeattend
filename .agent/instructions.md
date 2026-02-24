# Antigravity Instructions - LeeAttend Project

> [!IMPORTANT]
> **Single Source of Truth (SSOT):** [PROJECT_OS.md](../PROJECT_OS.md)
> You MUST read and follow `PROJECT_OS.md` before executing any command or making any implementation decision.

## ⚖️ Rules of Engagement
1. **No Host Commands:** NEVER run `cargo`, `npm`, `pytest`, or `sqlx` on the host.
2. **Docker Protocol:** ALWAYS use `./run-tests.sh --docker=ENV CMD` for all operations. NEVER run scripts in `scripts/` directly.
3. **RAM-First:** Ensure all build/test flows use the `tmpfs` mounts defined in the Docker workflow to avoid WSL2 freezes.
4. **Modularity:** Keep files < 200 lines. Refactor immediately if this limit is reached.
5. **Naming:** Use `kebab-case` for all files and directories.

## 🛠️ Common Workflows
- **Check Logs:** `./run-tests.sh --docker=dev logs -f [service]`
- **Run Tests:** `./run-tests.sh --all`
- **Run One Test:** `./run-tests.sh --api --test=TEST_NAME`
- **Build Service:** `./run-tests.sh --docker=dev build [service]`
- **Interactive Shell:** `./run-tests.sh --docker=dev exec [service] bash`
- **AI Toolkit:** Use `./scripts/ai-tools/*.sh` for direct DB/API/Redis access.

---
