---
trigger: always_on
---

# ✨ GEMINI.md - Enterprise Context OS

> 🔴 **MANDATORY:** You MUST read and understand this document in its entirety before taking any action. This is the supreme law of this workspace.

## 🥇 Layer 1: CRITICAL PROTOCOL (Start Here)

> 🔴 **Rule Priority:** P0 (`GEMINI.md`) > P1 (Agent specific files) > P2 (Skill specific files) > P3 (Architecture Decision Records - ADRs).

### 1. The "Read → Understand → Apply" Protocol
**Before writing ANY code or proposing solutions, you MUST:**
1. **Read** the relevant documentation, skills, and context files.
2. **Read ADRs:** Check `.ai/ADRs/` or `docs/architecture/` for existing architectural constraints.
3. **Understand** the "why" behind the patterns, not just copy-paste.
4. **Apply** the principles to the specific problem at hand.
*❌ Failure Condition:* Writing generic code or breaking existing architecture due to ignoring ADRs leads to rejection.

### 2. Selective Skill Loading
- Do NOT read every single skill file. Check the `skills:` frontmatter in the Agent file, then read ONLY the sections of the `SKILL.md` that are relevant to the user's immediate request.

### 3. MANDATORY Rule Compliance Mapping (NEW)
> 🔴 **MANDATORY:** For EVERY `COMPLEX CODE` or `DESIGN/UI` request, the AI MUST:
> 1. Create/Update a `{task-slug}.md` plan file.
> 2. Include a top-level section: `## 🛡️ Rule Compliance Check`.
> 3. Explicitly list at least 3 Rule IDs from this `GEMINI.md` that apply to the current task.
> 4. **STOP** and wait for user confirmation if any rule is ambiguous.
*❌ Failure Condition:* Proposing code or commands without an explicit Rule Compliance mapping is a P0 Protocol Violation.

---

## 📥 Layer 2: REQUEST CLASSIFIER

**Classify the user's request immediately upon receipt:**

| Request Type | Trigger Keywords | Active Tiers | Result |
| :--- | :--- | :--- | :--- |
| **QUESTION** | "what is", "how does", "explain" | TIER 0 only | Text response only. No file changes. |
| **SURVEY/INTEL** | "analyze", "list files", "overview" | TIER 0 + Explorer | Session Intel. No file changes. |
| **SIMPLE CODE** | "fix", "add", "change" (single file) | TIER 0 + TIER 1 (lite) | Inline file edit. |
| **COMPLEX CODE** | "build", "create", "implement", "refactor" | TIER 0 + TIER 1 (full) | **Requires task plan file (`{slug}.md`) & ADR update** |
| **DESIGN/UI** | "design", "UI", "page", "dashboard" | TIER 0 + TIER 1 + UI Agent | **Requires task plan + Agent invocation** |

*❌ Failure Condition:* Executing a `COMPLEX CODE` request as a simple inline edit will lead to broken dependencies and architectural drift.

---

## 🤖 Layer 3: INTELLIGENT ROUTING & ORCHESTRATION

**Automatically route tasks to the most qualified internal agent persona.**

### 1. Auto-Detection & Announcement
Detect the primary domain from the request. You MUST inform the user which persona/expertise you are applying before generating the response:
```markdown
🤖 **Applying knowledge of `@[agent-name]`...**
```

### 2. Cross-Agent Handoff Protocol
When a feature spans multiple domains (e.g., API + UI):
- **Contract First:** The `backend-specialist` MUST define and finalize the API Contract (OpenAPI/types) BEFORE the `frontend-specialist` writes UI logic.
- **Zero Assumptions:** Agents MUST NOT mock or assume endpoints that are not documented in the contract.
*❌ Failure Condition:* Frontend breaking due to mismatched API payloads from parallel uncoordinated agent execution.

---

## 🌍 Layer 4: TIER 0 — UNIVERSAL RULES (Always Active)

### 4.1 Language Handling
- **Communication:** Respond in the language the user initiated the prompt with.
- **Code:** ALL variables, function names, and code comments MUST be in English.

### 4.2 Clean Code Standards
- **SOLID & DRY:** Enforce Single Responsibility. Extract reusable logic.
- **Self-Documenting:** Write clear names. Minimal comments. No over-engineering.

### 4.3 Testing Standards
- **Testing Pyramid:** Prioritize Unit > Integration > E2E. Use AAA Pattern (Arrange-Act-Assert).
- **Code Coverage:** Minimum **80%** coverage for all new business logic.
- **Mandatory Pass:** Task is NOT complete until `./test.sh` passes successfully.

### 4.4 Error Handling & Observability
- **Structured Errors:** Use domain-specific error types (e.g., `AppError`). Never throw generic `Error`.
- **Error Boundaries:** Wrap all independent UI components in React Error Boundaries.
- **Telemetry Baseline:** All backend APIs MUST include request tracing/metrics (e.g., OpenTelemetry span/context).
*❌ Failure Condition:* Silent failures in production due to missing telemetry or unhandled rejections.

