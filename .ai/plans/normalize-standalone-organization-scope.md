# Normalize Standalone Organization Scope

Status: Approved

## Goal
Replace SaaS-facing `workspace` / `workspaces` / `ws_id` terminology and selectable scope with one canonical standalone organization context, while preserving `workspace_id` database storage until a separately approved schema migration.

## Constraints and interfaces
- Canonical public/runtime term: `organization`; one immutable organization resolved at startup/bootstrap.
- Keep DB `workspace_id` columns, `workspaces` table, workspace RBAC tables, and tenant predicates in this cutover. They remain compatibility storage and security scope, not user-selectable tenancy.
- Remove public workspace selection, creation, listing, switching, and path IDs. Canonical API routes omit `{ws_id}` and resolve organization from trusted server context.
- Authorization stays mandatory. Singleton organization does not remove authentication, membership, permission checks, audit logging, or forged-scope rejection.
- No compatibility aliases for removed SaaS routes after cutover. Existing clients receive the approved breaking-change response; regenerate contracts and migrate the dashboard in the same change.
- Rule IDs: `database-rules §1, §4`; `access-control §1–2`; `api-governance §1–2, §6`; `migration-and-rollback §1–6`; `development-workflow §2–7`.

## Tasks
- [ ] 1. Freeze canonical vocabulary and route map in ADR 0005. Record `organization` / `organization_id` public naming, internal `workspace_id` exception, removed routes, migration owner, rollback point, and consumer cutover. Verify architecture review accepts the breaking API contract.
- [ ] 2. Add one backend organization-context resolver. Resolve exactly one organization during bootstrap/startup; inject it into request context; reject missing, non-singleton, path/header/local-storage-selected scope. Rename context helpers and handler-local variables to organization terminology while mapping SQL parameters to `workspace_id` only at repository boundaries. Verify forged IDs cannot change authorization or query scope.
- [ ] 3. Flatten backend public routes in `apiserver/main.go` and `apiserver/internal/api/**`. Remove `/workspaces/{ws_id}` registrations and `pathUUID(..., "ws_id")` reads; retain only organization-scoped paths such as `/members`, `/employees`, `/attendance`, `/settings`. Remove workspace lifecycle/provisioning endpoints and dead SaaS handlers. Verify every retained endpoint has auth, permission, singleton context, and audit behavior.
- [ ] 4. Normalize repository/service contracts and response types. Rename exported runtime fields/interfaces from `Workspace*` / `WorkspaceID` to `Organization*` / `OrganizationID` where they are application contracts; keep SQL column names and explicit `workspace_id` predicates at persistence boundaries. Update auth/RBAC, pool manager, agent/device, files, reports, payroll, leave, attendance, and audit paths without introducing a second scope abstraction.
- [ ] 5. Regenerate and migrate API contracts. Update the canonical OpenAPI source/generator, generated Go/TypeScript clients, route parameter names, error keys, and contract tests. Verify schema diff contains no workspace path parameters and removed SaaS/tenant lifecycle operations are absent.
- [ ] 6. Normalize frontend context and URLs. Replace `useWorkspace`, `workspaceId`, `wsId`, workspace guard/storage selection, workspace route directories, and organization-wrapper legacy signatures with singleton organization context. Keep user-visible settings language as organization/company language; remove workspace switch/create UI and stale workspace translations. Verify refresh, login, logout, and direct navigation never depend on browser-persisted scope.
- [ ] 7. Update persistence/configuration/deployment only where scope is selectable. Remove dynamic tenant pool/cache/eviction and SaaS tenant configuration; bind repositories to the one configured database. Do not rename/drop `workspace_id` or `workspaces` in this phase. Verify fresh bootstrap creates exactly one organization and existing data remains queryable.
- [ ] 8. Update tests, fixtures, docs, data inventory, and changelog. Add regression coverage for singleton enforcement, forged scope denial, absent route parameter, authorization failure, migration row-count/admin preservation, and generated-contract compatibility. Remove obsolete SaaS wording and update standalone runbooks.

## Done when
- [ ] No supported runtime, generated contract, frontend route, or user-facing flow uses selectable `workspace`, `workspaces`, or `ws_id` scope.
- [ ] Database compatibility names remain only in documented persistence boundaries and migration history.
- [ ] All core HR, attendance, leave, payroll, files, agent, RBAC, and audit flows use one trusted organization context.
- [ ] Forward migration is reversible or has tested backup/restore rollback; no destructive schema rename occurs without separate approval.
- [ ] Backend, frontend, integration, contract, and browser smoke gates pass.

## Review focus
- Route and generated-client cutover is intentionally breaking; all consumers must migrate together.
- Verify no handler trusts a user-supplied organization/workspace ID after route flattening.
- Confirm SQL predicates still bind the singleton ID and cannot broaden scope.
- Confirm existing `workspace_roles` / `workspace_members` admin access survives bootstrap and upgrades.
