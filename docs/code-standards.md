# Code Standards & Conventions

## 🦀 Rust (Backend)

### 1. Linter & Quality
- **Multi-Tenancy Isolation**: All tables containing tenant (workspace) data **MUST** include a `workspace_id` column. This is automatically enforced through `schema_isolation_tests.rs`. Never query data without filtering by `workspace_id`.
- **Zero-Warning Policy**: The project enforces a strict no-warning policy. Code must pass `cargo clippy -- -D warnings` before commit.
- **Test Module Placement**: Always place the `mod tests` block at the end of the file to avoid Clippy warnings about item ordering after test modules.
- **Type Complexity**: Avoid overly deep nested data types (e.g., `OnceLock<Arc<Mutex<HashSet<...>>>>`). Use `type` aliases to improve readability and maintainability.
- **SQLx Validation (Online Mode)**: The project prioritizes using a real Database (on RAM/tmpfs) to validate SQL queries directly during compilation. Maintaining the `.sqlx/` directory is no longer required for development and testing environments.
- **Method Size (Rule of 30)**: A function should not exceed 30 lines of code. If it's longer, consider splitting it into private methods or helper functions.
- **Single Responsibility (SRP)**: A function should perform only one task. The function name must accurately reflect the action (e.g., `validate_agent_token` instead of `check_agent`).

### 1.1. Error Handling Best Practices
- **NO `.unwrap()` in Production Code**: Avoid `.unwrap()` and `.expect()` in handler/service/repository code. Use `?` operator or explicit error handling instead.
- **Mandatory `AppError` Pattern**: All API handlers MUST return `AppResult<T>` (alias for `Result<T, AppError>`).
  ```rust
  // ✅ CORRECT - Unified error handling
  pub async fn my_handler(State(state): State<Arc<AppState>>) -> AppResult<Json<MyResponse>> {
      let data = service::get_data(&state.db).await?; // Auto-converts to AppError
      Ok(Json(data))
  }
  ```
- **Conversion with `?`**: `AppError` implements `From<sqlx::Error>`, so database errors can be propagated using `?`. For other error types, use `.map_err()`:
  ```rust
  let result = some_external_lib().map_err(|e| AppError::Internal(e.to_string()))?;
  ```
- **Error Response Format**: Centralized in `errors.rs`, ensuring all errors follow the `{ "error": { "code": "...", "message": "..." } }` schema.
- **Exception for Constants**: `.expect("message")` is acceptable for infallible constant values:
  ```rust
  // ✅ OK - midnight is always valid
  NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is valid")
  ```
- **DST/Timezone Handling**: Always handle `LocalResult` variants when converting local time:
  ```rust
  // ✅ CORRECT - handles DST gaps and ambiguities
  match tz.from_local_datetime(&naive) {
      chrono::LocalResult::Single(dt) => Ok(dt),
      chrono::LocalResult::Ambiguous(dt, _) => Ok(dt), // Pick first
      chrono::LocalResult::None => {
          // DST gap - shift forward
          let shifted = naive + Duration::hours(1);
          tz.from_local_datetime(&shifted).single().ok_or("DST error")
      }
  }
  ```

### 1.2. Database Transaction Patterns
- **Atomic Operations**: Use transactions when multiple DB operations must succeed or fail together:
  ```rust
  // ✅ CORRECT - atomic user registration
  let mut tx = pool.begin().await?;
  create_user_tx(&mut *tx, &user).await?;
  create_session_tx(&mut *tx, user.id).await?;
  tx.commit().await?;
  ```
- **Generic Executor Pattern**: Use `Executor` trait for functions that work with both pools and transactions:
  ```rust
  pub async fn create_user_tx<'e, E>(executor: E, user: &User) -> Result<(), sqlx::Error>
  where
      E: Executor<'e, Database = Postgres>,
  {
      sqlx::query("INSERT INTO users ...").execute(executor).await?;
      Ok(())
  }
  ```
- **Rollback on Error**: Explicitly rollback on error before returning (though Drop also handles this):
  ```rust
  if let Err(e) = some_operation(&mut *tx).await {
      let _ = tx.rollback().await;
      return Err(e);
  }
  ```

### 4. Agent-Specific Patterns (agent-main)

#### 4.1. Retry Logic & Exponential Backoff
- **Module**: `src/retry.rs` provides standardized retry utilities
- **Constants**:
  - `SIDECAR_MAX_RETRY_DELAY_SECS = 60` (capped maximum)
  - `SIDECAR_STABILITY_THRESHOLD_SECS = 30` (time before resetting retry counter)
  - `SIDECAR_MAX_RETRY_COUNT = 10` (maximum retry attempts)
- **Function**: `get_sidecar_retry_delay(retry_count: u32) -> Duration` with ±25% jitter
- **Usage Pattern**:
  ```rust
  if last_stable < STABILITY_THRESHOLD {
      retry_count += 1;
  } else {
      retry_count = 0; // Reset after stability window
  }
  let backoff = get_sidecar_retry_delay(retry_count);
  tokio::time::sleep(backoff).await;
  ```

