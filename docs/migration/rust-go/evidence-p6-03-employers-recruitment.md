# P6-03 Employers & Recruitment Sources Evidence

Status: implemented; 10 route slice registered, unit tests added. Full integration verification deferred to parent agent.

## Implemented Endpoints

### Employers (5 routes)
1. `GET /api/v1/workspaces/{ws_id}/employers`
   - Requires authenticated workspace membership.
   - Paginates via `page` (default 1) and `limit` (default 50, max 3000).
   - Filters `deleted_at IS NULL` and orders `name ASC`.
   - Returns 200 OK with `EmployerListResponse`.
2. `POST /api/v1/workspaces/{ws_id}/employers`
   - Requires authenticated workspace membership.
   - Validates non-empty name, email format, 3-character ASCII country/currency codes, and date format.
   - Validates `parent_id` exists in same workspace.
   - Emits `employer.created` audit event when audit emitter is configured.
   - Returns 201 Created with `Employer`.
3. `GET /api/v1/workspaces/{ws_id}/employers/{id}`
   - Requires authenticated workspace membership.
   - Returns active employer by ID and workspace. Returns 404 Not Found if missing or soft-deleted.
4. `PUT /api/v1/workspaces/{ws_id}/employers/{id}`
   - Requires authenticated workspace membership.
   - Preserves omitted fields; supports explicit `null` clearing for `parent_id` and `relationship_type`.
   - Validates email, country code, currency code, and date when supplied.
   - Validates parent cycle: rejects self-parent and circular parent chains with validation error (400).
   - Emits `employer.updated` audit event when audit emitter is configured.
   - Returns 200 OK with updated `Employer`.
5. `DELETE /api/v1/workspaces/{ws_id}/employers/{id}`
   - Requires authenticated workspace membership.
   - Verifies active employments count; blocks deletion with validation error (400) if active employments reference employer.
   - Soft-deletes row (`deleted_at = CURRENT_TIMESTAMP`).
   - Emits `employer.deleted` audit event when audit emitter is configured.
   - Returns 204 No Content.

### Recruitment Sources (4 routes)
1. `GET /api/v1/workspaces/{ws_id}/recruitment-sources`
   - Requires authenticated workspace membership.
   - Paginates via `page` (default 1) and `limit` (default 50, max 3000).
   - Orders `created_at DESC`.
   - Returns 200 OK with `RecruitmentSourceListResponse`.
2. `POST /api/v1/workspaces/{ws_id}/recruitment-sources`
   - Requires `recruitment:manage` permission.
   - Validates `source_type` in `('direct', 'referral', 'agency')` and non-empty `name`.
   - Defaults `referral_bonus_paid=false` and `agency_fee_paid=false`.
   - Monetary values use `shopspring/decimal`.
   - Returns 201 Created with `RecruitmentSource`. No audit event (matching Rust).
3. `PUT /api/v1/workspaces/{ws_id}/recruitment-sources/{id}`
   - Requires `recruitment:manage` permission.
   - Updates supplied fields and updates `updated_at`.
   - Returns 404 Not Found if missing.
   - Returns 200 OK with `RecruitmentSource`.
4. `DELETE /api/v1/workspaces/{ws_id}/recruitment-sources/{id}`
   - Requires `recruitment:manage` permission.
   - Deletes matching workspace-scoped row. Returns 404 Not Found if not found.
   - Returns 204 No Content.

### Dashboard Breakdown (1 route)
1. `GET /api/v1/dashboard/recruitment-sources`
   - Uses `X-Workspace-Id` context and user authentication.
   - Requires `dashboard:read` permission.
   - Aggregates employee counts grouped by `source_type` and `source_name` for active employments.
   - Returns 200 OK with `[]RecruitmentSourceBreakdown`.

## Files Added & Modified

- `apiserver/internal/api/employers.go`
- `apiserver/internal/api/employers_test.go`
- `apiserver/internal/repository/employers.go`
- `apiserver/internal/api/recruitment_sources.go`
- `apiserver/internal/api/recruitment_sources_test.go`
- `apiserver/internal/repository/recruitment_sources.go`
- `apiserver/main.go`
- `docs/migration/rust-go/endpoint-coverage-gap.md`
- `docs/migration/rust-go/evidence-p6-03-employers-recruitment.md`

## Verification Status

- Unit tests added covering:
  - Employer list, get (200 / 404), create validation, update omitted vs explicit null semantics, circular parent rejection, delete active employment guard, non-member rejection (403).
  - Recruitment list, create validation and permission guard, update (200 / 404), delete (204 / 404), dashboard permission guard and breakdown response.
- Endpoint metrics after route registration:
  - Contract routes: **403**
  - Go route registrations: **167** (+10)
  - Contract routes covered: **144** (+10)
  - Contract routes missing: **259** (-10)

## Known Limitations

- Production audit sink is optional in `EmployerHandler`; background audit queue is not initialized in `main.go` pending global audit service wiring.
- Live database integration checks deferred to main parent verification gate.
