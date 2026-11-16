# ADR 0004: Retire SaaS Platform-Role Surfaces

## Status
Superseded by [ADR 0005](0005-single-organization-schema-cutover.md). This record preserves the narrower SaaS control-plane removal decision; current runtime and schema policy are defined by ADR 0005.

## Context
LeeAttend removed tenant lifecycle and platform-role administration while moving to one standalone organization. The retained administration model uses organization RBAC, explicit permission checks, ownership boundaries, and fail-closed authorization.

## Decision
1. Removed system tenant lifecycle and platform-role routes remain absent; clients use canonical `/api/v1/...` routes.
2. Retained system, SMTP, health, monitor, and RBAC routes require authenticated organization-admin authorization.
3. The later singleton cutover supersedes this record's compatibility-schema exception and removes request-selected scope from current contracts.
4. Removed API paths remain a breaking change. Generated artifacts, consumers, tests, and operator docs must move together when the authoritative contract source is restored.

## Consequences
- One organization RBAC source remains for administration.
- No tenant provisioning, switching, platform support, or billing-role surface remains.
- Existing installations require protected backup, admin reconciliation, forward migration, and restore rehearsal before destructive schema cleanup.
- Historical migration evidence remains immutable history, not current runtime contract.

## Verification
- Removed paths are absent from supported runtime routes and current generated contracts.
- Retained routes cover admin allow, non-admin/unauthenticated deny, and authorization dependency failure.
- Clean and representative migration runs preserve admin access and satisfy schema/reconciliation checks.
- Full LeeDevKit backend/frontend/integration/E2E gates pass.
