# Debug Report: Export Modal Stuck on "Preparing" Status

**Date:** 2026-02-03 09:56
**Agent:** debugger (a0bd5d8)
**Issue:** Report export modal stuck showing "Đang chuẩn bị..." (Preparing) indefinitely

---

## Executive Summary

**Root Cause:** Race condition between manual phase update in `handleStartExport` and automatic phase mapping from WebSocket status in `useEffect`.

**Business Impact:** Users cannot export reports - modal appears frozen at "Preparing" stage, preventing downloads.

**Fix Priority:** HIGH - Core export functionality broken

---

## Technical Analysis

### Timeline of Events

1. **User clicks "Start Export"** (ReportExportDialog.tsx:123)
   - `handleStartExport()` called
   - Sets `phase = "starting"`

2. **Job created successfully** (line 128-136)
   - API returns `job_id`
   - Sets `jobId` state
   - **Manually sets** `phase = "polling"` (line 137)

3. **WebSocket hook activates** (use-report-stream.ts:216)
   - useEffect detects `jobId` changed
   - Calls `connect()`
   - Sets internal `status = "connecting"` (line 144)

4. **Race condition occurs** (ReportExportDialog.tsx:85-98)
   - useEffect triggers on `streamStatus` change
   - Sees `streamStatus === "connecting"`
   - **Overwrites** `phase = "starting"` (line 90)

5. **Modal stuck:**
   - Phase is "starting" → displays "Đang chuẩn bị..."
   - Waiting for `streamStatus` to become "streaming"
   - If WebSocket connection delayed/fails, phase never updates

---

## Root Cause Identification

### Problem Code

**File:** `src/components/features/reports/ReportExportDialog.tsx`

**Lines 123-142:**
```typescript
const handleStartExport = async () => {
  setPhase("starting");  // ← Step 1
  setError(null);

  try {
    const response = await createReportExportJob(wsId, {
      start_date: startDate,
      end_date: endDate,
      report_type: reportType,
      employee_ids: employeeIds,
    });

    setJobId(response.job_id);        // ← Step 2: triggers useEffect
    setTotalBatches(response.total_batches);
    setPhase("polling");              // ← Step 3: sets polling BEFORE WebSocket connects
  } catch (err) {
    setPhase("failed");
    setError(err instanceof Error ? err.message : t("report_export.status.failed"));
  }
};
```

**Lines 85-98:**
```typescript
// Map WebSocket status to export phase
useEffect(() => {
  if (!jobId || useFallbackPolling) return;

  // Only update phase during streaming states
  if (streamStatus === "connecting") {
    setPhase("starting");  // ← Step 4: OVERWRITES to "starting"!
  } else if (streamStatus === "streaming") {
    setPhase("polling");   // ← Never reaches here if WebSocket delays
  } else if (streamStatus === "error" && streamError) {
    setPhase("failed");
    setError(streamError);
  }
  // Note: "completed" handled by onComplete callback
}, [streamStatus, streamError, jobId, useFallbackPolling]);
```

### Why It Fails

1. **Manual phase management conflicts with automatic mapping**
   - `handleStartExport` manually sets `phase = "polling"`
   - useEffect immediately overwrites to `phase = "starting"`

2. **WebSocket connection delay**
   - `useReportStream` sets status to "connecting"
   - Takes time to establish WebSocket connection
   - During this time, phase stays "starting"
   - If connection fails silently, phase never advances

3. **Translation mapping:**
   - `phase === "starting"` → displays "Đang chuẩn bị..." (vi.json:997)
   - `phase === "polling"` → displays "Đang tạo báo cáo..." (vi.json:998)

---

## Evidence from Logs

### Browser Console (Expected):
```
[ReportExport] Stream completed
[ReportStream] Connecting to ws://localhost:3000/api/ws/reports/{jobId}
[ReportStream] Connected
[ReportStream] Received chunk 1/5
```

### Actual Behavior:
- No "Connected" message appears
- Modal stuck at "Đang chuẩn bị..."
- Progress bar at 0%
- No chunks received

### Backend WebSocket Handler:
**File:** `apiserver/src/handlers/report_websocket.rs`

Potential issues:
- Line 39-46: Returns 503 if Redis unavailable (silent fail)
- Line 63-74: Subscription failure sends ERROR message but may not be received
- No initial "CONNECTED" message sent to client

---

## Proposed Fix

### Solution 1: Remove Manual Phase Update (RECOMMENDED)

**File:** `src/components/features/reports/ReportExportDialog.tsx`

