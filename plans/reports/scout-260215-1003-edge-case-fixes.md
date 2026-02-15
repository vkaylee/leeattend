# Scout Report: Edge Case Fixes Files Inventory

**Date:** 2026-02-15 | **Time:** 10:03 UTC  
**Task:** Locate all files relevant to 9 edge case fixes in leeattend codebase

---

## Search Results Summary

Successfully identified all 9 target files plus supporting infrastructure files.

---

## Core Target Files

### 1. **auth.rs** (Auth Handler - get_users endpoint)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/auth.rs`
- **Lines:** 418
- **Summary:** Authentication handlers including register, login, refresh token, logout; contains get_users endpoint (line 299) and get_user_profile handler

### 2. **report_export.rs** (Report Export - Job Polling)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/report_export.rs`
- **Lines:** 400
- **Summary:** Async report export job handlers including create_export_job, get_job_status, cancel_job, download_report with Redis queue integration

### 3. **import.rs** (Attendance Import - Job Polling)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/import.rs`
- **Lines:** 259
- **Summary:** Attendance Excel upload handler with file storage and job creation; includes get_latest_import_job and get_import_job_status endpoints

### 4. **report_websocket.rs** (WebSocket Report Handler)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/report_websocket.rs`
- **Lines:** 168
- **Summary:** WebSocket upgrade handler for real-time report streaming via Redis Pub/Sub; route: GET /ws/reports/:job_id

### 5. **payroll_service.rs** (Payroll - Batch Concurrency, Grace Period, OT, Deductions)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/services/payroll_service.rs`
- **Lines:** 395
- **Summary:** Monthly payslip calculation engine handling workspace timezone, grace period context, OT intervals, batch deductions, and payroll evaluator integration

### 6. **logs.rs** (Attendance Logs - Reprocess Endpoint)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/logs.rs`
- **Lines:** 166
- **Summary:** Attendance log retrieval handlers including get_attendance_logs, get_raw_attendance_logs, and force_reprocess_attendance (line 123) with date range processing

### 7. **store.rs** (Agent Memory Store - Signature Cache)
- **Path:** `/home/elt1541/lee/leeattend/agent-main/src/memory_store/store.rs`
- **Lines:** 2666
- **Summary:** In-memory FIFO attendance log store with signature deduplication (lines 50-67), composite key hashing, TTL eviction, and sync state tracking

### 8. **resolver.rs** (Policy Resolution - Grace Period Context)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/domain/policy/resolver.rs`
- **Lines:** 243
- **Summary:** Policy resolver with temporal hierarchy (employee > department > workspace > system defaults); resolves standard month config with grace period context

### 9. **lib.rs** (Route Definitions & Middleware)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/lib.rs`
- **Lines:** 621
- **Summary:** Router configuration with all route definitions (lines 116-565), UserId/MemberContext/EmployeeContext extractors, AppState definition, CORS setup

---

## Supporting Infrastructure Files

### 10. **main.rs** (Entry Point & Worker Setup)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/main.rs`
- **Lines:** 282
- **Summary:** Server initialization, database migration, worker setup, AttendanceImportService resume, ReportWorker/ReportCoordinator creation

### 11. **common.rs** (Extractor Definitions)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/common.rs`
- **Lines:** 182
- **Summary:** **MemberContext extractor** (lines 62-124), **UserId extractor** reference, WorkspaceId extractor, EmployeeContext extractor, health_check handler

### 12. **runner.rs** (Worker Concurrency Management)
- **Path:** `/home/elt1541/lee/leeattend/apiserver/src/worker/runner.rs`
- **Lines:** 594
- **Summary:** Report worker with **Semaphore** for concurrency control (line 11 import, line 182 usage), BLPOP polling, job dispatch, heartbeat monitoring

---

## Extractor Definitions (Key Infrastructure)

### MemberContext Extractor
- **Location:** `/home/elt1541/lee/leeattend/apiserver/src/handlers/common.rs` (lines 62-124)
- **Definition:** Struct holding `user_id`, `workspace_id`, `roles: Vec<MemberRole>`
- **Traits:** `FromRequestParts<S>` where `Arc<AppState>: FromRef<S>`
- **Usage:** Extracts from path + header with RBAC role resolution
- **Methods:** `has_role()`, `is_admin_or_owner()`

### UserId Extractor
- **Location:** `/home/elt1541/lee/leeattend/apiserver/src/lib.rs` (lines 57-112, re-exported as `ax_auth::UserId`)
- **Definition:** Newtype struct `UserId(pub Uuid)`
- **Traits:** `FromRequestParts<S>` where `Arc<AppState>: FromRef<S>`
- **Auth Flow:** 
  1. Debug impersonation check (X-Debug-Impersonate-Id header)
  2. Bearer token verification via AuthService::verify_token
  3. Debug X-User-Id header fallback
  4. Returns 401 Unauthorized if none match

---

## Concurrency & Rate Limiting Patterns

