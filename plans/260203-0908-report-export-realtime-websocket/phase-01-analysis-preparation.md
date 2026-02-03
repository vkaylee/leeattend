# Phase 1: Analysis & Preparation

**Status**: pending
**Priority**: P2
**Effort**: 1h

---

## Context

Read and analyze existing code to understand:
- Current polling implementation details
- WebSocket hook API and behavior
- State management flow in dialog
- Message type mappings

---

## Requirements

### Functional
- Understand `ExportPhase` enum states
- Map `StreamStatus` to `ExportPhase`
- Identify state variables used in UI rendering

### Non-Functional
- Zero disruption to existing UI/UX
- Maintain backward compatibility

---

## Files to Analyze

```
webdashboard/src/components/features/reports/ReportExportDialog.tsx (286 lines)
  - Lines 23-24: ExportPhase enum
  - Lines 35-49: State management
  - Lines 52-60: useQuery polling logic
  - Lines 63-73: Job status effect
  - Lines 117-118: Progress calculation

webdashboard/src/hooks/use-report-stream.ts (256 lines)
  - Lines 10-23: Hook interface
  - Lines 47-52: Hook options
  - Lines 53-70: Hook return type
  - Lines 75-127: Message handlers (CHUNK, DONE, ERROR)
  - Lines 132-201: WebSocket connection logic

webdashboard/src/types/report-stream.ts
  - StreamMessage types (CHUNK, DONE, ERROR)
  - StreamStatus enum (idle, connecting, streaming, completed, error)
```

---

## Implementation Steps

### 1. Read Current Dialog Implementation
```bash
# Already read in context
# Key findings:
# - useQuery polling at line 52-60 (1000ms interval)
# - Phase states: idle → starting → polling → downloading → completed/failed
# - Progress from jobStatus.progress.percentage
# - Batch count from jobStatus.progress.completed_batches
```

### 2. Analyze State Mapping

**ExportPhase → StreamStatus mapping**:
```typescript
// Current (polling)
type ExportPhase = "idle" | "starting" | "polling" | "downloading" | "completed" | "failed";

// WebSocket hook
type StreamStatus = "idle" | "connecting" | "streaming" | "completed" | "error";

// Mapping
idle → idle
starting → connecting
polling → streaming
completed → completed
failed → error
downloading → (keep as-is, not part of streaming)
```

### 3. Identify State Dependencies

**Current state variables**:
- `phase: ExportPhase` - UI rendering
- `jobId: string | null` - Job identifier
- `error: string | null` - Error messages
- `totalBatches: number` - Total batch count

**WebSocket hook provides**:
- `status: StreamStatus` - Maps to phase
- `progress: number` - 0-100 percentage
- `chunks: unknown[]` - Accumulated data
- `error: string | null` - Error messages
- `reconnect: () => void` - Manual reconnect

**Integration points**:
- Replace `useQuery` with `useReportStream`
- Map `status` → `phase` in effect
- Use `progress` directly (no calculation needed)
- Derive `completedBatches` from `totalBatchesRef` in hook

### 4. Identify UI Dependencies

**Progress bar** (line 152-162):
- Uses `percentage` from `jobStatus.progress.percentage`
- Will use `progress` from hook directly

**Batch counter** (line 191-198):
- Uses `progress.completed_batches` and `totalBatches`
- Need to calculate from hook's `progress` percentage

**Status text** (line 182-186):
- Uses `phase` enum
- Will map `status` to `phase`

### 5. Document Integration Points

**Add to dialog**:
```typescript
import { useReportStream } from "@/hooks/use-report-stream";

const { status, progress, error: streamError, reconnect } = useReportStream({
  jobId,
  onComplete: () => setPhase("completed"),
  onError: (msg) => {
    setPhase("failed");
    setError(msg);
  },
  maxRetries: 3,
});
```

**State effect**:
```typescript
useEffect(() => {
  if (status === "connecting") setPhase("starting");
  if (status === "streaming") setPhase("polling");
  if (status === "completed") setPhase("completed");
  if (status === "error") setPhase("failed");
}, [status]);
```

---

## Todo List

- [x] Read scout report
- [x] Read dialog implementation
- [x] Read WebSocket hook
- [ ] Document state mapping
- [ ] Document integration points
- [ ] Create refactor checklist
- [ ] Identify testing scenarios

---

## Success Criteria

✅ Full understanding of current polling flow
✅ Clear state mapping documented
✅ Integration points identified
✅ No missing edge cases

---

## Security Considerations

- WebSocket URL construction (protocol detection)
- Token/auth handling (check if needed)
- Error message sanitization

---

## Next Steps

→ Phase 2: Implement WebSocket integration
