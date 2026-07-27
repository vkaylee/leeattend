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

### Attendance Policy Engine Refactor (V5, 2026-07-27)

Layered Explicit Policy System — 6-layer cascade replacing fragmented hardcoded overrides in `shift_default_intervals` and runtime `ShiftPolicyOverrides`.

| ID | Title | Status | Scope |
|----|-------|--------|-------|
| [ADR-001](./ADR-001-foundation-up-layered-hierarchy.md) | Foundation-Up Layered Hierarchy & Explicit Priority Rank | Accepted | 6-layer cascade order, L2.5 priority_rank, EXCLUDE GIST constraint |
| [ADR-002](./ADR-002-field-level-option-cascade.md) | Field-Level `Option<T>` Cascade & Absolute Delta Caps | Accepted | Merge formula, delta semantics, clamp warnings |
| [ADR-003](./ADR-003-time-sliced-evaluation.md) | Time-Sliced Evaluation & Per-Rule Anchoring Matrix | Accepted | Overnight decomposition, 4-rule anchoring matrix, VN-only scope |
| [ADR-004](./ADR-004-immutable-snapshot-cascade-trace.md) | Hybrid Immutable Snapshot, Cascade Trace & 3-State Lifecycle | Accepted | DRAFT/ACTIVE/FROZEN, `rules_hash` generated column, supersede-recompute |
| [ADR-005](./ADR-005-dual-run-shadow-rollback.md) | Dual-Run Shadow Mode & Rollback Protocol | Accepted | Diff taxonomy A/B/C, 14-day shadow, 30-day dual-run, legacy edit migration |

## Related

- **Implementation plan:** `.claude/plans/snoopy-yawning-quasar.md` (V5 refactor breakdown).
- **Rulebooks impacted:** `.attendagent/rules/database-rules.md`, `.attendagent/rules/data-governance.md`, `.attendagent/rules/change-management.md`.
- **Code entry points:** `apiserver/src/domain/policy/resolver.rs`, `apiserver/src/services/pairing/service.rs`, `apiserver/src/models/attendance.rs`.

## Authoring Rules

1. Filename format: `ADR-NNN-kebab-case-title.md`. NNN is zero-padded, monotonically increasing.
2. Front-matter block at top: `Status`, `Date`, `Deciders`, `Supersedes` (optional), `Superseded-by` (optional).
3. Body sections: **Context**, **Decision**, **Consequences**, **Alternatives Considered**.
4. Accepted ADRs are effectively immutable. Corrections limited to typos or link fixes. Semantic changes require a superseding ADR.
