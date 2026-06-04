# AGENTS.md — Rules for AI Coding Assistants

> This file is read by AI coding tools (Claude Code, Cursor, Copilot, Windsurf, Gemini, etc.) to provide project-specific instructions.
> For the supreme laws of the project, refer to `GEMINI.md`. To prevent context window bloat, the actual rules have been separated into specific domain rulebooks.

## ⚠️ PTY Compatibility & Execution — CRITICAL

> [!CAUTION]
> **Prevent PTY Hangs.** AI MUST ALWAYS follow these rules when executing commands:
> 1. **Exclusive for `run_command` tool:** You MUST append the string ` > logs/run_{command_slug}.log 2>&1 </dev/null` directly to the `CommandLine` parameter when running **ANY COMMAND**. `{command_slug}` is a short snake_case name for the command (e.g., `cargo_check`, `npm_dev`, `ls`, `docker_up`). Failure to do this WILL hang the AI's terminal. Always prefix with `mkdir -p logs &&` to ensure the directory exists. Example: `CommandLine: "mkdir -p logs && ls -la > logs/run_ls.log 2>&1 </dev/null"`.
> 2. **MANDATORY WORKFLOW FOR MULTILINE FILES/SCRIPTS:** To prevent PTY hangs, AI MUST ALWAYS follow this 2-step workflow instead of using bash commands with multiline strings (like `echo "..." > file`, `git commit -m "..."`, or `python -c "..."`):
>    - **Step 1:** Use the AI's native tool (e.g., `write_to_file` or `multi_replace_file_content`) to create a physical file (e.g., `scratch/temp_script.py`, `scratch/commit_msg.txt`).
>    - **Step 2:** Use the `run_command` tool to execute the created file (e.g., `python scratch/temp_script.py`, or `git commit -F scratch/commit_msg.txt`).
>    - *Rationale:* Passing multiline strings directly into the terminal will break the shell's parser and cause a permanent system hang. Always consider `write_to_file` as your best friend for any task involving multiline text/code!
> 3. **HERMETIC WRAPPERS:** Absolutely do not call `cargo`, `npm`, or `diesel` directly. You must use wrapper scripts in `scripts/` (e.g., `./scripts/_cargo.sh`). See `execution-wrappers.md` for details.
> 4. **AI MUST BE CALM & PATIENT (CRITICAL):** This is a fundamental mindset for all situations. DO NOT RUSH.
>    - Read requirements carefully before starting.
>    - Cross-check risks before modifying source code.
>    - Never jump to conclusions based on assumptions or subjective guesses.
>    - Always verify with concrete evidence before responding.
>    - Always wait for processes to finish, carefully read error logs (if any), and only then report back to the user. Never say "it's done" without actually verifying the outcome.

## 📚 DOMAIN RULEBOOKS (Lazy Load Index)
If your task touches any of these domains, use the `view_file` or `cat` tool to read the corresponding rulebook FIRST and output "Read: [Rulebook]".

### 💻 Code & Architecture
- **Coding Standards:** `@[.agent/rules/coding-standards.md]` (Clean code, errors, security, enterprise grade)
- **Testing Standards:** `@[.agent/rules/testing-standards.md]`
- **Database Rules:** `@[.agent/rules/database-rules.md]` (Diesel, migrations, indexing)
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
