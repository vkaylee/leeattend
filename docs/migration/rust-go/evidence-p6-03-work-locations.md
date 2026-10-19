# P6-03 Work Locations Evidence

Status: implemented route slice; employee-location and full P6-03 differential parity remain open.

## Implemented

- `GET /api/v1/workspaces/{ws_id}/locations`
- `POST /api/v1/workspaces/{ws_id}/locations`
- `GET /api/v1/workspaces/{ws_id}/locations/{id}`
- `PUT /api/v1/workspaces/{ws_id}/locations/{id}`
- `DELETE /api/v1/workspaces/{ws_id}/locations/{id}`

All routes require authenticated workspace membership. Every repository query constrains `workspace_id` and active status.

Behavior:

- List uses page/limit pagination and headquarters-first/name ordering.
- Create defaults timezone to `UTC`, status to `active`, and rejects standard-endpoint headquarters creation.
- Update applies supplied metadata fields and rejects headquarters flag transitions.
- Delete soft-deletes by setting `status='deleted'`, rejects headquarters deletion, and rejects locations referenced by active employments.
- Missing rows map to `404`; invalid input and protected-state conflicts map to validation errors.

## Files

- `apiserver/internal/api/locations.go`
- `apiserver/internal/api/locations_test.go`
- `apiserver/internal/repository/locations.go`
- `apiserver/main.go`

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics: contract routes **403**, Go registrations **157**, covered **134**, missing **269**.

Audit event emission is not yet wired for this slice; Rust records `location.created`, `location.updated`, and `location.deleted`. This remains an explicit parity gap.
