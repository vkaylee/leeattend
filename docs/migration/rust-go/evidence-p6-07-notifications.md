# P6-07 Notifications Domain Evidence

Status: in-progress; notification list, cursor pagination, unread count, and read status endpoints implemented and verified through container build gates.

## Notification Delivery Slice

### Implemented Endpoints
1. `GET /v1/notifications` and `GET /api/v1/notifications`
   - Requires authenticated user context.
   - Enforces `limit` between 1 and 100 (defaults to 20).
   - Supports timestamp cursor pagination (`created_at < cursor`).
   - Returns unread count and `next_cursor` when results exceed limit.
2. `GET /v1/notifications/unread-count` and `GET /api/v1/notifications/unread-count`
   - Returns count of unread notifications for the caller.
3. `PATCH /v1/notifications/{id}/read` and `PATCH /api/v1/notifications/{id}/read`
   - Marks specific notification as read, returning 404 if not found or owned by user.
4. `POST /v1/notifications/read-all` and `POST /api/v1/notifications/read-all`
   - Atomically updates all unread notifications for the caller.

### Source Files
- `internal/api/notifications.go`: Typed handlers and DTOs.
- `internal/repository/notifications.go`: System database queries with cursor pagination and unread counts.
- `internal/api/notifications_test.go`: Auth requirement, list pagination, and mark-read coverage.
- `main.go`: Route registration for both `/v1/notifications` and `/api/v1/notifications` prefixes.

### Verification Status
- `scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
