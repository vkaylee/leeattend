# LeeAttend AI Context

## Current runtime

- Go 1.26+ `net/http` API in `apiserver/`
- React/Vite/TypeScript SPA in `webdashboard/`
- PostgreSQL, Redis, and MinIO
- No Rust runtime, Tauri desktop, or Rust migration archive

## Start here

1. [`source-of-truth.md`](source-of-truth.md)
2. [`architecture/system-architecture.md`](architecture/system-architecture.md)
3. [`architecture/codebase-summary.md`](architecture/codebase-summary.md)
4. [`api/api-reference.md`](api/api-reference.md)
5. [`operations/deployment-guide.md`](operations/deployment-guide.md)
6. Relevant active plan under [`plans/`](plans/)
7. Relevant rulebook under `.agent/rules/`

## Verification

```bash
leedevkit test api
leedevkit test web
leedevkit test all
```

Use `leedevkit --help` for available commands. Do not invent commands from old documents.

## Context boundaries

- `docs/archive/` is historical, not active instruction.
- LeeDevKit is an external global tool installed at `~/.leedevkit/current/`; use global `leedevkit` CLI. Do not rely on local `.leedevkit/`.
- `docs/dev-accounts.md` is operationally sensitive; read only for authenticated UI/API testing and never copy credentials into artifacts.
- Do not load every plan. Choose plan by task domain and status.
