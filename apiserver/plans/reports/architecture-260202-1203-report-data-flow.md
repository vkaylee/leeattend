# Report Generation Data Flow Architecture

**Type:** Architecture Documentation
**Date:** 2026-02-02 12:03
**Component:** Report Export System

## Executive Summary

Report system uses **3-phase architecture**: batch processing → merge → download. All intermediate and final data stored in Redis, not streamed directly to client during processing.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     PHASE 1: BATCH PROCESSING                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Worker 1       Worker 2       Worker 3       Worker 4         │
│     │              │              │              │              │
│     ├─ Batch 0     ├─ Batch 1    ├─ Batch 2    ├─ Batch 3    │
│     │   (176 emp)  │   (176 emp) │   (176 emp) │   (176 emp) │
│     │              │              │              │              │
│     └──────┬───────┴──────┬───────┴──────┬───────┴──────┬──────┘
│            │              │              │              │
│            ▼              ▼              ▼              ▼
│      ┌──────────────────────────────────────────────────┐
│      │         Redis Hash: report:results:{job_id}      │
│      ├──────────────────────────────────────────────────┤
│      │  batch_00000 → JSON (176 employees)              │
│      │  batch_00001 → JSON (176 employees)              │
│      │  batch_00002 → JSON (176 employees)              │
│      │  batch_00003 → JSON (176 employees)              │
│      └──────────────────────────────────────────────────┘
│                          │
│                          │ All batches stored in Redis
│                          │ Waiting for merge...
└─────────────────────────┼────────────────────────────────────────┘
                          │
                          │ Last batch triggers merge
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      PHASE 2: MERGE                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                     Merge Worker                                │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 1. get_all_results() │                     │              │
│    │    Fetch all batches │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          ▼                                      │
│              ┌───────────────────────┐                          │
│              │  Merge JSON Results   │                          │
│              │  704 employees total  │                          │
│              └───────────┬───────────┘                          │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 2. Generate Excel    │                     │              │
│    │    25KB .xlsx file   │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          ▼                                      │
│              ┌───────────────────────┐                          │
│              │ Base64 encode Excel   │                          │
│              └───────────┬───────────┘                          │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 3. set_result_data() │                     │              │
│    │    Store in Redis    │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          ▼                                      │
│      ┌──────────────────────────────────────────┐              │
│      │  Redis Keys:                             │              │
│      │  report:results:{job_id}:data            │              │
│      │    → base64(Excel) 25KB                  │              │
│      │  report:results:{job_id}:filename        │              │
│      │    → attendance_summary_...xlsx          │              │
│      │  TTL: 1 hour                             │              │
│      └──────────────────────────────────────────┘              │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 4. delete_results()  │                     │              │
│    │    Clean batch data  │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          │                                      │
└─────────────────────────┼────────────────────────────────────────┘
                          │
                          │ Client requests download
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PHASE 3: DOWNLOAD                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                      Client Request                             │
│    GET /api/workspaces/{id}/reports/jobs/{id}/download         │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 1. Check job status  │                     │              │
│    │    Must be completed │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 2. get_result_data() │                     │              │
│    │    Fetch from Redis  │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          ▼                                      │
│              ┌───────────────────────┐                          │
│              │  Base64 decode Excel  │                          │
│              │  25KB binary data     │                          │
│              └───────────┬───────────┘                          │
│                          │                                      │
│    ┌─────────────────────┼──────────────────────┐              │
│    │ 3. Send to client    │                     │              │
│    │    HTTP Response     │                     │              │
│    │    Content-Type:     │                     │              │
│    │    application/xlsx  │                     │              │
│    └─────────────────────┼──────────────────────┘              │
│                          ▼                                      │
│                      Client Browser                             │
│                  Downloads Excel File                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Data Storage Strategy

### Phase 1: Batch Results (JSON in Redis)

**Storage:**
```
Redis Key: report:results:{job_id}
Type: HASH
TTL: 24 hours

Fields:
  batch_00000 → JSON string (15-30KB per batch)
  batch_00001 → JSON string
  batch_00002 → JSON string
  batch_00003 → JSON string
```

