# 🏗️ Project Structure & Commits

## 1. Component Map
| Component | Language | Path |
|-----------|----------|------|
| API Server | Rust (Axum) | `apiserver/` |
| Agent (ZK Worker) | Rust | `agent-main/` |
| Web Dashboard | React + TypeScript | `webdashboard/` |
| Containers | Compose | `docker-compose*.yml` |
| Tests | Rust + Playwright | `./test.sh` |

## 2. Commit Convention
Use conventional commits for all changes: 
`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

## 3. AI Logging & Workspace Memory
- **Memory Retention:** Document major architectural shifts in `.ai/ADRs/` to retain long-term AI context.
- **Clean Workspace:** Do NOT save `.txt` or temporary planning files in project root. Use `/tmp/` or prefix `ai_` (which is added to `.gitignore`).
