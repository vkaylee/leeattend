# Architecture Decision Records (ADR)

> Immutable governance artifacts. Each ADR captures a decision that shapes the codebase — the *why*, the *tradeoffs*, and the *constraints* that flow from it. When a decision is superseded, do NOT edit the ADR: create a new one and mark the old one `Superseded by ADR-XXX`.

## Status Legend

| Status | Meaning |
|--------|---------|
| **Proposed** | Drafted, awaiting review. |
| **Accepted** | Approved, in effect. Code must comply. |
| **Superseded** | Replaced by a newer ADR (link in header). |
| **Deprecated** | No longer applicable; kept for historical record. |

## Index

### Client Infrastructure (Webdashboard, 2026-07-27)

Frontend architectural decisions for the React/TS SPA.

| ID | Title | Status | Scope |
|----|-------|--------|-------|
| [ADR-006](./ADR-006-client-state-model.md) | Client State Model — URL-First Layered State | Accepted | 4-layer state model (server/URL/session/persistent), Zod-validated search, Route-owned contract |

## Authoring Rules

1. Filename format: `ADR-NNN-kebab-case-title.md`. NNN is zero-padded, monotonically increasing.
2. Front-matter block at top: `Status`, `Date`, `Deciders`, `Supersedes` (optional), `Superseded-by` (optional).
3. Body sections: **Context**, **Decision**, **Consequences**, **Alternatives Considered**.
4. Accepted ADRs are effectively immutable. Corrections limited to typos or link fixes. Semantic changes require a superseding ADR.
