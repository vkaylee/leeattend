# P6-02 Standalone System Admin Evidence

Status: superseded for removed platform-role, tenant-lifecycle, and database-cluster surfaces. Retained SMTP routes remain implemented and covered by `apiserver/internal/api/system_admin_test.go`.

## Contract

Retained SMTP routes require JWT authentication plus the standalone `security:settings:manage` workspace permission. Missing authentication returns `401`; callers without the permission receive `403`.

## Retained SMTP endpoints

1. `GET /api/v1/system/smtp/config` reads `system_settings.smtp`, returns `{}` when absent, and masks credential-bearing keys.
2. `PUT /api/v1/system/smtp/config` validates JSON, persists the setting, invalidates the Redis cache key, and returns `200 {"saved":true}`.
3. `POST /api/v1/system/smtp/test` validates the recipient, delegates to the email service, records verification time on success, and returns transport failures as a structured `success:false` response.

Platform-role assignment, tenant lifecycle, and database-cluster management routes are removed from the standalone contract. The forward schema migration is `apiserver/migrations/20261106000000_remove_system_roles/`.

## Security notes

- Responses never include raw SMTP credentials.
- No email addresses or credentials are logged by retained handlers.
- SQL is parameterized.

## Verification

- Focused handler coverage remains in `apiserver/internal/api/system_admin_test.go`.
- Migration preconditions and restore-only rollback are documented in `docs/migrations/remove-system-roles-migration.md`.
