# Planner Report: Unix Timestamp Refactor

**Date**: 2026-02-01
**Plan**: `/home/elt1541/leeattend/plans/260201-1047-unix-timestamp-refactor/`

## Summary

Created detailed implementation plan for refactoring Rust Agent to use Unix timestamps (i64) and per-device timezone configuration.

## Codebase Analysis

### Current Architecture

| Component | Current Type | Location |
|-----------|-------------|----------|
| `zkt_raw_attendance.timestamp` | DATETIME | migrations/20260108000000_init_schema.sql |
| `RawAttendanceLog.timestamp` | NaiveDateTime | src/models.rs:11 |
| `SignatureRow.timestamp` | NaiveDateTime | src/models.rs:25 |
| `MemoryLog.timestamp` | NaiveDateTime | src/memory_store.rs:48 |
| `DeviceConfig` | No timezone | src/models.rs:48-56 |
| `DeviceConfigPush` | No timezone | src/models.rs:59-66 |

### Composite Key Flow

```
MemoryLog.composite_key()
    |
    v
format!("{}:{}:{}", device_id, user_id, timestamp.and_utc().timestamp())
    |
    v
parse_composite_key() in central_client.rs
    |
    v
DateTime::from_timestamp() -> format("%Y-%m-%d %H:%M:%S")
    |
    v
WHERE timestamp = ? (String binding - MISMATCH!)
```

**Problem**: Timestamp stored as DATETIME String, but composite key uses Unix i64. Conversion roundtrip causes format mismatches.

### Sync Logic

1. **Sync Loop** (central_client.rs:91-204): Gets unsynced logs from HybridDatabase, sends to server
2. **ACK Handling** (central_client.rs:316-376): Parses composite key, updates DB
3. **Memory Store** (memory_store.rs): In-memory cache with composite index for ACK correlation

### Signature Hash

Current signature hash uses:
- device_id
- user_id
- timestamp.and_utc().timestamp() (Unix i64)
- punch
- status

**Good**: Already uses Unix timestamp for dedup. Change is consistent.

## Plan Structure

```
plans/260201-1047-unix-timestamp-refactor/
├── plan.md                      # Overview (6h total effort)
├── phase-01-database-schema.md  # Schema changes (1h)
├── phase-02-dependencies.md     # Add chrono-tz (15m)
├── phase-03-data-models.md      # Update structs (30m)
├── phase-04-database-logic.md   # Timezone lookup (1.5h)
├── phase-05-sync-logic.md       # Composite key refactor (1.5h)
└── phase-06-testing.md          # Test updates (1h)
```

## Key Implementation Decisions

1. **Timezone Lookup Strategy**: Query devices table per-save, batch pre-fetch for batch ops
2. **Fallback**: Default to 'UTC' if device not found or timezone invalid
3. **DST Handling**: Use chrono_tz's earliest() for ambiguous times
4. **Server Compatibility**: Keep AttendanceData.timestamp as String (format before sending)
5. **No Backward Compat**: Fresh DB reset acceptable

## Dependencies

- **chrono-tz 0.9**: IANA timezone parsing, compatible with chrono 0.4
- No Python changes (sidecar continues sending local time strings)

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Timezone parsing failures | Default to UTC |
| DST ambiguity | Use earliest() fallback |
| Test failures | Systematic fixture updates |
| Server incompatibility | Format timestamp as String before sending |

## Unresolved Questions

1. **Server-side changes**: Does server need to parse i64 from composite key? (Likely yes, for ACK generation)
2. **Historical data**: If any prod data exists, need migration strategy (currently assumed fresh start)
3. **Timezone updates**: If device timezone changes mid-day, how to handle in-flight logs?

## Files to Modify

| File | Changes |
|------|---------|
| migrations/20260108000000_init_schema.sql | timestamp: INTEGER, add timezone column |
| Cargo.toml | Add chrono-tz = "0.9" |
| src/models.rs | RawAttendanceLog, SignatureRow, FailedLog, DeviceConfig, DeviceConfigPush |
| src/database.rs | save_raw_attendance, upsert_device, timezone lookup |
| src/memory_store.rs | MemoryLog, insert_with_timestamp, hash functions |
| src/central_client.rs | parse_composite_key, ACK handling, ConfigPush |
| tests/*.rs | All test fixtures |

## Testing Strategy

1. Use `./run-tests.sh --api agent` (WSL constraint)
2. Add specific timezone conversion tests
3. Test edge cases: invalid TZ, missing device, DST
4. Validate SQLite stores INTEGER values