#### 4.2. Database Query Patterns
- **FIFO Ordering**: Always use `ORDER BY id ASC` in log queries:
  ```rust
  // ✅ CORRECT
  SELECT * FROM attendance_logs WHERE is_synced = 0 ORDER BY id ASC

  // ❌ INCORRECT (may break ordering)
  SELECT * FROM attendance_logs WHERE is_synced = 0
  ```
- **Batch Operations**: Use `IN` clause for batch updates:
  ```rust
  UPDATE attendance_logs SET is_synced = 1 WHERE id IN (?, ?, ...)
  ```
- **Insert Strategy**: Don't set `updated_at` on INSERT; only set when marking in-flight:
  ```rust
  // ✅ CORRECT (created_at only)
  INSERT INTO attendance_logs (device_id, user_id, created_at) VALUES (...)

  // When marking in-flight:
  UPDATE attendance_logs SET updated_at = NOW() WHERE id = ?
  ```

#### 4.3. Semaphore-Based Concurrency
- **Pattern**: Use tokio `Semaphore` for bounded parallelism:
  ```rust
  let semaphore = Arc::new(Semaphore::new(max_workers));
  let mut tasks = vec![];
  for device in devices {
      let permit = semaphore.acquire().await;
      tasks.push(tokio::spawn(async move {
          let _guard = permit; // Held until task completes
          // Device polling work
      }));
  }
  futures::future::join_all(tasks).await;
  ```
- **Fair Scheduling**: Per-cycle acquire-release ensures no device starvation
- **Resource Cleanup**: Permit auto-releases on drop

#### 4.4. Sidecar Worker Management
- **Spawn Pattern**: Use `Command::new()` with piped stdout/stderr:
  ```rust
  let mut child = Command::new(&sidecar_path)
      .arg("--port").arg(port.to_string())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;
  ```
- **Cleanup**: Always call `child.kill()` before abandoning process handle
- **RPC Contract**: All workers must implement JSON-RPC 2.0 with these methods:
  - `common.ping()` - Heartbeat
  - `device.connect(device_id)` - Establish hardware connection
  - `device.get_logs()` - Fetch attendance logs
  - `device.get_time()` - Sync device time

#### 4.5. Duplicate Detection & Deduplication
The agent filters duplicate logs at the worker level using in-memory deduplication to reduce queue traffic and database operations:
- **Pattern**: Each worker maintains a `HashSet<String>` for the current session:
  ```rust
  let mut seen_logs: HashSet<String> = HashSet::new();

  for log in logs {
      let key = format!("{}:{}", attendance.user_id, attendance.timestamp);
      if seen_logs.insert(key) {
          // New log - send to queue
          let _ = log_tx.send((worker.name.clone(), attendance)).await;
      } else {
          // Duplicate - skip
          duplicate_count += 1;
      }
  }
  ```
- **Key Format**: `"user_id:timestamp"` (device_id is implicit per worker).
- **Memory Safety**: Approximate ~5MB per 50,000 logs; HashSet cleared on worker restart.
- **Logging**: Always log duplicate count when present: `"Fetched X logs (Y new, Z duplicates filtered)"`.
- **Testing**: Verify filter accuracy, memory bounds, and log output in worker tests.

#### 4.6. Signature Cache Deduplication (Agent-Main)
The main orchestrator uses hash-based signature caching for persistent duplicate protection with minimal memory overhead:
- **Data Structure**: `DashSet<u64>` for concurrent lock-free signature storage
- **Hash Function**: FxHash (rustc-hash) for fast non-cryptographic hashing
- **Signature Formula** (CRITICAL - must include ALL fields):
  ```rust
  fn compute_signature(
      device_id: &str,
      user_id: &str,
      timestamp: NaiveDateTime,
      punch: Option<i32>,     // REQUIRED (In/Out/Break)
      status: Option<i32>     // REQUIRED (Check-in/Check-out)
  ) -> u64 {
      let mut hasher = FxHasher::default();
      device_id.hash(&mut hasher);
      user_id.hash(&mut hasher);
      timestamp.and_utc().timestamp().hash(&mut hasher);
      punch.hash(&mut hasher);    // Prevents false positives
      status.hash(&mut hasher);   // Prevents false positives
      hasher.finish()
  }
  ```
- **Why punch+status**: Same employee can punch In/Out/Break at exact same timestamp - these MUST be distinct logs
- **Memory Savings**: 8 bytes per signature vs ~200 bytes per full log (96% reduction)
- **Persistence**: Signatures remain in memory even after logs are synced/cleaned up
- **Startup**: Two-phase loading (Phase A: all signatures, Phase B: unsynced queue)
- **Testing**: Validate no false positives when punch/status differ at same timestamp

#### 4.6. Memory Leak Detection (Testing)
- **Test Module**: `tests/memory_leak_detection_test.rs`
- **Strategy**: Spawn 1000+ iterations of channel/semaphore operations
- **Validation**: Monitor process memory before/after using `/proc/self/status` on Linux
- **Tolerance**: Assert `final_rss < initial_rss + 10MB` to catch gradual leaks
- **Example**:
  ```rust
  #[tokio::test]
  async fn test_channel_leak_detection() {
      let initial_rss = get_rss_mb();
      for _ in 0..1000 {
          let (tx, rx) = mpsc::channel(100);
          drop(tx); drop(rx); // Simulate usage
      }
      let final_rss = get_rss_mb();
      assert!(final_rss < initial_rss + 10.0, "Memory leak detected");
  }
  ```

