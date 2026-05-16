# AGENTS.md — Rules for AI Coding Assistants

> This file is read by AI coding tools (Claude Code, Cursor, Copilot, Windsurf, Gemini, etc.) to provide project-specific instructions.
> For the supreme laws of the project, refer to `GEMINI.md`. To prevent context window bloat, the actual rules have been separated into specific domain rulebooks.

## ⚠️ PTY Compatibility & Execution — CRITICAL

> [!CAUTION]
> **Tránh PTY Hang.** AI LUÔN LUÔN phải thêm chuyển hướng (redirect) `> run.log 2>&1 </dev/null` khi chạy các lệnh shell/bash (đặc biệt là lệnh khởi chạy test, docker) để tránh làm treo terminal của chính bạn. KHÔNG ĐƯỢC QUÊN.

## 📚 DOMAIN RULEBOOKS (Lazy Load Index)
If your task touches any of these domains, use the `view_file` or `cat` tool to read the corresponding rulebook FIRST and output "Read: [Rulebook]".

### 💻 Code & Architecture
- **Coding Standards:** `@[.agent/rules/coding-standards.md]` (Clean code, errors, security, enterprise grade)
- **Testing Standards:** `@[.agent/rules/testing-standards.md]`
- **Database Rules:** `@[.agent/rules/database-rules.md]` (Diesel, migrations, indexing)
- **Project Structure:** `@[.agent/rules/project-structure.md]`

### 🎨 Design
- **UI/UX Design:** `@[.agent/rules/design-rules.md]` (Design system, a11y, responsive)