**Change line 137:**
```typescript
const handleStartExport = async () => {
  setPhase("starting");
  setError(null);

  try {
    const response = await createReportExportJob(wsId, {
      start_date: startDate,
      end_date: endDate,
      report_type: reportType,
      employee_ids: employeeIds,
    });

    setJobId(response.job_id);
    setTotalBatches(response.total_batches);
    // ❌ REMOVE: setPhase("polling");  ← Let useEffect handle phase
  } catch (err) {
    setPhase("failed");
    setError(err instanceof Error ? err.message : t("report_export.status.failed"));
  }
};
```

**Why this works:**
- Single source of truth for phase management (useEffect)
- WebSocket status naturally drives phase transitions
- No race conditions

### Solution 2: Add Immediate Phase Transition

**Alternative approach:**

```typescript
// Update useEffect to handle jobId creation
useEffect(() => {
  if (!jobId || useFallbackPolling) return;

  // Immediately transition to polling when job is created
  if (streamStatus === "idle" || streamStatus === "connecting") {
    setPhase("polling");  // ← Show "processing" immediately
  } else if (streamStatus === "streaming") {
    setPhase("polling");
  } else if (streamStatus === "error" && streamError) {
    setPhase("failed");
    setError(streamError);
  }
}, [streamStatus, streamError, jobId, useFallbackPolling]);
```

**Tradeoff:** Less precise status but better UX

---

## Validation Steps

### 1. Test WebSocket Connection
```bash
# Check if WebSocket endpoint is accessible
wscat -c ws://localhost:3000/api/ws/reports/test-uuid
```

### 2. Check Redis Availability
```bash
redis-cli ping
# Expected: PONG
```

### 3. Test Export Flow
1. Open DevTools → Network → WS tab
2. Click "Start Export" in modal
3. Verify:
   - WebSocket connection established (status 101)
   - "Connected" log appears
   - Messages received
   - Phase transitions: starting → polling → completed

### 4. Add Debug Logging
```typescript
useEffect(() => {
  console.log('[DEBUG] Phase useEffect triggered:', {
    jobId,
    streamStatus,
    currentPhase: phase,
    useFallbackPolling
  });

  if (!jobId || useFallbackPolling) return;
  // ... rest of logic
}, [streamStatus, streamError, jobId, useFallbackPolling]);
```

---

## Backend Considerations

### WebSocket Handler Issues

**File:** `apiserver/src/handlers/report_websocket.rs`

**Potential improvements:**

1. **Send immediate CONNECTED message:**
```rust
// After line 77: "Client connected to report stream"
// Send initial message to confirm connection
let connected_msg = serde_json::json!({
    "type": "CONNECTED",
    "job_id": job_id.to_string()
});
let _ = sender.send(Message::Text(connected_msg.to_string())).await;
```

2. **Better error handling:**
- Line 41: Log Redis unavailable error
- Line 66: Ensure ERROR message reaches client before closing

3. **Connection timeout:**
- Add timeout for initial subscription (currently waits indefinitely)

---

## Risk Assessment

### High Risk
- Users cannot export reports (critical feature)
- Silent failure mode (no clear error message)
- May appear as system freeze

### Medium Risk
- WebSocket connection issues not visible to user
- Fallback polling not activating properly

### Low Risk
- Only affects new WebSocket streaming implementation
- Old polling system available as fallback

---

## Security Considerations

None - purely UI state management issue.

---

## Next Steps

1. **Immediate (within 1 hour):**
   - Remove manual `setPhase("polling")` at line 137
   - Test export flow end-to-end
   - Verify fallback polling still works

2. **Short-term (today):**
   - Add debug logging to trace phase transitions
   - Test with slow network conditions
   - Verify backend WebSocket endpoint health

3. **Long-term (next sprint):**
   - Add CONNECTED message to WebSocket protocol
   - Implement connection timeout with visible error
   - Add retry mechanism for WebSocket failures

---

## Unresolved Questions

1. **Is Redis Pub/Sub working correctly?**
   - Need to verify messages are being published to channel
   - Check Redis logs for subscription activity

2. **Does WebSocket handshake complete?**
   - Check nginx/proxy configuration for WebSocket support
   - Verify upgrade headers are passed through

3. **Why no fallback trigger?**
   - Fallback requires ERROR message from `onError` callback
   - Silent connection failures won't trigger fallback

4. **Backend job processing:**
   - Is job actually being processed by worker?
   - Are chunks being published to Redis channel?
   - Check worker logs for activity

---

**Recommended Immediate Action:** Remove line 137 `setPhase("polling")` and test. This is minimal change with high confidence of fixing the issue.
