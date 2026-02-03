# Report Export Real-Time Update Architecture

**Date**: 2026-02-03 09:03
**Issue**: Export report modal does not update realtime
**Type**: Scout Report

---

## Executive Summary

The report export system has **dual update mechanisms** but the modal currently uses **polling only**. WebSocket streaming infrastructure exists but is **not integrated** into `ReportExportDialog`.

**Current State**: Polling-based (1s interval via React Query)
**Available**: WebSocket streaming (`useReportStream` hook + backend Pub/Sub)
**Gap**: Frontend modal not connected to WebSocket system

---

## Architecture Overview

### Frontend Components

**1. ReportExportDialog.tsx** ⚠️ (Current Implementation)
- **Path**: `webdashboard/src/components/features/reports/ReportExportDialog.tsx`
- **Update Method**: Polling via `useQuery` (1000ms refetch interval)
- **Endpoint**: `GET /workspaces/{wsId}/reports/jobs/{jobId}` (REST)
- **Phases**: idle → starting → polling → downloading → completed/failed
- **Features**: Progress bar, batch counter, retry logic, background processing
- **Problem**: Uses HTTP polling instead of WebSocket streaming

**2. useReportStream Hook** ✅ (Available but Unused)
- **Path**: `webdashboard/src/hooks/use-report-stream.ts`
- **Purpose**: WebSocket-based real-time streaming
- **Features**:
  - Automatic reconnection (exponential backoff: 1s, 2s, 4s)
  - Progress tracking (0-100%)
  - Chunk accumulation
  - Error handling
  - Terminal states (DONE/ERROR)
- **Status**: Built but **not integrated** into export dialog

**3. Stream Types**
- **Path**: `webdashboard/src/types/report-stream.ts`
- **Messages**: `ChunkMessage`, `DoneMessage`, `ErrorMessage`
- **States**: idle → connecting → streaming → completed/error
- **Row Types**: `SummaryRow`, `DetailRow`

**4. Excel Builder** (Client-Side Generation)
- **Path**: `webdashboard/src/lib/excel-builder.ts`
- **Library**: ExcelJS + FileSaver
- **Features**: Progress callbacks, streaming support
- **Note**: Can accept chunks from WebSocket stream

**5. API Layer**
- **Path**: `webdashboard/src/lib/api/attendance.ts:439-490`
- **Functions**:
  - `createReportExportJob()` - POST job creation
  - `fetchExportJobStatus()` - GET status (used by polling)
  - `downloadExportReport()` - GET completed file
  - `cancelExportJob()` - DELETE job

---

### Backend Architecture

**1. REST Endpoints** (Polling Interface)
- **Path**: `apiserver/src/handlers/report_export.rs` (400 lines)
- **Routes**:
  - `POST /api/workspaces/:ws_id/reports/export` - Create job (202 Accepted)
  - `GET /api/workspaces/:ws_id/reports/jobs/:job_id` - Poll status
  - `GET /api/workspaces/:ws_id/reports/jobs/:job_id/download` - Download Excel
  - `DELETE /api/workspaces/:ws_id/reports/jobs/:job_id` - Cancel job
- **Response**: `JobStatusResponse` with progress percentage, batch counts

**2. WebSocket Handler** ✅ (Real-Time Interface)
- **Path**: `apiserver/src/handlers/report_websocket.rs` (168 lines)
- **Route**: `GET /ws/reports/:job_id` (WebSocket upgrade)
- **Features**:
  - Redis Pub/Sub integration
  - 10-minute timeout
  - JSON chunk streaming
  - Auto-cleanup on disconnect
- **Status**: Implemented, tested, **ready to use**

**3. Redis Pub/Sub System**
- **Path**: `apiserver/src/queue/pubsub.rs`
- **Components**:
  - `ReportPublisher` - Publishes chunks to `report:stream:{job_id}`
  - `ReportSubscriber` - Subscribes to job channels
  - `StreamMessage` enum: CHUNK, DONE, ERROR
