# P6-18 Documents & Folders Parity Evidence

Task 6.2 Documents & folders (13 routes) completed in Go API, advancing contract route coverage from 357 to 370 (missing routes reduced from 46 to 33).

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **408** (+26 with `/v1` and `/api/v1` aliases)
- Contract routes covered: **370** (+13)
- Contract routes missing: **33** (-13)

---

## 13 Newly Implemented & Wired Routes

All 13 routes wired in `apiserver/main.go` and implemented in `internal/api/documents.go`, `internal/api/files.go`, `internal/repository/documents.go`, and `internal/repository/files.go`:

### 1. Folders (5 routes)
1. `GET /api/v1/workspaces/{ws_id}/documents/folders` — 200 OK `[]DocumentFolderDto`. List workspace document folders ordered by name. Auth: workspace membership (`IsMember`).
2. `POST /api/v1/workspaces/{ws_id}/documents/folders` — 200 OK `DocumentFolderDto`. Create folder with optional `parent_folder_id` validation. Auth: workspace membership (`IsMember`).
3. `GET /api/v1/workspaces/{ws_id}/documents/folders/{id}` — 200 OK `DocumentFolderDto`. Retrieve folder details by ID; 404 if missing. Auth: workspace membership (`IsMember`).
4. `PUT /api/v1/workspaces/{ws_id}/documents/folders/{id}` — 200 OK `DocumentFolderDto`. Rename existing folder; 404 if missing. Auth: workspace membership (`IsMember`).
5. `DELETE /api/v1/workspaces/{ws_id}/documents/folders/{id}` — 200 OK `{"status":"success"}`. Delete folder (Postgres foreign keys handle cascade); 404 if missing. Auth: workspace membership (`IsMember`).

### 2. Distributions & Templates (3 routes)
6. `POST /api/v1/workspaces/{ws_id}/documents/distributions` — 200 OK `{"status":"success","count":N}`. Create distribution acknowledgements for target employee IDs in `file_acknowledgements`. Auth: workspace membership (`IsMember`).
7. `POST /api/v1/workspaces/{ws_id}/documents/templates/preview` — 200 OK `TemplatePreviewResponse`. Dynamic variable interpolation (`\{\{([A-Z0-9_]+)\}\}`) with workspace, employee, employment, contract tokens, and manual variable overrides. Auth: workspace membership (`IsMember`).
8. `POST /api/v1/workspaces/{ws_id}/documents/templates/{id}/generate` — 200 OK `{"status":"generated"}`. Template document generation endpoint. Auth: workspace membership (`IsMember`).

### 3. Files (5 routes)
9. `POST /api/v1/workspaces/{ws_id}/files/upload` — 201 Created `WorkspaceFileRecord`. Multipart upload parsing (20MB cap), category MIME/size validation, SHA-256 hash calculation, object storage upload, and metadata persistence. Auth: workspace membership (`IsMember`).
10. `PUT /api/v1/workspaces/{ws_id}/files/{id}` — 200 OK `WorkspaceFileRecord`. Copy-on-Write (CoW) versioning update. Short-circuits when content SHA-256 matches existing hash; creates new version (`is_latest=false` on previous, `version+1` on new) when content changes. Auth: workspace membership (`IsMember`).
11. `GET /api/v1/workspaces/{ws_id}/files/{id}/download` — 200 OK (streamed proxy) or 307 Temporary Redirect (presigned GET). Files `<= 1MB` streamed directly with `Content-Type` and `Content-Disposition`; files `> 1MB` redirect to S3 presigned URL. Auth: workspace membership (`IsMember`).
12. `GET /api/v1/workspaces/{ws_id}/files/{id}/versions` — 200 OK `[]WorkspaceFileRecord`. Recursive CTE lineage query returning version ancestry chain ordered by version DESC. Auth: workspace membership (`IsMember`).
13. `GET /api/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}` — 200 OK `[]WorkspaceFileRecord`. Queries active workspace files matching owner type and owner UUID. Auth: workspace membership (`IsMember`).

---

## Complete Route Map (13/13)

| # | Method | Path | Status | Handler | Auth Guard |
|---|---|---|---|---|---|
| 1 | GET | `/api/v1/workspaces/{ws_id}/documents/folders` | 200 OK | `DocumentHandler.ListFolders` | Workspace Member |
| 2 | POST | `/api/v1/workspaces/{ws_id}/documents/folders` | 200 OK | `DocumentHandler.CreateFolder` | Workspace Member |
| 3 | GET | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | 200 OK | `DocumentHandler.GetFolder` | Workspace Member |
| 4 | PUT | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | 200 OK | `DocumentHandler.RenameFolder` | Workspace Member |
| 5 | DELETE | `/api/v1/workspaces/{ws_id}/documents/folders/{id}` | 200 OK | `DocumentHandler.DeleteFolder` | Workspace Member |
| 6 | POST | `/api/v1/workspaces/{ws_id}/documents/distributions` | 200 OK | `DocumentHandler.Distribute` | Workspace Member |
| 7 | POST | `/api/v1/workspaces/{ws_id}/documents/templates/preview` | 200 OK | `DocumentHandler.PreviewTemplate` | Workspace Member |
| 8 | POST | `/api/v1/workspaces/{ws_id}/documents/templates/{id}/generate` | 200 OK | `DocumentHandler.GenerateTemplate` | Workspace Member |
| 9 | POST | `/api/v1/workspaces/{ws_id}/files/upload` | 201 Created | `FileHandler.UploadFile` | Workspace Member |
| 10 | PUT | `/api/v1/workspaces/{ws_id}/files/{id}` | 200 OK | `FileHandler.UpdateFile` | Workspace Member |
| 11 | GET | `/api/v1/workspaces/{ws_id}/files/{id}/download` | 200 OK / 307 | `FileHandler.DownloadFile` | Workspace Member |
| 12 | GET | `/api/v1/workspaces/{ws_id}/files/{id}/versions` | 200 OK | `FileHandler.ListVersions` | Workspace Member |
| 13 | GET | `/api/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}` | 200 OK | `FileHandler.ListByOwner` | Workspace Member |