### 2. Architecture Layers (Handler-Service-Repository)
To ensure flexibility and testability (Mockable), the project follows a 3-layer model:
- **Handlers (API Layer)**: Only extracts parameters from Request, calls Service, and returns Response.
- **Services (Domain Layer)**: Contains business logic. Services receive Repositories via Dependency Injection (DI) to interact with data.
- **Repositories (Data Layer)**: Defines Traits (using `#[async_trait]`) to abstract data querying (SQLx, In-memory, etc.).

### 2.1. Database Pagination Pattern
For optimal performance, pagination MUST be implemented at the database level, not in-memory:

**Anti-pattern (DO NOT USE):**
```rust
// ❌ INCORRECT - Loads all data then paginates in memory
let all_items = repository::get_all(pool).await?;
let total = all_items.len();
let data = all_items.into_iter().skip(offset).take(limit).collect();
```

**Correct Pattern:**
```rust
// ✅ CORRECT - Pagination at DB level
let (items, total) = repository::get_paginated(pool, page, limit).await?;
// Returns only requested page + total count in single query
```

**Implementation Example:**
```rust
pub async fn get_paginated_employee_ids(
    pool: &PgPool,
    workspace_id: Uuid,
    filter_ids: Option<&Vec<Uuid>>,
    page: i64,
    limit: i64,
    sort_desc: bool,
) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
    // Count query
    let total: i64 = query_scalar(
        "SELECT COUNT(*) FROM employees
         WHERE workspace_id = $1 AND is_active = true
         AND ($2::uuid[] IS NULL OR id = ANY($2))"
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .fetch_one(pool)
    .await?;

    // Data query with LIMIT/OFFSET at DB level
    let offset = (page - 1) * limit;
    let employees = query_as::<_, EmployeeBasicInfo>(
        "SELECT id, full_name FROM employees
         WHERE workspace_id = $1 AND is_active = true
         AND ($2::uuid[] IS NULL OR id = ANY($2))
         ORDER BY full_name ASC
         LIMIT $3 OFFSET $4"
    )
    .bind(workspace_id)
    .bind(filter_ids)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((employees, total))
}
```

**Benefits:**
- Memory: O(L) instead of O(N) where L=limit, N=total records
- Performance: Only fetches requested page from database
- Scalability: Handles large datasets without memory issues

### 2.2. Batch Fetching Pattern (N+1 Query Elimination)
To eliminate N+1 query problems, use batch fetching with HashMap returns:

**Anti-pattern (N+1 Query Problem):**
```rust
// ❌ INCORRECT - N queries in loop
for employee in employees {
    let entries = get_employee_time_entries(pool, employee.id).await?;
    process(entries);
}
```

**Correct Pattern:**
```rust
// ✅ CORRECT - Single batch query
let employee_ids: Vec<Uuid> = employees.iter().map(|e| e.id).collect();
let entries_map = get_workspace_time_entries_batch(pool, &employee_ids).await?;

for employee in employees {
    let entries = entries_map.get(&employee.id).cloned().unwrap_or_default();
    process(entries);
}
```

**Implementation Example:**
```rust
pub async fn get_workspace_time_entries_batch(
    pool: &PgPool,
    employee_ids: &[Uuid],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<HashMap<Uuid, Vec<TimeEntry>>, sqlx::Error> {
    if employee_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let entries = query_as::<_, TimeEntry>(
        "SELECT * FROM time_entries
         WHERE employee_id = ANY($1)
         AND punch_in_time >= $2 AND punch_in_time <= $3
         ORDER BY employee_id, punch_in_time DESC"
    )
    .bind(employee_ids)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    // Group by employee_id for O(1) lookup
    let mut result: HashMap<Uuid, Vec<TimeEntry>> = HashMap::new();
    for entry in entries {
        result.entry(entry.employee_id).or_default().push(entry);
    }

    Ok(result)
}
```

**Benefits:**
- Queries: O(1) instead of O(N)
- Performance: Single database roundtrip
- Lookup: O(1) via HashMap

### 2.3. Composite-Key HashMap Pattern (Multi-Dimensional Indexing)
For scenarios where lookups require multiple key components (e.g., pattern_id + day_of_week), use composite tuples as HashMap keys to achieve O(1) lookups:

**Pattern:**
```rust
// ✅ CORRECT - Composite key for multi-dimensional lookups
let details_map: HashMap<(Uuid, i32), Option<Uuid>> = query_results
    .into_iter()
    .map(|row| ((row.pattern_id, row.day_of_week), row.shift_id))
    .collect();

// O(1) lookup with compound key
if let Some(&shift_opt) = details_map.get(&(pattern_id, dow)) {
    // shift_opt = Some(uuid) or None (day off)
}
```

