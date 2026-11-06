# P6-11 Standalone System Health Evidence

Status: retained standalone route; platform-role authorization contract superseded.

## Contract

`GET /api/v1/system/health` requires JWT authentication, singleton workspace context, and the standalone `infra:monitor` permission. Missing user or workspace context returns `401`/`403`; authorization lookup failure denies access.

Success body:

```json
{"status":"ok","viewer_user_id":"<uuid>"}
```

## Implementation

- `apiserver/internal/api/system_health.go`
- `apiserver/internal/api/system_health_test.go`
- `apiserver/main.go`

The handler uses the standalone workspace authorizer; it does not read legacy system-role tables.
