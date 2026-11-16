# Code Standards & Conventions

## Go API

- Resolve the single organization from trusted server configuration; never add a client-selectable organization or tenant predicate. Use parameterized SQL through `pgx`/`sqlc`.
- Keep handlers focused on HTTP transport, services on orchestration, and repositories on persistence.
- Return structured API errors. Wrap operational failures with `%w`; never panic or silently discard errors in request or worker paths.
- Validate request bodies, UUIDs, pagination bounds, uploads, and configuration at trust boundaries.
- Use explicit transactions for related mutations and rollback on every error.
- Paginate in PostgreSQL with deterministic ordering and bounded limits.
- Batch related reads; never issue queries inside loops or hold a DB connection during external I/O.

## Frontend

- Use Vite, React Query, TypeScript, and Tailwind CSS 4 conventions already present in `webdashboard/`.
- Keep data access and business logic in hooks/services; UI components render explicit props and state.
- Use generated OpenAPI types as the API source of truth. Do not duplicate generated schemas in hand-written types.
- Use approved UI components, accessible labels, loading skeletons, empty states, and i18n for user-visible changes.
- Use `apiFetch` for API calls, bounded request timeouts, stable test IDs, and Playwright fixtures for browser workflows.

## Python tooling

- Run repository Python tooling through `scripts/_python.sh`.
- Keep dependencies declared in the repository's Python dependency manifest.
- Use configured formatters and linters; do not add ad-hoc host-only commands.

## Security and observability

- Enforce authentication, authorization, rate limits, and entity/department boundaries at the API boundary.
- Propagate request IDs and emit bounded structured logs. Never log secrets or raw PII.
- Treat dependency failures explicitly; preserve data integrity and fail closed for security-sensitive operations.
- Clamp pagination and request sizes; audit sensitive reads, exports, and state-changing operations.

## Verification

- Use `./leedevkit test api`, `./leedevkit test web`, or `./leedevkit test all`.
- Run integration and end-to-end phases after focused checks. Unit-only output is insufficient.
- Capture output under `logs/` and inspect warnings, skipped phases, and integration failures.

## Frontend testing

- Use Playwright for cross-component workflows and verify accessible, user-visible behavior.
- Prefer stable test IDs and semantic roles over localized text selectors.
- Keep tests deterministic and isolated; mock external dependencies through approved fixtures.
- Verify success, validation, authorization, dependency failure, persistence, and compatibility behavior where applicable.

## Performance and data integrity

- Use O(1) lookup maps for repeated frontend joins instead of nested linear searches.
- Store payroll and work-unit calculations using the existing integer precision contract; snapshot finalized values.
- Never recalculate historical payroll from mutable organization settings.

## Real-time features

- WebSocket consumers must expose connection state, bounded retries, cleanup, and a documented polling fallback when required by the feature.
- Use discriminated JSON message types with request/job IDs and timestamps.
- Surface connection failures to users; never leave a silent failed state.
