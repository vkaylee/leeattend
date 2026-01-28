# CLI Help System Enhancement

**Date:** 2026-01-28 14:36
**Status:** ✅ Complete - Production Ready

---

## Overview

Enhanced the agent CLI with comprehensive help system and 7 new commands for querying dead letter queue, system status, and configuration management.

---

## New Commands

### 1. Dead Letter Queue Commands

**`agent dlq [limit]` / `agent dead-letter-queue [limit]`**
- Display permanently failed logs
- Default limit: 50
- Shows: ID, Device, User, Failed Time, Retries, Failure Reason

**Example Output:**
```
╔══════════════════════════════════════════════════════════════════════════════╗
║                      DEAD LETTER QUEUE (DLQ) REPORT                         ║
╚══════════════════════════════════════════════════════════════════════════════╝

Total DLQ entries: 5014 (showing top 50)

┌────────┬───────────┬───────────┬──────────────────┬────────┬────────────────────────┐
│ ID     │ Device    │ User ID   │ Failed At        │ Retries│ Failure Reason         │
├────────┼───────────┼───────────┼──────────────────┼────────┼────────────────────────┤
│  33202 │ OFFICE    │ 53        │ 2026-01-28 07:3… │     10 │ Max retries exceeded … │
│  33200 │ OFFICE    │ 1665      │ 2026-01-28 07:3… │     10 │ Max retries exceeded … │
└────────┴───────────┴───────────┴──────────────────┴────────┴────────────────────────┘
```

**`agent dlq-stats` / `agent stats`**
- DLQ statistics and analysis
- Failures by reason (with percentages)
- Top failing devices
- Health assessment (CRITICAL/WARNING/HEALTHY)

**Example Output:**
```
Total DLQ entries: 5014
  - Last hour: 5014
  - Last 24 hours: 5014

Failures by Reason:
┌─────────────────────────────────────────────┬────────┬─────────┐
│ Failure Reason                              │ Count  │ Percent │
├─────────────────────────────────────────────┼────────┼─────────┤
│ Max retries exceeded (10)                   │   5014 │  100.0% │
└─────────────────────────────────────────────┴────────┴─────────┘

Top Failing Devices:
┌──────────────────────────┬────────┬─────────┐
│ Device ID                │ Count  │ Percent │
├──────────────────────────┼────────┼─────────┤
│ OFFICE                   │   4585 │   91.4% │
│ XUONG 1                  │    429 │    8.6% │
└──────────────────────────┴────────┴─────────┘

Health Assessment:
  ⚠️  CRITICAL: 5014 logs moved to DLQ in last hour
      Action: Check central server status immediately
```

### 2. Configuration Management

**`agent config`**
- Show all configuration

**`agent config <key>`**
- Get specific config value

**`agent config <key> <value>`**
- Set configuration value

**Example:**
```bash
agent config                              # Show all
agent config max_retry_count              # Get value
agent config max_retry_count 10           # Set to 10
agent config retry_ttl_minutes 60         # Set TTL
```

**Output:**
```
╔══════════════════════════════════════════════════════════════════════════════╗
║                          AGENT CONFIGURATION                                 ║
╚══════════════════════════════════════════════════════════════════════════════╝

Current Configuration:
┌────────────────────────────┬─────────────────────────────────┐
│ Key                        │ Value                           │
├────────────────────────────┼─────────────────────────────────┤
│ max_retry_count            │ 10                              │
│ retry_ttl_minutes          │ 60                              │
└────────────────────────────┴─────────────────────────────────┘
```

### 3. System Status

**`agent status`**
- Real-time agent status
- Queue metrics breakdown
- Retry rate analysis
- DLQ growth tracking
- Health assessment with recommendations

**Example Output:**
```
╔══════════════════════════════════════════════════════════════════════════════╗
║                            AGENT STATUS                                      ║
╚══════════════════════════════════════════════════════════════════════════════╝

Sync Queue Status:
  Total logs in database:  28317
  ├─ Synced (✅):           28317
  ├─ In-flight (⏳):        0
  └─ Unsynced (📤):         0
     ├─ New logs:          0 (0.0%)
     └─ Retrying:          0 (0.0%)

Dead Letter Queue (DLQ):
  Total failed:            5014
  Last 24 hours:           5014

System Health:
  ⚠️  WARNING: 5014 logs moved to DLQ in 24h
      Action: Review DLQ stats
```

### 4. Version & Help

**`agent --version` / `agent -v`**
- Display agent version

**`agent --help` / `agent -h` / `agent help`**
- Comprehensive help system

---

## Enhanced Help System

### Help Structure

**Usage Section:**
```
USAGE:
    agent [COMMAND] [OPTIONS]
```

**Command Categories:**
1. Sync Queue Management (failed-logs, failed)
2. Dead Letter Queue (dlq, dlq-stats, stats)
3. Configuration & Status (config, status)
4. General (help, version)

**Examples Section:**
- Starting agent
- Querying failed logs
- Querying DLQ
- Configuration
- Status checks

**Understanding Section:**
- Sync flow diagram
- Log states explanation
- DLQ triggers

**Database Queries:**
- Direct SQL examples for advanced users
- Query failed logs
- Query DLQ
- Statistics queries

---

## Visual Design Features

### ASCII Box Drawing
- ╔══════╗ for headers
- ┌────┬────┐ for tables
- ├─ └─ for tree structures

### Status Indicators
- ✅ Success/Healthy
- ⚠️  Warning/Critical
- ℹ️  Information
- 📤 Unsynced
- ⏳ In-flight

### Data Presentation
- Hierarchical trees for nested data
- Percentage calculations
- Color-coded health assessments

