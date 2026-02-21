# Antigravity Instructions - LeeAttend Project

> [!IMPORTANT]
> **Single Source of Truth (SSOT):** [PROJECT_OS.md](../PROJECT_OS.md)
> You MUST read and follow `PROJECT_OS.md` before executing any command or making any implementation decision.

## ⚖️ Rules of Engagement
1. **No Host Commands:** NEVER run `cargo`, `npm`, `pytest`, or `sqlx` on the host.
2. **Docker Protocol:** ALWAYS use `./scripts/docker-helper.sh` for all operations (up, down, logs, exec, build).
3. **RAM-First:** Ensure all build/test flows use the `tmpfs` mounts defined in the Docker workflow to avoid WSL2 freezes.
4. **Modularity:** Keep files < 200 lines. Refactor immediately if this limit is reached.
5. **Naming:** Use `kebab-case` for all files and directories.

## 🛠️ Common Workflows
- **Check Logs:** `./scripts/docker-helper.sh dev logs [service]`
- **Run Tests:** `./run-tests.sh --all`
- **Build Service:** `./scripts/docker-helper.sh dev build [service]`
- **Interactive Shell:** `./scripts/docker-helper.sh dev exec [service] bash`
- **AI Toolkit:** Use `./scripts/ai-tools/*.sh` for direct DB/API/Redis access.

---
