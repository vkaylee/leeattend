# Fix all tests

- TEST-1: Run full LeeDevKit suite, including integration/E2E.
- TEST-2: Preserve unrelated uncommitted changes and avoid weakening assertions.
- EXEC-1: Use repository wrappers only; report incomplete environment gates.
- SEC-1: Do not expose credentials, tokens, or PII in logs or fixtures.

## Test impact matrix

- Changed behavior: restore failing test contracts without changing product behavior unless failure proves a regression.
- Existing behavior: preserve current API and UI behavior.
- Boundaries: inspect malformed, empty, and missing translation/mock inputs where failures expose them.
- Failures: preserve dependency and timeout assertions.
- Security: no auth bypass or secret changes.
- Compatibility: keep locale parity and existing API contracts.
- Operations: verify full suite and distinguish environment failures from code failures.
