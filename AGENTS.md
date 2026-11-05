---
trigger: always_on
---

# ✨ GEMINI.md - Enterprise Context OS

> 🔴 **MANDATORY:** You MUST read and understand this document in its entirety before taking any action. This is the supreme law of this workspace.

<project_soul>
**LeeAttend** is an Enterprise-Grade Premium Pro Max HRM & Attendance system.
- **Architecture:** Go 1.26+ `net/http` API backend + React/Vite/TypeScript SPA frontend.
- **Runtime:** PostgreSQL, Redis, and MinIO support the Go API and React dashboard. Rust is historical migration context only, not an active LeeAttend runtime component.
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
   - **Execution Pattern:** At the beginning of each task/session, ensure the `logs/` directory exists once with `mkdir -p logs`. For ALL terminal commands after that, you **MUST** append ` > logs/run_{command_slug}.log 2>&1 </dev/null` directly inside the `CommandLine` argument; do not repeat `mkdir -p logs &&` unless the directory may have been removed. Example: `CommandLine: "ls -la > logs/run_ls.log 2>&1 </dev/null"`.
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
9. **TESTING COMPLETENESS (CRITICAL - NO FALSE POSITIVES):**
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
- **Backend / Go / API:** `@[.agent/agents/backend-specialist.md]` (Dependencies: `coding-standards.md`, `database-rules.md`, `access-control.md`, `data-governance.md`)
- **Database / Postgres:** `@[.agent/agents/database-expert.md]` (Dependencies: `database-rules.md`, `data-governance.md`, `encryption-rules.md`)
- **DevOps / Infra:** `@[.agent/agents/devops-engineer.md]` (Dependencies: `execution-pty-safety.md`, `testing-standards.md`, `observability-rules.md`, `business-continuity.md`, `supply-chain-security.md`)
- **Security / Audit:** `@[.agent/agents/security-auditor.md]` (Dependencies: `access-control.md`, `encryption-rules.md`, `incident-response.md`, `data-governance.md`)


<!-- leedevkit:begin -->
## LeeDevKit base context

This repository uses LeeDevKit. Before making changes, read and apply:
`.leedevkit/templates/CLAUDE.base.md`.

Source of truth for this project's AI context lives in project `.agent/` and
the installed devkit `.leedevkit/`:

| Purpose | Location |
|---------|----------|
| Rulebooks (lazy-load by domain) | `.agent/rules/*.md` |
| Built-in skills | `.leedevkit/.agent/skills/*/SKILL.md` |
| Installed community skills | `.leedevkit/skills.d/*/SKILL.md` |
| Built-in agents | `.leedevkit/.agent/agents/*.md` |
| Project agents | `.agent/agents/*.md` |
| Devkit commands | `./leedevkit --help` |

Capability fallback: if this harness cannot run shell commands, dispatch
subagents, or track todos, follow the skill's prose directly and report the
missing capability. Never invent a tool call the harness does not provide.

### Available skills

