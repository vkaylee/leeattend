# ADR: Chi Router and Domain Route Modules

| Property | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-12 |
| Owner | LeeAttend backend |
| Scope | Go API HTTP routing |

## Context

`apiserver/main.go` owns bootstrap, dependency construction, and 379 HTTP route registrations. This forces an agent changing one domain route to load unrelated infrastructure and route code. Public-route authorization currently relies on a string prefix list, which makes route ownership and middleware scope harder to review.

LeeAttend handlers and middleware already use `net/http` types. The migration must preserve HTTP paths, methods, handlers, error responses, authorization, tenant isolation, request ID, rate limiting, CORS, body limits, streaming, and WebSocket behavior.

## Decision

Use `github.com/go-chi/chi/v5` as HTTP router and split route registration into small domain-owned modules. Keep `http.Handler`, `http.HandlerFunc`, and existing middleware contracts. Use chi route groups for public, authenticated, and system scopes. Add a narrow adapter that copies chi route parameters into Go `Request.PathValue` so existing handlers and tests remain unchanged.

Route modules own registration only. Dependency construction remains outside route modules and is passed through a typed dependency boundary. Route modules must not open database/Redis connections or construct services.

## Alternatives

### Keep `net/http.ServeMux` and only split files

Lowest dependency and migration risk. This remains rollback path and fallback if the chi spike fails. It does not provide equally clear nested middleware groups for public, authenticated, and system routes.

### Echo or Gin

Rejected. Their framework context and handler signatures add abstraction and migration surface without reducing LeeAttend handler complexity.

### Fiber

Rejected. It changes transport semantics from `net/http` to `fasthttp`, increasing compatibility and streaming/WebSocket risk.

### Gorilla/mux

Rejected. Project is archived and is not suitable for a new long-lived dependency.

## Consequences

### Positive

- Route ownership becomes visible in file/module boundaries.
- Public, authenticated, and system middleware scope can be reviewed structurally.
- Existing handlers keep `net/http` signatures and request context behavior.
- Agents can modify one domain without reading full bootstrap and route inventory.
- Route contract tests can compare method/path, authorization, and parameter behavior.

### Negative

- Adds one runtime dependency and transitive supply-chain review.
- Chi path parameters need a compatibility adapter while handlers use `Request.PathValue`.
- Migration must test method precedence, 404/405, trailing slash, uploads, streaming, and WebSocket upgrades.
- Router behavior is no longer supplied by Go standard library.

## Security and Operational Impact

- Authentication must be attached to the authenticated route group; public routes must be explicitly listed.
- System routes must retain system authorization and tenant-boundary tests.
- Request ID, rate limiting, locale, CORS, body limits, tracing, and error behavior must retain order and response semantics.
- No PII or credentials enter route tests, logs, or generated artifacts.
- Dependency license, vulnerability, module integrity, and maintenance checks are required before completion.

## Migration and Rollback

1. Record baseline route inventory and run `./leedevkit test api`.
2. Add chi and path-value compatibility tests.
3. Migrate one low-risk route group and compare status/method/path behavior.
4. Migrate remaining groups in small diffs, preserving handlers and middleware.
5. Remove old ServeMux registration only after route parity and negative authorization tests pass.
6. Roll back by reverting router/module commits. No database or API data migration is required.

Removal criterion for transitional adapter: remove only after all handlers stop using `Request.PathValue` or after a deliberate API handler migration with equivalent tests.

## Acceptance Criteria

- All existing API paths and methods remain registered exactly once.
- Existing handler signatures and response/error contracts remain unchanged.
- Public routes do not require authentication; protected routes fail closed without valid auth.
- System and tenant authorization negative tests pass.
- Request path parameters remain available through `Request.PathValue`.
- CORS, body limits, rate limits, request IDs, streaming, and WebSocket upgrade tests pass.
- `./leedevkit test api` passes with integration phases included.
- No unapproved dead code or unused dependency remains.
