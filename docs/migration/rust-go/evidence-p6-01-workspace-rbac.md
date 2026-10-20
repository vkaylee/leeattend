# P6-01 Workspace RBAC & Storage Settings Evidence

Status: implemented; members, roles, permission catalog, and storage-settings routes registered with unit tests. Build/test/gate verification deferred to the parent agent.

## Contract

Authentication is required on every route. Context-only routes are allowed for any authenticated caller, matching Rust `MemberContext` behavior. Mutation routes enforce workspace permissions through `WorkspaceAuthorizer.HasPermission`:

| Scope | Permission |
|---|---|
| Add member | `members:invite` |
| Role change, extra role/permission mutations, role CRUD/permission updates, catalogs | `members:roles:manage` |
| Remove member | `members:remove` |
| Read member extra permissions | `members:read` |
| Members list, member roles, roles list, storage settings | none beyond authentication |

All SQL runs on the system database (`workspace_members`, `workspace_roles`, `workspace_role_permissions`, `member_extra_roles`, `workspace_member_extra_permissions`, `storage_configs`). No tenant database access.

## Implemented Endpoints

### Members (10)

1. `GET /api/v1/workspaces/{ws_id}/members` — paginated (`page`, `limit`, default 50, max 3000) join of `workspace_members` and `users`; attaches active (`expires_at > NOW()`) extra role IDs per member.
2. `POST /api/v1/workspaces/{ws_id}/members` — `members:invite`; resolves user by case-insensitive email (`404 error.user_not_found` when absent); inserts membership with `ON CONFLICT (workspace_id, user_id) DO NOTHING`; returns `201`.
3. `PATCH /api/v1/workspaces/{ws_id}/members/{user_id}` — `members:roles:manage`; updates primary `role_id`; missing membership returns `404`; returns `200`.
4. `DELETE /api/v1/workspaces/{ws_id}/members/{user_id}` — `members:remove`; self-removal rejected with `400 error.cannot_remove_self`; missing membership returns `404`; returns `204`.
5. `GET /api/v1/workspaces/{ws_id}/members/{user_id}/roles` — returns `{ "primary": WorkspaceRole|null, "extra": MemberExtraRole[] }`; extra roles filtered to future expiry.
6. `POST /api/v1/workspaces/{ws_id}/members/{user_id}/roles` — `members:roles:manage`; requires future `expires_at` (`400 error.expiration_future`); inserts `member_extra_roles`; returns `201` with the record.
7. `DELETE /api/v1/workspaces/{ws_id}/members/{user_id}/roles/{role_id}` — `members:roles:manage`; deletes by extra-role ID scoped to workspace; zero rows returns `404`; returns `204`.
8. `GET /api/v1/workspaces/{ws_id}/members/{user_id}/permissions` — `members:read`; returns non-expired extra permissions ordered `assigned_at DESC`.
9. `POST /api/v1/workspaces/{ws_id}/members/{user_id}/permissions` — `members:roles:manage`; rejects non-canonical permission (`400 error.unknown_permission`) and past expiry (`400 error.expiration_future`); records `assigned_by`; returns `201`.
10. `DELETE /api/v1/workspaces/{ws_id}/members/{user_id}/permissions/{perm_id}` — `members:roles:manage`; deletes by permission ID scoped to workspace; zero rows returns `404`; returns `204`.

### Roles (6 + 2 catalogs)

11. `GET /api/v1/workspaces/{ws_id}/roles` — paginated; includes workspace roles plus system roles (`workspace_id IS NULL` semantics through `is_system = true`), ordered `is_system DESC, created_at ASC`.
12. `POST /api/v1/workspaces/{ws_id}/roles` — `members:roles:manage`; `scope_type` defaults to `scoped` and only accepts `global`/`scoped`; non-system role created; returns `201`.
13. `PUT /api/v1/workspaces/{ws_id}/roles/{role_id}` — `members:roles:manage`; updates only non-system roles in the workspace; system/unknown role returns `404 error.role_not_found_or_system`; returns `200`.
14. `DELETE /api/v1/workspaces/{ws_id}/roles/{role_id}` — `members:roles:manage`; deletes only non-system roles; zero rows returns `404 error.role_not_found_or_system`; returns `204`.
15. `GET /api/v1/workspaces/{ws_id}/roles/{role_id}/permissions` — `members:roles:manage`; returns `[]string`, allowing workspace roles and system roles.
16. `PUT /api/v1/workspaces/{ws_id}/roles/{role_id}/permissions` — `members:roles:manage`; rejects system/unknown roles (`404`); replaces permissions, empty list clears; duplicates deduplicated by primary key; returns `200` with the permission list.
17. `GET /api/v1/workspaces/{ws_id}/permissions/catalog` — `members:roles:manage`; returns 103 ordered catalog items `{id, group, label_key}` with the Rust group mapping and `permissions.items.<id>.name` label keys.
18. `GET /api/v1/workspaces/{ws_id}/permissions/available` — `members:roles:manage`; returns the 103 canonical permission IDs in order.

### Storage settings (4)

19. `GET /api/v1/workspaces/{ws_id}/settings/storage` — returns `null` when unconfigured; otherwise `StorageConfigResponse` with `access_key_masked = "***"`, never encrypted or plaintext credentials.
20. `PUT /api/v1/workspaces/{ws_id}/settings/storage` — requires `ENCRYPTION_KEYS_JSON`; AES-256-GCM encrypts access and secret keys independently; `encryption_key_version` is the `version:nonce:ciphertext` prefix; upserts on `workspace_id`; response masks credentials.
21. `DELETE /api/v1/workspaces/{ws_id}/settings/storage` — deletes the workspace row and returns `200 {}`.
22. `POST /api/v1/workspaces/{ws_id}/settings/storage/test` — builds an S3 client from request credentials and performs a bucket health request; returns `200 true` on success and `500` on connection failure. When both request and server S3 endpoints are empty (S3 disabled), succeeds without network access.

## Files Added & Modified

- `apiserver/internal/api/workspace_rbac.go`
- `apiserver/internal/api/storage_tester.go`
- `apiserver/internal/api/workspace_rbac_test.go`
- `apiserver/internal/repository/workspace_rbac.go`
- `apiserver/internal/apperror/errors.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`
- `docs/migration/rust-go/evidence-p6-01-workspace-rbac.md`

## Security Notes

- Storage credentials are encrypted before persistence; API responses always mask access key and never expose secret/encrypted columns.
- Storage test uses request plaintext only in memory and never persists it.
- System roles cannot be updated, deleted, or have permissions replaced; protection is enforced in SQL predicates and handler checks.
- Member removal cannot target the acting user.
- Extra permissions accept only canonical permission strings.
- All SQL is parameterized; extra-role/permission deletes include workspace scoping.

## Tests

`apiserver/internal/api/workspace_rbac_test.go` covers:

- `members:invite` denial and successful invitation (`403` → `201`).
- Unknown permission and past expiration rejection (`400`).
- Self-removal rejection (`400` + `VALIDATION_ERROR`) and removal of another member (`204`).
- System-role update/delete/permission replacement protection (`404`).
- Storage credential encryption round-trip, plaintext absence from the repository record, response masking, connection-test success/failure, and deletion.

## Deferred Verification

`go build`, `go test`, linters, formatters, and live database checks are intentionally deferred to the parent agent.