| Skill | When to use | Read |
|---|---|---|
| `2d-games` | 2D game development principles. Sprites, tilemaps, physics, camera. | `.leedevkit/.agent/skills/game-development/2d-games/SKILL.md` |
| `3d-games` | 3D game development principles. Rendering, shaders, physics, cameras. | `.leedevkit/.agent/skills/game-development/3d-games/SKILL.md` |
| `api-and-interface-design` | Guides stable API and interface design. Use when designing APIs, module boundaries, or any public interface. Use when creating REST or GraphQL endpoints, defining type contracts between modules, or establishing boundaries between frontend and backend. | `.leedevkit/skills.d/agent-skills/skills/api-and-interface-design/SKILL.md` |
| `api-patterns` | API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination. | `.leedevkit/.agent/skills/api-patterns/SKILL.md` |
| `app-builder` | Main application building orchestrator. Creates full-stack applications from natural language requests. Determines project type, selects tech stack, coordinates agents. | `.leedevkit/.agent/skills/app-builder/SKILL.md` |
| `architecture` | Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design. | `.leedevkit/.agent/skills/architecture/SKILL.md` |
| `bash-linux` | Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems. | `.leedevkit/.agent/skills/bash-linux/SKILL.md` |
| `behavioral-modes` | AI operational modes (brainstorm, implement, debug, review, teach, ship, orchestrate). Use to adapt behavior based on task type. | `.leedevkit/.agent/skills/behavioral-modes/SKILL.md` |
| `brainstorming` | Targeted clarification and concise user communication for ambiguous or high-impact work. | `.leedevkit/.agent/skills/brainstorming/SKILL.md` |
| `browser-testing-with-devtools` | Tests in real browsers via Chrome DevTools MCP. Use when building or debugging anything that runs in a browser. Use when you need to inspect the DOM, capture console errors, analyze network requests, profile performance, or verify visual output with real runtime data. Requires the chrome-devtools MCP server to be configured. | `.leedevkit/skills.d/agent-skills/skills/browser-testing-with-devtools/SKILL.md` |
| `ci-cd-and-automation` | Automates CI/CD pipeline setup. Use when setting up or modifying build and deployment pipelines. Use when you need to automate quality gates, configure test runners in CI, or establish deployment strategies. | `.leedevkit/skills.d/agent-skills/skills/ci-cd-and-automation/SKILL.md` |
| `clean-code` | Pragmatic coding standards - concise, direct, no over-engineering, no unnecessary comments | `.leedevkit/.agent/skills/clean-code/SKILL.md` |
| `code-review-and-quality` | Conducts multi-axis code review. Use before merging any change. Use when reviewing code written by yourself, another agent, or a human. Use when you need to assess code quality across multiple dimensions before it enters the main branch. | `.leedevkit/skills.d/agent-skills/skills/code-review-and-quality/SKILL.md` |
| `code-review-checklist` | Code review guidelines covering code quality, security, and best practices. | `.leedevkit/.agent/skills/code-review-checklist/SKILL.md` |
| `code-simplification` | Simplifies code for clarity. Use when refactoring code for clarity without changing behavior. Use when code works but is harder to read, maintain, or extend than it should be. Use when reviewing code that has accumulated unnecessary complexity. | `.leedevkit/skills.d/agent-skills/skills/code-simplification/SKILL.md` |
| `context-engineering` | Optimizes agent context setup. Use when starting a new session, when agent output quality degrades, when switching between tasks, or when you need to configure rules files and context for a project. | `.leedevkit/skills.d/agent-skills/skills/context-engineering/SKILL.md` |
| `database-design` | Database design principles and decision-making. Schema design, indexing strategy, ORM selection, serverless databases. | `.leedevkit/.agent/skills/database-design/SKILL.md` |
| `debugging-and-error-recovery` | Guides systematic root-cause debugging. Use when tests fail, builds break, behavior doesn't match expectations, or you encounter any unexpected error. Use when you need a systematic approach to finding and fixing the root cause rather than guessing. | `.leedevkit/skills.d/agent-skills/skills/debugging-and-error-recovery/SKILL.md` |
| `deployment-procedures` | Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts. | `.leedevkit/.agent/skills/deployment-procedures/SKILL.md` |
| `deprecation-and-migration` | Manages deprecation and migration. Use when removing old systems, APIs, or features. Use when migrating users from one implementation to another. Use when deciding whether to maintain or sunset existing code. | `.leedevkit/skills.d/agent-skills/skills/deprecation-and-migration/SKILL.md` |
| `documentation-and-adrs` | Records decisions and documentation. Use when making architectural decisions, changing public APIs, shipping features, or when you need to record context that future engineers and agents will need to understand the codebase. | `.leedevkit/skills.d/agent-skills/skills/documentation-and-adrs/SKILL.md` |
| `documentation-templates` | Documentation templates and structure guidelines. README, API docs, code comments, and AI-friendly documentation. | `.leedevkit/.agent/skills/documentation-templates/SKILL.md` |
| `doubt-driven-development` | Subjects every non-trivial decision to a fresh-context adversarial review before it stands. Use when correctness matters more than speed, when working in unfamiliar code, when stakes are high (production, security-sensitive logic, irreversible operations), or any time a confident output would be cheaper to verify now than to debug later. | `.leedevkit/skills.d/agent-skills/skills/doubt-driven-development/SKILL.md` |
| `frontend-design` | Design thinking and decision-making for web UI. Use when designing components, layouts, color schemes, typography, or creating aesthetic interfaces. Teaches principles, not fixed values. | `.leedevkit/.agent/skills/frontend-design/SKILL.md` |
| `frontend-ui-engineering` | Builds production-quality, accessible, responsive user-facing UIs. Use when building or modifying interfaces and pages, creating components, implementing layouts, meeting WCAG accessibility requirements, managing state, or when the output needs to look and feel production-quality rather than AI-generated. | `.leedevkit/skills.d/agent-skills/skills/frontend-ui-engineering/SKILL.md` |
| `game-art` | Game art principles. Visual style selection, asset pipeline, animation workflow. | `.leedevkit/.agent/skills/game-development/game-art/SKILL.md` |
| `game-audio` | Game audio principles. Sound design, music integration, adaptive audio systems. | `.leedevkit/.agent/skills/game-development/game-audio/SKILL.md` |
| `game-design` | Game design principles. GDD structure, balancing, player psychology, progression. | `.leedevkit/.agent/skills/game-development/game-design/SKILL.md` |
| `game-development` | Game development orchestrator. Routes to platform-specific skills based on project needs. | `.leedevkit/.agent/skills/game-development/SKILL.md` |
| `geo-fundamentals` | Generative Engine Optimization for AI search engines (ChatGPT, Claude, Perplexity). | `.leedevkit/.agent/skills/geo-fundamentals/SKILL.md` |
| `git-workflow-and-versioning` | Structures git workflow practices. Use when making any code change. Use when committing, branching, resolving conflicts, or when you need to organize work across multiple parallel streams. Use when cutting a release, choosing a semantic version bump, tagging, or writing a changelog. | `.leedevkit/skills.d/agent-skills/skills/git-workflow-and-versioning/SKILL.md` |
| `i18n-localization` | Internationalization and localization patterns. Detecting hardcoded strings, managing translations, locale files, RTL support. | `.leedevkit/.agent/skills/i18n-localization/SKILL.md` |
| `idea-refine` | Refines raw ideas into sharp, actionable concepts through structured divergent and convergent thinking. Use when an idea is still vague, when you need to stress-test assumptions before committing to a plan, or when you want to expand options before converging on one. Triggers on "ideate", "refine this idea", or "stress-test my plan". | `.leedevkit/skills.d/agent-skills/skills/idea-refine/SKILL.md` |
| `incremental-implementation` | Delivers changes incrementally. Use when implementing any feature or change that touches more than one file. Use when you're about to write a large amount of code at once, or when a task feels too big to land in one step. | `.leedevkit/skills.d/agent-skills/skills/incremental-implementation/SKILL.md` |
| `intelligent-routing` | Select smallest suitable agent or agent set from task intent, domain, and complexity. | `.leedevkit/.agent/skills/intelligent-routing/SKILL.md` |
| `interview-me` | Extracts what the user actually wants instead of what they think they should want. Achieves this through one-question-at-a-time interview until ~95% confidence about the underlying intent. Use when an ask is underspecified ("build me X" without "for whom" or "why now"), when the user explicitly invokes ("interview me", "grill me", "are we sure?", "stress-test my thinking"), or when you catch yourself silently filling in ambiguous requirements before any plan, spec, or code exists. | `.leedevkit/skills.d/agent-skills/skills/interview-me/SKILL.md` |
| `lint-and-validate` | Automatic quality control, linting, and static analysis procedures. Use after code modifications to ensure syntax correctness and project standards. Triggers on lint, format, check, validate, types, static analysis. | `.leedevkit/.agent/skills/lint-and-validate/SKILL.md` |
| `mcp-builder` | MCP (Model Context Protocol) server building principles. Tool design, resource patterns, best practices. | `.leedevkit/.agent/skills/mcp-builder/SKILL.md` |
| `mobile-design` | Mobile-first design thinking and decision-making for iOS and Android apps. Touch interaction, performance patterns, platform conventions. Teaches principles, not fixed values. Use when building React Native, Flutter, or native mobile apps. | `.leedevkit/.agent/skills/mobile-design/SKILL.md` |
| `mobile-games` | Mobile game development principles. Touch input, battery, performance, app stores. | `.leedevkit/.agent/skills/game-development/mobile-games/SKILL.md` |
| `multiplayer` | Multiplayer game development principles. Architecture, networking, synchronization. | `.leedevkit/.agent/skills/game-development/multiplayer/SKILL.md` |
| `nextjs-react-expert` | React and Next.js performance optimization from Vercel Engineering. Use when building React components, optimizing performance, eliminating waterfalls, reducing bundle size, reviewing code for performance issues, or implementing server/client-side optimizations. | `.leedevkit/.agent/skills/nextjs-react-expert/SKILL.md` |
| `nodejs-best-practices` | Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying. | `.leedevkit/.agent/skills/nodejs-best-practices/SKILL.md` |
| `observability-and-instrumentation` | Instruments code so production behavior is visible and diagnosable. Use when adding logging, metrics, tracing, or alerting. Use when shipping any feature that runs in production and you need evidence it works. Use when production issues are reported but you can't tell what happened from the available data. | `.leedevkit/skills.d/agent-skills/skills/observability-and-instrumentation/SKILL.md` |
| `parallel-agents` | Coordinate specialized subagents for independent analysis, dependent implementation, verification, and synthesis. | `.leedevkit/.agent/skills/parallel-agents/SKILL.md` |
| `pc-games` | PC and console game development principles. Engine selection, platform features, optimization strategies. | `.leedevkit/.agent/skills/game-development/pc-games/SKILL.md` |
| `performance-optimization` | Optimizes application performance across frontend, backend, queries, and databases. Use when performance requirements exist, when you suspect performance regressions, when Core Web Vitals or load times need improvement, when N+1 query patterns need fixing, or when profiling reveals bottlenecks. | `.leedevkit/skills.d/agent-skills/skills/performance-optimization/SKILL.md` |
| `performance-profiling` | Performance profiling principles. Measurement, analysis, and optimization techniques. | `.leedevkit/.agent/skills/performance-profiling/SKILL.md` |
| `plan-writing` | Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work. | `.leedevkit/.agent/skills/plan-writing/SKILL.md` |
| `planning-and-task-breakdown` | Breaks work into ordered tasks. Use when you have a spec or clear requirements and need to break work into implementable tasks. Use when a task feels too large to start, when you need to estimate scope, or when parallel work is possible. | `.leedevkit/skills.d/agent-skills/skills/planning-and-task-breakdown/SKILL.md` |
| `powershell-windows` | PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling. | `.leedevkit/.agent/skills/powershell-windows/SKILL.md` |
| `python-patterns` | Python development principles and decision-making. Framework selection, async patterns, type hints, project structure. Teaches thinking, not copying. | `.leedevkit/.agent/skills/python-patterns/SKILL.md` |
| `red-team-tactics` | Red team tactics principles based on MITRE ATT&CK. Attack phases, detection evasion, reporting. | `.leedevkit/.agent/skills/red-team-tactics/SKILL.md` |
| `rust-pro` | Master Rust 1.75+ with modern async patterns, advanced type system features, production-ready systems programming, Tokio, axum, and cutting-edge crates. Use proactively for Rust development, performance optimization, or systems programming. | `.leedevkit/.agent/skills/rust-pro/SKILL.md` |
| `security-and-hardening` | Hardens code against vulnerabilities. Use when handling user input, authentication, data storage, or external integrations. Use when building any feature that accepts untrusted data, manages user sessions, or interacts with third-party services. | `.leedevkit/skills.d/agent-skills/skills/security-and-hardening/SKILL.md` |
| `seo-fundamentals` | SEO fundamentals, E-E-A-T, Core Web Vitals, and Google algorithm principles. | `.leedevkit/.agent/skills/seo-fundamentals/SKILL.md` |
| `server-management` | Server management principles and decision-making. Process management, monitoring strategy, and scaling decisions. Teaches thinking, not commands. | `.leedevkit/.agent/skills/server-management/SKILL.md` |
| `shipping-and-launch` | Prepares production launches. Use when preparing to deploy to production. Use when you need a pre-launch checklist, when setting up monitoring, when planning a staged rollout, or when you need a rollback strategy. | `.leedevkit/skills.d/agent-skills/skills/shipping-and-launch/SKILL.md` |
| `source-driven-development` | Grounds every implementation decision in official documentation. Use when you want authoritative, source-cited code free from outdated patterns. Use when building with any framework or library where correctness matters. | `.leedevkit/skills.d/agent-skills/skills/source-driven-development/SKILL.md` |
| `spec-driven-development` | Creates specs before coding. Use when starting a new project, feature, or significant change and no specification exists yet. Use when requirements are unclear, ambiguous, or only exist as a vague idea. | `.leedevkit/skills.d/agent-skills/skills/spec-driven-development/SKILL.md` |
| `systematic-debugging` | 4-phase systematic debugging methodology with root cause analysis and evidence-based verification. Use when debugging complex issues. | `.leedevkit/.agent/skills/systematic-debugging/SKILL.md` |
| `tailwind-patterns` | Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture. | `.leedevkit/.agent/skills/tailwind-patterns/SKILL.md` |
| `tdd-workflow` | Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle. | `.leedevkit/.agent/skills/tdd-workflow/SKILL.md` |
| `technical-debt-management` | Systematically analyze, assess, and resolve technical debt. Use when the user requests refactoring, legacy-code optimization, or cleanup of obsolete code or architecture. | `.leedevkit/.agent/skills/technical-debt-management/SKILL.md` |
| `templates` | Project scaffolding templates for new applications. Use when creating new projects from scratch. Contains 12 templates for various tech stacks. | `.leedevkit/.agent/skills/app-builder/templates/SKILL.md` |
| `test-driven-development` | Drives development with tests. Use when implementing any logic, fixing any bug, or changing any behavior. Use when you need to prove that code works, when a bug report arrives, or when you're about to modify existing functionality. | `.leedevkit/skills.d/agent-skills/skills/test-driven-development/SKILL.md` |
| `testing-patterns` | Testing patterns and principles. Unit, integration, mocking strategies. | `.leedevkit/.agent/skills/testing-patterns/SKILL.md` |
| `using-agent-skills` | Discovers and invokes agent skills. Use when starting a session or when you need to discover which skill applies to the current task. This is the meta-skill that governs how all other skills are discovered and invoked. | `.leedevkit/skills.d/agent-skills/skills/using-agent-skills/SKILL.md` |
| `vr-ar` | VR/AR development principles. Comfort, interaction, performance requirements. | `.leedevkit/.agent/skills/game-development/vr-ar/SKILL.md` |
| `vulnerability-scanner` | Advanced vulnerability analysis principles. OWASP 2025, Supply Chain Security, attack surface mapping, risk prioritization. | `.leedevkit/.agent/skills/vulnerability-scanner/SKILL.md` |
| `web-design-guidelines` | Review UI code for Web Interface Guidelines compliance. Use when asked to "review my UI", "check accessibility", "audit design", "review UX", or "check my site against best practices". | `.leedevkit/.agent/skills/web-design-guidelines/SKILL.md` |
| `web-games` | Web browser game development principles. Framework selection, WebGPU, optimization, PWA. | `.leedevkit/.agent/skills/game-development/web-games/SKILL.md` |
| `webapp-testing` | Web application testing principles. E2E, Playwright, deep audit strategies. | `.leedevkit/.agent/skills/webapp-testing/SKILL.md` |
<!-- leedevkit:end -->
