# Phase 3: Add Fallback Mechanism

**Status**: pending
**Priority**: P2
**Effort**: 1.5h

---

## Context

Add graceful fallback to HTTP polling if WebSocket connection fails after max retries.

---

## Requirements

### Functional
- Detect WebSocket max retry exhaustion
- Switch to polling mode automatically
- Display fallback notification to user
- Ensure seamless UX transition

### Non-Functional
- No manual user intervention required
- Preserve all data and progress
- Log fallback events for monitoring

---

## Architecture

```
Start Export
    ↓
Try WebSocket (3 retries)
    ↓
Success? → Stream updates
    ↓
Failure? → Fallback to polling
    ↓
Poll every 2s (less aggressive)
    ↓
Complete normally
```

---

## Related Files

**Modify**:
- `webdashboard/src/components/features/reports/ReportExportDialog.tsx`

**Keep as-is**:
- `webdashboard/src/hooks/use-report-stream.ts` (already has retry logic)
- `webdashboard/src/lib/api/attendance.ts` (polling API exists)

---

## Implementation Steps

### 1. Add Fallback State

**Location**: After existing state declarations (line ~39)

```typescript
const [useFallbackPolling, setUseFallbackPolling] = useState(false);
```

### 2. Detect WebSocket Failure

**Modify** `useReportStream` error callback:

```typescript
// REPLACE onError callback (line ~60):
onError: (message) => {
  console.error("[ReportExport] Stream error:", message);

  // Check if max retries exceeded
  if (message.includes("Max retries exceeded") ||
      message.includes("Connection lost")) {
    console.warn("[ReportExport] WebSocket failed, falling back to polling");
    setUseFallbackPolling(true);
    setError(null); // Clear error, fallback will handle
  } else {
    setPhase("failed");
    setError(message);
  }
},
```

### 3. Add Polling Fallback

**Location**: After `useReportStream` hook

```typescript
// Fallback polling when WebSocket fails
const { data: pollingStatus } = useQuery({
  queryKey: ["export-job-fallback", jobId],
  queryFn: () => fetchExportJobStatus(wsId, jobId!),
  enabled: !!jobId && useFallbackPolling && phase === "polling",
  refetchInterval: (query) => {
    const status = query.state.data?.status;
    return status === "processing" || status === "pending" ? 2000 : false; // 2s (less aggressive)
  },
});
```

### 4. Handle Polling Status

**Location**: After fallback polling query

```typescript
// Handle fallback polling status
useEffect(() => {
  if (!useFallbackPolling || !pollingStatus) return;

  if (pollingStatus.status === "completed") {
    setPhase("completed");
  } else if (pollingStatus.status === "failed") {
    setPhase("failed");
    setError(pollingStatus.error || t("report_export.status.failed"));
  }
}, [pollingStatus, useFallbackPolling, t]);
```

### 5. Update Progress from Polling

**Modify** percentage calculation (line ~117):

```typescript
// Calculate percentage from appropriate source
const percentage = jobId && phase === "polling"
  ? (useFallbackPolling && pollingStatus?.progress?.percentage
      ? pollingStatus.progress.percentage
      : streamProgress)
  : 0;

// Calculate batch count
const completedBatches = useFallbackPolling && pollingStatus?.progress
  ? pollingStatus.progress.completed_batches
  : (totalBatches > 0 ? Math.floor((streamProgress / 100) * totalBatches) : 0);
```

### 6. Add Fallback Notification

**Location**: After error message div (line ~205)

```typescript
{/* Fallback Mode Notification */}
{useFallbackPolling && phase === "polling" && (
  <div className="bg-amber-50 border border-amber-200 rounded-xl p-3">
    <p className="text-xs text-amber-700">
      ⚠️ {t("report_export.fallback_mode")}
    </p>
  </div>
)}
```

### 7. Add Translation Keys

**File**: `webdashboard/src/i18n/locales/en.json`

```json
{
  "report_export": {
    "fallback_mode": "Real-time updates unavailable. Using standard polling."
  }
}
```

**File**: `webdashboard/src/i18n/locales/vi.json`

```json
{
  "report_export": {
    "fallback_mode": "Cập nhật thời gian thực không khả dụng. Đang sử dụng polling."
  }
}
```

**File**: `webdashboard/src/i18n/locales/zh.json`

```json
{
  "report_export": {
    "fallback_mode": "实时更新不可用。使用标准轮询。"
  }
}
```

### 8. Reset Fallback on New Job

**Modify** reset effect (line ~41-49):

```typescript
useEffect(() => {
  if (isOpen) {
    setPhase("idle");
    setJobId(null);
    setError(null);
    setTotalBatches(0);
    setUseFallbackPolling(false); // Reset fallback state
  }
}, [isOpen]);
```

---

## Fallback Decision Logic

```typescript
// WebSocket attempt flow:
1. Start export → open WebSocket
2. Connection fails → retry (1s, 2s, 4s backoff)
3. Max retries (3) exceeded → trigger fallback
4. Switch to polling (2s interval)
5. Continue until completion

// Fallback triggered when:
- WebSocket error: "Max retries exceeded"
- WebSocket error: "Connection lost"
- WebSocket closed abnormally (code !== 1000)
```

---

## Todo List

- [ ] Add `useFallbackPolling` state
- [ ] Modify `onError` callback to detect max retries
- [ ] Add fallback polling query (2s interval)
- [ ] Add polling status effect
- [ ] Update progress calculation for fallback
- [ ] Add fallback notification UI
- [ ] Add translation keys (en, vi, zh)
- [ ] Reset fallback state on dialog reopen
- [ ] Test fallback scenario manually
- [ ] Log fallback event for monitoring

---

## Success Criteria

✅ WebSocket failure triggers fallback automatically
✅ Polling takes over seamlessly
✅ Progress continues updating (slower)
✅ User sees fallback notification
✅ No data loss during transition
✅ Completion/error states work in fallback mode

---

## Testing Scenarios

### Manual Testing

**Scenario 1: WebSocket Failure**
1. Block WebSocket port (browser DevTools → Network → Block URL pattern `/ws/`)
2. Start export
3. Verify fallback notification appears
4. Verify polling starts (check Network tab for REST calls)
5. Verify progress updates every 2s
6. Verify completion works

**Scenario 2: Network Interruption**
1. Start export via WebSocket
2. Disable network mid-stream
3. Wait for max retries (1s + 2s + 4s = 7s)
4. Re-enable network
5. Verify fallback polling resumes

**Scenario 3: Normal Flow**
1. Start export with working WebSocket
2. Verify no fallback notification
3. Verify real-time updates
4. Verify completion

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Fallback not triggered | High | Explicit error message checks |
| Dual updates (WS + poll) | Medium | Disable polling when WS active |
| Progress out of sync | Low | Use conditional source selection |
| Translation missing | Low | Add all locale keys |

---

## Monitoring

**Log Events**:
```typescript
console.warn("[ReportExport] WebSocket failed, falling back to polling");
console.info("[ReportExport] Fallback polling status:", pollingStatus);
```

**Future Enhancement**:
- Add telemetry event: `export.fallback.triggered`
- Track fallback frequency in monitoring dashboard

---

## Security Considerations

- Polling uses same auth as WebSocket (session cookies)
- Fallback doesn't bypass permissions
- Error messages sanitized (no sensitive data leaked)

---

## Next Steps

→ Phase 4: Add data-testid attributes