**Anti-pattern (DO NOT USE):**
```rust
// ❌ INCORRECT - Nested iteration for compound lookups
let shift = query_results.iter()
    .find(|d| d.pattern_id == pattern_id && d.day_of_week == dow)
    .map(|d| d.shift_id); // O(N) per lookup
```

**Semantics Note**: Use `Option<T>` as the HashMap value to distinguish:
- **Key exists with None**: Entity mapped to "day off" (explicit null, must return early)
- **Key missing**: No mapping found (fall through to next priority level)

**Use Case**: Schedule resolution where inner loops repeatedly query pattern details across multiple (pattern_id, day_of_week) combinations. Pre-fetching and indexing eliminates per-lookup DB queries or linear scans.

### 3. Dependency Injection
Use `Arc<dyn RepositoryTrait>` to inject into Services. Initialize all dependencies in `main.rs` and pass them into `AppState`.

## 🐍 Python (Worker Agents)

### 1. Environment Management
- **Venv Required**: Always use a virtual environment (`venv`) for development and testing. The standard directory is `venv/` within the module.
- **Dependencies**: Managed via `requirements.txt`.
- **Formatting**: Use `black` or `ruff` to format code.

### 2. Testing
- **Pytest**: Use `pytest` combined with `pytest-asyncio` for asynchronous tasks.
- **Execution**: Always run tests via the virtual environment: `source venv/bin/activate && pytest`.

## 🐍 Python (Worker Agents)

### 1. Environment Management (Linux Priority)
- **Linux Venv**: Mandatory use of `python3 -m venv venv`. Activate with `source venv/bin/activate`.
- **Windows Note**: Avoid sharing the `venv` directory between Linux and Windows due to differences in script structures (`bin/` vs `Scripts/`) and library paths.
- **Dependencies**: Always update `requirements.txt` after installing new packages.

### 2. Testing & Execution
- **Pytest**: Run tests in the activated environment: `pytest -v`.
- **Hardware Access**: On Linux, verify access permissions for Serial/USB ports (typically belonging to the `dialout` group).

## Testing Standards

### 🧪 Unified Test Orchestrator (`run-tests.sh`)

To ensure high-quality delivery and fast feedback loops, the project uses a unified test orchestrator. This tool handles environment setup (Postgres in Docker), parallel execution, and result aggregation.

#### ⚠️ WSL Performance Protocol (CRITICAL)

> **This project runs on WSL with severe Disk I/O issues (1GB/s read causing system freeze).**

**Mandatory Rules:**
1. **NEVER** run `cargo test`, `cargo nextest`, `pytest`, or any test command directly on the host.
2. **Humans MUST** use `./run-tests.sh` for ALL testing requests (unit, integration, e2e). 
3. **AI Agents MUST** use `./ai-test.sh` (a PTY-safe wrapper) to prevent terminal hangs.
4. All tests run inside Docker containers to avoid WSL2 Disk I/O bottlenecks.

#### 1. Test Execution & Tools
- **Nextest**: The project uses `cargo nextest` for all Rust tests. It provides better parallelism, faster execution, and isolated process management.
- **Docker Isolation**: All tests (Unit & Integration) run inside Docker containers.
- **Internal Networking**: Database connections use internal hostnames (e.g., `test_db:5432`). Host port mapping is disabled.
- **Database Performance**: Test databases are tuned for speed with `fsync=off`, `synchronous_commit=off`, and `full_page_writes=off`.

#### 2. SQLx & Online Validation
- **Container-First Setup**: All migrations happen inside the `apiserver_test` container against a live database.
- **Online Compilation**: Host-side pre-compilation and test execution uses `SQLX_OFFLINE=false` to ensure queries are validated against the actual database schema. This eliminates the risk of outdated cache files.

#### 3. Logs & Debugging
- **Memory-Backed Logs**: All logs are written to `.test_logs/` (backed by RAM).
- **Container Logs**: Use `docker compose logs` or inspect `.test_logs/` to debug failures within the Docker environment.

### 🚀 Mandatory Passing Tests
- **Zero Tolerance**: All tests (Unit tests, Integration tests, Frontend tests) **MUST** pass before any commit or merge command. Code with failing tests will not be accepted.
- **Automated Linting (Phase 0)**: The system automatically executes code quality checks (Linting) in the first step of the pipeline.
    - **Rust**: `cargo clippy -- -D warnings` (Deny all warnings).
    - **Frontend**: `npm run lint` (ESLint with cache) and `npm run type-check`.
    - Any warnings from the linter or type errors are treated as errors and will stop the pipeline immediately.
- **Unit Tests (Phase 1)**: 
    - **Vitest**: Timeouts 10s, `isolate: false` to speed up execution.
- **Recommended Execution**: 
  - For Humans:
    ```bash
    ./run-tests.sh
    ```
  - For AI Agents:
    ```bash
    ./ai-test.sh
    ```

### Integration Tests (Rust)

We use a **Dynamic Test Database** pattern to ensure test isolation and allow parallel execution.

