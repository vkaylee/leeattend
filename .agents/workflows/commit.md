---
description: Commit staged changes with a conventional commit message, bypassing PTY issues from GPG agent
---

# Git Commit Workflow

## Why this workflow exists
When running `git commit` via agent tools (`run_command`), GPG signing spawns
a `gpg-agent` process that inherits the PTY file descriptors. Even after the
commit completes, `gpg-agent` holds the PTY open, causing `command_status` to
report "running" indefinitely.

## Steps

1. Stage the changes (if not already staged):
```bash
git add <files>
```

// turbo
2. Commit with GPG agent workaround — close inherited file descriptors:
```bash
git commit --no-gpg-sign -m "<message>" </dev/null
```

> **Note**: `</dev/null` detaches stdin from the PTY, preventing GPG agent from
> inheriting and holding the file descriptor. `--no-gpg-sign` avoids spawning
> `gpg-agent` which would hold the PTY open indefinitely.

// turbo
3. Verify the commit was created:
```bash
git log --oneline -1
```