### Semaphore Usage (Existing)
- **File:** `/home/elt1541/lee/leeattend/apiserver/src/worker/runner.rs`
- **Import:** Line 11: `use tokio::sync::{Mutex, Semaphore};`
- **Usage:** 
  - Line 182: `Arc::new(Semaphore::new(self.config.max_concurrent_jobs))`
  - Used in worker loop to limit concurrent job processing
  - Prevents > max_concurrent_jobs tasks running simultaneously
  - Default: 2 concurrent jobs (configurable via WORKER_MAX_CONCURRENT)

### No Existing Rate Limiter Found
- No dedicated rate-limiting middleware found in codebase
- Semaphore is used only for internal job concurrency
- Rate limiting for API endpoints is NOT currently implemented

---

## Route Definitions Location

**File:** `/home/elt1541/lee/leeattend/apiserver/src/lib.rs` (lines 157-565)

Key routes for edge cases:
- Line 188: `"/users"` → `get(handlers::get_users)` — **Target #1**
- Line 423-432: Report export routes — **Target #2**
- Line 401-406: Import job polling — **Target #3**
- Line 436-437: WebSocket report route — **Target #4**
- Line 397-398: Reprocess endpoint — **Target #6**

---

## Critical Implementation Details

### Grace Period Context
- **Resolver:** `/home/elt1541/lee/leeattend/apiserver/src/domain/policy/resolver.rs`
- **Method:** `PolicyResolver::resolve_standard_month()` (lines 18-78)
- **Hierarchy:** Employee override > Department override > Workspace temporal > System default
- **Returns:** `StandardMonthConfig` with `std_working_days`, `hours_per_day`, `source`

### Batch Payroll Concurrency
- **File:** `/home/elt1541/lee/leeattend/apiserver/src/services/payroll_service.rs`
- **Method:** `calculate_monthly_payslip()` (lines 22-50+)
- **Context:** Receives workspace timezone for proper period definition
- **OT Integration:** Uses `OvertimeEngine` and `PolicyResolver` for grace period resolution

### Signature Deduplication Cache
- **File:** `/home/elt1541/lee/leeattend/agent-main/src/memory_store/store.rs`
- **Method:** `compute_signature()` (lines 50-61)
- **Signature:** FxHash of (device_id, user_id, timestamp_unix, punch, status)
- **Storage:** `DashMap<u64, i64>` mapping signature → timestamp_utc
- **API:** `has_signature()` (line 64), `count_signatures()` (line 70)

### Job Polling Architecture
- **Export Jobs:** Redis queue + ReportCoordinator service
- **Import Jobs:** AttendanceImportService with job status tracking
- **Dispatch:** BLPOP polling in runner with configurable timeout
- **Concurrency:** Semaphore-gated task execution

---

## Route Table (from lib.rs)

```
POST   /users                              → get_users (line 299 in auth.rs) ⚠️ NO AUTH CHECK
POST   /auth/login                         → login
POST   /auth/register                      → register
POST   /auth/refresh                       → refresh_token
POST   /auth/logout                        → logout

POST   /workspaces/:ws_id/reports/export   → create_export_job
GET    /workspaces/:ws_id/reports/jobs/:job_id → get_job_status
DELETE /workspaces/:ws_id/reports/jobs/:job_id → cancel_job
GET    /workspaces/:ws_id/reports/jobs/:job_id/download → download_report

POST   /workspaces/:ws_id/attendance/upload → upload_attendance_excel (import)
GET    /workspaces/:ws_id/attendance/import-jobs/latest → get_latest_import_job
GET    /workspaces/:ws_id/attendance/import-jobs/:id → get_import_job_status

GET    /ws/reports/:job_id                 → report_stream_ws (WebSocket)

POST   /workspaces/:ws_id/attendance/reprocess/:employee_id → force_reprocess_attendance
```

---

## Edge Case Implementation Files Summary Table

| # | Edge Case | Primary File | Support Files | Key Function |
|---|-----------|--------------|----------------|--------------|
| 1 | get_users auth | auth.rs (299) | lib.rs (UserId), common.rs | `get_users()` - NO RBAC |
| 2 | report_export polling | report_export.rs | queue.rs, runner.rs, coordinator | `create_export_job()` |
| 3 | import polling | import.rs | runner.rs, service | `upload_attendance_excel()` |
| 4 | websocket handler | report_websocket.rs | queue.rs, RedisPool | `report_stream_ws()` |
| 5 | payroll batch | payroll_service.rs | resolver.rs, OvertimeEngine | `calculate_monthly_payslip()` |
| 6 | reprocess logs | logs.rs (123) | PairingService | `force_reprocess_attendance()` |
| 7 | memory signature | store.rs (50-67) | agent-main | `compute_signature()` |
| 8 | grace period | resolver.rs (18-78) | payroll_service | `resolve_standard_month()` |
| 9 | routes/middleware | lib.rs (157-565) | common.rs extractors | `create_app()` |

---

## Unresolved Questions

1. Is there a pre-existing rate limiter middleware not found in this search (e.g., in tower middleware layer)?
2. What is the exact panic/error behavior expected when Semaphore.acquire() fails in worker runner?
3. Are there integration tests for concurrent payroll batch processing that need to be updated?

