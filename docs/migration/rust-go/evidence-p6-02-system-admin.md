# P6-02 System Admin (SMTP, Roles, DB Clusters, Tenants) Evidence

Status: implemented; 16-route slice registered, unit tests added. Build/test verification deferred to parent agent.

## Contract

All 16 routes require JWT authentication plus the `platform_admin` system role. Missing authentication returns `401`; a valid user without `platform_admin` (including `support` and `billing_admin`) receives `403`.

## Implemented Endpoints

### SMTP (3 routes)

1. `GET /api/v1/system/smtp/config`
   - Reads `system_settings` key `smtp`.
   - Missing key returns `200` with `{}`.
   - Credential-bearing keys (`password`, `secret`, `token`, `auth_value`, including nested `http_api`) are masked with `***` before the response; raw stored JSON is unchanged.
2. `PUT /api/v1/system/smtp/config`
   - Requires a valid JSON document (any JSON value accepted by Rust).
   - Persists the raw document under `system_settings.smtp`.
   - Deletes Redis key `smtp:config` on a best-effort basis.
   - Returns `200 {"saved":true}`.
3. `POST /api/v1/system/smtp/test`
   - Rejects empty `to_email` or values without `@` with `400` validation.
   - Delegates to `service.EmailService.SendTestEmail` (SMTP or configured `http_api`).
   - On transport failure returns `200` with `{"success":false,"message":...}` — never `500`.
   - On success writes `last_verified_at` (RFC3339) into the stored `smtp` document and returns `200 {"success":true,...}`.
   - Does not log recipient address or credentials.

### System Roles (4 routes)

1. `GET /api/v1/system/roles` — returns `200` with `[]SystemRoleInfo` ordered by role name; assignees ordered by `assigned_at`; empty assignee lists serialize as `[]`.
2. `GET /api/v1/system/users/{user_id}/roles` — returns `200` with role names ordered by name; no assignments returns `[]`.
3. `POST /api/v1/system/users/{user_id}/roles`
   - Body `{"role_name": "<platform_admin|support|billing_admin>"}`.
   - Unknown/empty/mixed-case role names rejected with `400` (`error.unknown_system_role`); prevents privilege escalation by typo.
   - Idempotent insert via `ON CONFLICT (user_id, role_id) DO NOTHING`.
   - Missing role row returns `404` (`error.system_role_not_found`).
   - Invalidates Redis `cache:system:roles:{user_id}` and returns `204`.
4. `DELETE /api/v1/system/users/{user_id}/roles/{role_name}`
   - Unknown role name returns `400` (`error.unknown_system_role`).
   - Revoking `platform_admin` when the global count is `<= 1` returns `400` (`error.cannot_revoke_last_admin`).
   - Zero deleted rows returns `404` (`error.system_role_not_found`).
   - Invalidates Redis `cache:system:roles:{user_id}` and returns `204`.

### DB Clusters (4 routes)

1. `GET /api/v1/system/db-clusters` — returns `200` with clusters ordered `created_at DESC`.
2. `POST /api/v1/system/db-clusters` — generates UUID, `active_tenants = 0`, `created_at`/`updated_at` now, returns `200` with the created cluster. Admin database URL passwords are always masked as `***` in responses.
3. `DELETE /api/v1/system/db-clusters/{id}` — no affected-row check; returns `204` even when the ID is unknown, matching Rust.
4. `PATCH /api/v1/system/db-clusters/{id}/status` — updates `is_active` and `updated_at`, returns `200` with the cluster; missing ID returns `404`.

### Tenants (5 routes)

Tenants are workspaces. All queries run against the system pool; workspace row access is not tenant-scoped.

1. `GET /api/v1/system/tenants` — returns `200` with `id`, `name`, `is_byod`, `timezone`, `created_at`. Never includes the database URL.
2. `POST /api/v1/system/tenants` — optional `database_url` is AES-256-GCM encrypted using `ENCRYPTION_KEYS_JSON`, then inserted as `custom_db_url_encrypted`; falls back to plaintext only when no crypto service is configured (matching Rust). Inserts the owner membership. Returns `201` with the tenant response.
3. `DELETE /api/v1/system/tenants/{tenant_id}` — deletes the workspace row only; never drops the physical database. Returns `204`.
4. `GET /api/v1/system/tenants/{tenant_id}/stats` — returns `200` with connection info from `PGBOUNCER_HOST`/`DATABASE_HOST` and `PGBOUNCER_PORT`/`PGPORT`, defaulting to `localhost:5432`.
   - `masked_db_url` decrypts the stored URL and masks the password as `***`.
   - Malformed URLs yield `*** REDACTED ***`.
   - Decryption failure omits `masked_db_url` entirely.
   - Missing workspace returns `404` (`error.workspace_not_found`).
5. `PUT /api/v1/system/tenants/{tenant_id}/database` — encrypts the URL, updates `custom_db_url_encrypted`, returns `200` with an empty body. Missing workspace returns `404`.

## Files Added & Modified

- `apiserver/internal/api/system_admin.go`
- `apiserver/internal/api/system_admin_test.go`
- `apiserver/internal/repository/system_admin.go`
- `apiserver/internal/service/email.go`
- `apiserver/internal/infra/crypto/crypto.go`
- `apiserver/internal/apperror/errors.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`
- `docs/migration/rust-go/evidence-p6-02-system-admin.md`

## Security Notes

- Responses never include raw `admin_database_url` or tenant BYOD URLs; only password-masked forms.
- SMTP GET response masks stored credentials.
- No email addresses, database URLs, or credentials are logged by the new handlers.
- All SQL is parameterized; no string interpolation of user-controlled values.
- Unknown system role names are rejected before any DB lookup (assign) or delete (revoke).
- `platform_admin` guard is enforced per handler; Support/Billing roles cannot reach any of the 16 routes.

## Verification Status

- Focused unit tests added in `apiserver/internal/api/system_admin_test.go`:
  - 401 unauthenticated and 403 for `support` on SMTP config.
  - SMTP empty-config `{}`, PUT persistence, invalid recipient rejection, transport failure `200/false`, success `200/true` plus `last_verified_at`.
  - Role list/get, unknown role `400`, assignment `204`, last-admin revoke guard `400`, valid revoke `204`.
  - Cluster list/create/password mask, PATCH status, delete missing ID `204`.
  - Tenant create `201` with BYOD, stats password masking and connection fallback, database update `200` empty body, delete `204`.
  - `MaskDatabaseURL` masking, malformed URL redaction, empty input redaction.
- Endpoint metrics after route registration:
  - Contract routes: **403**
  - Go route registrations: **183** (+16)
  - Contract routes covered: **160** (+16)
  - Contract routes missing: **243** (−16)
- `./scripts/_go-gates.sh`, `go test`, formatters, and live database checks are intentionally deferred to the parent agent.
