# Hybrid Schedule Lifecycle

Status: Awaiting approval

## Goal
Keep the rule engine for previews; establish published schedules as operational evidence, retain amendments, and preserve approved attendance results.

## Confirmed decisions
1. Publication: automatic for approved fixed-office policies; explicit batch publication for rotating rosters.
2. Closure: per attendance month scoped by employer; reopening requires explicit authority, reason and retained history.
3. Delivery: staged backend/schema/contracts before UI and consumer cutover. The overall workflow remains incomplete until consumers are migrated and end-to-end verification passes.

These choices are confirmed by the user. Detailed implementation approval remains pending.

## Constraints and rule references
- Architecture Governance §1–3: reuse current resolver and work-segment infrastructure where suitable; record lifecycle and ownership in an ADR before implementation.
- Database Rules §3–4: transactional publication/amendment/audit; additive migrations with tested rollback and data preservation.
- Access Control §1–2: explicit publication, amendment, closure and reopening permissions; enforce ownership/department boundaries.
- Data Governance §1–3: minimize captured employee attributes, protect evidence, define retention; do not log raw PII.
- Development Workflow §2–7: verify state transitions, migration, concurrency, failures and compatibility; do not report unit-only evidence as completion.

## Tasks
- [ ] Inspect existing attendance approval/period locks, audit records, work-segment status and consumers; select reuse points and document confirmed gaps.
- [ ] Approve lifecycle/API contract and ADR: preview, publication revision, superseding amendment, attendance closure and authorized correction; distinguish off-day from missing schedule and leave from base schedule.
- [ ] Add minimum persistence and guards: snapshot selected schedule and relevant resolution inputs, publication metadata, immutable revisions, closure references; preserve existing data without fabricating historical publication evidence.
- [ ] Connect resolver to idempotent transactional batch publication; detect stale previews and concurrent changes, preserve manual overrides, record audit atomically; automatically publish only under approved policy.
- [ ] Route official schedule views, attendance matching and no-show reporting through the same published schedule contract; expose missing publication explicitly and remove synthetic roster hours.
- [ ] Add scoped UI actions/history and evidence retrieval with i18n, accessible states and permission checks; show provisional vs published vs closed results clearly.
- [ ] Verify unit then full applicable integration/E2E suites via discovered repository wrappers; test migrations, rollback, repeat requests, concurrent publication, forbidden edits, retroactive corrections, overnight/multi-segment days, and historical reconstruction after rule/profile changes.

## Acceptance
- A closed attendance result can be explained using the original published schedule, relevant rule/pattern/shift inputs, original punches and corrections, calculation policy, and approval records.
- Editing present-day configuration cannot silently alter published schedules or closed attendance.
- Automatic office scheduling requires no per-person daily approval; rotating rosters follow the approved publication policy.
- Unknown/missing schedules are not treated as approved off-days or automatically counted as absence.
- No separate engine, new service, or unbounded pre-generation; discovery must determine bounded publication coverage and consumer cutover strategy.

Implementation has not started. Runtime tests are not applicable to this planning-only artifact.
