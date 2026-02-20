# 🚀 LeeAttend Project OS (AI-Standard V2)

> [!IMPORTANT]
> **Single Source of Truth** for technical protocols. Failure to follow these rules causes system instability and build freezes.

---

## 1. 🏗️ BUILD & EXECUTION (RAM-First)

**WSL2 Bottleneck:** VirtioFS I/O freezes the system. **RAM-based (tmpfs)** mounts are REQUIRED.

### 1.1 Docker Helper Protocol
ALWAYS use `./scripts/docker-helper.sh` to ensure correct mounting and layering.

| Goal | Command |
| :--- | :--- |
| **Start Dev** | `./scripts/docker-helper.sh dev up -d` |
| **Stop Dev** | `./scripts/docker-helper.sh dev down` |
| **Full Logs** | `./scripts/docker-helper.sh dev logs -f [service]` |
| **Shell Access**| `./scripts/docker-helper.sh dev exec [service] bash` |

### 1.2 Development Cycle
| Service | Build (Cached) | Common Commands |
| :--- | :--- | :--- |
| **apiserver** | `./scripts/docker-helper.sh dev build apiserver` | `./scripts/docker-helper.sh dev exec apiserver cargo [test\|clippy\|add\|sqlx]` |
| **agent-main**| `./scripts/docker-helper.sh dev build agent-main` | `./scripts/docker-helper.sh dev exec agent-main cargo [test\|clippy]` |
| **dashboard** | `./scripts/docker-helper.sh dev exec webdashboard npm install` | `./scripts/docker-helper.sh dev exec webdashboard npm run [build\|lint]` |

---

## 2. 🧪 TESTING PROTOCOL

### 2.1 Unified Runner
Use `./run-tests.sh` for all gated automation.
- `--all`: Full suite (Gatekeeper)
- `--api [pattern]`: Rust/Python backend logic
- `--web [pattern]`: Playwright E2E with mock interception

### 2.2 Manual Verification
If running manually, you MUST use the `test` environment layer:
1. `up`: `./scripts/docker-helper.sh test up -d`
2. `exec`: `./scripts/docker-helper.sh test exec [service] [cmd]`

---

## 3. 🔍 LOGGING & OBSERVABILITY

| Tool | Purpose |
| :--- | :--- |
| `./scripts/debug-logs.sh --errors` | Scans all components for FATAL/ERROR |
| `./scripts/debug-logs.sh --last` | Shows tail of the last failed component |
| `./scripts/debug-logs.sh --cat [svc]` | Full dump of component logs from RAM |

---

## 4. 📐 ARCHITECTURE LIMITS

- **Size Constraint:** Files MUST stay < **200 lines**. Refactor into sub-modules immediately if exceeded.
- **Naming:** `kebab-case` for all files/folders.
- **SSOT Policy:** If you modify infra, update `PROJECT_OS.md` in the SAME token response.

---

## 🎯 DEFINITION OF DONE (AI Checklist)
- [ ] Code is formatted (`cargo fmt`, `prettier`)
- [ ] Linter is clean (`clippy`, `eslint`)
- [ ] **Manual Verification**: Test run inside Docker succeeds
- [ ] **Regression**: `./run-tests.sh` passes relevant suites
- [ ] **Documentation**: `PROJECT_OS.md` or `README.md` updated if logic changed
