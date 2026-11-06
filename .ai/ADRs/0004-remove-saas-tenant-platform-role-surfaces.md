# ADR 0004: Remove SaaS Tenant and Platform-Role Surfaces

## Status
Accepted

## Context
LeeAttend now runs as a standalone single-organization appliance. The repository still contains SaaS control-plane surfaces for tenant lifecycle and platform-level role assignment. These surfaces conflict with the standalone contract, expose obsolete UI and API concepts, and retain a second authorization model beside organization RBAC.

Affected legacy surfaces include `/system/tenants`, `/system/roles`, `system_roles`, `user_system_roles`, `platform_admin`, `support`, and `billing_admin`. The supported standalone role model keeps organization/workspace RBAC and its global `admin` role. SMTP, system health, and infrastructure monitor routes remain supported but require standalone organization-admin authorization.

## Decision
1. Remove tenant lifecycle and platform-role management routes, pages, API clients, generated contracts, tests, fixtures, translations, bootstrap code, repository methods, cache keys, and documentation.
2. Authorize retained system routes through the standalone organization-admin RBAC primitive, failing closed on missing identity, missing membership/role, or authorization dependency failure.
3. Preserve the existing internal `workspace_id` compatibility schema until the broader single-database unification work removes it. This ADR does not flatten all workspace tables or business routes.
4. Migrate existing installations by preserving organization admin access through `workspace_members` and the global `admin` workspace role before deleting obsolete system-role tables.
5. Treat removed API paths as an approved breaking change. Update source contracts, generated clients, consumer tests, and operator documentation in the same cutover.

## Alternatives considered
- Keep the legacy endpoints hidden behind the UI: rejected; dead public contracts and second authorization model remain.
- Keep `system_roles` as a compatibility alias: rejected; standalone has no platform tenant boundary and aliases preserve obsolete privilege paths.
- Replace all workspace schema and routes in this change: rejected; scope is too broad and would combine unrelated data-contract migration risk.

## Consequences
### Positive
- One supported standalone organization model.
- One organization RBAC source for retained system administration.
- No tenant provisioning/switching or SaaS support/billing role assignment surface.
- Smaller frontend, backend, schema, and contract surface.

### Negative
- Clients using removed paths require migration before upgrade.
- Existing installations need a destructive schema migration after backup and admin-access reconciliation.
- Historical migration evidence mentioning Rust-era endpoints must be marked historical or updated without rewriting unrelated evidence.

## Security impact
- Removed role-management endpoints cannot be used for privilege assignment.
- Retained system endpoints require authenticated organization admin membership and deny on authorization lookup errors.
- Migration MUST verify at least one usable organization admin before dropping legacy role tables.
- No raw credentials, tokens, or PII may appear in migration logs or test artifacts.

## Operational and migration approach
1. Inventory all consumers and freeze the standalone contract.
2. Switch retained system-route guards to organization-admin RBAC and add allow/deny/failure tests.
3. Remove frontend/backend contracts and runtime consumers.
4. Stop seeding or assigning legacy system roles.
5. Back up existing databases, reconcile admin assignments, run forward migration, and verify login plus retained admin routes.
6. Roll back by restoring the pre-migration backup if exact legacy role data must be recovered; roll-forward is safer once new-version data is written because the removed API has no supported compatibility path.

## Verification
- Repository-wide search finds no supported runtime or generated-contract references to removed tenant/platform-role surfaces.
- Contract tests prove removed paths are absent and retained system paths remain present.
- Authorization tests cover admin allow, non-admin/unauthenticated deny, and dependency failure deny.
- Clean and representative migration runs preserve admin access and satisfy schema/reconciliation checks.
- Full LeeDevKit backend/frontend/integration/E2E gates pass.

## Owner
LeeAttend platform maintainers.
