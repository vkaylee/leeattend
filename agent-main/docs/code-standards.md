# Code Standards & Architecture Guidelines

**Project:** Agent Attendance Sync System
**Language:** Rust
**Version:** 1.0
**Last Updated:** 2026-01-29

---

## File Organization & Naming

### Directory Structure

```
src/
├── main.rs              # Application entry point and orchestration
├── lib.rs               # Module declarations and public exports
├── database.rs          # SQLite operations and schema
├── hybrid_database.rs   # Memory + disk facade
├── memory_store.rs      # In-memory concurrent storage
├── crash_recovery.rs    # Crash detection and recovery
├── central_client.rs    # Server sync via WebSocket
├── worker_client.rs     # Worker service requests
├── zk_client.rs         # Device communication (ZKTeco)
├── config.rs            # Configuration management
├── models.rs            # Data structures and types
├── retry.rs             # Retry logic and backoff
└── utils.rs             # Helper utilities

tests/
├── hybrid_database_integration_test.rs
├── push_logs_integration_test.rs
├── retry_limit_test.rs
├── startup_config_clear_test.rs
├── device_clear_safety_test.rs
├── employee_id_normalization_test.rs
├── fair_polling_test.rs
├── memory_leak_detection_test.rs
├── queue_backpressure_test.rs
├── sync_log_classification_test.rs
├── db_writer_singleton_test.rs
├── duplicate_detection_test.rs
├── batch_worker_test.rs
├── config_push_test.rs
├── integration_test.rs
└── ... (14+ total)

migrations/
├── 20260108000000_init_schema.sql
├── 20260126000000_add_updated_at_to_logs.sql
├── 20260126000001_add_unique_index.sql
├── 20260126000002_create_parsing_errors.sql
├── 20260127000000_add_brand_to_devices.sql
├── 20260127130000_create_agent_config.sql
└── 20260128000000_add_retry_tracking.sql
```

### File Naming Conventions

- **Modules:** snake_case, descriptive names
  - `memory_store.rs` (not `store.rs` or `mem.rs`)
  - `central_client.rs` (not `client.rs`)
  - `crash_recovery.rs` (not `recovery.rs`)

- **Test Files:** `*_test.rs` or `*_integration_test.rs`
  - `hybrid_database_integration_test.rs`
  - `retry_limit_test.rs`

- **Migrations:** Timestamp prefix for ordering
  - `20260128000000_add_retry_tracking.sql`
  - Ensure chronological execution

### File Size Limits

- **Production Code:** Target <200 lines per file
  - Current: All files <1000 lines
  - Largest: database.rs (973 lines) - acceptable, monolithic
  - Others: 50-600 lines - well-balanced

- **Test Files:** No strict limit, but organize tests logically
  - Current: 50-500 lines per test file
  - Group related tests in same file

---

## Rust Coding Conventions

### 1. Imports & Module Organization

**Do:**
```rust
// Standard library first
use std::sync::Arc;
use std::collections::HashMap;

// External crates
use tokio::task;
use sqlx::SqlitePool;
use dashmap::DashMap;

// Internal modules (organized by module, then alphabetically)
use crate::crash_recovery::detect_crash;
use crate::models::{AttendanceData, MemoryLog};
use crate::utils::{format_error, normalize_employee_id};
```

**Don't:**
```rust
// Don't mix import types without grouping
use tokio::task;
use std::sync::Arc;
use dashmap::DashMap;
use crate::models::AttendanceData;

// Don't use wildcard imports except in tests
use crate::*;
```

### 2. Type Definitions & Structs

**Do:**
```rust
/// Represents a single attendance log in memory
///
/// # Fields
/// - `id`: Unique identifier, generated sequentially
/// - `is_synced`: 0=unsynced, 1=synced, 2=in_flight
/// - `updated_at`: Last sync attempt timestamp, None for new logs
/// - `retry_count`: Number of failed sync attempts
#[derive(Clone, Debug)]
pub struct MemoryLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: NaiveDateTime,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub retry_count: i64,
}

// Tuple structs for newtype pattern
pub struct LogId(pub i64);
pub struct DeviceId(pub String);
```

**Don't:**
```rust
// Don't omit documentation on public types
pub struct MemoryLog { /* ... */ }

// Don't use ambiguous field names
pub struct Log {
    pub s: i64,           // Unclear what 's' means
    pub d: String,        // 'd' could be device or data
}
```

### 3. Function Signatures

