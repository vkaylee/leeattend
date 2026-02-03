# Phase 2: Backend Logic Implementation

## Overview
Implement fingerprint calculation and routing logic for attendance logs.

## Priority
High - Core business logic

## Related Files
- `apiserver/src/services/attendance_log_service.rs` - Update routing logic
- `apiserver/src/utils/` - New fingerprint utility (or inline)

## Fingerprint Logic

### Function: `calculate_fingerprint()`
Unified helper for both staging and error tables.

```rust
use sha2::{Sha256, Digest};

/// Calculate fingerprint for deduplication
/// Format: SHA256(device_id|raw_user_id|timestamp_unix|punch|status)
pub fn calculate_fingerprint(
    device_id: Option<&str>,
    raw_user_id: &str,
    punch_time: &str,  // Raw timestamp string
    punch: Option<i32>,
    status: Option<i32>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(device_id.unwrap_or(""));
    hasher.update("|");
    hasher.update(raw_user_id);
    hasher.update("|");
    hasher.update(punch_time);
    hasher.update("|");
    hasher.update(punch.map(|p| p.to_string()).unwrap_or_default());
    hasher.update("|");
    hasher.update(status.map(|s| s.to_string()).unwrap_or_default());
    format!("{:x}", hasher.finalize())
}
```

## Routing Logic Update

### Current Flow (process_log)
```
Employee not found → create_error_log → Err(msg)  ❌ Agent retries
Parse error → create_error_log → Err(msg)  ❌ Agent retries
Success → create_raw_log → Ok(true)
```

### New Flow
```
Case 1: Employee found → create_raw_log → Ok(true)  ✓ Normal
Case 2: Employee NOT found → create_staging_log (fingerprint) → Ok(false)  ✓ Swallow
Case 3: Parse/validation error → create_error_log (fingerprint) → Ok(false)  ✓ Swallow
```

### Code Changes in `attendance_log_service.rs`

```rust
// Case 2: Employee not found → Staging (NOT error)
Ok(None) => {
    let fingerprint = calculate_fingerprint(
        device_id.as_deref(),
        employee_id_number,
        punch_time_raw,
        raw_data.as_ref().and_then(|d| d.get("punch").and_then(|p| p.as_i64()).map(|p| p as i32)),
        raw_data.as_ref().and_then(|d| d.get("status").and_then(|s| s.as_i64()).map(|s| s as i32)),
    );
    let _ = staging_log_repository::create_staging_log(
        pool,
        workspace_id,
        device_id,
        employee_id_number,
        punch_time_raw,  // Store raw time before timezone conversion
        raw_data,
        &fingerprint,
    ).await;
    warn!("Unknown user {} staged for workspace {}", employee_id_number, workspace_id);
    return Ok(false);  // SUCCESS - don't retry
}

// Case 3: Parse error → Error table with fingerprint
Err(e) => {
    let fingerprint = calculate_fingerprint(...);
    let _ = attendance_log_error_repository::create_error_log_with_fingerprint(
        pool,
        workspace_id,
        device_id.clone(),
        raw_data,
        &msg,
        &format!("{:?}", source),
        &fingerprint,
    ).await;
    return Ok(false);  // SUCCESS - don't retry
}
```

## Todo
- [ ] Add sha2 dependency to Cargo.toml
- [ ] Create calculate_fingerprint() utility
- [ ] Update process_log() routing logic
- [ ] Update ingest_agent_log() return handling

## Success Criteria
- Unknown users go to staging, return Ok(false)
- Parse errors go to errors with fingerprint, return Ok(false)
- Duplicates silently ignored (ON CONFLICT DO NOTHING)
