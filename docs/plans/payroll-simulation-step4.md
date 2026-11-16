# Payroll simulation step 4

- `TEST-01`: focused unit plus PostgreSQL integration proof.
- `DB-03`: simulation transaction always rolls back; no payslip, segment, or calculation-run writes survive.
- `ARCH-01`: reuse existing `calculateEmploymentPayslip`; no second calculation engine.
- `AC-01`: simulation endpoint checks `payroll:calculate` before execution.

Scope: current effective policy only. Draft-policy overrides, batch aggregation, and stale-preview metadata remain later steps.