**Do:**
```rust
/// Insert a new attendance log into memory
///
/// # Arguments
/// * `device_id` - Unique device identifier
/// * `data` - Attendance record with user/timestamp
///
/// # Returns
/// * `Result<i64, String>` - Log ID on success, error message on failure
///
/// # Panics
/// Panics if capacity exceeded (should not happen due to backpressure)
pub fn insert(&self, device_id: String, data: AttendanceData) -> Result<i64, String> {
    // Implementation
}

// Clear parameter naming
fn mark_in_flight(&self, ids: &[i64]) -> Result<(), String> { }

// Return meaningful types
fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> { }
```

**Don't:**
```rust
// Don't use vague parameter names
fn insert(&self, d: String, x: AttendanceData) -> Result<i64, String> { }

// Don't omit return type when unclear
pub fn process(log: &MemoryLog) { } // Unclear if this returns anything

// Don't use cryptic abbreviations
fn rst_if() -> bool { } // Should be reset_in_flight()
```

### 4. Error Handling

**Do:**
```rust
use std::fmt;

#[derive(Debug)]
pub enum SyncError {
    CapacityExceeded { max: usize, current: usize },
    DatabaseError(String),
    NetworkError(String),
    SerializationError(String),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CapacityExceeded { max, current } => {
                write!(f, "Capacity exceeded: {} of {}", current, max)
            }
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            // ... other variants
        }
    }
}

// Use Result<T, E> consistently
pub fn save_logs(&self, logs: &[MemoryLog]) -> Result<usize, SyncError> {
    // Use ? operator for propagation
    let count = self.database.insert(logs)?;
    Ok(count)
}

// Handle errors explicitly
match result {
    Ok(id) => {
        info!("Log inserted: {}", id);
    }
    Err(e) => {
        error!("Failed to insert log: {}", e);
        // Handle or propagate
    }
}
```

**Don't:**
```rust
// Don't use panic! in production code paths
fn insert(&self, data: AttendanceData) -> i64 {
    self.logs.insert(data).expect("Database panicked")
}

// Don't ignore errors silently
let _ = self.save_logs(&logs);

// Don't use generic error strings
Err("Something went wrong".to_string())
```

### 5. Async/Concurrency Patterns

**Do:**
```rust
// Use tokio for async tasks
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        sync_loop(store).await
    });

    // Graceful shutdown
    if let Err(e) = handle.await {
        error!("Worker panicked: {}", e);
    }
}

// Use Arc for shared state across threads
let store = Arc::new(InMemoryStore::new(100_000));
let store_clone = Arc::clone(&store);

// Use DashMap for concurrent access
let logs: DashMap<i64, MemoryLog> = DashMap::new();
let entry = logs.entry(id).or_insert(log);

// Use parking_lot RwLock (faster than std)
use parking_lot::RwLock;
let index: RwLock<BTreeSet<i64>> = RwLock::new(BTreeSet::new());
let guard = index.read();  // Shared read
let mut guard = index.write();  // Exclusive write
```

**Don't:**
```rust
// Don't use std::sync::Mutex when parking_lot is available
use std::sync::Mutex;
let index = Mutex::new(BTreeSet::new());

// Don't hold locks across await points
let guard = locks.write();
async_operation().await;  // Lock held during await = deadlock risk
drop(guard);

// Don't spawn unbounded tasks
for log in logs {
    tokio::spawn(process_log(log));  // Unbounded growth
}

// Instead: use a bounded semaphore or channel
let semaphore = Arc::new(Semaphore::new(100));
```

### 6. Logging Standards

**Do:**
```rust
use tracing::{info, warn, error, debug};

// Info: Major state changes
info!("Checkpoint: {} logs persisted", count);
info!("Sync cycle completed in {}ms", elapsed);

// Warn: Recoverable issues
warn!("Checkpoint delayed: database busy");
warn!("50 logs still in backoff period");

// Error: Failures requiring attention
error!("Failed to sync logs: {}", e);
error!("Database connection lost, retrying...");

// Debug: Detailed diagnostics (disabled in production by default)
debug!("Processing {} unsynced logs", logs.len());
debug!("Backoff delay: {}s for retry_count={}", delay, retry_count);
```

**Don't:**
```rust
// Don't use println! in library code
println!("Log inserted"); // Use logging instead

// Don't log sensitive data
error!("Auth failed for user: {}", user_id);  // OK: generic
error!("Token rejected: {}", token);          // WRONG: exposes secrets

// Don't log at wrong level
info!("Processing loop iteration 5023");  // Too verbose (use debug)
error!("Could not create directory (it already exists)");  // Not an error (use debug)
```

### 7. Testing Standards

