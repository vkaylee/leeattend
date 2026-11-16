# Full schedule lifecycle chain integration

Scope: prove one real PostgreSQL workflow from rotating preview through publication, replan, attendance closure, reopen, reclose and evidence export.

Rules:
- Development Workflow §4: state transition, persistence, idempotency and failure dimensions.
- Testing Standards §1/§3: integration evidence required; unit-only is insufficient.
- Database Rules §3: related mutations stay transactional and immutable evidence remains preserved.
- API Governance §4: lifecycle mutations use preview/revision tokens and idempotency.

Acceptance:
- Rotating preview creates provisional output without official revision.
- Publication creates current official revision and segment.
- Replan supersedes old revision and links `previous_revision_id`.
- Close snapshots replacement revision; reopen preserves old closure; reclose creates new closure.
- Evidence export returns both revisions, segments, diff lineage and closure days.
