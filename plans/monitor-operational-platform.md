# Monitor Operational Platform Compliance

## Scope
Implement and verify the `/infra/monitor` operational monitoring contract across the Axum API, OpenAPI/generated frontend types, React dashboard, and unit/integration tests.

## Rule IDs
- **C-15 / C-16** — structured `AppError` handling and zero-panic production code.
- **O-23** — backend observability contract must expose measured health/freshness and request tracing-compatible data.
- **T-09 / T-42** — unit tests are insufficient; run full wrapper-based integration/E2E verification.
- **A-04 / A-15** — preserve authorization and fail-closed behavior for infrastructure monitoring.
- **D-68 / D-70** — generated OpenAPI types are canonical; no hand-written duplicate schema drift.
- **U-33 / U-35** — frontend loading, empty, unavailable, and accessible state presentation.

## Contract decisions
- Use nullable metric values for source-dependent numbers; `null` means unknown/unavailable and must not be coerced to zero.
- Return known worker records with explicit `live`, `stale`, or `offline` state and numeric heartbeat age in seconds.
- Use measured `observed_at` and `snapshot_age_seconds`; derive freshness from configured thresholds rather than a hardcoded label.
