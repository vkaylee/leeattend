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
- Detailed UI audit protocol: `.agent/rules/ui-route-audit.md`.

## 💎 Layer 0: CORE PRINCIPLES (The "Big Five")
> These universal rules CANNOT be overridden by any external skill or lazy-loaded rule.
1. **Enterprise Premium Standard:** EVERY feature MUST be production-ready (Skeleton loaders, Empty States, i18n, Rate Limiting, UUID validation).
2. **Security & Privacy:** NEVER log PII (emails, passwords, phones) in raw text. Mask them (e.g., `user_***@domain.com`).
3. **Structured Errors:** Use domain-specific error types (`AppError`). NEVER throw generic `Error` or `unwrap()` blindly.
4. **Boy Scout Rule:** If you see a typo, bad pattern, or visible bug nearby, FIX IT IMMEDIATELY. Never say "it's out of scope".
5. **Hermetic Wrappers (CRITICAL):** Do not call host dependency tools directly. Use `./leedevkit test` for tests and the repository wrappers in `scripts/` for Go, Bun, Python, and container operations. See `.agent/rules/execution-wrappers.md`.
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
    - **Anti-pattern:** Running `./leedevkit test <target> --unit-only` and declaring the task complete. This creates false confidence.
    - **Mandatory workflow:** After unit tests pass, you MUST run the full test suite: `./leedevkit test <target>` (without `--unit-only`) to ensure integration/E2E tests also pass.
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
- **Frontend / React / UI:** `@[.agent/agents/frontend-specialist.md]` (Dependencies: `design-rules.md`, `coding-standards.md`)
- **Backend / Go / API:** `@[.agent/agents/backend-specialist.md]` (Dependencies: `coding-standards.md`, `database-rules.md`, `access-control.md`, `data-governance.md`)
- **Database / Postgres:** `@[.agent/agents/database-expert.md]` (Dependencies: `database-rules.md`, `data-governance.md`, `encryption-rules.md`)
- **DevOps / Infra:** `@[.agent/agents/devops-engineer.md]` (Dependencies: `testing-standards.md`, `observability-rules.md`, `business-continuity.md`, `supply-chain-security.md`)
- **Security / Audit:** `@[.agent/agents/security-auditor.md]` (Dependencies: `access-control.md`, `encryption-rules.md`, `incident-response.md`, `data-governance.md`)


<!-- leedevkit:begin -->
## LeeDevKit base context

This repository uses LeeDevKit. Before making changes, read and apply:
`.leedevkit/templates/CLAUDE.base.md`.

Project policy lives in `.agent/`; devkit sources live at:
- Built-in skills: `.leedevkit/.agent/skills/*/SKILL.md`
- Community skills: `.leedevkit/skills.d/*/SKILL.md`
- Built-in agents: `.leedevkit/.agent/agents/*.md`
- Project agents: `.agent/agents/*.md`

Run `./leedevkit --help` for available project commands.

### Available skills

