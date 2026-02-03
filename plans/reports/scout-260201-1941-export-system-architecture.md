# Export System Architecture Research Report

**Date**: 2026-02-01  
**Scout Agent**: afe3492  
**Task**: Research current export system for refactor planning

---

## Executive Summary

Current architecture: **Store-and-Forward with Redis-based batch processing**
- Workers compute full attendance data, serialize to JSON, store in Redis
- Merge worker aggregates all JSON batches into final Excel file
- File uploaded to S3/local storage, download URL returned to client
- No streaming, no WebSocket, no Redis Pub/Sub currently implemented

---

## 1. Report Coordinator (`apiserver/src/services/report_coordinator.rs`)

### Key Constants
```rust
BATCH_SIZE: usize = 20  // Employees per micro-job
JOB_TIMEOUT_SECS: u64 = 3600
ESTIMATED_SECONDS_PER_BATCH: u64 = 2
```

### Job Scheduling Flow
1. `create_report_job()`: Splits employee list into batches of 20
2. Creates parent `JobMetadata` in Redis with total batch count
3. Enqueues all micro-jobs atomically via `ReportQueue::enqueue_jobs()`
4. Returns `job_id`, `status_url`, `estimated_seconds` to client

### Result Storage Mechanism
- **Batch completion**: Increments counters in Redis (`increment_completed`, `increment_failed`)
- **Merge trigger**: When `completed_count >= total_batches`, enqueues `MergeResults` job
- **Final URL storage**: `queue.set_result_url(job_id, url)` stores download link in Redis key `report:results:{job_id}:url`

---

## 2. Worker Handlers (`apiserver/src/worker/handlers/`)

### Attendance Handler (`attendance.rs`)
**Current Output**: Stores JSON-serialized result in Redis

```rust
// Line 128-131
queue.store_result(job.parent_job_id, job.batch_index, &result_json)
```

**Result Types**:
- **Summary**: `Vec<AttendanceSummary>` → JSON
- **Detail**: `Vec<DailyAttendanceReport>` → JSON

**Progress Tracking**: No explicit progress updates mid-batch. Worker completes entire batch before reporting.

### Merge Handler (`merge.rs`)
**Current Process**:
1. `queue.get_all_results(parent_job_id)` → Retrieves all batch JSON from Redis
2. Deserialize and merge into single collection
3. `ExcelWriter::write_summary()` / `::write_detail()` → Generate Excel bytes
4. `storage.upload(key, excel_data, content_type)` → Upload to S3/local
5. `storage.get_download_url(key)` → Generate presigned URL (S3) or API path (local)
6. `queue.set_result_url(job_id, url)` → Store download link in Redis
7. `queue.update_job_status(JobStatus::Completed)` → Mark job done
8. `queue.delete_results(parent_job_id)` → Cleanup batch JSON

---

## 3. Redis Infrastructure (`apiserver/src/queue/`)

### Current Keys
```rust
QUEUE_KEY = "report:jobs:attendance"           // Job queue (RPUSH/BLPOP)
META_PREFIX = "report:meta:"                   // Job metadata hash
RESULTS_PREFIX = "report:results:"             // Batch results (HSET)
COUNTER_PREFIX = "report:counter:"             // Atomic counters
```

### Redis Operations
- **Job enqueue**: `RPUSH report:jobs:attendance {json}`
- **Job dequeue**: `BLPOP report:jobs:attendance 30` (workers)
- **Store result**: `HSET report:results:{job_id} batch_{index} {json}`
- **Get all results**: `HGETALL report:results:{job_id}`
- **Set URL**: `SET report:results:{job_id}:url {url}` + `EXPIRE 86400`

### Pub/Sub Capability
**Status**: NOT currently used
- Redis client supports Pub/Sub (ConnectionManager available)
- No `PUBLISH` or `SUBSCRIBE` commands found in codebase
- **Gap**: Real-time progress notifications would require implementing Pub/Sub subscriber

---

## 4. WebSocket Infrastructure

**Status**: **Not Implemented for Reports**

Found WebSocket reference only in:
- `/home/elt1541/leeattend/apiserver/src/handlers/agents.rs` (agent heartbeat monitoring)
- **Gap**: No WebSocket handler for report progress streaming exists

---

## 5. Frontend Export Flow

**Location**: `/home/elt1541/leeattend/webdashboard/` (React/Vite)

**Current API Pattern** (inferred from handlers):
1. POST `/api/workspaces/{ws_id}/reports/export` → Returns `{ job_id, status_url }`
2. Client polls GET `/api/workspaces/{ws_id}/reports/jobs/{job_id}` → Returns progress
3. When `status: "completed"`, GET `/api/workspaces/{ws_id}/reports/jobs/{job_id}/download` → 307 Redirect to presigned URL

