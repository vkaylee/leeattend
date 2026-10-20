# P6-03 Dashboard Preferences Evidence

Status: implemented; database integration and live differential comparison remain required before final parity closure.

## Contract

Implemented `GET` and `PUT /api/v1/dashboard/preferences` using `X-Workspace-Id` and authenticated user context. Both require exact `dashboard:preferences` permission.

Response fields: `id`, `user_id`, `workspace_id`, `active_view`, `visible_widgets`, `widget_order`, and `collapsed_sections`.

Normalization follows Rust:

- `active_view`: `overview`, `attendance`, `trends`, or `custom`.
- Known widget keys merge with defaults; unknown widget keys drop.
- Widget order accepts strings, drops unknown IDs, removes duplicates, then appends missing canonical IDs.
- Collapsed sections merge boolean values over `summary=false`, `charts=false`, `actions=true`; unknown keys remain.
- Invalid JSON shape/value returns `400` with `VALIDATION_ERROR`.
- Missing GET row returns UUID nil plus defaults without insert.

## Implementation

- `apiserver/internal/api/dashboard_preferences.go`
- `apiserver/internal/api/dashboard_preferences_test.go`
- `apiserver/internal/repository/dashboard_preferences.go`
- `apiserver/main.go`

Repository queries constrain both `user_id` and `workspace_id`. PUT inserts defaults on first use with `ON CONFLICT (user_id, workspace_id) DO NOTHING`, then updates only supplied fields. No Redis, audit, queue, or schema change.

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: previously passed before final normalization adjustment; rerun required after commit.
- Endpoint metrics after route registration: contract routes **403**, Go registrations **143**, covered **120**, missing **283**.
