# 🚀 LeeAttend Project OS (AI-Standard V3)

> [!IMPORTANT]
> **Single Source of Truth** for technical protocols. Failure to follow these rules causes system instability and build freezes.

---

## 1. 🏗️ BUILD & EXECUTION (RAM-First)

**WSL2 Bottleneck:** VirtioFS I/O freezes the system. **RAM-based (tmpfs)** mounts are REQUIRED.

### 1.1 Docker Management via `run-tests.sh --docker=ENV`

`./run-tests.sh` is the **ONLY** entry point for all Docker and test operations.

| Goal | Command |
| :--- | :--- |
| **Start Dev** | `./run-tests.sh --docker=dev up -d` |
| **Stop Dev** | `./run-tests.sh --docker=dev down` |
| **Full Logs** | `./run-tests.sh --docker=dev logs -f` |
| **Service Logs** | `./run-tests.sh --docker=dev logs -f SERVICE` |
| **Shell Access** | `./run-tests.sh --docker=dev exec SERVICE bash` |

### 1.2 Development Cycle

| Service | Build (Cached) | Ad-hoc Commands |
| :--- | :--- | :--- |
| **apiserver** | `./run-tests.sh --docker=dev build apiserver` | `./run-tests.sh --docker=dev exec apiserver cargo [test\|clippy\|add\|sqlx]` |
| **agent-main** | `./run-tests.sh --docker=dev build agent-main` | `./run-tests.sh --docker=dev exec agent-main cargo [test\|clippy]` |
| **dashboard** | `./run-tests.sh --docker=dev exec webdashboard npm install` | `./run-tests.sh --docker=dev exec webdashboard npm run [build\|lint]` |

---

## 2. 🧪 TESTING PROTOCOL

### 2.1 Unified Runner — `./run-tests.sh`

**NEVER run scripts/ files directly.** Always use `./run-tests.sh`.

| Goal | Command |
| :--- | :--- |
| Full suite | `./run-tests.sh --all` |
| API tests only | `./run-tests.sh --api` |
| Web/E2E only | `./run-tests.sh --web` |
| Run one test | `./run-tests.sh --api --test=TEST_NAME` |
| Skip lint, run test | `./run-tests.sh --api --test-only --test=TEST_NAME` |
| Quick smoke check | `./run-tests.sh --api --smoke` |
| Dry run (preview) | `./run-tests.sh --api --dry-run` |
| Read test logs | `./run-tests.sh --logs` |

### 2.2 Test Pattern Format

- **Rust (cargo nextest):** `./run-tests.sh --api --test=module::test_name`
- **Playwright:** `./run-tests.sh --web --test=login_page`

---

## 3. 🔍 LOGGING & OBSERVABILITY

All logs are stored in Docker volume `test-logs`. Access via:

| Tool | Purpose |
| :--- | :--- |
| `./run-tests.sh --logs` | Dump all test logs from Docker volume |
| `./run-tests.sh --tail-logs` | Live-tail test logs |
| `./run-tests.sh --docker=dev logs -f SERVICE` | Live-tail service container logs |

---

## 4. 📐 ARCHITECTURE LIMITS

- **Size Constraint:** Files MUST stay < **200 lines**. Refactor into sub-modules immediately if exceeded.
- **Naming:** `kebab-case` for all files/folders.
- **SSOT Policy:** If you modify infra, update `PROJECT_OS.md` in the SAME token response.

---

## 5. 🛠️ AI DEVELOPER TOOLKIT

Wrapper scripts for direct host → container access:

| Tool | Purpose |
| :--- | :--- |
| `./scripts/ai-tools/redis-cli.sh [args]` | Execute Redis commands (auto-auth) |
| `./scripts/ai-tools/psql.sh [args]` | Execute Postgres commands |
| `./scripts/ai-tools/sqlite.sh [args]` | Query Agent local SQLite database |
| `./scripts/ai-tools/curl-api.sh [args]` | Call API from within `apiserver` net |
| `./scripts/ai-tools/shell.sh [service]` | Quick bash login to service container |

---

## 🎯 DEFINITION OF DONE (AI Checklist)
- [ ] Code is formatted (`cargo fmt`, `prettier`)
- [ ] Linter is clean (`clippy`, `eslint`)
- [ ] **Manual Verification**: Test run inside Docker succeeds
- [ ] **Regression**: `./run-tests.sh` passes relevant suites
- [ ] **Documentation**: `PROJECT_OS.md` or `README.md` updated if logic changed

---

## 🤖 AI INTERACTION & OVERRIDES

This project integrates an **AI Standard Layer** (Cursor, Antigravity, Claude Code) to ensure system safety and build performance.

### 1. AI Protocols:
- `.agent/instructions.md`: Specialized instructions for Antigravity/Gemini.
- `.cursorrules`: Rules specific to Cursor AI.
- `CLAUDE.md`: Core directives applicable to all AI agents.

### 2. Override Authority (For Teammates):
These files are designed to help you work faster, not to hinder you. If you need to change the standards:
- **Temporary**: Instruct the AI in chat: *"Ignore the line limit for this file"* or *"Bypass docker for this run"*.
- **Permanent**: Directly modify the content in the AI Protocol files above to suit the team's evolving workflow.

> [!TIP]
> Treat AI Rules as code - they need to be updated and maintained alongside the project's evolution.
