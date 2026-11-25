# Feature flags

| Key | Default | Owner | Removal target | Purpose |
|---|---:|---|---|---|
| `ff_econtracts` | off | platform | 2026-12-13 | Controls e-Contracts workflow and settings |
| `ff_ess` | off | platform | 2026-12-15 | Controls Employee Self-Service workflows and routes |
| `ff_employee_identity_match` | off | platform | 2026-12-25 | Controls employee identity preflight (`POST /api/v1/employees/match`), explicit tenure reuse (`POST /api/v1/employees/{employee_id}/tenures`), and `/employees/new` preflight UI (pilot: `admin`, `hr_manager` via `employees:identity:match`) |

## Operations

- Manage flags at `Settings → Feature flags`.
- Reading requires `feature_flags:read`; changing state requires `feature_flags:manage`.
- Runtime consumers fail closed when the effective-flags API is unavailable.
- Production does not honor `localStorage` overrides. DEV-only override remains available for manual testing.
- Every create/update/upsert/delete writes an audit event with actor, flag key or ID, and resulting state.
- Unknown DB flags appear with `Unregistered consumer` in admin UI, but never create routes, menus, permissions, or feature behavior.
- Runtime consumers must register known keys in `webdashboard/src/lib/feature-flags.ts` and gate behavior explicitly.
- Remove flag registry entry, consumers, migration seed, and this row within 30 days after full rollout.

### `ff_employee_identity_match` Rollout & Rollback

- **Prerequisites:** Configure `IDENTITY_LOOKUP_HMAC_SECRET` (>= 32 bytes) and run the idempotent lookup hash backfill before enabling.
- **Pilot scope:** Enabling `ff_employee_identity_match` exposes preflight and explicit tenure reuse only to roles holding `employees:identity:match` (`admin`, `hr_manager`); all other roles remain denied (`403`).
- **Observability:** Structured `employee_identity_metric` logs emit aggregate counts and latency only (`operation`, `outcome` tier/conflict/rollback, `candidate_count`, `latency_ms`) — zero raw PII, lookup hashes, or candidate IDs.
- **Instant rollback (kill switch):** Set `ff_employee_identity_match` to `false` in `Settings → Feature flags` (or via `PATCH /api/v1/feature-flags/{id}`). Within the 60s backend gate cache TTL, `/api/v1/employees/match` and `/api/v1/employees/{employee_id}/tenures` return `403 feature_disabled` and `/employees/new` hides identity preflight. Existing `POST /api/v1/employees` and `POST /api/v1/employees/{id}/rehire` continue working; additive `employee_profiles.*_lookup_hash` columns remain untouched without destructive schema changes.