**Do:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test name describes what and expected result
    #[test]
    fn test_insert_new_log_returns_sequential_id() {
        let store = InMemoryStore::new(1000);
        let data = create_test_data();

        let id1 = store.insert("D1".to_string(), data.clone()).unwrap();
        let id2 = store.insert("D2".to_string(), data).unwrap();

        assert_eq!(id2, id1 + 1);
    }

    // Test error conditions
    #[test]
    fn test_capacity_exceeded_blocks_insert() {
        let store = InMemoryStore::new(1);  // Capacity of 1

        let result = store.insert("D1".to_string(), create_test_data());
        assert!(result.is_ok());

        let result = store.insert("D2".to_string(), create_test_data());
        assert!(result.is_err());  // Should fail
    }

    // Use helper functions for common setup
    fn create_test_data() -> AttendanceData {
        AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        }
    }
}
```

**Don't:**
```rust
// Don't omit assertions
#[test]
fn test_insert() {
    let store = InMemoryStore::new(1000);
    store.insert("D1".to_string(), data);
    // Missing assertion!
}

// Don't test multiple things in one test
#[test]
fn test_store_operations() {
    // Tests insert, update, delete - should be 3 tests
    store.insert(...);
    assert!(...);
    store.update(...);
    assert!(...);
}

// Don't use hardcoded magic values
assert_eq!(result.len(), 42);  // Where does 42 come from?
```

---

## Architecture Patterns

### 1. Dual-Write Pattern (Memory → Disk)

**Used in:** HybridDatabase, central_client

**Pattern:**
```rust
pub fn save_logs(&self, logs: &[MemoryLog]) -> Result<usize, Error> {
    // 1. Write to fast path (memory) first
    for log in logs {
        self.memory.insert(log.id, log.clone())?;
    }

    // 2. Write to durable path (disk) asynchronously
    // Failures here don't block the critical path
    tokio::spawn({
        let db = self.database.clone();
        let logs = logs.to_vec();
        async move {
            if let Err(e) = db.save_logs(&logs).await {
                error!("Checkpoint failed: {}", e);
                // Logs stay in memory, retry on next checkpoint
            }
        }
    });

    Ok(logs.len())
}
```

**Rationale:** Speed + Durability tradeoff

### 2. Facade Pattern (HybridDatabase)

**Used in:** HybridDatabase wrapping InMemoryStore + Database

**Pattern:**
```rust
pub struct HybridDatabase {
    memory: Arc<InMemoryStore>,
    database: Arc<Database>,
}

impl HybridDatabase {
    // Present identical interface to Database
    pub fn get_unsynced_logs(&self, limit: usize) -> Result<Vec<MemoryLog>, Error> {
        // Delegate to memory (fast path)
        Ok(self.memory.get_unsynced(limit))
    }

