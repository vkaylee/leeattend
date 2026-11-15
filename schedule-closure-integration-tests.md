# Attendance closure integration tests

- `development-workflow.md` §4/§6: cover state transitions, stale versions, persistence, and failure paths.
- `database-rules.md` §3: verify transaction boundaries and immutable closure evidence.
- `testing-standards.md` §1/§4: run focused and full API gates; report tagged DB integration execution separately.

Scope: test existing attendance closure repository against migrated PostgreSQL using a minimal employer/payroll-period fixture. Cover close, reopen, reclose, stale version, payroll lock rejection, and immutable closure rows. Do not modify unrelated payroll work.