---

## Implementation Details

1. **Category & MIME Validation Rules**:
   - `employee_avatar`, `user_avatar`: <= 5MB; `image/jpeg`, `image/png`, `image/webp`.
   - `workspace_logo`: <= 5MB; `image/jpeg`, `image/png`, `image/webp`, `image/svg+xml`.
   - `attendance_photo`: <= 10MB; `image/jpeg`, `image/png`, `image/webp`.
   - `employee_doc`, `onboarding_attachment`: <= 50MB; PDF, JPEG, PNG, WEBP, DOC, DOCX.
   - `contract`: <= 50MB; PDF, DOC, DOCX.
   - `leave_attachment`, `attendance_adjustment_attachment`, `overtime_request_attachment`, `ot_claim_attachment`: <= 20MB; PDF.
   - `payslip`: <= 10MB; PDF.
   - `report_export`: <= 100MB; XLSX, CSV.
   - `import_file`: <= 20MB; XLSX, XLS.
   - `policy`: <= 50MB; PDF.
   - `template_output`: <= 50MB; PDF, DOCX.
   - `system_asset`: <= 50MB; JPEG, PNG, WEBP, SVG, PDF.

2. **Copy-on-Write (CoW) Versioning**:
   - `UpdateFile` hashes incoming bytes via SHA-256.
   - If `content_hash == current_file.content_hash`, bypasses S3 upload and DB mutation, returning current record (200 OK).
   - If changed, uploads new S3 key and runs DB transaction: marks previous version `is_latest = false` and inserts new version with `version = previous.version + 1`, `parent_file_id = previous.id`, and `is_latest = true`.

3. **Download Proxy vs Redirect**:
   - 1MB threshold (`1024 * 1024` bytes).
   - `<= 1MB`: proxies bytes directly from storage, sets `Content-Type: mime_type` and `Content-Disposition: attachment; filename="{file_name}"`, returns HTTP 200.
   - `> 1MB`: generates 15-minute presigned GET URL and issues HTTP 307 Temporary Redirect.

4. **Template Substitution**:
   - Regex `\{\{([A-Z0-9_]+)\}\}` extracts placeholder tokens.
   - Populates `COMPANY_NAME` (workspace name or fallback), `EMPLOYEE_NAME`, `EMPLOYEE_ID_NUMBER`, `HIRE_DATE` (`DD/MM/YYYY`), `JOB_TITLE`, `CONTRACT_NUMBER`, `DOCUMENT_DATE` (`DD/MM/YYYY`).
   - Merges caller `manual_variables` as overrides.
   - Preserves unrecognized placeholders intact.

---

## Unit Tests

1. `internal/api/files_test.go`:
   - `TestFileUploadMultipart`: tests valid multipart upload, category validation, 201 Created status, SHA-256 computation, and metadata generation.
   - `TestFileUpdateCoW`: tests CoW bypass on identical content (same version, no new S3 upload) and version increment on modified content (`version = 2`, `parent_file_id` link).
   - `TestFileDownload`: verifies 200 OK proxy streaming for `<= 1MB` files and 307 Temporary Redirect for `> 1MB` files.
   - `TestFileListVersionsAndByOwner`: verifies recursive ancestry lineage query and active file filtering by owner type and ID.
   - `TestFilePresignedUploadAndMetadata`: preserves existing tests for presigned upload and metadata retrieval.
   - `TestSignatureRequestLifecycle`: preserves existing signature workflow tests.

2. `internal/api/documents_test.go`:
   - `TestDocumentFolders_CRUD`: tests listing folders, root folder creation, subfolder creation with parent validation, invalid parent rejection (400 Bad Request), get folder, rename folder, delete folder, and 404 for deleted folder.
   - `TestDocumentDistribution`: tests creating acknowledgement requests for multiple target employee IDs.
   - `TestDocumentTemplates_PreviewAndGenerate`: tests template preview variable interpolation with standard variables, manual overrides, current date formatting, unmatched placeholder preservation, and template generation.
   - `TestDocumentAuth_Forbidden`: verifies non-members receive 403 Forbidden.

---

## Verification Status

All 13 routes wired and tested. Ready for verification via:
- `./scripts/_go-gates.sh`
- `./leedevkit test api-go`
