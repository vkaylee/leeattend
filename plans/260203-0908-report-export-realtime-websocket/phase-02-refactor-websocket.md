# Phase 2: Refactor to WebSocket

**Status**: pending
**Priority**: P2
**Effort**: 2h

---

## Context

Replace HTTP polling with WebSocket streaming using existing `useReportStream` hook.

---

## Requirements

### Functional
- Import and use `useReportStream` hook
- Map `StreamStatus` to `ExportPhase`
- Update progress display from WebSocket messages
- Remove polling logic completely

### Non-Functional
- Preserve all existing UI components
- Maintain state transitions
- Keep error handling behavior

---

## Related Files

**Modify**:
- `webdashboard/src/components/features/reports/ReportExportDialog.tsx`

**Reference**:
- `webdashboard/src/hooks/use-report-stream.ts`
- `webdashboard/src/types/report-stream.ts`

---

## Implementation Steps

### 1. Add Import

**Location**: `ReportExportDialog.tsx` line 1-11

```typescript
import { useReportStream } from "../../../hooks/use-report-stream";
```

### 2. Remove Polling Logic

**Remove** lines 52-60:
```typescript
// DELETE THIS:
const { data: jobStatus } = useQuery({
  queryKey: ["export-job", jobId],
  queryFn: () => fetchExportJobStatus(wsId, jobId!),
  enabled: !!jobId && phase === "polling",
  refetchInterval: (query) => {
    const status = query.state.data?.status;
    return status === "processing" || status === "pending" ? 1000 : false;
  },
});
```

**Remove** lines 63-73 (status effect):
```typescript
// DELETE THIS:
useEffect(() => {
  if (!jobStatus) return;

  if (jobStatus.status === "completed") {
    setPhase("completed");
  } else if (jobStatus.status === "failed") {
    setPhase("failed");
    setError(jobStatus.error || t("report_export.status.failed"));
  }
}, [jobStatus, t]);
```

### 3. Add WebSocket Hook

**Location**: After line 48 (after reset effect)

```typescript
// WebSocket streaming
const {
  status: streamStatus,
  progress: streamProgress,
  error: streamError,
  reconnect,
} = useReportStream({
  jobId,
  onComplete: () => {
    console.debug("[ReportExport] Stream completed");
    setPhase("completed");
  },
  onError: (message) => {
    console.error("[ReportExport] Stream error:", message);
    setPhase("failed");
    setError(message);
  },
  maxRetries: 3,
});
```

### 4. Map Stream Status to Phase

**Location**: After WebSocket hook

```typescript
// Map WebSocket status to export phase
useEffect(() => {
  if (!jobId) return;

  // Only update phase during streaming states
  if (streamStatus === "connecting") {
    setPhase("starting");
  } else if (streamStatus === "streaming") {
    setPhase("polling"); // Keep "polling" name for UI consistency
  } else if (streamStatus === "error" && streamError) {
    setPhase("failed");
    setError(streamError);
  }
  // Note: "completed" handled by onComplete callback
}, [streamStatus, streamError, jobId]);
```

### 5. Update Progress Calculation

**Replace** lines 117-118:
```typescript
// OLD:
const progress = jobStatus?.progress;
const percentage = progress?.percentage ?? 0;

// NEW:
const percentage = jobId && phase === "polling" ? streamProgress : 0;
```

### 6. Calculate Batch Count

**Add** after percentage calculation:

```typescript
// Calculate batch count from progress and total
const completedBatches = totalBatches > 0
  ? Math.floor((streamProgress / 100) * totalBatches)
  : 0;
```

### 7. Update Batch Display

**Replace** lines 191-198:
```typescript
// OLD:
{phase === "polling" && progress && totalBatches > 0 && (
  <p className="text-center text-sm text-slate-500">
    {t("report_export.batches_completed", {
      completed: progress.completed_batches,
      total: totalBatches,
    })}
  </p>
)}

// NEW:
{phase === "polling" && totalBatches > 0 && (
  <p className="text-center text-sm text-slate-500">
    {t("report_export.batches_completed", {
      completed: completedBatches,
      total: totalBatches,
    })}
  </p>
)}
```

### 8. Update handleStartExport

Keep existing logic, only ensure `jobId` triggers WebSocket connection:

```typescript
// No changes needed - setting jobId via setJobId(response.job_id)
// automatically triggers useReportStream connection via useEffect
```

---

## State Flow

### Before Refactor
```
idle → starting (POST) → polling (useQuery loop) → completed/failed
```

### After Refactor
```
idle → starting (POST + WS connect) → polling (WS streaming) → completed/failed
```

**Phase names unchanged for UI compatibility**

---

## Todo List

- [ ] Import `useReportStream` hook
- [ ] Remove `useQuery` polling logic
- [ ] Remove old status effect
- [ ] Add WebSocket hook with callbacks
- [ ] Add status mapping effect
- [ ] Update progress calculation
- [ ] Add batch count calculation
- [ ] Update batch counter display
- [ ] Test compilation
- [ ] Manual smoke test (dev environment)

---

## Success Criteria

✅ No TypeScript errors
✅ Dialog compiles successfully
✅ No polling logic remains
✅ WebSocket hook integrated
✅ Progress updates in real-time (visual test)
✅ Batch counter updates live (visual test)

---

## Testing Notes

**Manual Testing**:
1. Open report export dialog
2. Start export job
3. Observe progress bar (should update smoothly without 1s delay)
4. Check batch counter (should increment per chunk)
5. Verify completion state
6. Test error scenario (invalid date range)

**Expected Behavior**:
- Progress bar updates < 100ms after worker publishes
- No 1s polling delay visible
- Batch counter increments per CHUNK message

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Breaking existing UI | Preserve all phase names and UI components |
| Missing progress data | Use hook's `progress` directly (0-100) |
| Batch count inaccurate | Calculate from percentage * totalBatches |
| State sync issues | Use effects to map status → phase |

---

## Security Considerations

- WebSocket URL auto-uses wss: for HTTPS (line 29-32 in hook)
- No auth token needed (session-based via cookies)

---

## Next Steps

→ Phase 3: Add fallback mechanism
