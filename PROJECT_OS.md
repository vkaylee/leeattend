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
| Quick check | `./run-tests.sh --api` |
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

## 🎯 DEFINITION OF DONE — MANDATORY AI PROTOCOL

> [!IMPORTANT]
> These are **non-negotiable automated steps**. After any code change, the AI **MUST** execute these steps autonomously without waiting for user instruction.

**After EVERY code change, the AI MUST run the following checklist:**
- [ ] **Format**: Code is formatted (`cargo fmt`, `prettier`)
- [ ] **Lint**: Linter is clean (`clippy`, `eslint`)
- [ ] **Write/Update Tests**: (see "Test Authoring" section below)
- [ ] **Test**: Run the test scope relevant to the change:
  - Changed `apiserver/` or `agent-main/` → `./run-tests.sh --api`
  - Changed `webdashboard/` → `./run-tests.sh --web`
  - Major refactor → `./run-tests.sh --all`
- [ ] **Verify**: If tests fail → run full suite for the changed component to identify exact failure
- [ ] **Report**: Summarize pass/fail + which tests were run

> [!WARNING]
> **NEVER mark a task as "done" or hand off to the user if tests are still failing.**
> If tests fail after a fix, continue iterating until they pass or explicitly ask the user for guidance.

---

## 🧪 TEST AUTHORING — AI MUST WRITE TESTS PROACTIVELY

> [!IMPORTANT]
> Whenever the AI **adds or modifies code**, it MUST autonomously write or update tests alongside the implementation. Do NOT wait for the user to ask.

### Rules

| Scenario | What AI Must Do |
|---|---|
| **New function/method added** | Write at least 1 unit test for the happy path + 1 for the error/edge case |
| **Bug fixed** | Write a regression test that would have caught the bug |
| **API endpoint added/changed** | Write/update integration test covering the endpoint |
| **Frontend component added** | Write a Vitest unit test for the component |
| **UI flow changed** | Update or add a Playwright E2E test |
| **Refactor (behavior unchanged)** | Verify existing tests still cover the refactored code; add new ones if coverage drops |

### Test File Conventions

- **Rust (apiserver / agent-main):**
  - Unit tests: inline `#[cfg(test)] mod tests { ... }` in the same file
  - Integration tests: `apiserver/tests/integration/` or `agent-main/tests/`
  - Pattern: `./run-tests.sh --api --test=module::test_name`

- **Web (webdashboard):**
  - Unit tests: `src/**/__tests__/*.test.tsx` adjacent to component
  - E2E tests: `tests/*.spec.ts` (Playwright)
  - Pattern: `./run-tests.sh --web --test=ComponentName`

### Minimum Coverage Expectation

| Component | Coverage Target |
|---|---|
| `apiserver` | ≥ 70% line coverage trên toàn crate |
| `agent-main` | ≥ 70% line coverage trên toàn crate |
| `webdashboard` | ≥ 60% line coverage (Vitest) |
| **Từng file vừa sửa** | 100% các function được thêm/sửa phải có test |

> Check coverage: `./run-tests.sh --api --coverage` hoặc `./run-tests.sh --web --coverage`

### Legacy Code Policy

Khi sửa file không có test:
- Chỉ cần viết test **cho đoạn code vừa sửa/thêm**, không bắt buộc cover toàn bộ file.
- Nếu việc add test sẽ làm thay đổi quá lớn scope → note rõ trong summary và tạo task riêng.


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
