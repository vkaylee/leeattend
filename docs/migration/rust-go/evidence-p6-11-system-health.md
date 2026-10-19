# P6-11 System Health Evidence

Status: implemented; middleware-level live differential checks remain pending.

## Contract

`GET /api/v1/system/health` requires JWT authentication and permits only `platform_admin` or `support` system roles. Workspace context is not required.

Success body:

```json
{"status":"ok","viewer_user_id":"<uuid>","viewer_roles":["platform_admin"]}
```

Unknown role names are ignored in the response. `billing_admin` is recognized but does not authorize this endpoint. Missing user context returns `401`; repository errors return `500`; unauthorized system roles return `403`.

## Implementation

- `apiserver/internal/api/system_health.go`
- `apiserver/internal/api/system_health_test.go`
- `apiserver/main.go`

The handler reuses `AuthRepository.GetSystemRolesForUser`; no DB mutation, Redis mutation, audit event, or workspace lookup occurs.

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics: contract routes **403**, Go registrations **147**, covered **124**, missing **279**.
