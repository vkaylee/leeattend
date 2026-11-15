# Schedule lifecycle P0 fixes

Scope: publication response propagation and amendment revision lineage.

Rules:
- Testing Standards §1: behavior and regression tests required.
- Development Workflow §3–§4: focused tests, rollback/state coverage.
- API Governance §1/§4: preserve response and retry semantics.
- Database Rules §3: revision chain mutation stays transactional.

Acceptance:
- Human publish returns committed batch ID, revisions, timestamp, and affected days.
- System publication keeps existing behavior.
- Replan replacement stores previous_revision_id.
- Existing first publication remains without previous revision.
- Failed writes roll back through existing transaction boundary.