**Handler**: `/home/elt1541/leeattend/apiserver/src/handlers/report_export.rs`

---

## 6. Storage Infrastructure (`apiserver/src/services/storage.rs`)

### Storage Providers
```rust
trait StorageProvider {
    async fn upload(key, data, content_type) -> Result<String>
    async fn get_download_url(key) -> Result<String>
}
```

**Implementations**:
- **S3Storage**: Uses `S3Client::upload_report()`, generates presigned URL (24hr expiry)
- **LocalStorage**: Writes to `./storage/reports/{workspace_id}/{year}/{month}/{filename}`, returns `/api/reports/local/{key}`

**Local Download Handler**: `serve_local_report()` in `report_export.rs`
- Validates workspace access via MemberContext
- Streams file with `ReaderStream<File>`
- Security: Canonical path validation to prevent traversal

---

## Current Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│ Client (webdashboard/)                                       │
│   1. POST /reports/export → job_id                           │
│   2. Poll GET /reports/jobs/{id} → progress (HTTP polling)   │
│   3. GET /reports/jobs/{id}/download → 307 Redirect          │
└───────────────────────────────┬─────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────┐
│ ReportCoordinator                                            │
│   - Splits employees into batches (BATCH_SIZE = 20)         │
│   - Creates JobMetadata in Redis                            │
│   - Enqueues micro-jobs to Redis LIST                       │
└───────────────────────────────┬─────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────┐
│ ReportWorker (runner.rs)                                     │
│   - BLPOP jobs from queue (30s timeout)                     │
│   - Routes to AttendanceHandler or MergeHandler             │
│   - Max 2 concurrent jobs per worker (Semaphore)            │
└───────────────────────────────┬─────────────────────────────┘
                                │
         ┌──────────────────────┴──────────────────────┐
         │                                             │
┌────────▼─────────┐                         ┌────────▼─────────┐
│ AttendanceHandler│                         │  MergeHandler    │
│  - Fetch DB data │                         │  - HGETALL Redis │
│  - Calculate     │                         │  - Merge JSON    │
│  - Serialize JSON│                         │  - ExcelWriter   │
│  - HSET Redis    │                         │  - Upload S3     │
└────────┬─────────┘                         │  - SET URL Redis │
         │                                   └────────┬─────────┘
         └──────────────┬────────────────────────────┘
                        │
         ┌──────────────▼──────────────┐
         │ Redis Storage                │
         │  - results:{id}:batch_{idx} │ → JSON batches
         │  - results:{id}:url         │ → Download URL
         │  - counter:{id}             │ → Completion count
         └─────────────────────────────┘
```

---

## Reusable Infrastructure

✅ **Already Implemented**:
- Redis connection pool (ConnectionManager)
- Job queue (RPUSH/BLPOP)
- Atomic counters (INCR)
- Storage abstraction (S3 + Local fallback)
- Batch processing workers with semaphore
- Progress tracking via JobMetadata

---

## Gaps for Streaming Architecture

❌ **Missing Components**:
1. **WebSocket Handler**: No WS endpoint for real-time progress
2. **Redis Pub/Sub**: Not used (would enable worker → WS broadcast)
3. **Streaming Excel Writer**: Current `ExcelWriter` generates full file in memory
4. **Client-side streaming**: Frontend needs SSE/WebSocket listener + XLSX assembly
5. **Incremental download**: No HTTP range request support

---

## Key Files Reference

### Backend Core
- `apiserver/src/services/report_coordinator.rs` - Job orchestration (484 lines)
- `apiserver/src/queue/report_queue.rs` - Redis operations (569 lines)
- `apiserver/src/worker/runner.rs` - Worker loop + heartbeat (577 lines)
- `apiserver/src/worker/handlers/attendance.rs` - Batch computation (610 lines)
- `apiserver/src/worker/handlers/merge.rs` - Merge + upload (387 lines)

### API Layer
- `apiserver/src/handlers/report_export.rs` - HTTP endpoints (526 lines)
- `apiserver/src/services/storage.rs` - S3/Local abstraction (289 lines)
- `apiserver/src/services/excel_writer.rs` - Excel generation (not analyzed)

### Frontend
- `webdashboard/` - React/Vite app (structure not explored)

---

## Unresolved Questions

1. Current Excel file size limits with BATCH_SIZE=20?
2. Redis memory usage for 7000 employees × 31 days detail report?
3. Frontend polling interval and timeout logic?
4. Worker scaling strategy (how many workers deployed)?
5. ExcelWriter implementation details (in-memory vs streaming capability)?

