# AGENTS.md — Rules for AI Coding Assistants

> This file is read by AI coding tools (Claude Code, Cursor, Copilot, Windsurf, Gemini, etc.) to provide project-specific instructions.
> To prevent context window bloat, the actual rules have been separated into specific domain rulebooks.

## ⚠️ PTY Compatibility & Execution — CRITICAL

> [!CAUTION]
> **CRITICAL:** When running commands via terminal tools, child processes can inherit PTY file descriptors and prevent command completion detection. The agent will hang waiting for a "done" status that never arrives.

### 1. The Hermetic Toolbox (MANDATORY)
All standard development tools run inside isolated Podman containers. You MUST execute them via these wrapper scripts instead of using native commands on the host:

| Tool | ❌ WRONG (Direct on Host) | ✅ RIGHT (Inside Container) | Target Container |
| :--- | :--- | :--- | :--- |
| Rust/Cargo | `cargo build` | `./scripts/_cargo.sh build` | `apiserver_test` |
| Node/NPM | `npm install` | `./scripts/_npm.sh install` | `web_test` |
| SQLx/DB | `sqlx migrate run` | `./scripts/_sqlx.sh migrate run` | `apiserver_test` |

### 2. Git & Multiline Strings
- **Git Protocol:** NEVER run `git` directly. ALWAYS use the `./scripts/git.sh` wrapper (e.g., `./scripts/git.sh commit -m "msg"`) to prevent PTY hangs.
- **Multiline Strings:** NEVER pass strings containing literal newlines into terminal commands. Write multi-line content to a temp file (`/tmp/msg.txt`), then execute the command reading from that file (`-F /tmp/msg.txt`).

### 3. Background Process Stream Redirection
When executing any background command (`&`) or long-running pipe (`tail -f`), you MUST sever it from the PTY stream:
- **ALWAYS append:** `</dev/null >/dev/null 2>&1`
- **Example:** `./start-server.sh </dev/null >/dev/null 2>&1 &`

### 4. Banned Inline Commands & Patterns
> **Do NOT use these via AI terminal tools.** They cause infinite PTY hangs due to unclosed quotes or waiting for stdin.
- Heredocs (`cat <<EOF` or `cat >> file <<EOF`)
- Pager commands (`git log`, `less`, `more`). Always use `--no-pager`.
- Piped commands holding PTY (`rg | awk`, `grep | sed`). Append `</dev/null`.

### 5. Safe Inline Code Runner
> **NEVER use `python -c`, `python3 -c`, `node -e`, `bash -c` for inline code.**
Use `run-inline` instead — writes code to a temp file, executes via `_safe_run.py`:
- `echo 'print(1)' | ./scripts/ai-tools/run-inline -l python`
- `./scripts/ai-tools/run-inline -l bash -c 'echo $HOME'`

## 📚 DOMAIN RULEBOOKS (Lazy Load Index)
If your task touches any of these domains, use the `view_file` or `cat` tool to read the corresponding rulebook FIRST.

### 💻 Code & Architecture
- **Coding Standards:** `@[.agent/rules/coding-standards.md]` (Clean code, errors, security, enterprise grade)
- **Testing Standards:** `@[.agent/rules/testing-standards.md]`
- **Database Rules:** `@[.agent/rules/database-rules.md]` (SQLx, migrations, indexing)
- **Project Structure:** `@[.agent/rules/project-structure.md]`

### 🎨 Design
- **UI/UX Design:** `@[.agent/rules/design-rules.md]` (Design system, a11y, responsive)
