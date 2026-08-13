# Dashboard Metrics Shared Contract Plan

## Scope
Build shared backend Rust and frontend TypeScript/Zod dashboard metric contract without changing existing dashboard endpoints or metric semantics.

## Rule IDs
- `coding-standards.md §2, §3, §10`: typed Rust DTOs, explicit status/null semantics, no unsafe frontend casts.
- `testing-standards.md §1, §2, §3`: run tests only through `./leedevkit test`; unit plus full integration verification required.
- `access-control.md §1, §2`: contract must not weaken existing handler authorization; future endpoint migration keeps handler checks before services.
- `data-governance.md §1, §2`: aggregate contract contains no employee PII and restricted state carries no inferable metric values.
- `project-structure.md §5, §6`: preserve handler/service layering and generated-type ownership conventions.

## Files
- Add `apiserver/src/models/dashboard_metrics.rs` with serde/utoipa contract types and serialization/null-handling tests.
- Update `apiserver/src/models/mod.rs` to expose new module while preserving existing re-exports.
- Add `webdashboard/src/hooks/dashboard-metric-contract.ts` with TypeScript types and strict Zod schemas.
- Add `webdashboard/src/hooks/__tests__/dashboard-metric-contract.test.ts` for valid, null, missing, and invalid payloads.
- Add `docs/dashboard-metrics-contract-migration.md` documenting endpoint compatibility and migration path.

## Contract decisions
- Canonical metric identifier: `key`.
- Numerator/denominator: nullable structured quantity objects with value and unit.
- `status`: availability (`available`, `not_applicable`, `data_gap`, `restricted`).
- `health`: optional decision state (`ok`, `warning`, `critical`, `not_applicable`, `data_gap`).
- Missing/null values are explicit and status-dependent; restricted metrics omit value and quantities.
- Supported units include count, rate, hours, currency, duration, percent, and percentage-point delta.

## Verification
1. Run targeted backend and frontend tests through project wrapper.
2. Run full API and web integration suites through wrapper.
3. Report exact pass/fail output; do not modify generated OpenAPI/types or old endpoints.
