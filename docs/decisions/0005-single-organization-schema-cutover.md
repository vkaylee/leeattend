# ADR 0005: Single-Organization Schema and Runtime Cutover

## Status
Accepted

## Date
2026-11-08

## Context
LeeAttend already runs as a standalone appliance with one PostgreSQL database and flat API routes, but runtime callers, RBAC storage, documentation, tooling, and the compatibility schema still expose selectable-workspace assumptions. Keeping a second tenant context would preserve obsolete client scope and make authorization, bootstrap, and persistence disagree. The approved single-database checkpoint replaces that compatibility schema in source; no destructive operation against live data is authorized by this ADR.

## Decision
1. Model one implicit organization. The server loads its settings from the singleton `organization_settings` row; the row ID is an internal persistence key only. Clients MUST NOT select organization scope through routes, headers, JWT claims, request DTOs, or browser storage.
2. Keep flat `/api/v1/...` application routes. Organization settings continue to use `GET /api/v1` and `PATCH /api/v1`; do not invent a replacement endpoint. Reject public `/workspaces/{id}` paths with 404. Preserve `/api/v1/ws/reports/{job_id}` because `ws` denotes WebSocket, not workspace.
3. Cut persistence over to `organization_settings`, `user_roles`, `roles`, `role_permissions`, `user_extra_roles`, `user_extra_permissions`, `files`, `compliance_rules`, and `config_entries`; remove tenant ID columns and workspace-only objects from the final schema. Keep existing organization data, settings, user/entity IDs, and opaque stored object keys. New object keys contain no workspace prefix.
4. Preserve the existing authorization guarantees while removing the workspace selector: authenticated user identity, explicit permission checks, deny-by-default/fail-closed permission decisions, role scope and department boundaries, ESS ownership, extra-grant expiry, revocation, audit behavior, and last-administrator protection. JWT identity remains the user subject with expiry/issued-at claims; it carries no organization selector. External API keys remain permission-scoped and bind to the one implied organization, not a client-selected ID.
5. Replace active current rules that require workspace predicates or selectable workspace scope with single-organization rules. Tracked semantic references are rewritten or removed in this cleanup; Git history remains unchanged. Technical names remain only when explicitly allowlisted (for example, container mounts, CloudBeaver state, or WebSocket `ws`).
6. Deploy schema cutover only in a single maintenance window: stop and drain every writer, backfill and reconcile once, switch all readers/writers, validate permissions/data/settings/business totals, then reopen. Reject migration when organization count exceeds one or role names conflict; do not merge conflicting grants silently.

## Alternatives considered
- Keep `workspace_id` storage/predicates as long-term compatibility: rejected; current sources continue to depend on a client-like tenant abstraction and conflict with the standalone contract.
- Replace workspace IDs with a client-selectable `organization_id`: rejected; a singleton does not justify another untrusted scope selector.
- Leave old routes as aliases: rejected; callers must move to flat paths and legacy paths remain rejected.
- Rewrite Git history: rejected; tracked files may be cleaned, but immutable Git history preserves audit traceability. Rename unavoidable technical names such as `/workspace` container mounts, CloudBeaver metadata, or WebSocket `ws`: rejected unless they are semantic tenant contracts.
- Apply destructive schema changes directly to live data: rejected without a separately approved live operational change.

## Consequences
- API, database, and frontend consumers cut over together; no adapters, ID shims, or dual writes remain.
- SQL no longer uses tenant-ID predicates, but authorization, entity ownership, department scope, soft deletion, transactions, constraints, and error mapping remain mandatory.
- Existing installations need a protected backup and tested restore path. After destructive contract, restoring the prior database and matching object-store state is safer than reverting only application code.
- A clean database may begin with no organization settings row; bootstrap creates the singleton. More than one existing row is a hard migration error.

## Verification and abort conditions
- Verify schema and permissions on disposable clean and upgraded databases before release; verify settings, users, role grants/revocations, department/ESS boundaries, expiry, and business totals.
- Abort on multiple organizations, ambiguous role-name collisions, lost/changed grants, failed reconciliation, or failed backup/restore rehearsal.
- Do not treat an existing development process as evidence for current source behavior. Run API and migration smoke only against isolated rebuilt services and disposable databases.

## Owner
LeeAttend maintainers; implementation owners are assigned by the approved single-database unification checkpoint.
