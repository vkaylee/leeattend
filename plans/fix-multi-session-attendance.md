# Fix multi-session attendance details

## Rule IDs

- RULE-CODE-ERRORS: preserve explicit Rust error handling and clean domain boundaries.
- RULE-TEST-PIPELINE: add regression coverage and verify through `./leedevkit test`.
- RULE-UI-A11Y: keep the existing accessible, responsive report-table interaction.
- RULE-DATA-GOVERNANCE: expose only workspace-scoped attendance data already authorized by the report endpoint.

## Scope

- Preserve one web row per resolved work segment.
- Return all matched time entries and authoritative day totals from the segment report.
- Calculate work from actual work-session durations without counting gaps as work.
- Make the drawer consume backend day totals once per employment/date.