1. **Isolation**: Every integration test file (or test case) MUST use `TestDatabase::new().await`. This creates a unique Postgres database for that specific session.
2. **Migrations**: The helper automatically runs `sqlx::migrate!()`. No manual setup required.
3. **Parallelism**: Do NOT use `#[serial]` unless absolutely necessary for non-database resources. Tests are safe to run in parallel.
4. **CPU Management**: To prevent system oversubscription during unified test runs, always limit test threads using `cargo test -- --test-threads $N` where $N$ matches the shared worker pool size.
5. **Cleanup**: Databases are automatically dropped on `Drop` via a detached background shell process. This allows the test to finish immediately while the database is cleaned up shortly after.
5. **Signal Safety**: A global **Signal Trap** (SIGINT/SIGTERM) is initialized on the first `TestDatabase` creation. If the test process is interrupted (e.g., Ctrl+C), it will attempt to clean up all currently active test databases.
6. **Robustness**: The cleanup logic terminates active database backends before dropping to prevent "database is being accessed by other users" errors.

### End-to-End Tests (Playwright)

For E2E stability and performance, we adhere to the **Isolated Sequential Environment** standard:

1. **Sequential Execution (Rule of 1)**: In Docker or restricted environments, Playwright **MUST** run with exactly one worker (`--workers 1`). This eliminates CPU/RAM contention and race conditions in Mock API intercepts.
2. **Mock API Architecture**: Frontend E2E tests are 100% decoupled from the backend using Playwright's `page.route`. All API responses must be mocked in `tests/mocks/api-intercept.ts`.
3. **Architecture (POM & Fixtures)**:
    - **Page Object Model (POM)**: BẮT BUỘC đóng gói selector và hành động vào các class trong `tests/pom/` (ví dụ: `LoginPage`, `EmployeesPage`).
    - **Custom Fixtures**: Sử dụng Playwright Fixtures để inject Page Objects vào test. Ưu tiên sử dụng `authPage` để tự động login cho các test yêu cầu quyền admin.
4. **Hybrid Locators (Priority Order)**: 
    - **Level 1: Data-TestID (Primary Strategy)**: **MUST** use `page.getByTestId('sidebar-link-members')` for navigation components, primary actions, and page headers. This is the only way to ensure tests don't fail when language (i18n) changes or CSS is refactored.
        - **Standard Naming Convention**: `[context]-[sub-element]-[type/action]`
        - **Examples**: `sidebar-link-employees`, `login-email-input`, `employees-btn-edit`, `employees-row-123`.
        - **Format**: Always use `kebab-case`. Avoid dynamic IDs unless specifically needed to identify a specific record (use DB ID).
    - **Level 2: Semantic Attributes**: Use `page.locator('input[name="email"]')` or `getByPlaceholder`.
    - **Level 3: Stable Roles**: Use `getByRole('button', { name: 'submit' })` if the `type="submit"` attribute is stable.
    - **Avoid**: Absolutely avoid using `getByText(/Regex/i)` for critical elements as localization changes (copy) will frequently break tests.
5. **Navigation Reliability**: 
    - Prioritize using `page.goto('/target-path')` in regression tests to bypass sidebar animations or nested menus, making tests faster and more stable.
6. **Resilient Timeouts**: 
    - **Global Test Timeout**: 90 giây (Docker environment overhead).
    - **Expect Timeout**: 10 giây.
    - Các test phức tạp có thể sử dụng `test.slow()` để nhân 3 timeout mặc định.

### Unit Testing Standards (Frontend)

To ensure tests pass across different locales and environments:

1. **Multi-language Matchers**: Always use regex for text matching that supports both English and Vietnamese.
   - ❌ `getByText("Submit")`
   - ✅ `getByText(/Submit|Gửi/i)`
2. **Stable Date Logic**: Always use `en-GB` with `hourCycle: h23` for ISO date string conversions to avoid "24:00" vs "00:00" discrepancies.
3. **Query Mocks**: Mock all `useQuery` dependencies (like `fetchEmployees`) even if they aren't the primary focus of the test, to avoid unexpected loading states or "No Data" messages.
4. **Bail Policy**: Vitest is configured with `bail: 1` to fail fast and save CI resources.

### API Standardization (Frontend)

To prevent security leaks and ensure reliable connectivity, all frontend API calls MUST follow these rules:

1. **Centralized Wrapper**: Use `apiFetch` from `@/lib/api/helpers` instead of the native `window.fetch`.
2. **Auto-Headers**: `apiFetch` automatically attaches `Authorization` (JWT), `X-Workspace-Id` (from localStorage), and `Content-Type: application/json`.
3. **Validation**: API implementations in `src/lib/api` are protected by a regression test (`standardization.test.ts`) that forbids direct `fetch()` usage.
4. **Timeout Enforcement**: All requests inherit a 10s timeout via `AbortController` implemented in the helper.

**Example Pattern:**
```typescript
// ✅ CORRECT
import { apiFetch } from "./helpers";
const response = await apiFetch(`${API_BASE_URL}/my-endpoint`);

// ❌ INCORRECT (Forbidden)
const response = await fetch(`${API_BASE_URL}/my-endpoint`); 
```

### End-to-End Tests (Playwright)

**Example Pattern:**
```rust
#[tokio::test]
async fn my_feature_test() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    // ... test logic ...
}
```

