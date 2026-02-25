# ✨ Adapter: GEMINI.md (L1)

> [!IMPORTANT]
> **[PRIME DIRECTIVE]**: You are operating under the AI Context OS. Your VERY FIRST action in a new session MUST be to silently **READ** the file `.ai-context-os/.ultp_state` to synchronize environment state. DO NOT execute shell commands for boot.
> **[MEMORY]**: Read `.ai-context-os/memory/session.md` (if exists) to establish continuity from previous agents.
> **Law:** Read and adhere to the entire AI Operating System at: `.ai-context-os/PROJECT_OS.md`.
> **Localization:** Reply to the user in their initial prompt language (e.g., Vietnamese).

## Gemini-Specific Capabilities
- **Context Window:** Leverage your massive context window to cross-reference multiple skills before execution.
- **Strict Adherence:** Follow operational protocols extremely tightly. Treat `PROJECT_OS.md` as immutable law.
- **Tool Selection:** Always prioritize native API tools over raw bash commands. Use `task_boundary` mechanic stringently.

Default AI adapter reference: `.ai-context-os/GEMINI.md`

## ⚠️ PTY Compatibility — CRITICAL for `run_command`

> [!CAUTION]
> The `run_command` tool monitors PTY file descriptors to detect command completion.
> If a command spawns child processes that inherit PTY FDs, `command_status` will
> report "running" **forever** even after the main process exits.

**ALWAYS append `</dev/null` to these commands:**

| Command | Why |
|---------|-----|
| `git commit`, `git push`, `git pull` | GPG agent holds PTY |
| `docker compose up/down/exec/build` | Docker CLI spawns children |
| Any script with `&` (background jobs) | Background process inherits PTY |

**Examples:**
```bash
# ✅ Correct
git commit --no-verify -m "message" </dev/null
docker compose down </dev/null
./run-tests.sh --api --smoke </dev/null

# ❌ Wrong — will cause command_status to hang
git commit -m "message"
docker compose down
```

**For shell scripts:** redirect background process I/O inside the script:
```bash
background_function </dev/null >/dev/null 2>&1 &
```