| Skill | When to use | Read |
|---|---|---|
| `2d-games` | 2D game development principles. Sprites, tilemaps, physics, camera. | `.leedevkit/.agent/skills/game-development/2d-games/SKILL.md` |
| `3d-games` | 3D game development principles. Rendering, shaders, physics, cameras. | `.leedevkit/.agent/skills/game-development/3d-games/SKILL.md` |
| `api-patterns` | API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination. | `.leedevkit/.agent/skills/api-patterns/SKILL.md` |
| `app-builder` | Main application building orchestrator. Creates full-stack applications from natural language requests. Determines project type, selects tech stack, coordinates agents. | `.leedevkit/.agent/skills/app-builder/SKILL.md` |
| `architecture` | Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design. | `.leedevkit/.agent/skills/architecture/SKILL.md` |
| `bash-linux` | Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems. | `.leedevkit/.agent/skills/bash-linux/SKILL.md` |
| `behavioral-modes` | AI operational modes (brainstorm, implement, debug, review, teach, ship, orchestrate). Use to adapt behavior based on task type. | `.leedevkit/.agent/skills/behavioral-modes/SKILL.md` |
| `brainstorming` | Targeted clarification and concise user communication for ambiguous or high-impact work. | `.leedevkit/.agent/skills/brainstorming/SKILL.md` |
| `clean-code` | Pragmatic coding standards - concise, direct, no over-engineering, no unnecessary comments | `.leedevkit/.agent/skills/clean-code/SKILL.md` |
| `code-review-checklist` | Code review guidelines covering code quality, security, and best practices. | `.leedevkit/.agent/skills/code-review-checklist/SKILL.md` |
| `database-design` | Database design principles and decision-making. Schema design, indexing strategy, ORM selection, serverless databases. | `.leedevkit/.agent/skills/database-design/SKILL.md` |
| `deployment-procedures` | Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts. | `.leedevkit/.agent/skills/deployment-procedures/SKILL.md` |
| `documentation-templates` | Documentation templates and structure guidelines. README, API docs, code comments, and AI-friendly documentation. | `.leedevkit/.agent/skills/documentation-templates/SKILL.md` |
| `frontend-design` | Design thinking and decision-making for web UI. Use when designing components, layouts, color schemes, typography, or creating aesthetic interfaces. Teaches principles, not fixed values. | `.leedevkit/.agent/skills/frontend-design/SKILL.md` |
| `game-art` | Game art principles. Visual style selection, asset pipeline, animation workflow. | `.leedevkit/.agent/skills/game-development/game-art/SKILL.md` |
| `game-audio` | Game audio principles. Sound design, music integration, adaptive audio systems. | `.leedevkit/.agent/skills/game-development/game-audio/SKILL.md` |
| `game-design` | Game design principles. GDD structure, balancing, player psychology, progression. | `.leedevkit/.agent/skills/game-development/game-design/SKILL.md` |
| `game-development` | Game development orchestrator. Routes to platform-specific skills based on project needs. | `.leedevkit/.agent/skills/game-development/SKILL.md` |
| `geo-fundamentals` | Generative Engine Optimization for AI search engines (ChatGPT, Claude, Perplexity). | `.leedevkit/.agent/skills/geo-fundamentals/SKILL.md` |
| `i18n-localization` | Internationalization and localization patterns. Detecting hardcoded strings, managing translations, locale files, RTL support. | `.leedevkit/.agent/skills/i18n-localization/SKILL.md` |
| `intelligent-routing` | Select smallest suitable agent or agent set from task intent, domain, and complexity. | `.leedevkit/.agent/skills/intelligent-routing/SKILL.md` |
| `lint-and-validate` | Automatic quality control, linting, and static analysis procedures. Use after code modifications to ensure syntax correctness and project standards. Triggers on lint, format, check, validate, types, static analysis. | `.leedevkit/.agent/skills/lint-and-validate/SKILL.md` |
| `mcp-builder` | MCP (Model Context Protocol) server building principles. Tool design, resource patterns, best practices. | `.leedevkit/.agent/skills/mcp-builder/SKILL.md` |
| `mobile-design` | Mobile-first design thinking and decision-making for iOS and Android apps. Touch interaction, performance patterns, platform conventions. Teaches principles, not fixed values. Use when building React Native, Flutter, or native mobile apps. | `.leedevkit/.agent/skills/mobile-design/SKILL.md` |
| `mobile-games` | Mobile game development principles. Touch input, battery, performance, app stores. | `.leedevkit/.agent/skills/game-development/mobile-games/SKILL.md` |
| `multiplayer` | Multiplayer game development principles. Architecture, networking, synchronization. | `.leedevkit/.agent/skills/game-development/multiplayer/SKILL.md` |
| `nextjs-react-expert` | React and Next.js performance optimization from Vercel Engineering. Use when building React components, optimizing performance, eliminating waterfalls, reducing bundle size, reviewing code for performance issues, or implementing server/client-side optimizations. | `.leedevkit/.agent/skills/nextjs-react-expert/SKILL.md` |
| `nodejs-best-practices` | Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying. | `.leedevkit/.agent/skills/nodejs-best-practices/SKILL.md` |
| `parallel-agents` | Coordinate specialized subagents for independent analysis, dependent implementation, verification, and synthesis. | `.leedevkit/.agent/skills/parallel-agents/SKILL.md` |
| `pc-games` | PC and console game development principles. Engine selection, platform features, optimization strategies. | `.leedevkit/.agent/skills/game-development/pc-games/SKILL.md` |
| `performance-profiling` | Performance profiling principles. Measurement, analysis, and optimization techniques. | `.leedevkit/.agent/skills/performance-profiling/SKILL.md` |
| `plan-writing` | Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work. | `.leedevkit/.agent/skills/plan-writing/SKILL.md` |
| `powershell-windows` | PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling. | `.leedevkit/.agent/skills/powershell-windows/SKILL.md` |
| `python-patterns` | Python development principles and decision-making. Framework selection, async patterns, type hints, project structure. Teaches thinking, not copying. | `.leedevkit/.agent/skills/python-patterns/SKILL.md` |
| `red-team-tactics` | Red team tactics principles based on MITRE ATT&CK. Attack phases, detection evasion, reporting. | `.leedevkit/.agent/skills/red-team-tactics/SKILL.md` |
| `seo-fundamentals` | SEO fundamentals, E-E-A-T, Core Web Vitals, and Google algorithm principles. | `.leedevkit/.agent/skills/seo-fundamentals/SKILL.md` |
| `server-management` | Server management principles and decision-making. Process management, monitoring strategy, and scaling decisions. Teaches thinking, not commands. | `.leedevkit/.agent/skills/server-management/SKILL.md` |
| `systematic-debugging` | 4-phase systematic debugging methodology with root cause analysis and evidence-based verification. Use when debugging complex issues. | `.leedevkit/.agent/skills/systematic-debugging/SKILL.md` |
| `tailwind-patterns` | Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture. | `.leedevkit/.agent/skills/tailwind-patterns/SKILL.md` |
| `tdd-workflow` | Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle. | `.leedevkit/.agent/skills/tdd-workflow/SKILL.md` |
| `technical-debt-management` | Systematically analyze, assess, and resolve technical debt. Use when the user requests refactoring, legacy-code optimization, or cleanup of obsolete code or architecture. | `.leedevkit/.agent/skills/technical-debt-management/SKILL.md` |
| `templates` | Project scaffolding templates for new applications. Use when creating new projects from scratch. Contains 12 templates for various tech stacks. | `.leedevkit/.agent/skills/app-builder/templates/SKILL.md` |
| `testing-patterns` | Testing patterns and principles. Unit, integration, mocking strategies. | `.leedevkit/.agent/skills/testing-patterns/SKILL.md` |
| `vr-ar` | VR/AR development principles. Comfort, interaction, performance requirements. | `.leedevkit/.agent/skills/game-development/vr-ar/SKILL.md` |
| `vulnerability-scanner` | Advanced vulnerability analysis principles. OWASP 2025, Supply Chain Security, attack surface mapping, risk prioritization. | `.leedevkit/.agent/skills/vulnerability-scanner/SKILL.md` |
| `web-design-guidelines` | Review UI code for Web Interface Guidelines compliance. Use when asked to "review my UI", "check accessibility", "audit design", "review UX", or "check my site against best practices". | `.leedevkit/.agent/skills/web-design-guidelines/SKILL.md` |
| `web-games` | Web browser game development principles. Framework selection, WebGPU, optimization, PWA. | `.leedevkit/.agent/skills/game-development/web-games/SKILL.md` |
| `webapp-testing` | Web application testing principles. E2E, Playwright, deep audit strategies. | `.leedevkit/.agent/skills/webapp-testing/SKILL.md` |
<!-- leedevkit:end -->