### Mocking vs Fakes
- Prefer **Fakes** (like `TestFactory` for DB data) over Mocks.
- Use `ZKService_mock.py` for external hardware simulation.


### ⏱️ Time-Tracking Component Architecture

#### Component Structure
Time-tracking functionality is organized into modular, reusable components under `src/components/time-tracking/`:

1. **PunchCard.tsx**: Primary UI for active time session
   - Props: `activeEntry`, `isOnBreak`, `userName`, `onPunchIn`, `onPunchOut`, `onRequestAdjustment`, `isPunching`
   - Displays: Status indicator, punch timestamps, action buttons (Punch In/Out, Start/End Break)
   - No internal state: Receives state and callbacks from parent hook

2. **TimeEntryRow.tsx**: Reusable table row for time entry display
   - Props: `entry` (TimeEntry), `onAdjust?` (callback), `showAdjustButton?` (boolean)
   - Displays: Entry date, type badge (Work/Break), punch-in/out times with status indicator
   - Supports optional adjustment button with hover reveal pattern

3. **AdjustmentRequestModal.tsx**: Form for submitting attendance corrections
   - Props: `isOpen`, `onClose`, `onSubmit`, `timezone`
   - Features: Date picker, time inputs, reason textarea, UTC conversion helper
   - Validates punch-in before punch-out chronologically

#### Hook Integration
- **useMyClock()**: Central state management for clock operations
   - Fetches 7-day time entries with 30s auto-refresh (prevents stale status)
   - Exposes: `entries[]`, `activeEntry`, `isOnBreak`, `punchIn()`, `punchOut()` methods
   - Auto-invalidates React Query on punch mutations for UI sync

#### Data Flow
```
MyClockPage (Page)
  ↓
useMyClock() hook → React Query (30s refresh)
  ↓
PunchCard + TimeEntryRow (UI Components)
  ↓
AdjustmentRequestModal (Form)
  ↓
submitAttendanceAdjustment() API
```

#### Reusability Patterns
- **PunchCard**: Can be embedded in admin dashboards to display employee clock status
- **TimeEntryRow**: Used in reports and activity feeds for consistent time entry rendering
- **AdjustmentRequestModal**: Can be triggered from admin interfaces or automated alert systems

---

### 🔢 Integer Work Unit (WU) System

To ensure absolute precision (no floating-point errors) and preserve historical data, the system adheres to the following principles:

1.  **Atomic Unit**: The smallest unit for work and payroll calculation is the **Work Unit (Integer)**. Never store or calculate hours as decimal numbers (floats) at the Service/Database layer.
2.  **Calculation Rule**: 
    - `Minutes / work_unit_minutes = Units`. 
    - The Workspace's `rounding_policy` must be applied at the moment of conversion to Units.
3.  **Temporal Integrity (Snapshotting)**: 
    - When finalizing payroll, you **MUST** snapshot the unit counts (`standard_units`, `overtime_units`) into the `payslips` table. 
    - Never recalculate units from raw logs when viewing old payslips, as Workspace configurations (unit length, rounding rules) may have changed over time.
4.  **UI Representation**: 
    - The frontend displays `Decimal Hours` for UX: `Hours = Units * (work_unit_minutes / 60)`.
    - However, during data entry or detailed viewing, raw Unit counts should also be displayed for reconciliation.

---

## ⚛️ React (Frontend)

### 1. Development
- **Vite**: Use Vite as the build tool.
- **Server State (React Query)**: Use `@tanstack/react-query` to manage Server State. All fetching, caching, and mutation logic must reside in **Domain-specific Custom Hooks** (e.g., `useAgents`, `useDevices`).
- **Component Design**: Components should only be responsible for display (UI) and receiving user events. Avoid using `useEffect` and `useState` manually for data fetching.
- **Tailwind CSS 4**: Use Tailwind 4 utilities for styling.
- **Lucide React**: Standard icon library.

### 📅 Effective Schedule Logic

When calculating an employee's effective shift at a given time, you **MUST** follow this 5-tier priority order:

1.  **Tier 1: Manual Roster** - `shift_rosters`. Record assigned to a specific date. If assigned as NULL, it means the employee is OFF (Day Off) regardless of lower tiers.
2.  **Tier 2: Individual Working Pattern** - `employee_working_patterns`. Recurring schedule pattern assigned to an employee for a period.
3.  **Tier 3: Department Working Pattern** - Default schedule pattern of the department the employee currently belongs to.
4.  **Tier 4: Fixed Assignment (Legacy)** - `employee_shifts`. Use records with a matching `start_date` and the latest `created_at`.
5.  **Tier 5: Department Default Shift** - The final default shift of the department.

**Note**: Never calculate this logic on the Frontend. Always use the `/api/workspaces/:id/schedules` API to retrieve pre-calculated results from the Backend.

---

### 🔍 Unified Search Pattern

To ensure a consistent user experience, employee searching must follow this design pattern:

1.  **Component**: Always use the `EmployeeSearch` (Combobox) component instead of simple text inputs.
2.  **Behavior**:
    - **Debounced Remote Search**: Server-side search with a 400ms delay for performance optimization.
    - **Offline Result Shadowing**: Disable the client library's automatic filtering (like `cmdk`) to accurately display results from the Server (especially when searching by employee ID).
    - **Label Persistence**: Must capture and store the label (employee name) of the selected item to display correctly even if that item is no longer in the current search result set.
