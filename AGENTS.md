# AGENTS.md — Rules for AI Coding Assistants

> This file is read by AI coding tools (Claude Code, Cursor, Copilot, Windsurf, Gemini, etc.) to provide project-specific instructions.
> For the supreme laws of the project, refer to `GEMINI.md`. To prevent context window bloat, the actual rules have been separated into specific domain rulebooks.

## ⚠️ PTY Compatibility & Execution — CRITICAL

> [!CAUTION]
> **Tránh PTY Hang.** AI LUÔN LUÔN phải tuân thủ các quy tắc sau khi thực thi lệnh:
> 1. Thêm chuyển hướng (redirect) `> run.log 2>&1 </dev/null` khi chạy các lệnh shell/bash (đặc biệt là lệnh khởi chạy test, docker) để tránh làm treo terminal của chính bạn. KHÔNG ĐƯỢC QUÊN.
> 2. **TUYỆT ĐỐI KHÔNG** sử dụng lệnh `python -c` (hoặc `python3 -c`) kết hợp với code/chuỗi nhiều dòng (multiline strings). Lệnh này sẽ làm treo PTY ngay lập tức do lỗi parsing của shell. Nếu cần chạy script python, hãy tạo một file `.py` riêng biệt (ví dụ trong thư mục `scratch/`) rồi mới chạy lệnh `python file.py`.
> 3. **HERMETIC WRAPPERS:** Tuyệt đối không gọi trực tiếp `cargo`, `npm`, `diesel`. Phải dùng wrapper scripts trong `scripts/` (ví dụ: `./scripts/_cargo.sh`). Chi tiết xem ở `execution-wrappers.md`.

## 📚 DOMAIN RULEBOOKS (Lazy Load Index)
If your task touches any of these domains, use the `view_file` or `cat` tool to read the corresponding rulebook FIRST and output "Read: [Rulebook]".

### 💻 Code & Architecture
- **Coding Standards:** `@[.agent/rules/coding-standards.md]` (Clean code, errors, security, enterprise grade)
- **Testing Standards:** `@[.agent/rules/testing-standards.md]`
- **Database Rules:** `@[.agent/rules/database-rules.md]` (Diesel, migrations, indexing)
- **Project Structure:** `@[.agent/rules/project-structure.md]`
- **Execution Wrappers:** `@[.agent/rules/execution-wrappers.md]` (cargo, npm, diesel, python scripts)

### 🎨 Design
- **UI/UX Design:** `@[.agent/rules/design-rules.md]` (Design system, a11y, responsive)
