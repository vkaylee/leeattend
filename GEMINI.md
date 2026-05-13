---
trigger: always_on
---

# ✨ GEMINI.md - Enterprise Context OS

> 🔴 **MANDATORY:** You MUST read and understand this document in its entirety before taking any action. This is the supreme law of this workspace.

<project_soul>
**LeeAttend** is an Enterprise-Grade Premium Pro Max HRM & Attendance system. 
- **Architecture:** Zero-compromise Rust/Axum backend + React/TS SPA frontend.
- **Quality:** No MVP code. UI must feature premium micro-interactions, skeletons, and flawless i18n. Backend must have zero N+1 queries, strict transaction boundaries, and explicit AppError handling.
- **Security:** Total PII masking. Zero hardcoded secrets. Strict Hermetic containerized execution.
- **Mindset:** Project Context is King. External skills serve the project, not the other way around. Leave the codebase cleaner than you found it (Boy Scout).
</project_soul>

## 💎 Layer 0: CORE PRINCIPLES (The "Big Four")
> These universal rules CANNOT be overridden by any external skill or lazy-loaded rule.
1. **Enterprise Premium Standard:** EVERY feature MUST be production-ready (Skeleton loaders, Empty States, i18n, Rate Limiting, UUID validation).
2. **Security & Privacy:** NEVER log PII (emails, passwords, phones) in raw text. Mask them (e.g., `user_***@domain.com`).
3. **Structured Errors:** Use domain-specific error types (`AppError`). NEVER throw generic `Error` or `unwrap()` blindly.
4. **Boy Scout Rule:** If you see a typo, bad pattern, or visible bug nearby, FIX IT IMMEDIATELY. Never say "it's out of scope".
## 🥇 Layer 1: CRITICAL PROTOCOL
**Before writing ANY code or proposing solutions, you MUST:**
1. **Read → Understand → Apply:** Read relevant files, check ADRs, understand, apply.
2. **Selective Loading:** Only load rule files when needed for the task.
3. **MANDATORY Rule Compliance Check:** For COMPLEX CODE or DESIGN tasks, create `{task-slug}.md` and list at least 3 Rule IDs.

## 📥 Layer 2: REQUEST CLASSIFIER & ROUTING
- Classify request: QUESTION, SURVEY, SIMPLE CODE, COMPLEX CODE, DESIGN/UI.
- Detect domain and announce: `🤖 **Applying knowledge of @[agent-name]...**`
- Cross-Agent Handoff: Contract First (API before UI), Zero Assumptions.

## 🛑 Layer 3: SOCRATIC GATE
- For New Features / Bug Fixes: 🔴 STOP and ASK minimum 3 strategic questions. Confirm understanding. Do not implement based on assumptions.

## 📚 Layer 4: DOMAIN RULEBOOKS (Lazy Load Index)
> 🔴 **MANDATORY:** Before executing a task, you MUST read the relevant dictionary files using the `view_file` tool. Do NOT guess the rules.

### 🛡️ Core Safety & Execution (READ BEFORE RUNNING COMMANDS)
> [!CAUTION]
> **CRITICAL PTY HANG WARNING:** 
> 1. NEVER pass strings containing literal newlines into terminal commands (like `git commit -m "multi\nline"` or heredocs). THIS WILL HANG THE AGENT FOREVER. Write multi-line content to a temp file (`/tmp/msg.txt`), then execute the command reading from that file (`-F /tmp/msg.txt` or `< /tmp/msg.txt`).
> 2. When running commands via terminal tools, child processes can inherit PTY file descriptors and prevent command completion detection.

**1. The Hermetic Toolbox (MANDATORY)**
All standard development tools run inside isolated Podman containers. You MUST execute them via these wrapper scripts instead of using native commands on the host:
- Rust/Cargo: `./scripts/_cargo.sh build` (Container: `apiserver_test`)
- Node/NPM: `./scripts/_npm.sh install` (Container: `web_test`)
- SQLx/DB: `./scripts/_sqlx.sh migrate run` (Container: `apiserver_test`)
- SQLx Prepare: `./scripts/_sqlx.sh prepare --workspace` (Must be run after tests fail with SQLX_OFFLINE=true error, requires test DB to be up)
- Diesel/DB: `./scripts/_diesel.sh print-schema` (Container: `apiserver_test`)