3.  **Backend API**: Always use the `/workspaces/:ws_id/employees?q=...` endpoint, which supports case-insensitive searching on First Name, Last Name, and Employee ID.

---

### 🌐 Workspace Identification (Path Priority)

The system prioritizes organization identification based on the URI for clarity and compatibility:

1.  **Path over Header**: Prioritize using the `:ws_id` or `:id` parameter in the URL.
2.  **Client Responsibility**: The frontend must ensure the ID is passed in the request path. The `apiFetch` utility automatically supports this when combined with `WorkspaceGuard`.
3.  **Optional Header**: The `X-Workspace-Id` header is for backup only and is not mandatory for routes that already include the ID in the path.

---

### 🎨 Frontend: Notifications (Toasts)
- **No Browser Dialogs**: Absolutely **DO NOT** use `window.alert()`, `window.confirm()`, or `window.prompt()` for routine action notifications.
- **Global Toast**: Use the custom `useToast()` hook to display notifications.
  - `toast.success()`: On successful actions (Save, Update).
  - `toast.error()`: On API errors or critical logic errors.
  - `toast.warning()`: When user input is missing data.
- **Transitions**: Use Tailwind `animate-in` for all Modals and Toast items to create a smooth experience.

### 🕒 Date & Time Display
- **Standardized Formatter**: Absolutely **DO NOT** use `toLocaleDateString()`, `toLocaleTimeString()`, or hardcoded format functions from external libraries directly in Components.
- **Unified Hook**: Always use the custom `useDateTimeFormatter()` hook for displaying time. This hook automatically retrieves the `date_format` and `time_format` settings of the current Workspace.
  - `formatDate(date)`: Displays date (e.g., 14/01/2026).
  - `formatTime(date)`: Displays time (e.g., 08:30 or 08:30 AM).
  - `formatDateTime(date)`: Displays both date and time.
  - `formatMonthYear(date)`: Displays month and year (e.g., 01/2026 or 2026-01) depending on workspace config.

- **Internal/API Format**: 
  - **MANDATORY** use `format(date, 'yyyy-MM-dd')` from `date-fns` when needing a date string for the API or internal state. 
  - **FORBIDDEN** use `date.toISOString().split('T')[0]` because this method converts to UTC, causing date shifts in positive timezones (like Vietnam UTC+7) during early morning hours.

---

## 🛡️ Security

- **Multi-Tenancy**: Always check `workspace_id` in every query.
- **RBAC**: Use `MemberContext` to verify permissions. Avoid hardcoded role checks at the handler level if a common permission logic can be used.

### Security Anti-Patterns (Rust Backend)

The following patterns are **FORBIDDEN** and enforced by Clippy lints in `Cargo.toml`:

| Anti-Pattern | Why Dangerous | Correct Alternative |
|---|---|---|
| `.unwrap()` in handler/service/repo | Panics crash the server | Use `?` or `.map_err()` |
| `let _ = db_operation.await` | Silently swallows DB errors | Use `?` or `if let Err(e) = ... { tracing::error!(...) }` |
| Query without `workspace_id` | Cross-tenant data leak | Always include `WHERE workspace_id = $N` |
| `limit.unwrap_or(20)` without clamp | Memory DoS with limit=999999 | `limit.unwrap_or(20).clamp(1, 100)` |
| Password without length check | bcrypt DoS with >1MB input | Validate 8-128 chars before hashing |
| Unbounded file upload | Memory DoS (OOM crash) | Use `field.bytes().await` with `MAX_FILE_SIZE` (e.g. 10MB) |
| Public ID-based command | Unauthorized control (Broken Auth) | Use `MemberContext` and verify ID belongs to `workspace_id` |
| TCP `readline()` | Memory DoS on long lines | Use `readuntil(b'\n')` with `limit` (e.g. 64KB) |

### 2.4. Frontend Performance: O(1) Lookup Pattern
Khi hiển thị danh sách lớn (ví dụ: Logs) cần tham chiếu thông tin từ danh sách khác (ví dụ: Employees), **KHÔNG** sử dụng `.find()` bên trong vòng lặp render (độ phức tạp O(N*M)).

**Correct Pattern:**
```typescript
// ✅ CORRECT - Use useMemo to create a Map for O(1) lookup
const lookupMap = useMemo(() => {
  const map = new Map<string, Entity>();
  items.forEach(item => map.set(item.id, item));
  return map;
}, [items]);

// Inside render loop:
const name = lookupMap.get(id)?.name;
```

**Anti-pattern:**
```typescript
// ❌ INCORRECT - O(N) lookup inside O(M) loop
{logs.map(log => (
  <div key={log.id}>
    {employees.find(e => e.id === log.employee_id)?.name}
  </div>
))}
```

**Compile-Time Enforcement** (`apiserver/Cargo.toml`):
```toml
[lints.clippy]
unwrap_used = "deny"  # Blocks .unwrap() at compile time (allowed in tests via #![cfg_attr])
```
**AI-Enforced** (code review): `let _ =` on critical DB ops should use error logging instead.

