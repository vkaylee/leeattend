# Scout Report: rustzk-debug

## Exploration Scope
- Target: rustzk 0.2.6 attendance fetching
- Boundaries: `agent-main/src/zk_worker.rs`, `agent-main/examples/debug_zk.rs`, `agent-main/Cargo.toml`

## Patterns Discovered
### Pattern: Sync Task Execution
- **Location**: `agent-main/src/zk_worker.rs`, `agent-main/examples/debug_zk.rs`
- **Usage**: Since `rustzk` is a synchronous library, calls are wrapped in `tokio::task::spawn_blocking`.
- **Must Follow**: Yes, to avoid blocking the async runtime.

## Integration Points
| Point  | File   | Function | New Code Location |
| ------ | ------ | -------- | ----------------- |
| ZK Connection | `src/zk_worker.rs` | `connect` | N/A |
| Attendance Fetch | `src/zk_worker.rs` | `get_attendance_logs` | N/A |

## Conventions
- Device IP: `192.168.12.13` (Confirmed by user)
- Port: `4370` (Standard ZK port)
- Protocol: `UDP` or `Auto`

## Warnings
- ⚠️ `rustzk` binary contains "Buffered response size" string, confirming it's a library-side message.
- ⚠️ Timeouts are set to 30s-60s, suggesting large logs might be slow or problematic.
