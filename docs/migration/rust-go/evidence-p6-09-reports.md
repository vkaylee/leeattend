# P6-09 Reports and Dashboard Domain Evidence

Status: in-progress; report export job creation (202 Accepted) and status polling endpoints implemented and verified through container build gates.

## Reports and Analytics Slice

### Implemented Endpoints
1. `POST /api/v1/workspaces/{ws_id}/reports/export`
   - Requires `reports:export` permission.
   - Enforces valid date range (max 366 days), report types (`summary`, `detail`), and output formats (`xlsx`, `csv`).
   - Persists async export job into `outbox_jobs` table with `AttendanceReport` type and payload.
   - Enqueues to Redis queue `worker:jobs:high` and stores metadata for polling.
   - Returns 202 Accepted with `job_id`, `status: pending`, `total_batches`, and `status_url`.
2. `GET /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}`
   - Requires `reports:export` permission.
   - Resolves job status and batch progress from Redis metadata or fallback storage.
   - Returns 200 OK with progress percentages and RFC3339 timestamps.
   - Returns 404 Not Found on unknown job ID.

### Source Files
- `internal/api/reports.go`: Typed handlers, DTOs, date range validation, and JSON responses.
- `internal/api/reports_test.go`: Unit tests for job creation, progress calculation, and 404 behavior.
- `internal/repository/reports.go`: Postgres `outbox_jobs` persistence and Redis queue client.
- `main.go`: Registered routes and wired `ReportRepository` and `RedisReportQueue`.

### Verification Status
- `scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.
