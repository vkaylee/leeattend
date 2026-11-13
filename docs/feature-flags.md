# Feature flags

| Key | Default | Owner | Removal target | Purpose |
|---|---:|---|---|---|
| `ff_econtracts` | off | platform | 2026-12-13 | Controls e-Contracts workflow and settings |

## Operations

- Manage flags at `Settings → Feature flags`.
- Reading requires `feature_flags:read`; changing state requires `feature_flags:manage`.
- Runtime consumers fail closed when the effective-flags API is unavailable.
- Production does not honor `localStorage` overrides. DEV-only override remains available for manual testing.
- Every create/update/upsert/delete writes an audit event with actor, flag key or ID, and resulting state.
- Remove flag registry entry, consumers, migration seed, and this row within 30 days after full rollout.
