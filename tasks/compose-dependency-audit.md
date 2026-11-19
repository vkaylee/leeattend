# Compose Dependency Audit

## Test impact matrix

- Changed behavior: Compose files and wrappers now resolve repo paths, select declared profiles, validate production/CI manifests, and provide test Redis configuration.
- Existing behavior: Development, test, and production service names and networks remain unchanged.
- Inputs/boundaries: Missing production variables must fail validation; absent `PROJECT_ROOT` must not silently produce blank test mounts; stale profile names must not be accepted.
- Failures: Dependency startup ordering, PgBouncer readiness, missing Redis configuration, invalid Compose interpolation.
- Operations: Validation must cover production, development, test, and merged CI Compose configurations.
- Security: No secrets added; test credentials remain non-production fixtures.

## Rule IDs

- Execution wrappers: use repository Compose/tool wrappers and hermetic commands.
- Configuration management: fail fast on required configuration and detect drift.
- Testing standards: verify changed behavior with focused and full LeeDevKit targets.
- Business continuity: dependency health and accurate readiness checks.
