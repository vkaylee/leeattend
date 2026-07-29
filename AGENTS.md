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

## 💎 Layer 0: CORE PRINCIPLES (The "Big Five")
> These universal rules CANNOT be overridden by any external skill or lazy-loaded rule.
1. **Enterprise Premium Standard:** EVERY feature MUST be production-ready (Skeleton loaders, Empty States, i18n, Rate Limiting, UUID validation).
2. **Security & Privacy:** NEVER log PII (emails, passwords, phones) in raw text. Mask them (e.g., `user_***@domain.com`).
3. **Structured Errors:** Use domain-specific error types (`AppError`). NEVER throw generic `Error` or `unwrap()` blindly.
4. **Boy Scout Rule:** If you see a typo, bad pattern, or visible bug nearby, FIX IT IMMEDIATELY. Never say "it's out of scope".
5. **PTY Safety & File Creation (CRITICAL):** 
   - **Execution Pattern:** For ALL terminal commands, you **MUST** append ` > logs/run_{command_slug}.log 2>&1 </dev/null` directly inside the `CommandLine` argument. Always prefix with `mkdir -p logs &&`. Example: `CommandLine: "mkdir -p logs && ls -la > logs/run_ls.log 2>&1 </dev/null"`.
   - **Anti-pattern:** Leaving stdout/stderr un-redirected, which hangs the system.
   - **Script Execution Pattern (2-Step Workflow):** Any code execution (Python, Bash, JS) MUST follow a strict 2-step workflow:
     1. Use the native `write_to_file` tool to create a temporary script (e.g., `scratch/temp_script.py` or `scratch/commit_msg.txt`).
     2. Execute the file via `run_command` (e.g., `python scratch/temp_script.py > logs/run_script.log 2>&1`).
   - **Pre-Flight Check for Multiline:** ALWAYS check if your command requires multiline inputs, complex formatting, or inline execution (`python -c`, `bash -c`, multiline `echo`/`cat`). If it does, YOU MUST USE the `write_to_file` tool to save it as a physical file first. Multiline strings in the terminal break the PTY parser and cause system hangs. Use `write_to_file` as your best friend!
   - **Log Monitoring Pattern:** Use **periodic sampling** (e.g., one-shot reads via `tail -n 100`, `cat`, or native `view_file` tool) to inspect states safely.
   - **Anti-pattern:** Using infinite-blocking streams (`tail -f`, `watch`, `top`, `ping`) via `run_command` in background mode. These commands hang the PTY and pollute the task list forever.
6. **Hermetic Wrappers (CRITICAL):** DO NOT call `cargo`, `npm`, or `diesel` directly. Use `./leedevkit test` for testing and wrapper scripts in `scripts/` (e.g., `./scripts/_cargo.sh`) for individual tool commands. See `execution-wrappers.md` for details.
7. **AI MUST BE CALM & PATIENT (CRITICAL):** This is a fundamental mindset for all situations. DO NOT RUSH.
    - Read requirements carefully before starting.
    - Cross-check risks before modifying source code.
    - Never jump to conclusions based on assumptions or subjective guesses.
    - Always verify with concrete evidence before responding.
    - Always wait for processes to finish, carefully read error logs (if any), and only then report back to the user. Never say "it's done" without actually verifying the outcome.
8. **AI MUST EXPLORE BEFORE CREATING (ANTI-REINVENTION):**
    - You MUST use `list_dir`, `grep_search`, or read root-level scripts to verify that a tool doesn't already exist BEFORE creating any new scripts (bash/python).
    - AI tends to ignore existing large codebase tools and writes new ones, causing context bloat and technical debt. DO NOT REINVENT THE WHEEL.

## 🥇 Layer 1: CRITICAL PROTOCOL
**Before writing ANY code or proposing solutions, you MUST:**
1. **MANDATORY Lazy-Load:** You MUST run `view_file` on related rulebooks and explicitly output 'Read: [Rule Name]' in your Socratic response BEFORE generating any code.
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


### 💻 Code & Architecture
- **Coding Standards:** `@[.agent/rules/coding-standards.md]` (Clean code, errors, security, enterprise grade)
- **Testing Standards:** `@[.agent/rules/testing-standards.md]`
- **Database:** `@[.agent/rules/database-rules.md]` (Diesel, migrations, indexing)
- **Project Structure:** `@[.agent/rules/project-structure.md]`
- **Execution Wrappers:** `@[.agent/rules/execution-wrappers.md]` (cargo, npm, diesel, python scripts)
- **Change Management:** `@[.agent/rules/change-management.md]` (Code review, branch protection, deployment gates, feature flags)
- **Configuration Management:** `@[.agent/rules/configuration-management.md]` (Env vars, fail-fast validation, environment parity)

### 🎨 Design
- **UI/UX Design:** `@[.agent/rules/design-rules.md]` (Design system, a11y, responsive)

### 🔒 Security & Compliance
- **Data Governance:** `@[.agent/rules/data-governance.md]` (Data classification, retention, GDPR, PII handling)
- **Access Control:** `@[.agent/rules/access-control.md]` (RBAC enforcement, JWT, password policy, MFA)
- **Encryption:** `@[.agent/rules/encryption-rules.md]` (TLS, encryption at rest, key management, security headers)
- **Incident Response:** `@[.agent/rules/incident-response.md]` (Security events, breach notification, post-mortems)
- **Supply Chain:** `@[.agent/rules/supply-chain-security.md]` (Dependency audit, license policy, Docker image security, SBOM)

### 🏗️ Operations & Infrastructure
- **Observability:** `@[.agent/rules/observability-rules.md]` (Structured logging, monitoring, alerting, tracing)
- **Business Continuity:** `@[.agent/rules/business-continuity.md]` (RPO/RTO, backup policy, failover, DR testing)

## 🔌 External Skill Integration (Plugin Protocol)
To prevent capability limitations, this workspace supports external AI skills:
1. **Persona-Bound:** External skills must be registered directly inside the specific `.agent/agents/*.md` file. Do NOT load global skills randomly.
2. **Conflict Resolution:** Project internal rules (`.agent/rules/`) **ALWAYS OVERRIDE** external skill instructions. Internal context is king.

## 🤖 Layer 5: AGENTS & SKILLS (Concurrent Pre-fetching)
To activate an agent, use the `view_file` tool to load their profile AND their dependencies IN PARALLEL:
- **Frontend / React / UI:** `@[.agent/agents/frontend-specialist.md]` (Dependencies: `design-rules.md`, `coding-standards.md`)
- **Backend / Rust / API:** `@[.agent/agents/backend-specialist.md]` (Dependencies: `coding-standards.md`, `database-rules.md`, `access-control.md`, `data-governance.md`)
- **Database / Postgres:** `@[.agent/agents/database-expert.md]` (Dependencies: `database-rules.md`, `data-governance.md`, `encryption-rules.md`)
- **DevOps / Infra:** `@[.agent/agents/devops-engineer.md]` (Dependencies: `execution-pty-safety.md`, `testing-standards.md`, `observability-rules.md`, `business-continuity.md`, `supply-chain-security.md`)
- **Security / Audit:** `@[.agent/agents/security-auditor.md]` (Dependencies: `access-control.md`, `encryption-rules.md`, `incident-response.md`, `data-governance.md`)