- **Messages**:
  - `CHUNK`: Batch data with progress (`batch_index`, `total_batches`)
  - `DONE`: Completion signal
  - `ERROR`: Failure notification

**4. Report Coordinator**
- **Path**: `apiserver/src/services/report_coordinator.rs` (478 lines)
- **Config**:
  - `BATCH_SIZE`: 200 employees/batch
  - `JOB_TIMEOUT`: 3600s (1 hour)
  - `ESTIMATED_SECONDS_PER_BATCH`: 2s
- **Functions**: Job splitting, progress tracking, cancellation

**5. Report Queue**
- **Path**: `apiserver/src/queue/report_queue.rs` (668 lines)
- **Queue Type**: Redis LIST (FIFO)
- **Operations**:
  - `enqueue_jobs()` - RPUSH atomic batch
  - `dequeue_job()` - BLPOP with 30s timeout (non-polling)
  - `get_result_data()` - Fetch completed results
- **Redis Keys**:
  - `report:jobs:*` - Job queue
  - `report:meta:*` - Job metadata
  - `report:results:*` - Result storage (24hr TTL)
  - `report:stream:*` - Pub/Sub channels

**6. Worker Handlers**
- **Path**: `apiserver/src/worker/handlers/attendance.rs`
- **Flow**:
  1. Process batch (calculate attendance)
  2. Store result in Redis
  3. **Publish chunk via Pub/Sub** (lines 147-150)
  4. Update progress counters
- **Path**: `apiserver/src/worker/handlers/merge.rs` (similar pattern)

**7. Worker Runner**
- **Path**: `apiserver/src/worker/runner.rs`
- **Config**:
  - `BLPOP_TIMEOUT`: 30s (blocking dequeue)
  - `MAX_CONCURRENT_JOBS`: 2
  - Worker heartbeat: 5s intervals, 15s TTL
- **Status**: Idle / Processing

---

## Data Flow Comparison

### Current (Polling) 🐌
```
User triggers export
    ↓
POST /reports/export (job_id received)
    ↓
React Query polling (1000ms interval)
    ↓
GET /reports/jobs/{job_id} (REST)
    ↓
Response: { status, progress, batch_index, total_batches }
    ↓
Update UI → Poll again (repeat every 1s)
    ↓
Status = "completed" → Download file
```

**Pros**: Simple, no WebSocket management
**Cons**: High latency, unnecessary API calls, delayed updates

### Available (WebSocket Streaming) ⚡
```
User triggers export
    ↓
POST /reports/export (job_id received)
    ↓
useReportStream(job_id) opens WebSocket
    ↓
GET /ws/reports/{job_id} (WebSocket upgrade)
    ↓
Subscribe to Redis Pub/Sub: report:stream:{job_id}
    ↓
Worker publishes CHUNK messages (per batch)
    ↓
Frontend receives real-time updates:
  - CHUNK: { batch_index, total_batches, data }
  - DONE: { total_rows }
  - ERROR: { message }
    ↓
Accumulate chunks → Build Excel client-side (optional)
    ↓
DONE message → Close WebSocket → Complete
```

**Pros**: Real-time, low latency, reduced API load, live progress
**Cons**: Requires WebSocket management, more complex state

---

## Root Cause Analysis

### Why Modal Doesn't Update Real-Time

**1. Implementation Gap**
- `ReportExportDialog` uses polling (`useQuery` with 1s refetch)
- `useReportStream` hook exists but **never imported/used** in dialog
- WebSocket infrastructure fully functional but **disconnected from UI**

**2. Missing Integration**
- Dialog at line 52-60: `useQuery` calls `fetchExportJobStatus()` in loop
- No WebSocket connection established
- No subscription to `report:stream:{job_id}`

**3. Architecture Mismatch**
- Backend publishes to Pub/Sub (lines in `attendance.rs:147-150`)
- Frontend ignores Pub/Sub channel
- Modal relies on HTTP endpoint that only reflects **last known state**

