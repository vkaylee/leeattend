# Documentation Policy

## Placement

- Runtime architecture, API, operations, development, business, audit, domain, decisions, and plans belong under `docs/`.
- Root keeps only repository entrypoints and governance files.
- `.agent/` contains project rules; `.leedevkit/` contains external tooling. Do not duplicate either under `docs/`.
- Generated artifacts stay beside their owning source or in their existing generated location.

## Status

Every plan or decision must state status: `Draft`, `Awaiting approval`, `Approved`, `Blocked`, `Completed`, or `Superseded`.

Completed or superseded docs move to `docs/archive/` when still useful for audit/history. Delete only confirmed obsolete scratch material with no consumer.

## Truth and links

- Documentation must describe current Go + React/Vite runtime.
- Commands must exist in current project CLI or be marked conditional.
- Paths must exist, or link must point to an explicitly external/conditional resource.
- Update inbound links when moving files.
- Do not store secrets, tokens, credentials, or raw PII in docs or test evidence.

## AI context

Start with `docs/ai-context.md`, then load only source-of-truth docs relevant to task. Do not treat `docs/archive/` as active instructions.
