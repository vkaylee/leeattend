# Schedule resolver integration tests

- `development-workflow.md` §4/§6: cover effective-date boundaries, persistence, and interval edge cases.
- `database-rules.md` §3: use migrated PostgreSQL fixture and parameterized SQL.
- `testing-standards.md` §1/§4: run tagged integration through `scripts/_go-gates.sh --integration`.

Scope: verify `loadShifts` reads SCD shift versions and all default intervals from PostgreSQL, then verify resolver selects correct version at effective-date boundary. Keep concurrent API/employee changes out of this phase.
