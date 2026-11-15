# Payroll historical rerun snapshot

Status: verified

- `TEST-01`: append integration and regression coverage for immutable calculation history, point-in-time policy resolution, and locked-period protection; disposable PostgreSQL rehearsal passed.
- `DB-03`: change persistence/index contract without overwriting historical snapshots; reversible migration; forward migration chain passed after fixing a PostgreSQL-invalid `DISTINCT` plus `FOR UPDATE` query; disposable rollback rehearsal passed for clean history and failed closed for duplicate history.
- `ARCH-01`: keep calculation provenance in existing `payroll_calculation_runs`; no second snapshot store.

Scope:
- Make policy-only calculation runs append-only.
- Remove only policy-only `(payslip_id)` uniqueness that forces overwrite.
- Preserve formula-linked idempotency and legacy payroll arithmetic.
- Verify focused repository tests and applicable API gate; do not touch unrelated dirty files.