**2. Test Execution & Coverage**
- NEVER run `cargo test` directly on the host (especially on WSL2 due to I/O freezes). ALWAYS use `./test.sh`.
- When asked to check coverage, append `--coverage` (e.g., `./test.sh api --coverage`). This enforces the 100% threshold.

**3. Git Protocol**
- NEVER run `git` directly. ALWAYS use the `./scripts/git.sh` wrapper.
- NEVER use multi-line strings in commit messages. Create `/tmp/msg.txt` using file-write tools, then run `./scripts/git.sh commit -F /tmp/msg.txt`.

**4. Background Execution & Services**
- **BANNED:** Never use the bash background operator (`&`) for ANY command (tests, servers, builds). 
- **BANNED:** Never use `tail -f` or similar blocking pagers.
- *Note:* The AI's native terminal tool automatically handles long-running processes safely. You must run all commands normally without manual shell redirection or backgrounding.

**5. Banned Inline Commands & Patterns**
- Heredocs (`cat <<EOF` or `cat >> file <<EOF`)
- Pager commands (`git log`, `less`, `more`). Always use `--no-pager`.
- Piped commands holding PTY (`rg | awk`, `grep | sed`). Append `</dev/null`.

**6. Safe Inline Code Runner**
- **NEVER use `python -c`, `python3 -c`, `node -e`, `bash -c` for inline code.**
- Use `run-inline` instead: `echo 'print(1)' | ./scripts/ai-tools/run-inline -l python`


**7. Avoiding PTY Hangs on Scripts (MANDATORY)**
- When running long-running scripts (`./test.sh`, `./scripts/_cargo.sh`, `npm run`, etc.), the PTY may hang indefinitely.
- **ALWAYS redirect script output to a file** and read it via your native `view_file` tool:
  ✅ `> run.log 2>&1 </dev/null`
- **NEVER** pipe commands together if they spawn containers or heavy child processes (e.g., `cargo check | grep` is ❌).

- **Testing Standards:** `@[.agent/rules/testing-standards.md]`

### 💻 Code & Architecture
- **Coding Standards:** `@[.agent/rules/coding-standards.md]` (Clean code, errors, security, enterprise grade)
- **Database:** `@[.agent/rules/database-rules.md]` (SQLx, migrations, indexing)
- **Project Structure:** `@[.agent/rules/project-structure.md]`

### 🎨 Design
- **UI/UX Design:** `@[.agent/rules/design-rules.md]` (Design system, a11y, responsive)

## 🔌 External Skill Integration (Plugin Protocol)
To prevent capability limitations, this workspace supports external AI skills:
1. **Persona-Bound:** External skills must be registered directly inside the specific `.agent/agents/*.md` file. Do NOT load global skills randomly.
2. **Conflict Resolution:** Project internal rules (`.agent/rules/`) **ALWAYS OVERRIDE** external skill instructions. Internal context is king.

## 🤖 Layer 5: AGENTS & SKILLS (Concurrent Pre-fetching)
To activate an agent, use the `view_file` tool to load their profile AND their dependencies IN PARALLEL:
- **Frontend / React / UI:** `@[.agent/agents/frontend-specialist.md]` (Dependencies: `design-rules.md`, `coding-standards.md`)
- **Backend / Rust / API:** `@[.agent/agents/backend-specialist.md]` (Dependencies: `coding-standards.md`, `database-rules.md`)
- **Database / Postgres:** `@[.agent/agents/database-expert.md]` (Dependencies: `database-rules.md`)
- **DevOps / Infra:** `@[.agent/agents/devops-engineer.md]` (Dependencies: `execution-pty-safety.md`, `testing-standards.md`)
