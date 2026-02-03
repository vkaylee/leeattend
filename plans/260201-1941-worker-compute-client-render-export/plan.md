# Export System Refactor: Worker Compute - Client Render

**Created**: 2026-02-01 19:41
**Status**: Planning
**Branch**: main

---

## Overview

Refactor export system from "Store-and-Forward" to "Streaming Data & Client-Side Generation":
- **Current**: Worker → JSON → Redis → Merge → Excel → S3 → Client Download
- **Target**: Worker → JSON Chunks → Redis Pub/Sub → WebSocket → Client → ExcelJS Build

---

## Phase Summary

| Phase | Title | Status | Priority |
|-------|-------|--------|----------|
| 1 | [Backend Optimization](./phase-01-backend-optimization.md) | Pending | P0 |
| 2 | [Redis Pub/Sub Infrastructure](./phase-02-redis-pubsub.md) | Pending | P0 |
| 3 | [WebSocket Endpoint](./phase-03-websocket-endpoint.md) | Pending | P0 |
| 4 | [Frontend Streaming Hook](./phase-04-frontend-streaming.md) | Complete | P0 |
| 5 | [Client-Side Excel Generation](./phase-05-client-excel.md) | Complete | P0 |
| 6 | [Integration & Testing](./phase-06-integration-testing.md) | Complete | P1 |

---

## Architecture Change

```
┌──────────────────────────────────────────────────────────────┐
│ NEW: Worker Compute - Client Render                          │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Client                                                      │
│    │                                                         │
│    ├── 1. POST /reports/export → job_id                     │
│    │                                                         │
│    ├── 2. Connect WS /ws/reports/{job_id}                   │
│    │                                                         │
│    │   ┌─────────────────────────────────────────────┐      │
│    │   │ Worker (per batch)                           │      │
│    │   │  - Query DB (200 employees)                  │      │
│    │   │  - Calculate attendance                      │      │
│    │   │  - PUBLISH report:stream:{job_id}           │      │
│    │   └─────────────┬───────────────────────────────┘      │
│    │                 │                                       │
│    │   ┌─────────────▼───────────────────────────────┐      │
│    │   │ Redis Pub/Sub                                │      │
│    │   │  Channel: report:stream:{job_id}            │      │
│    │   └─────────────┬───────────────────────────────┘      │
│    │                 │                                       │
│    │   ┌─────────────▼───────────────────────────────┐      │
│    │   │ WebSocket Handler                            │      │
│    │   │  - Subscribe channel                         │      │
│    │   │  - Forward to client                         │      │
│    │   └─────────────┬───────────────────────────────┘      │
│    │                 │                                       │
│    ◀─────────────────┘ {type: CHUNK, data: [...], progress} │
│    │                                                         │
│    ├── 3. Accumulate rows[]                                 │
│    │                                                         │
│    ├── 4. On DONE → ExcelJS.writeBuffer()                   │
│    │                                                         │
│    └── 5. Trigger download (.xlsx)                          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Key Files (Modified/Created)

### Backend
- `apiserver/src/services/report_coordinator.rs` - BATCH_SIZE → 200
- `apiserver/src/domain/policy/resolver.rs` - Batch resolver (NEW)
- `apiserver/src/worker/handlers/attendance.rs` - Pub/Sub publish
- `apiserver/src/handlers/report_websocket.rs` - WS endpoint (NEW)
- `apiserver/src/queue/pubsub.rs` - Pub/Sub helpers (NEW)

### Frontend
- `webdashboard/src/hooks/use-report-stream.ts` - WS hook (NEW)
- `webdashboard/src/lib/excel-builder.ts` - ExcelJS wrapper (NEW)
- `webdashboard/src/components/features/reports/ReportExportDialog.tsx` - Streaming UI

---

## Dependencies

- **Backend**: No new crates (redis already has Pub/Sub)
- **Frontend**: `exceljs`, `file-saver` (NEW)

---

## Success Criteria

1. BATCH_SIZE = 200 employees per job (10x improvement)
2. WebSocket delivers chunks in real-time (< 500ms latency)
3. Client builds Excel locally (no S3 upload for streaming mode)
4. Progress bar shows real percentage (not just spinner)
5. Fallback to polling if WebSocket fails
6. All existing tests pass

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| WebSocket disconnect | Medium | Fallback to polling mode |
| Browser memory (large files) | High | Chunk size limit, streaming writes |
| Redis Pub/Sub scaling | Low | Separate subscriber per WS connection |
| ExcelJS bundle size | Medium | Dynamic import on export trigger |

---

## Constraints

- Performance: < 100ms per 200 records batch
- Scalability: Redis Pub/Sub for multi-container
- Cleanup: Worker sends DONE signal to close connections