**Example:**
```json
{
  "batch_00000": {
    "employees": [
      {
        "id": "uuid",
        "name": "Employee 1",
        "attendance": {...}
      },
      // ... 176 employees
    ]
  }
}
```

### Phase 2: Final Excel (Binary in Redis)

**Storage:**
```
Redis Keys:
  report:results:{job_id}:data     → base64(Excel bytes)
  report:results:{job_id}:filename → "attendance_summary_...xlsx"
Type: STRING
TTL: 1 hour
```

**Why base64?**
- Redis stores strings efficiently
- Binary data needs encoding
- Easy to decode on retrieval

### Phase 3: Client Download (Binary Stream)

**HTTP Response:**
```http
HTTP/1.1 200 OK
Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet
Content-Disposition: attachment; filename="attendance_summary_...xlsx"
Content-Length: 25283

[binary Excel data]
```

## Code Flow

### 1. Worker: Store Batch Result

**File:** `src/worker/handlers/attendance.rs:132-156`

```rust
// Store batch result in Redis
queue.store_result(job.parent_job_id, job.batch_index, &result_json).await?;

// Also publish to Pub/Sub for real-time streaming (optional UI feature)
publisher.publish_chunk(job.parent_job_id, job.batch_index, metadata.total_batches, chunk_data).await?;

info!("Batch result stored and published");
```

**Redis Operation:**
```
HSET report:results:{job_id} batch_00000 "{json_data}"
EXPIRE report:results:{job_id} 86400
```

### 2. Merge: Retrieve & Merge All Batches

**File:** `src/worker/handlers/merge.rs:52-137`

```rust
// Fetch all batch results
let batch_results = queue.get_all_results(parent_job_id).await?;

// Merge JSON results
let merged = merge_batch_results(batch_results, report_type)?;

// Generate Excel file
let excel_bytes = generate_excel(merged)?;

// Store Excel in Redis (base64 encoded)
queue.set_result_data(parent_job_id, &excel_bytes, &filename).await?;

// Clean up batch results
queue.delete_results(parent_job_id).await?;
```

**Redis Operations:**
```
# Retrieve
HGETALL report:results:{job_id}  → Get all batch_* fields

# Store Excel
SET report:results:{job_id}:data "base64(excel)"
EXPIRE report:results:{job_id}:data 3600
SET report:results:{job_id}:filename "attendance_summary_...xlsx"
EXPIRE report:results:{job_id}:filename 3600

# Cleanup
DEL report:results:{job_id}  → Remove batch results hash
```

### 3. Download: Send to Client

**File:** `src/handlers/report_export.rs:267-311`

```rust
// Get result data from Redis
let (data, filename) = queue.get_result_data(job_id).await?
    .ok_or("Report data not found or expired")?;

// Send as Excel download
Response::builder()
    .header(CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    .header(CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
    .body(Body::from(data))
```

## Why This Architecture?

### ✅ Advantages

1. **Decoupled Processing**
   - Workers don't need to know about clients
   - Can retry without client connection issues
   - Horizontal scaling of workers

2. **Fault Tolerance**
   - Batch results persisted in Redis
   - Merge can retry if it fails
   - Client can download multiple times (within TTL)

3. **Resource Efficiency**
   - Workers process in parallel
   - Merge only happens once
   - Client doesn't wait during processing

4. **Flexible Retrieval**
   - Client downloads when ready
   - No need for long-lived connections
   - Supports resume/retry downloads

### ⚠️ Trade-offs

1. **Redis Memory Usage**
   - Each job stores batch results (~100KB total)
   - Final Excel also stored (~25KB)
   - TTL manages cleanup

2. **Two-Step Download**
   - Client must poll for completion
   - Then download when ready
   - Could use WebSocket for notifications

3. **Temporary Storage**
   - 1-hour TTL on Excel files
   - Client must download within window
   - Could extend TTL or use S3 for longer retention

## Data Size Analysis

### Small Job (704 employees)

```
Batch Results (4 batches):
  batch_00000: ~15KB JSON
  batch_00001: ~15KB JSON
  batch_00002: ~15KB JSON
  batch_00003: ~15KB JSON
  Total: ~60KB in Redis

Final Excel:
  Binary: 25KB
  Base64: ~34KB in Redis

Peak Redis Usage: ~60KB + ~34KB = ~94KB
After Cleanup: ~34KB (until download)
```

