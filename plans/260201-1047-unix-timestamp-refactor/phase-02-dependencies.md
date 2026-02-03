# Phase 2: Add chrono-tz Dependency

## Context Links
- Plan: [plan.md](./plan.md)
- Cargo.toml: `agent-main/Cargo.toml`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 15m
- **Depends on**: Phase 1

Add `chrono-tz` crate for IANA timezone parsing and conversion.

## Key Insights

1. **chrono-tz**: Industry-standard crate for timezone handling in Rust
2. **IANA Database**: Contains all standard timezones (Asia/Ho_Chi_Minh, etc.)
3. **Version Compatibility**: Use latest stable (0.9.x) compatible with chrono 0.4

## Requirements

### Functional
- Parse IANA timezone strings into Tz objects
- Convert local DateTime to UTC DateTime
- Extract Unix timestamp from UTC DateTime

### Non-functional
- No additional runtime dependencies
- Compile-time timezone database (no external files)

## Architecture

```rust
use chrono::NaiveDateTime;
use chrono_tz::Tz;

// Parse timezone from device config
let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap_or(Tz::UTC);

// Parse local time string
let local_naive = NaiveDateTime::parse_from_str("2025-01-31 08:00:00", "%Y-%m-%d %H:%M:%S")?;

// Convert to timezone-aware, then to UTC, then to Unix timestamp
let local_dt = tz.from_local_datetime(&local_naive).unwrap();
let utc_dt = local_dt.with_timezone(&chrono::Utc);
let unix_timestamp: i64 = utc_dt.timestamp();
```

## Related Code Files

### Modify
- `agent-main/Cargo.toml`

## Implementation Steps

### Step 1: Add chrono-tz to dependencies

In `agent-main/Cargo.toml`, add under `[dependencies]`:

```toml
chrono-tz = "0.9"
```

Current chrono version is 0.4 which is compatible.

### Step 2: Verify cargo check

```bash
cd agent-main && cargo check
```

### Step 3: Document common timezone strings

```
UTC                     - Default fallback
Asia/Ho_Chi_Minh       - Vietnam (UTC+7)
Asia/Bangkok           - Thailand (UTC+7)
Asia/Singapore         - Singapore (UTC+8)
Asia/Tokyo             - Japan (UTC+9)
America/New_York       - US Eastern
Europe/London          - UK
```

## Todo List

- [ ] Add chrono-tz = "0.9" to Cargo.toml
- [ ] Run cargo check to verify compatibility
- [ ] Test timezone parsing in isolation

## Success Criteria

1. `cargo check` passes without errors
2. `use chrono_tz::Tz;` compiles
3. `"Asia/Ho_Chi_Minh".parse::<Tz>()` returns valid timezone

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Version incompatibility | Low | Low | chrono-tz 0.9 supports chrono 0.4 |
| Increased compile time | Low | Low | One-time cost, ~30s |

## Security Considerations
- chrono-tz is a well-audited crate from the Rust community
- No network calls; timezone database compiled into binary

## Next Steps
After this phase, proceed to [Phase 3: Data Models](./phase-03-data-models.md)