---

## File Locations

### Frontend (Need Changes)
```
webdashboard/src/components/features/reports/ReportExportDialog.tsx
webdashboard/src/hooks/use-report-stream.ts (unused)
webdashboard/src/types/report-stream.ts
webdashboard/src/lib/excel-builder.ts
webdashboard/src/lib/api/attendance.ts
```

### Backend (Already Working)
```
apiserver/src/handlers/report_websocket.rs
apiserver/src/handlers/report_export.rs
apiserver/src/queue/pubsub.rs
apiserver/src/queue/report_queue.rs
apiserver/src/services/report_coordinator.rs
apiserver/src/worker/handlers/attendance.rs
apiserver/src/worker/handlers/merge.rs
apiserver/src/worker/runner.rs
```

### Tests
```
apiserver/tests/report_streaming_tests.rs (backend)
webdashboard/tests/report-export-stream.spec.ts (E2E)
webdashboard/src/lib/__tests__/excel-builder.test.ts (unit)
```

---

## Recommended Solution

### Option A: Replace Polling with WebSocket (Preferred)
**Changes Required**:
1. Import `useReportStream` in `ReportExportDialog.tsx`
2. Replace `useQuery` polling logic with WebSocket hook
3. Map `StreamMessage` types to existing `ExportPhase` states
4. Handle CHUNK messages for progress updates
5. Process DONE message for completion

**Pros**: Real-time, efficient, uses existing infrastructure
**Cons**: Requires state refactoring

### Option B: Hybrid Approach
**Changes Required**:
1. Keep initial status check via REST
2. Open WebSocket after job starts
3. Switch to streaming during processing phase
4. Fallback to polling if WebSocket fails

**Pros**: Graceful degradation
**Cons**: More complex logic

### Option C: Server-Sent Events (SSE)
**Changes Required**:
1. Add SSE endpoint (`/reports/jobs/{job_id}/stream`)
2. Frontend uses EventSource API
3. Simpler than WebSocket for one-way streaming

**Pros**: Built-in reconnection, simpler than WebSocket
**Cons**: Requires new backend endpoint

---

## Configuration Constants

**Backend**:
- `BATCH_SIZE`: 200 employees
- `JOB_TIMEOUT`: 3600s
- `ESTIMATED_SECONDS_PER_BATCH`: 2s
- `WEBSOCKET_TIMEOUT`: 600s (10 min)
- `BLPOP_TIMEOUT`: 30s

**Frontend**:
- Polling interval: 1000ms
- Retry delays: 1s, 2s, 4s (exponential backoff)
- Max retries: 3

---

## Testing Infrastructure

**Backend Tests**:
- `apiserver/tests/websocket_tests.rs` - WebSocket connection tests
- `apiserver/tests/report_streaming_tests.rs` - Streaming logic tests

**Frontend Tests**:
- `webdashboard/tests/report-export-stream.spec.ts` - E2E streaming tests
- `webdashboard/src/lib/__tests__/excel-builder.test.ts` - Excel generation

---

## Unresolved Questions

1. **Migration Strategy**: Phased rollout or immediate switch?
2. **Fallback Behavior**: What happens if WebSocket connection fails?
3. **Browser Support**: Any concerns with older browsers?
4. **Mobile Support**: WebSocket performance on mobile networks?
5. **Excel Download**: Use streaming chunks or wait for backend merge?
6. **User Preference**: Should users choose polling vs streaming?
7. **Error Recovery**: How to handle mid-stream disconnections?
8. **Testing Coverage**: Need E2E tests for WebSocket failure scenarios?

---

## Next Steps

1. **Immediate**: Review `ReportExportDialog.tsx` polling logic
2. **Plan**: Design WebSocket integration approach
3. **Implement**: Replace polling with `useReportStream`
4. **Test**: Validate real-time updates in dev environment
5. **Deploy**: Rollout with feature flag (optional)
6. **Monitor**: Track WebSocket connection stability
