# Schedule legacy trigger integration tests

- `development-workflow.md` §4/§6: cover success, negative, persistence, migration behavior.
- `database-rules.md` §3/§4: validate transactional integrity and migration rollback.
- `testing-standards.md` §1/§4: run focused and full API gates; report integration limitations.

Scope: add database integration coverage for all eight legacy schedule mutation triggers without touching unrelated payroll changes. Verify official dates reject mutation, inactive/unrelated scope remains writable, and guarded migration rollback fails when official evidence exists.
