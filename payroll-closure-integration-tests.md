# Payroll lock and attendance closure integration tests

- `development-workflow.md` §4/§6: cover dependency success/failure and transaction state.
- `database-rules.md` §3: migrated PostgreSQL fixture.
- `testing-standards.md` §1/§4: tagged integration wrapper required.

Payroll lock must reject open attendance and succeed only after attendance period is closed.
