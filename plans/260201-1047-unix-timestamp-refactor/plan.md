---
title: "Unix Timestamp Refactor for Agent Sync"
description: "Standardize time storage to Unix Timestamp (i64 UTC) and add per-device timezone configuration"
status: pending
priority: P1
effort: 6h
branch: main
tags: [agent, timestamp, timezone, sync, database]
created: 2026-02-01
---

# Unix Timestamp Refactor Plan

## Overview

Standardize time storage to Unix Timestamp (UTC seconds) to fix synchronization issues caused by string formatting mismatches. Implement per-device timezone configuration to correctly parse local time strings from sidecar.

## Problem Statement

- Current: `timestamp` stored as `DATETIME` (NaiveDateTime string) in SQLite
- Sync issues: String format mismatches between Agent and Server cause ACK failures
- No timezone context: Sidecar sends local time strings, Agent treats them as UTC

## Solution Architecture

```
Input: Sidecar sends "2025-01-31 08:00:00" (Local Time String)
                        |
                        v
Configuration: Device has timezone = "Asia/Ho_Chi_Minh"
                        |
                        v
Processing: Parse String + Timezone -> DateTime<Tz>
                        |
                        v
Convert: DateTime<Tz> -> DateTime<Utc> -> Unix Timestamp (i64)
                        |
                        v
Storage: SQLite stores INTEGER (i64)
                        |
                        v
Sync: Send i64 to Server, parse ACK i64 directly
```

## Phases

| Phase | Status | Description | Effort |
|-------|--------|-------------|--------|
| [Phase 1](./phase-01-database-schema.md) | pending | Database Schema Changes | 1h |
| [Phase 2](./phase-02-dependencies.md) | pending | Add chrono-tz Dependency | 15m |
| [Phase 3](./phase-03-data-models.md) | pending | Update Data Models | 30m |
| [Phase 4](./phase-04-database-logic.md) | pending | Database Logic Changes | 1.5h |
| [Phase 5](./phase-05-sync-logic.md) | pending | Sync & Composite Key Refactor | 1.5h |
| [Phase 6](./phase-06-testing.md) | pending | Testing & Validation | 1h |

## Dependencies

- **chrono-tz**: Required for IANA timezone parsing
- **No backward compatibility**: DB will be reset (dev mode)
- **Python sidecar**: NO CHANGES (continues sending local time strings)

## Key Files to Modify

```
agent-main/
├── Cargo.toml                                    # Add chrono-tz
├── migrations/20260108000000_init_schema.sql     # Modify schema
├── src/
│   ├── models.rs                                 # RawAttendanceLog, DeviceConfig
│   ├── database.rs                               # save_raw_attendance, upsert_device
│   ├── memory_store.rs                           # MemoryLog, composite_key
│   ├── hybrid_database.rs                        # HybridDatabase methods
│   └── central_client.rs                         # parse_composite_key, ACK handling
└── tests/                                        # Update all test fixtures
```

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Timezone parsing failures | High | Default to 'UTC' if invalid |
| Migration breaks existing data | Low | DB reset is acceptable |
| Composite key format change | Medium | Update both Agent & Server together |
| Test failures | Medium | Update all test fixtures with i64 |

## Success Criteria

1. All timestamps stored as INTEGER (Unix seconds) in SQLite
2. Each device has configurable timezone column
3. Local time strings correctly converted to UTC before storage
4. Sync ACKs use i64 timestamps without string conversion
5. All existing tests pass after fixture updates