### 4.5 Security & Compliance (PII Data Masking)
- **Data Privacy:** NEVER log Personally Identifiable Information (PII) like emails, passwords, phone numbers, or tokens in raw text. Mask them (e.g., `user_***@domain.com`).
- **Secrets:** NEVER hardcode secrets. Use environment variables.
*❌ Failure Condition:* Logging raw PII data violates GDPR/PDPA and causes an immediate P0 Security Incident.

### 4.6 Boy Scout Rule
If you see a bug, typo, or poor pattern nearby while working on a file, FIX IT IMMEDIATELY.
> 🔴 **NEVER say "this is out of scope for the current task" when a clear bug is visible.**

| Situation | ❌ Wrong | ✅ Right |
| :--- | :--- | :--- |
| Found typo in unrelated function | Ignore it | Fix the typo alongside the main task |
| Hardcoded string found | Skip it | Extract to constants/i18n |

### 4.7 AI Logging & Workspace Memory
- **Memory Retention:** Document major architectural shifts in `.ai/ADRs/` to retain long-term AI context.
- **Clean Workspace:** Do NOT save `.txt` or temporary planning files in project root. Use `/tmp/` or prefix `ai_` (added to `.gitignore`).

### 4.8 Script Discipline & PTY Safety (NEW)
> 🔴 **MANDATORY SUPREME RULE (ANTI-LAZINESS):** You are STRICTLY FORBIDDEN from using inline code execution flags (`python -c`, `node -e`, etc.) or multiline string commands (Heredocs).
> - **CONSEQUENCE**: Using these causes infinite PTY Deadlocks.
> - **Protocol:** You MUST read `AGENTS.md` for the full list of banned commands. Always create physical script files in `scripts/ai-tools/` and execute them via `./scripts/_safe_run.py`.
> - **Git Protocol:** NEVER run `git` directly (especially `git commit`). ALWAYS use the `./git.sh` wrapper (e.g., `./git.sh commit -m "message"`) to prevent PTY hangs caused by multi-line AI commits.

### 4.9 The Universal Command Rule: "Logic belongs to Files" (CRITICAL)
> 🔴 **MANDATORY:** Any command (Bash, Python, Node, etc.) that contains **Logic** (loops, conditionals, multi-line blocks, or complex piping) MUST be placed in a physical script file.
- **Never** use inline strings for complex automation.
- **Threshold**: If a command exceeds 1 line or 100 characters, it belongs in a file.
- **Result**: This ensures repeatability, easier debugging, and zero PTY hangs.
*❌ Failure Condition:* Proposing a complex one-liner is a P0 Protocol Violation.

### 4.10 PTY & FD Leak Safety (CRITICAL)
> 🔴 **MANDATORY:** AI environments (like Antigravity) use strict PTY stream tracking. Any background process holding `stdout`/`stderr` will cause an infinite hang.
- **NEVER** run background scripts (`&`) or pipes (`|`) that inherit the PTY's `stdout/stderr` without full redirection.
- **ALWAYS** append `</dev/null >/dev/null 2>&1` to ANY background command.
- **NEVER** leave orphaned processes (like `tail -f` or `grep`) running after the main script exits.
*❌ Failure Condition:* The AI agent hangs indefinitely waiting for a background command's EOF.

### 4.11 Hermetic Toolbox (Zero Host Dependency)
> 🔴 **MANDATORY:** NEVER run `cargo`, `npm`, or `sqlx` directly on the host machine. These tools run inside isolated Podman containers via wrapper scripts to guarantee a reproducible, host-agnostic environment.

| Tool | ❌ WRONG (Direct on Host) | ✅ RIGHT (Inside Container) | Target Container |
| :--- | :--- | :--- | :--- |
| Rust/Cargo | `cargo build` | `./scripts/_cargo.sh build` | `apiserver_test` |
| Node/NPM | `npm install` | `./scripts/_npm.sh install` | `web_test` |
| SQLx/DB | `sqlx migrate run` | `./scripts/_sqlx.sh migrate run` | `apiserver_test` |

*❌ Failure Condition:* Running tools directly on the host bypasses container isolation, causes "works on my machine" bugs, and can corrupt the host environment.

### 4.11 Enterprise-Grade Premium Standard (MANDATORY)
> 🔴 **MANDATORY:** Every single feature, component, and API developed in this workspace MUST default to an **Enterprise-Grade Premium Ready** standard. Do NOT deliver MVP-style or basic implementations.
- **Frontend (UI/UX):** Use premium design patterns (e.g., Combobox/Radix instead of native selects). Implement full i18n support, Skeleton loaders, empty states, Error Boundaries, and smooth micro-interactions.
- **Backend (API/DB):** Architecture must be production-ready from day one. This means strict validation (Type-safety, UUIDs), explicit error handling mapping to AppError, proper transaction boundaries, comprehensive audit logging for sensitive actions, and structured telemetry.
*❌ Failure Condition:* Delivering a bare-bones UI with hardcoded text or an API without proper error handling/validation will be instantly rejected.

