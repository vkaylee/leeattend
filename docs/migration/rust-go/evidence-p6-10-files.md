# P6-10 Files, Documents, and E-Signature Evidence

Status: in-progress; presigned upload URLs, file metadata, download URLs, soft delete, and signature request lifecycle implemented and verified through container build gates.

## Files and Signatures Slice

### Implemented Endpoints
1. `POST /api/v1/workspaces/{ws_id}/files/presigned`
   - Requires `files:manage` permission.
   - Validates file name, MIME type, positive size up to 50 MB, and owner ID.
   - Creates a `workspace_files` row with `scan_status = pending`, version 1, and active latest flag.
   - Generates an S3/MinIO presigned PUT URL with configurable TTL.
2. `GET /api/v1/workspaces/{ws_id}/files/{id}`
   - Requires `files:read` permission.
   - Tenant-scoped lookup restricted to active files; returns 404 for unknown or soft-deleted rows.
3. `GET /api/v1/workspaces/{ws_id}/files/{id}/url`
   - Requires `files:read` permission.
   - Returns presigned download URL (`{"url": "..."}`).
4. `DELETE /api/v1/workspaces/{ws_id}/files/{id}`
   - Requires `files:manage` permission.
   - Performs soft delete (`is_active = false`) and returns 404 when no active row matches.
5. `POST /api/v1/workspaces/{ws_id}/documents/{document_id}/signature-requests`
   - Requires `documents:manage` permission.
   - Persists a pending `signature_requests` row with signer UUID.
6. `GET /api/v1/ess/workspaces/{ws_id}/signatures/pending`
   - Lists pending signature requests for the authenticated signer.
7. `POST /api/v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign`
   - Transitions a pending signature request to `signed`, stamping `signed_at`.

### Source Files
- `internal/api/files.go`: Typed handlers, DTOs, permission checks, and S3 presigning.
- `internal/api/files_test.go`: Presigned upload, metadata, URL, delete, and signature lifecycle tests.
- `internal/repository/files.go`: Tenant-scoped Postgres queries for `workspace_files` and `signature_requests`.
- `main.go`: Registered `/v1` and `/api/v1` route aliases and wired `FileRepository` with the S3 client.

### Verification Status
- `scripts/_go-gates.sh`: ALL GATES PASSED (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`).
- `./leedevkit test api-go`: ALL GATES PASSED.

### Remaining Parity Work
- Multipart upload proxy path (`/files/upload`) and version update (`PUT /files/{id}`) are not yet ported.
- Malware/ClamAV rejection path and infected-file quarantine policy require a dedicated fixture run.