    pub fn save_logs(&self, logs: &[MemoryLog]) -> Result<usize, Error> {
        // Delegate to appropriate store
        self.memory.insert_batch(logs)?;
        // Async checkpoint handled separately
        Ok(logs.len())
    }
}
```

**Rationale:** Transparent optimization without changing client code

### 3. Marker File Pattern (Crash Detection)

**Used in:** CrashRecovery

**Pattern:**
```rust
pub fn detect_crash(db_path: &Path) -> Result<bool, Error> {
    let marker_path = db_path.parent().unwrap().join(".agent_running");

    let crashed = marker_path.exists();  // If marker exists, we crashed

    // Create new marker for this run
    std::fs::File::create(marker_path)?;

    Ok(crashed)
}
```

**Rationale:** Simple, robust crash detection without database queries

### 4. Backoff Pattern (Exponential)

**Used in:** memory_store.rs get_unsynced()

**Pattern:**
```rust
pub fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> {
    let unsynced = self.unsynced_index.read();
    let mut result = Vec::with_capacity(limit);
    let now = chrono::Utc::now().naive_utc();

    for id in unsynced.iter() {
        if result.len() >= limit {
            break;
        }
        if let Some(log) = self.logs.get(id) {
            // Calculate backoff delay
            let delay_secs = (1i64 << log.retry_count.min(6)).min(60);
            let eligible_at = log.updated_at.unwrap_or(now)
                + chrono::Duration::seconds(delay_secs);

            // Skip if still in backoff
            if now < eligible_at {
                continue;
            }
            result.push(log.clone());
        }
    }

    result
}
```

**Rationale:** Prevents retry spam while allowing new logs through

---

## Performance Considerations

### 1. Lock Contention
- Use DashMap instead of Mutex<HashMap>
- Batch operations to reduce lock frequency
- Separate read and write indices

### 2. Memory Efficiency
- Bounded capacity prevents OOM
- Use references when possible (borrowed vs owned)
- String interning for device IDs (future optimization)

### 3. Latency Budget
- Memory operations: <1ms (target met)
- Checkpoint async: 5-10s interval (non-blocking)
- Sync loop: 500ms cycle (adaptive sleep)

### 4. Thread Safety
- DashMap for concurrent reads/writes
- parking_lot RwLock for read-heavy indices
- Atomic counters for backpressure
- No deadlock due to clear lock hierarchy

---

## Security Guidelines

### 1. Credential Management
- Never log authentication tokens
- Store credentials in environment variables
- Use .env files locally only (not in git)

### 2. Input Validation
- Validate device IDs (length, format)
- Validate user IDs (length, format)
- Validate timestamps (parse with chrono)
- Use type system (i64 instead of String for IDs where possible)

### 3. SQL Injection Prevention
- Always use parameterized queries (sqlx)
- Never concatenate strings into SQL
- Validate input lengths before insert

### 4. Error Messages
- Never expose internal paths
- Never show raw database errors
- Never log sensitive data
- Use generic error messages to users

---

## Documentation Standards

### 1. Module Documentation
```rust
//! Manages in-memory storage of attendance logs
//!
//! Uses DashMap for concurrent, lock-free reads and sharded writes.
//! Maintains FIFO ordering via BTreeSet index. Supports backpressure
//! via bounded capacity.
//!
//! # Example
//! ```
//! let store = InMemoryStore::new(100_000);
//! let id = store.insert("device1".into(), data)?;
//! let logs = store.get_unsynced(100);
//! ```
```

### 2. Function Documentation
```rust
/// Inserts a new attendance log into the in-memory store
///
/// # Arguments
/// * `device_id` - Device identifier
/// * `data` - Attendance data (user, timestamp, status)
///
/// # Returns
/// Result with:
/// - Ok(id): Log ID assigned
/// - Err(msg): Capacity exceeded or other error
///
/// # Example
/// ```
/// let id = store.insert("D1".into(), data)?;
/// ```
pub fn insert(&self, device_id: String, data: AttendanceData) -> Result<i64, String>
```

### 3. Complex Logic Comments
```rust
// Skip logs still in backoff period
// Delay = min(2^retry_count, 60) seconds
// New logs (updated_at=None) pass through immediately
if let Some(updated_at) = log.updated_at {
    let delay_secs = (1i64 << log.retry_count.min(6)).min(60);
    let eligible_at = updated_at + chrono::Duration::seconds(delay_secs);
    if now < eligible_at {
        continue;  // Still in backoff
    }
}
```

---

## Code Review Checklist

Before submitting a PR, verify:

- [ ] Code compiles without warnings
- [ ] All tests pass (cargo test --all)
- [ ] New public functions documented
- [ ] Error handling comprehensive (no unwrap in production code)
- [ ] No hardcoded credentials
- [ ] Logging at appropriate level (info, warn, error, debug)
- [ ] Performance-critical code reviewed for O(n) issues
- [ ] Thread safety verified for concurrent code
- [ ] Backward compatible (no breaking API changes)
- [ ] File size <200 lines (or justified exception)
- [ ] Comments explain "why", not "what"
- [ ] Test coverage >95% for new code

---

## Common Patterns & Anti-Patterns

| Pattern | Good | Bad | Reason |
|---------|------|-----|--------|
| Error handling | `result?` | `result.unwrap()` | Graceful failure |
| Logging | `info!()` | `println!()` | Production use |
| Concurrency | DashMap | Mutex<HashMap> | Performance |
| Locks | parking_lot | std::sync | Speed |
| Tests | Unit + integration | Only mocks | Real-world coverage |
| Comments | "Why reset?" | "Reset id" | Clarify intent |
| Types | i64 for ID | String for ID | Type safety |
| Ownership | Arc<DashMap> | Mutex<Box<Vec>> | Ergonomics |

---

## Version Control & Commits

### Commit Message Format
```
feat: Add exponential backoff for sync retries

Implement stateless time-based backoff in get_unsynced() to prevent
rapid-fire retries while allowing new logs through immediately.

Formula: delay = min(2^retry_count, 60) seconds
- New logs (retry_count=0): sync immediately
- Retries: exponential backoff, capped at 60s
- Prevents 99.9% of spam

Fixes #1234
```

### Commit Message Guidelines
- **feat:** New feature
- **fix:** Bug fix
- **perf:** Performance improvement
- **refactor:** Code restructuring
- **test:** Test additions/changes
- **docs:** Documentation
- **chore:** Build/dependency changes

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio async patterns](https://tokio.rs/)
- [SQLx type safety](https://sqlx.rs/)
- [DashMap documentation](https://docs.rs/dashmap/)

---

**Document Version:** 1.0
**Status:** ACTIVE
**Last Updated:** 2026-01-29
