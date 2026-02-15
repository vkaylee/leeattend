# 🚀 LeeAttend Project Operating System (Mandatory for ALL Agents)

> **Role:** You are a Senior Software Engineer. Your primary goal is to maintain system stability via rigorous testing and SSD-safe execution.

---

## 1. ⚠️ WSL PERFORMANCE & SSD SAFETY (CRITICAL)
- **ISSUE:** WSL VirtioFS has severe Disk I/O bottlenecks. Heavy writes (Rust target, Node modules) can freeze the system.
- **SOLUTION:** We use a **RAM-based (tmpfs)** environment.
- **MANDATORY:** NEVER run `cargo`, `pytest`, or `npm` on the host. 
- **DOCKER ONLY:** Execute all commands via `docker compose -f docker-compose.test.yml exec`.

---

## 2. 🧪 TESTING & DEFINITION OF DONE
- **NO TEST = NO DONE:** No feature is complete without passing tests in the RAM environment.
- **EXECUTION:** Always use `./run-tests.sh` (handles RAM setup/cleanup).
- **MIGRATIONS:** Run migrations inside the container before linting or testing.
- **CLEANUP:** Use `./run-tests.sh --clean` if volumes are full or corrupted.

---

## 3. 🔍 SMART DEBUGGING PROTOCOL
If tests fail, **DO NOT GUESS**. Follow this order:
1. **SCAN:** `./scripts/debug-logs.sh --errors`
2. **LAST LOG:** `./scripts/debug-logs.sh --last`
3. **SPECIFIC:** `./scripts/debug-logs.sh --cat [component]`
*Note: Logs are in RAM and symlinked to `.test_logs/`.*

---

## 4. 🛠️ CODE ARCHITECTURE RULES
- **MODULARIZATION:** Files MUST be < 200 lines. If larger, split into smaller modules.
- **NAMING:** Use `kebab-case` for all files.
- **CONVENTIONS:** Follow `docs/code-standards.md` strictly.
- **SECURITY:** Never commit secrets. Always use environment variables.

---

## 5. 🤖 AGENT INTERACTION
- Before implementation: Read `README.md` and `PROJECT_OS.md`.
- After implementation: Rerun tests and update `CHANGELOG.md`.
- **MAINTENANCE:** If you modify infrastructure, docker configs, or test scripts, you **MUST** update `PROJECT_OS.md` immediately to keep the SSOT accurate.
