# ADR: Employment Relationship, Assignment Segments, and Contract Boundaries

| Property | Value |
|---|---|
| Status | Accepted for implementation |
| Date | 2026-11-12 |
| Owner | LeeAttend HRM domain |
| Scope | Employee lifecycle, employment, contract, e-contract, employment phases |

## Context

LeeAttend currently stores employment-like assignment history in `employments`. A department transfer splits one record into dated segments and can create a new segment ID. Contracts reference `employee_contracts.employment_id`; attendance, leave, payroll, schedules, phases, and other records also reference employment IDs.

The dashboard currently renders the ordinary contract registry from `/econtracts`, while e-signature provider APIs are not wired. This mixes legal employment contracts with electronic-signing workflow.

Business decisions:

- One employee MAY have simultaneous employment relationships with different employers.
- One employee MUST NOT have overlapping relationships with the same employer.
- A retroactive change affecting a locked payroll period MUST be rejected. Corrections use a separate adjustment process.
- This implementation separates e-contract surfaces but does not connect a signing provider.

## Decision

Use five explicit concepts:

1. **Employee** — persistent person record.
2. **Employment relationship** — stable identity for one employment episode with one employer. Rehire creates a new relationship, including rehire by the same employer.
3. **Employment segment** — dated assignment within a relationship: department, title, location, and manager. Existing `employments` rows remain segments during migration to avoid rewriting historical foreign keys.
4. **Contract** — legal employment document linked to the relationship. Existing `employment_id` remains a compatibility link until all consumers can use relationship context.
5. **Signing envelope** — an electronic-signing attempt/history for a contract. Envelope state does not determine employment or contract effective state.

Employment phases remain lifecycle milestones. A phase MAY attach to a segment where assignment-specific behavior is required; it MUST NOT replace relationship or contract lifecycle state.

### Identity and history rules

- Department transfer creates or updates a dated segment within the same relationship. It does not create a relationship or contract automatically.
- Contract renewal or replacement creates a new legal document record. It does not create a relationship automatically.
- Termination closes the relationship and applies explicit contract/phase transitions. It does not delete attendance, leave, payroll, or signed-document history.
- Resolver code MUST require relationship/segment context when an employee has multiple employers. It MUST NOT choose the newest employment record as an implicit employer.
- Contract content that is effective or signed is immutable. Amend, replace, terminate, or archive operations preserve prior versions.

## Alternatives

### Rename `employments` and use it as the relationship

Rejected. Existing FK consumers and department-transfer behavior identify `employments` as dated segments. Renaming or reinterpreting it would risk changing historical attendance, leave, payroll, schedule, and phase meaning.

### Keep only the current segment model

Rejected as the final model. It can support a `/contracts` page quickly but cannot provide stable identity across department transfers or unambiguous rehire history.

### Create a new relationship table and migrate all employment FKs immediately

Rejected for the first migration slice. It has the largest blast radius and weak rollback characteristics. Add relationship links first, preserve existing segment IDs, then contract consumers incrementally.

## Consequences

### Positive

- `/employments` can manage employment relationships and dated assignment history without confusing them with legal documents.
- `/contracts` can manage ordinary contracts independently of the e-signature feature flag.
- `/econtracts` can later manage signing envelopes without pretending provider actions succeeded.
- Historical employment IDs remain valid for attendance and payroll consumers during migration.
- Different-employer concurrent employment becomes explicit instead of inferred.

### Negative

- Transitional responses contain both relationship context and legacy employment IDs.
- Resolver and authorization code must handle relationship, segment, employer, and department scope explicitly.
- Existing contract overlap and auto-close behavior needs replacement with explicit legal-document transitions.
- Backfill may produce ambiguous cases that require HR decisions; the system MUST NOT guess those mappings.

## Security, privacy, and operational impact

- Handler authorization MUST verify the employee/relationship/segment/contract parent before business logic. Membership alone is insufficient.
- HR contract and document fields are confidential/internal according to data-governance rules. Collections MUST omit unnecessary sensitive fields; detail and download actions require permission and audit logging.
- State-changing operations require audit records without raw PII or document contents.
- Mutations affecting a locked payroll period fail closed with a conflict/validation error and produce no partial side effect.
- Overlap constraints and transaction locks MUST protect concurrent relationship, segment, and contract writes.

## Migration and rollback

1. Add relationship schema and nullable compatibility links.
2. Reconcile existing rows; preserve existing UUIDs and record ambiguous mappings separately.
3. Read both old and new links while writers are gated by the new domain rules.
4. Move consumers one bounded domain at a time: employment/contract API, phase/lifecycle, assignments, attendance/leave/schedule, payroll/reports.
5. Require reconciliation and integration evidence before enforcing non-null links or removing compatibility fields.
6. Roll back UI and behavior switches without deleting new relationship data. Roll forward when an older binary cannot safely interpret relationship rows.

No destructive schema cleanup is part of this ADR.

## Acceptance criteria

- Employee, relationship, segment, contract, envelope, and phase have distinct ownership and lifecycle semantics.
- Department transfer preserves relationship and contract identity.
- Rehire creates a new relationship; different employers can overlap.
- Locked payroll periods reject retroactive mutations without side effects.
- `/contracts` is separate from `/econtracts`; provider integration remains unavailable until separately implemented.
- API, authorization, migration, rollback, and affected-consumer tests verify the decision.
