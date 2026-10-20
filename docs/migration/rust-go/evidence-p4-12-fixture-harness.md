# P4-12 Fixture/Differential Harness Evidence

Implementation: `apiserver/internal/testutil/fixture_runner.go`.

Capabilities:
- Loads canonical fixture JSON with required schema version, fixture ID, method, and path checks.
- Substitutes `{param}` path placeholders and encodes query values.
- Selects Rust `http://127.0.0.1:3000` or Go `http://127.0.0.1:3001` through `BaseURL`.
- Captures status, all response headers, body bytes, fixture ID, base URL, and duration.
- Leaves response comparison exact; P2-08 normalization is applied only by a future comparator from declared fixture rules.

Verification:
- `TestRunFixtureSubstitutesPathAndQuery` uses `httptest.Server`; no external network.
- `TestBaseURL` verifies fixed Rust/Go runtime defaults.
- Full fixture schema preflight: `scripts/validate_migration_fixtures.py` passed 59 fixtures.

Limitations:
- DB, Redis, outbox, audit, and rollback comparators remain interfaces specified in `fixtures/state-comparators.md`; integration requires P4-06/P4-07 containers.
- No Go business endpoint is mounted yet; this runner does not claim end-to-end parity.
