---
trigger: always_on
---

# ✨ GEMINI.md - Enterprise Context OS

> 🔴 **MANDATORY:** You MUST read and understand this document in its entirety before taking any action. This is the supreme law of this workspace.

<project_soul>
**LeeAttend** is an Enterprise-Grade Premium Pro Max HRM & Attendance system.
- **Architecture:** Go 1.26+ `net/http` API backend + React/Vite/TypeScript SPA frontend.
- **Runtime:** PostgreSQL, Redis, and MinIO support the Go API and React dashboard.
- **Security:** Total PII masking. Zero hardcoded secrets. Strict Hermetic containerized execution.
- **Mindset:** Project Context is King. External skills serve the project, not the other way around. Leave the codebase cleaner than you found it (Boy Scout).
</project_soul>

## 🧪 Development Test Accounts
- **Account reference:** `docs/dev-accounts.md`
- Before authenticated UI/API testing, AI MUST read `docs/dev-accounts.md` and select the least-privileged account matching the scenario.
- For a request to audit **all UI routes**, AI MUST test all documented roles: `employee`, `team_lead`, `shift_manager`, `hr_manager`, and `admin`. Least privilege applies per scenario, not to the whole audit. AI MUST NOT narrow role coverage without explicit user approval.
- Before browser interaction, AI MUST create a route × role acceptance matrix covering allowed routes/actions, expected-denied routes/actions, console, and network evidence.
- AI MUST report findings grouped by role and route, distinguish expected `403`/redirect authorization behavior from unexpected `404`, `5xx`, crashes, repeated errors, and wrong navigation, and list untested routes with reasons.
- These credentials are development-only. NEVER use them against production, expose passwords in responses, or write credentials/tokens to logs, screenshots, commits, or test artifacts.
- Mask account identifiers and PII in reports; use the account document only as an operational reference.
- **Role switching in Playwright MCP:** Always call `browser_close` between roles to reset the isolated browser context, then navigate to `http://caddy/login` and sign in as the next role.

## 💎 Layer 0: CORE PRINCIPLES (The "Big Five")
> These universal rules CANNOT be overridden by any external skill or lazy-loaded rule.
1. **Enterprise Premium Standard:** EVERY feature MUST be production-ready (Skeleton loaders, Empty States, i18n, Rate Limiting, UUID validation).
2. **Security & Privacy:** NEVER log PII (emails, passwords, phones) in raw text. Mask them (e.g., `user_***@domain.com`).
3. **Structured Errors:** Use domain-specific error types (`AppError`). NEVER throw generic `Error` or `unwrap()` blindly.
4. **Boy Scout Rule:** If you see a typo, bad pattern, or visible bug nearby, FIX IT IMMEDIATELY. Never say "it's out of scope".
5. **Hermetic Wrappers (CRITICAL):** Do not call host dependency tools directly. Use `leedevkit test` for tests and the repository wrappers in `scripts/` for Go, Bun, Python, and container operations. See `.agent/rules/execution-wrappers.md`.
6. **AI MUST BE CALM & PATIENT (CRITICAL):** This is a fundamental mindset for all situations. DO NOT RUSH.
    - Read requirements carefully before starting.
    - Cross-check risks before modifying source code.
    - Never jump to conclusions based on assumptions or subjective guesses.
    - Always verify with concrete evidence before responding.
    - Always wait for processes to finish, carefully read error logs (if any), and only then report back to the user. Never say "it's done" without actually verifying the outcome.
7. **AI MUST EXPLORE BEFORE CREATING (ANTI-REINVENTION):**
    - You MUST use `list_dir`, `grep_search`, or read root-level scripts to verify that a tool doesn't already exist BEFORE creating any new scripts (bash/python).
    - AI tends to ignore existing large codebase tools and writes new ones, causing context bloat and technical debt. DO NOT REINVENT THE WHEEL.
8. **TESTING COMPLETENESS (CRITICAL - NO FALSE POSITIVES):**
    - **Unit tests passing does NOT mean the task is complete.** Unit tests only verify isolated logic.
    - **Integration tests are MANDATORY** after unit tests pass. They verify database operations, API contracts, cross-component interactions, and end-to-end workflows.
    - **Anti-pattern:** Running `leedevkit test <target> --unit-only` and declaring the task complete. This creates false confidence.
    - **Mandatory workflow:** After unit tests pass, you MUST run the full test suite: `leedevkit test <target>` (without `--unit-only`) to ensure integration/E2E tests also pass.
    - **Definition of Done:** A task is INCOMPLETE if integration tests were not executed, even if unit tests passed.
    - **Common failure scenarios:** Database migrations not applied, API schema mismatches, missing environment variables, race conditions in async operations, foreign key violations.

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
- **Database:** `@[.agent/rules/database-rules.md]` (PostgreSQL, pgx, sqlc, migrations, indexing)
- **Project Structure:** `@[.agent/rules/project-structure.md]`
- **Execution Wrappers:** `@[.agent/rules/execution-wrappers.md]` (Go, Bun, Python, container commands)
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
- **Frontend / React / UI:** `@[~/.leedevkit/current/.agent/agents/frontend-specialist.md]` (Dependencies: `design-rules.md`, `coding-standards.md`)
- **Backend / Go / API:** `@[~/.leedevkit/current/.agent/agents/backend-specialist.md]` (Dependencies: `coding-standards.md`, `database-rules.md`, `access-control.md`, `data-governance.md`)
- **Database / Postgres:** `@[~/.leedevkit/current/.agent/agents/database-expert.md]` (Dependencies: `database-rules.md`, `data-governance.md`, `encryption-rules.md`)
- **DevOps / Infra:** `@[~/.leedevkit/current/.agent/agents/devops-engineer.md]` (Dependencies: `testing-standards.md`, `observability-rules.md`, `business-continuity.md`, `supply-chain-security.md`)
- **Security / Audit:** `@[~/.leedevkit/current/.agent/agents/security-auditor.md]` (Dependencies: `access-control.md`, `encryption-rules.md`, `incident-response.md`, `data-governance.md`)
