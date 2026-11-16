# Phase D — Policy ownership B1–B10

Test impact matrix:

- Ownership: policy membership resolves by employment and business date; no cross-employer fallback.
- Temporal boundaries: membership dates must fall inside employment validity; policy version uses half-open effective ranges.
- Mutation: policy management remains separate from shift management; invalid references and uncovered temporal boundaries reject.
- Historical reads: valid historical employment dates retain historical policy resolution.
- Conflict/failure: missing membership/version returns setup error; overlapping rows remain DB conflicts; no silent latest-row choice.
- Security: handlers keep payroll policy permissions; no policy details leak through shift-only routes.
- Compatibility: existing valid memberships and payloads remain accepted.

Rule IDs: Access Control §§1–2; Data Governance §§1–3; Database Rules §§1–4; API Governance §§1–6; Development Workflow §§2–7.