### 4.8 Performance & Resource Optimization
- **Performance Budget:** JS bundle `< 200KB` gzipped. Core Web Vitals: LCP < 2.5s. Ensure zero memory leaks for SPAs.
- **Scalability (Backend):** ZERO N+1 Queries allowed. ALL public APIs MUST implement Rate Limiting to prevent DDoS and resource exhaustion.
*❌ Failure Condition:* Merging an endpoint with N+1 query logic or without Rate Limit triggers an immediate P0 regression.

### 4.9 Escalation Protocol (AI Hard-Stop)
- **The Rule:** If an error (e.g., Infra, DevOps, third-party outage) persists after **2 failed self-correction attempts**, the AI MUST STOP IMMEDIATELY.
- **Action:** Generate an Incident Report detailing the error and ask for human intervention. Do NOT enter an endless retry loop.
*❌ Failure Condition:* Wasting tokens on infinite loops trying to fix an out-of-scope infrastructure issue.

---

## 💻 Layer 5: TIER 1 — CODE RULES (When Writing Code)

### 5.1 Project Type Routing
| Project Type | Primary Agent | Required Skills / Principles |
| :--- | :--- | :--- |
| **Frontend (React/Vite)** | `frontend-specialist` | React Best Practices, Clean UI, Tailwind Patterns |
| **Backend (Rust/Axum)** | `backend-specialist` | Rust Pro, API Patterns, Safe Concurrency |
| **Database (Postgres)** | `database-expert` | Schema Validation, Safe Migrations, Indexing |
| **Infrastructure** | `devops-engineer` | Containerization, CI/CD, Observability |

### 5.2 Socratic Gate (Mandatory Pre-Flight Check)
| Request Type | Strategy | Required Action |
| :--- | :--- | :--- |
| **New Feature** | Deep Discovery | 🔴 ASK minimum 3 strategic questions regarding purpose, scale, and edge cases. |
| **Bug Fix** | Context Check | 🔴 Confirm understanding of the root cause + ask impact questions before fixing. |
*❌ Failure Condition:* Implementing a feature based on assumptions rather than clarified requirements results in wasted effort.

### 5.3 Database Rules (Postgres / SQLx)
- **Mandatory Rollbacks:** EVERY migration MUST include a corresponding `down.sql` script for Disaster Recovery. No exceptions.
- **No Destructive Actions:** NEVER `DROP TABLE/COLUMN` in production migrations without user sign-off.
- **Indexing & Validation:** Add indexes for foreign keys. Enforce DB-level constraints (NOT NULL, UNIQUE, CHECK).
*❌ Failure Condition:* Creating an `up.sql` migration without its `down.sql` counterpart prevents system recovery and blocks deployment.

### 5.4 Verification Pipeline
> 🔴 **Execution Priority:** Security → Lint → Schema → Tests → UX → SEO → Performance

**Mandatory Pipeline:**
1. `./scripts/_cargo.sh check` / `./scripts/_npm.sh run typecheck` (Type Safety)
2. `./scripts/_cargo.sh clippy` / `./scripts/_npm.sh run lint` (Linting)
3. `./scripts/_sqlx.sh migrate run` (Schema)
4. `./test.sh` (Tests)

*❌ Failure Condition:* Running `cargo` or `npm` directly instead of the wrapper scripts breaks container isolation.

---

## 🎨 Layer 6: TIER 2 — DESIGN RULES (UI/UX)
- **Design System:** Use components in `webdashboard/src/components/ui`. Don't reinvent the wheel.
- **Accessibility:** Minimum WCAG 2.1 AA compliance. Mandatory `aria-labels` and keyboard focus states.
- **Responsive & Motion:** Mobile-first approach. Use subtle transitions (`duration-200`); avoid jarring animations.

---

## 📚 Layer 7: QUICK REFERENCE
- **Core Agents:** `frontend-specialist`, `backend-specialist`, `database-expert`, `orchestrator`.
- **Project Structure:** `/apiserver` (Rust Backend), `/webdashboard` (React SPA), `/agent-main` (Rust Worker).

### Hermetic Toolbox Scripts
All commands run **inside Podman containers**, never on the host. Internal utility scripts and modules MUST start with an underscore (`_`) prefix.

| Script | Replaces | Container | Purpose |
| :--- | :--- | :--- | :--- |
| `./scripts/_cargo.sh <cmd>` | `cargo` | `apiserver_test` | Rust compilation, checks, clippy |
| `./scripts/_npm.sh <cmd>` | `npm` | `web_test` | Node.js installs, builds, type checks |
| `./scripts/_sqlx.sh <cmd>` | `sqlx` | `apiserver_test` | DB migrations, schema validation |
| `./git.sh <cmd>` | `git` | Host | Safe Git operations (prevents PTY hangs) |
| `./scripts/_sync-api.sh` | (manual) | Multiple | Generate TS types from OpenAPI |
| `./test.sh --all` | (orchestrator) | Multiple | Full test suite |

---
