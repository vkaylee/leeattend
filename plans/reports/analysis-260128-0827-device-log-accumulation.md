# Device Log Accumulation Analysis

**Date:** 2026-01-28 08:27
**Status:** Issue Identified - Solution Needed

## Problem

Device repeatedly reports same 10,460 logs because they're never cleared from device memory:

```
[2026-01-28T01:25:25Z] [OFFICE] Fetched 10460 logs
[2026-01-28T01:25:51Z] [OFFICE] Fetched 10460 logs
[2026-01-28T01:26:17Z] [OFFICE] Fetched 10460 logs
```

## Current Flow

1. ✅ Sidecar fetches 10,460 logs from ZKTeco device
2. ✅ Logs sent to agent → saved to SQLite (is_synced=0)
3. ✅ Sync loop fetches 500 logs → marks in-flight (is_synced=2) → sends to central
4. ❓ Central server sends ACK back
5. ❓ Agent marks as synced (is_synced=1)
6. ❌ **Device logs never cleared** → same logs fetched repeatedly

## Root Cause

No mechanism to clear device logs after successful sync. The `pyzk2` library provides `clear_attendance()` method but agent never calls it.

## Available Methods

**pyzk2 library:**
- `conn.get_attendance()` - Get all attendance logs ✅ Currently used
- `conn.clear_attendance()` - Clear all attendance logs ⚠️ NOT used

**ZK Worker current methods:**
```python
- device.connect
- device.get_logs       # Calls get_attendance()
- device.get_time
- device.sync_time
```

**Missing method:**
```python
- device.clear_logs     # Should call clear_attendance()
```

## Solutions Considered

### Option 1: Clear Immediately After Fetch (❌ RISKY)
```rust
// In main.rs polling loop
match client.call("device.get_logs", json!({})) {
    Ok(res) => {
        // Send to DB writer
        // Clear device immediately
        client.call("device.clear_logs", json!({}))?;
    }
}
```

**Risk:** If agent crashes before sync completes, logs are lost forever.

### Option 2: Clear After Confirmed Sync (✅ SAFER - RECOMMENDED)
Track which device logs have been fully synced, then clear device in batches.

**Flow:**
1. Fetch logs → Save to SQLite with device_timestamp
2. Sync to central → Get ACK → Mark synced
3. Periodically check: "All logs before timestamp T are synced?"
4. If yes → Call `device.clear_logs` to clear old logs

**Pros:**
- No data loss risk
- Can handle partial syncs
- Idempotent (safe to retry)

**Cons:**
- More complex
- Device accumulates logs until sync completes

### Option 3: Manual Clear (Current Workaround)
Operator manually clears device when safe:
```bash
# SSH to agent server, connect to device, clear manually
```

**Pros:** No code changes needed
**Cons:** Not automated, requires manual intervention

## Recommended Solution: Option 2

### Implementation Plan

1. **Add device.clear_logs method to ZK worker**
   ```python
   # zk_worker.py
   async def clear_attendance_logs(self):
       conn = await self.connect()
       await asyncio.to_thread(conn.clear_attendance)
       return True

   # In process_request:
   elif method == "device.clear_logs":
       if not self.service: raise Exception("Device not connected")
       result = await self.service.clear_attendance_logs()
   ```

2. **Track device log state in database**
   Add migration:
   ```sql
   ALTER TABLE devices ADD COLUMN last_clear_timestamp TIMESTAMP;
   ```

3. **Add periodic clear task to main.rs**
   ```rust
   tokio::spawn(async move {
       loop {
           sleep(Duration::from_secs(300)).await; // Every 5 min

           // For each device:
           // 1. Check if all logs before last_fetch are synced (is_synced=1)
           // 2. If yes → call device.clear_logs
           // 3. Update last_clear_timestamp
       }
   });
   ```

4. **Add safety check**
   - Only clear if logs are > 5 minutes old (safety buffer)
   - Log clearing operation prominently for audit trail
   - Handle errors gracefully (clearing failure shouldn't crash agent)

## Alternative: Incremental Clear

Instead of clearing ALL logs, clear only confirmed-synced logs:
- **Problem:** pyzk2 `clear_attendance()` clears ALL logs, not selective
- **Workaround:** Accept that device accumulates until all logs synced, then bulk clear

## Questions

1. **Is data loss acceptable if agent crashes?** → No → Use Option 2
2. **How often should device be cleared?** → After ALL pending logs synced + 5min safety buffer
3. **Should clearing be automatic or manual?** → Automatic (with safety checks)
4. **What if device is offline during clear?** → Skip, retry next cycle

## Current Workaround

Until automated solution implemented:
1. Monitor device log count
2. When device has too many logs (>50k), manually clear via device UI or SSH
3. Or let logs accumulate (device can hold ~100k logs typically)

## Impact Assessment

**Current behavior:**
- ✅ No data loss (logs saved to SQLite)
- ✅ Sync works (logs reach central server)
- ❌ Device memory fills up (10,460 → 20,000 → ... → 100k+)
- ❌ Unnecessary network/CPU usage (re-fetching same logs)
- ❌ Slower polling (10k logs take time to fetch and process)

**With fix:**
- ✅ Device stays clean (only unsent logs present)
- ✅ Faster polling (fewer logs to fetch each cycle)
- ✅ Lower network usage
- ✅ Better performance

## Next Steps

1. User confirms: Auto-clear is desired?
2. User confirms: Wait for full sync before clear? (Option 2)
3. Implement `device.clear_logs` in zk_worker.py
4. Implement periodic clear task in main.rs
5. Add database migration for tracking
6. Test with safety checks
7. Deploy and monitor

## Open Questions

- Should we track per-device last fetch timestamp to know what's safe to clear?
- Should clearing be triggered after N logs synced, or time-based?
- What's the device's max log capacity? (Typical ZKTeco: 100k-200k)