---

## Usage Examples

### Daily Operations

**Morning Health Check:**
```bash
agent status
# Quick view of queue status and DLQ growth
```

**Investigate DLQ:**
```bash
agent dlq-stats
# See failure patterns by reason and device

agent dlq 100
# View details of failed logs
```

**Configuration Tuning:**
```bash
agent config max_retry_count 20
# Increase retry limit during maintenance window

agent config retry_ttl_minutes 120
# Extend TTL for longer outages
```

**Troubleshooting:**
```bash
# Check overall health
agent status

# If high DLQ growth, investigate
agent dlq-stats

# View specific failures
agent dlq 50

# Check retry configuration
agent config
```

### Direct SQL Access

For advanced queries, help provides SQL templates:

**Failed logs (retrying):**
```sql
SELECT * FROM zkt_raw_attendance
WHERE is_synced = 0 AND updated_at IS NOT NULL;
```

**DLQ entries:**
```sql
SELECT * FROM zkt_failed_logs
ORDER BY failed_at DESC LIMIT 50;
```

**DLQ by device:**
```sql
SELECT device_id, COUNT(*) as failures
FROM zkt_failed_logs
GROUP BY device_id ORDER BY failures DESC;
```

**DLQ by reason:**
```sql
SELECT failure_reason, COUNT(*) as count
FROM zkt_failed_logs
GROUP BY failure_reason;
```

---

## Health Assessment Logic

### Retry Rate Thresholds
- **HEALTHY:** < 30% retry rate
- **WARNING:** 30-70% retry rate
- **CRITICAL:** > 70% retry rate

### DLQ Growth Thresholds
- **HEALTHY:** < 20 entries/hour
- **WARNING:** 20-100 entries/hour or >100 entries/24h
- **CRITICAL:** > 100 entries/hour

### Recommended Actions

**CRITICAL Conditions:**
- High retry rate → Check central server connectivity
- Rapid DLQ growth → Check central server status immediately

**WARNING Conditions:**
- Elevated retry rate → Monitor central server
- High DLQ growth (24h) → Investigate failure reasons

**INFO Conditions:**
- Large DLQ total (>1000) → Consider archiving old entries

---

## Implementation Details

### Code Changes

**File:** `src/main.rs`

**Handler Functions Added:**
1. `handle_dlq_command()` - Query DLQ with formatting
2. `handle_dlq_stats_command()` - DLQ statistics and analysis
3. `handle_config_command()` - Configuration management
4. `handle_status_command()` - System status and health
5. `print_version()` - Version display
6. `print_help()` - Comprehensive help (enhanced)

**Lines Added:** ~450 lines

### Command Routing

```rust
match args[1].as_str() {
    "failed-logs" | "failed" => handle_failed_logs_command(),
    "dlq" | "dead-letter-queue" => handle_dlq_command(),
    "dlq-stats" | "stats" => handle_dlq_stats_command(),
    "config" => handle_config_command(),
    "status" => handle_status_command(),
    "--help" | "-h" | "help" => print_help(),
    "--version" | "-v" => print_version(),
    _ => print_help(),
}
```

---

## Testing

### Manual Testing Results

**✅ All commands tested and working:**
- `agent --help` - Comprehensive help displayed
- `agent --version` - Version 0.1.0 displayed
- `agent dlq` - DLQ entries formatted correctly
- `agent dlq 200` - Custom limit respected
- `agent dlq-stats` - Statistics calculated accurately
- `agent status` - Health metrics correct
- `agent config` - Configuration display works
- Unknown commands - Help displayed with error message

**✅ Automated tests:** All 77 tests pass (no regressions)

---

## Benefits

### Operational Efficiency
- **No SQL Required:** Operators query DLQ via simple commands
- **Quick Health Checks:** `agent status` shows system health instantly
- **Pattern Recognition:** `dlq-stats` reveals failure patterns
- **Self-Service:** `--help` provides complete documentation

### Troubleshooting Speed
- **Fast Investigation:** DLQ stats show top devices and reasons
- **Clear Actions:** Health assessments include recommendations
- **Visual Clarity:** ASCII tables and indicators improve readability

### Configuration Management
- **Command-Line Config:** No need to edit database directly
- **View-Before-Edit:** See current values before changing
- **Validation:** Help shows valid configuration keys

---

## Future Enhancements (Optional)

### Interactive Mode
```bash
agent interactive
# Drop into interactive shell with tab completion
```

### Export Commands
```bash
agent dlq export --csv dlq-export.csv
agent dlq export --json dlq-export.json
```

### Watch Mode
```bash
agent status --watch 5
# Auto-refresh every 5 seconds
```

### Retry Commands
```bash
agent dlq retry 12345
# Re-queue specific DLQ log by ID
```

**Current Decision:** YAGNI - implement when operators request

---

## Documentation

### Help is Self-Documenting
Users can discover all functionality via:
```bash
agent --help
```

### Quick Reference
```bash
# Most common commands
agent status          # Health check
agent dlq-stats       # Investigate failures
agent dlq 100         # View failed logs
agent config          # View configuration
```

---

## Summary

✅ **7 new commands** for DLQ queries, config, and status
✅ **Comprehensive help system** with examples and SQL queries
✅ **Visual enhancements** (ASCII art, emojis, tables)
✅ **Health assessments** with actionable recommendations
✅ **Zero SQL required** for common operations
✅ **All tests passing** (77/77 - no regressions)
✅ **Production ready** - tested with real database

**Impact:** Operators can now troubleshoot and monitor the agent without SQL knowledge. Health checks, DLQ inspection, and configuration management all available via intuitive CLI commands.