---

## 22. Real-Time Features & WebSocket Standards (NEW - 2026-02-03)

### 22.1. WebSocket Implementation Requirements

**Always Provide Fallback:**
- WebSocket connections MUST have an automatic fallback mechanism
- After 3 connection failures, switch to HTTP polling (or equivalent)
- User MUST be notified when using fallback mode (e.g., amber notification banner)
- Fallback ensures 100% reliability (zero feature failures)

**Retry Strategy:**
```typescript
const RETRY_DELAYS = [1000, 2000, 4000]; // Exponential backoff
const MAX_RETRIES = 3;
```

**Message Protocol:**
- Use discriminated union types (JSON with `type` field)
- Message types: `CHUNK`, `DONE`, `ERROR`, `ACK`
- Include timestamp and job/request ID for traceability

**Example Rust Implementation:**
```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ReportStreamMessage {
    #[serde(rename = "CHUNK")]
    Chunk {
        job_id: Uuid,
        batch_index: usize,
        total_batches: usize,
        progress_percentage: u32,
    },
    #[serde(rename = "DONE")]
    Done {
        job_id: Uuid,
        download_url: String,
    },
    #[serde(rename = "ERROR")]
    Error {
        job_id: Uuid,
        error_message: String,
    },
}
```

**Example React Hook Implementation:**
```typescript
export function useReportStream(options: UseReportStreamOptions) {
    const [status, setStatus] = useState<'idle' | 'connecting' | 'streaming' | 'completed' | 'error'>('idle');
    const [progress, setProgress] = useState(0);
    const [chunks, setChunks] = useState<ReportChunk[]>([]);
    const [error, setError] = useState<string | null>(null);
    const [retryCount, setRetryCount] = useState(0);

    // Auto-fallback after 3 failed retries
    useEffect(() => {
        if (retryCount >= 3) {
            // Switch to HTTP polling
            switchToPollingMode();
        }
    }, [retryCount]);

    // Return reconnect function for user-initiated retries
    return { status, progress, chunks, error, reconnect };
}
```

### 22.2. Frontend Testing Standards for Real-Time Features

**Test Requirements:**
- Happy path: Connection established, messages received in order
- Fallback path: WebSocket fails, HTTP polling takes over
- Retry logic: Exponential backoff with jitter (no thundering herd)
- Message parsing: Invalid JSON or unknown types handled gracefully
- Connection cleanup: No memory leaks on disconnect/reconnect

**Example E2E Test:**
```typescript
test('Report export with WebSocket fallback', async ({ page }) => {
    // Navigate to export dialog
    await page.goto('/employees');
    await page.click('button:has-text("Export Report")');

    // Simulate WebSocket connection failure
    await page.context().route('**/ws/reports/**', route => {
        route.abort('failed'); // Simulate WebSocket failure
    });

    // Submit export - should trigger fallback
    await page.click('[data-testid="report-export-start-button"]');

    // Wait for fallback notice
    const notice = await page.waitForSelector('[data-testid="report-export-fallback-notice"]');
    expect(notice).toContainText('Real-time updates unavailable');

    // Continue polling and eventually complete
    const download = await page.waitForSelector('[data-testid="report-export-download-button"]');
    expect(download).toBeVisible();
});
```

### 22.3. Data-TestID Standards for Real-Time Components

**Naming Convention for WebSocket/Real-Time Features:**
```
[component-name]-[sub-element]-[type/action]

Examples:
"report-export-dialog"
"report-export-progress-bar"
"report-export-progress-fill"
"report-export-batch-counter"
"report-export-status-text"
"report-export-start-button"
"report-export-cancel-button"
"report-export-download-button"
"report-export-retry-button"
"report-export-fallback-notice"
"report-export-close-button"
```

**Kebab-Case Always:**
```typescript
// ✅ CORRECT
<div data-testid="report-export-dialog">

// ❌ INCORRECT
<div data-testid="reportExportDialog">
<div data-testid="report_export_dialog">
<div data-testid="ReportExportDialog">
```

### 22.4. Error Handling for Real-Time Features

**Never Silent Failures:**
```typescript
// ❌ INCORRECT - Silent failure
socket.onerror = () => {
    // log only, no user notification
};

// ✅ CORRECT - User-visible error
socket.onerror = (error) => {
    toast.error('Connection failed. Switching to polling mode.');
    switchToFallback();
};
```

**Graceful Degradation Pattern:**
```typescript
// YAGNI: Don't try to be too clever
// Keep fallback logic simple and reliable

export async function withFallback<T>(
    primary: () => Promise<T>,
    fallback: () => Promise<T>,
    maxRetries: number = 3
): Promise<T> {
    for (let i = 0; i < maxRetries; i++) {
        try {
            return await primary();
        } catch (error) {
            if (i === maxRetries - 1) {
                // All retries exhausted, use fallback
                return await fallback();
            }
            const delay = Math.pow(2, i) * 1000;
            await new Promise(resolve => setTimeout(resolve, delay));
        }
    }
    throw new Error('All retry attempts exhausted');
}
```

---