### Large Job (10,000 employees)

```
Batch Results (57 batches @ 176 each):
  57 × 15KB = ~855KB JSON in Redis

Final Excel:
  Binary: ~300KB
  Base64: ~400KB in Redis

Peak Redis Usage: ~855KB + ~400KB = ~1.3MB
After Cleanup: ~400KB (until download)
```

## Redis Key Lifecycle

```
Job Created (T=0)
│
├─ report:meta:{job_id}                    [Created]
├─ report:counter:{job_id}                 [Created]
└─ report:merge_triggered:{job_id}         [Not yet]

Batches Processing (T=0 to T+60s)
│
├─ report:results:{job_id}                 [Growing]
│   ├─ batch_00000 → JSON                  [Added]
│   ├─ batch_00001 → JSON                  [Added]
│   ├─ batch_00002 → JSON                  [Added]
│   └─ batch_00003 → JSON                  [Added]
└─ report:merge_triggered:{job_id}         [Set when done]

Merge Processing (T+60s to T+90s)
│
├─ report:results:{job_id}                 [Deleted]
├─ report:results:{job_id}:data            [Created]
└─ report:results:{job_id}:filename        [Created]

Client Download (T+100s)
│
├─ report:results:{job_id}:data            [Retrieved]
└─ report:results:{job_id}:filename        [Retrieved]

After TTL (T+3600s)
│
├─ report:results:{job_id}:data            [Expired]
└─ report:results:{job_id}:filename        [Expired]
```

## Alternative: Streaming to Client

**Current:** Batch → Redis → Merge → Redis → Client
**Alternative:** Batch → Stream → Client (Real-time)

### Why Not Stream?

1. **Client must stay connected**
   - Long-running HTTP connection
   - Network issues cause retry complexity

2. **No retry for clients**
   - If client disconnects, must restart
   - Can't download again later

3. **Harder to scale**
   - Each client ties up a worker thread
   - Load balancer complexity

4. **Merge still needed**
   - Excel generation requires all data
   - Can't incrementally build Excel file

### When Streaming Makes Sense

- **Preview/Progressive Display**
  - Show data as it arrives
  - Don't need final Excel yet
  - UI can render JSON chunks

- **Very Large Reports**
  - Excel won't fit in memory
  - Stream CSV line-by-line
  - Client writes to disk

## Current Pub/Sub (Optional Feature)

**Note:** System already has Pub/Sub for real-time updates!

```rust
// After storing batch in Redis
publisher.publish_chunk(job.parent_job_id, batch_index, total_batches, chunk_data).await?;
```

**Used for:**
- WebSocket notifications to UI
- Progress updates
- Real-time preview (if UI wants)

**Not used for:**
- Final download (uses Redis GET)
- Primary data delivery

## Recommendations

### Current Architecture: ✅ GOOD

**Reasons:**
1. Works reliably in production
2. Handles retries gracefully
3. Scales horizontally
4. Simple client implementation

### Future Enhancements

1. **Optional S3 Storage**
   ```
   If file > 10MB:
     - Store in S3
     - Return presigned URL
     - Longer retention (7 days)
   ```

2. **WebSocket Progress**
   ```
   Already have Pub/Sub
   Add WebSocket handler
   Stream progress to UI
   ```

3. **Incremental Excel**
   ```
   Research: Can Excel be built incrementally?
   If yes: Stream Excel chunks
   If no: Keep current approach
   ```

## Conclusion

**Current Architecture:** Batch → Redis → Merge → Redis → Client

**Why It Works:**
- ✅ Reliable (retryable at each stage)
- ✅ Scalable (parallel batch processing)
- ✅ Simple (clear separation of concerns)
- ✅ Production-proven (handling 704+ employee reports)

**Trade-offs Accepted:**
- ⚠️ Not real-time download (client polls status)
- ⚠️ Redis storage overhead (~100KB per job)
- ⚠️ 1-hour download window (acceptable for use case)

**Verdict:** Architecture is well-designed for current requirements. No changes needed unless moving to very large reports (10K+ employees) or requiring instant download.
