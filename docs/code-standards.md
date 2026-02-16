# Code Standards & Conventions

## 🦀 Rust (Backend)

### 1. Linter & Quality
- **Multi-Tenancy Isolation**: Tất cả các bảng chứa dữ liệu của tenant (workspace) **PHẢI** bao gồm cột `workspace_id`. Điều này được thực thi tự động thông qua `schema_isolation_tests.rs`. Không bao giờ truy vấn dữ liệu mà không lọc theo `workspace_id`.
- **Zero-Warning Policy**: Dự án thực thi chính sách không cảnh báo nghiêm ngặt. Code phải vượt qua `cargo clippy -- -D warnings` trước khi commit.
- **Test Module Placement**: Luôn đặt khối `mod tests` ở cuối tệp tin để tránh cảnh báo Clippy về việc sắp xếp các mục sau module test.
- **Type Complexity**: Tránh các kiểu dữ liệu lồng nhau quá sâu (ví dụ: `OnceLock<Arc<Mutex<HashSet<...>>>>`). Hãy sử dụng `type` alias để cải thiện khả năng đọc và bảo trì.
- **SQLx Validation (Online Mode)**: Dự án ưu tiên sử dụng Database thật (trên RAM/tmpfs) để kiểm tra lỗi câu lệnh SQL trực tiếp trong quá trình biên dịch. Không còn yêu cầu duy trì thư mục `.sqlx/` cho môi trường phát triển và kiểm thử.
- **Method Size (Rule of 30)**: Một hàm không nên dài quá 30 dòng code. Nếu dài hơn, hãy cân nhắc tách nhỏ ra các private methods hoặc helper functions.
- **Single Responsibility (SRP)**: Một hàm chỉ làm một việc duy nhất. Tên hàm phải phản ánh đúng hành động (ví dụ: `validate_agent_token` thay vì `check_agent`).

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
Để đảm bảo tính linh hoạt và dễ kiểm thử (Mockable), dự án tuân thủ mô hình 3 lớp:
- **Handlers (API Layer)**: Chỉ trích xuất tham số từ Request, gọi Service và trả về Response.
- **Services (Domain Layer)**: Chứa logic nghiệp vụ. Services nhận các Repositories thông qua Dependency Injection (DI) để tương tác với dữ liệu.
- **Repositories (Data Layer)**: Định nghĩa các Trait (sử dụng `#[async_trait]`) để trừu tượng hóa việc truy vấn dữ liệu (SQLx, In-memory, etc.).

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
Sử dụng `Arc<dyn RepositoryTrait>` để inject vào Services. Khởi tạo toàn bộ dependencies tại `main.rs` và đưa vào `AppState`.

## 🐍 Python (Worker Agents)

### 1. Environment Management
- **Venv Required**: Luôn sử dụng môi trường ảo (`venv`) để phát triển và chạy test. Thư mục chuẩn là `venv/` bên trong module.
- **Dependencies**: Quản lý qua `requirements.txt`.
- **Formatting**: Sử dụng `black` hoặc `ruff` để format code.

### 2. Testing
- **Pytest**: Sử dụng `pytest` kết hợp với `pytest-asyncio` cho các task bất đồng bộ.
- **Execution**: Luôn chạy test thông qua môi trường ảo: `source venv/bin/activate && pytest`.

## 🐍 Python (Worker Agents)

### 1. Environment Management (Linux Priority)
- **Linux Venv**: Bắt buộc sử dụng `python3 -m venv venv`. Kích hoạt bằng `source venv/bin/activate`.
- **Windows Note**: Tránh dùng chung thư mục `venv` giữa Linux và Windows do sự khác biệt về cấu trúc script (`bin/` vs `Scripts/`) và đường dẫn thư viện.
- **Dependencies**: Luôn cập nhật `requirements.txt` sau khi cài package mới.

### 2. Testing & Execution
- **Pytest**: Chạy test trong môi trường đã kích hoạt: `pytest -v`.
- **Hardware Access**: Trên Linux, cần kiểm tra quyền truy cập cổng Serial/USB (thường thuộc group `dialout`).

## Testing Standards

### 🧪 Unified Test Orchestrator (`run-tests.sh`)

To ensure high-quality delivery and fast feedback loops, the project uses a unified test orchestrator. This tool handles environment setup (Postgres in Docker), parallel execution, and result aggregation.

#### ⚠️ WSL Performance Protocol (CRITICAL)

> **This project runs on WSL with severe Disk I/O issues (1GB/s read causing system freeze).**

**Mandatory Rules:**
1. **NEVER** run `cargo test`, `cargo nextest`, `pytest`, or any test command directly.
2. **ALWAYS** use `./run-tests.sh` for ALL testing requests (unit, integration, e2e).
3. All tests run inside Docker containers to avoid WSL2 Disk I/O bottlenecks.

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
- **Zero Tolerance**: Tất cả các bài kiểm tra (Unit tests, Integration tests, Frontend tests) **BẮT BUỘC** phải vượt qua (PASS) trước khi thực hiện bất kỳ lệnh commit hoặc merge nào. Code có test thất bại sẽ không được chấp nhận.
- **Automated Linting (Phase 0)**: Hệ thống thực thi kiểm tra code quality tự động (Linting) ở bước đầu tiên của pipeline.
    - **Rust**: `cargo clippy -- -D warnings` (Deny all warnings).
    - **Frontend**: `npm run lint` (ESLint with cache) và `npm run type-check`.
    - Bất kỳ cảnh báo (warning) nào từ linter hoặc lỗi type đều bị coi là lỗi và sẽ dừng pipeline ngay lập tức.
- **Unit Tests (Phase 1)**: 
    - **Vitest**: Timeouts 10s, `isolate: false` để tăng tốc độ thực thi.
- **Recommended Execution**: Sử dụng script hợp nhất để chạy toàn bộ suite một cách nhanh nhất (hỗ trợ thực thi song song và tự động setup môi trường):
  ```bash
  ./run-tests.sh
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
    - **Level 1: Data-TestID (Primary Strategy)**: BẮT BUỘC sử dụng `page.getByTestId('sidebar-link-members')` cho các thành phần điều hướng, hành động chính và tiêu đề trang. Đây là cách duy nhất đảm bảo test không bị fail khi thay đổi ngôn ngữ (i18n) hoặc refactor CSS.
        - **Standard Naming Convention**: `[context]-[sub-element]-[type/action]`
        - **Examples**: `sidebar-link-employees`, `login-email-input`, `employees-btn-edit`, `employees-row-123`.
        - **Format**: Luôn sử dụng `kebab-case`. Tránh gán ID động trừ khi cần xác định cụ thể một bản ghi (dùng DB ID).
    - **Level 2: Semantic Attributes**: Sử dụng `page.locator('input[name="email"]')` hoặc `getByPlaceholder`.
    - **Level 3: Stable Roles**: Sử dụng `getByRole('button', { name: 'submit' })` nếu attribute `type="submit"` ổn định.
    - **Avoid**: Tuyệt đối tránh sử dụng `getByText(/Regex/i)` cho các element quan trọng vì nội dung dịch (copy) thay đổi thường xuyên sẽ làm gãy test.
5. **Navigation Reliability**: 
    - Ưu tiên sử dụng `page.goto('/target-path')` trong các test regression để bỏ qua các animation của sidebar hoặc menu lồng nhau, giúp test chạy nhanh và ổn định hơn.
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

### 🔢 Hệ thống Đơn vị công (Integer Work Unit - WU)

Để đảm bảo tính chính xác tuyệt đối (không sai số floating-point) và bảo toàn dữ liệu lịch sử, hệ thống tuân thủ các nguyên tắc sau:

1.  **Atomic Unit**: Đơn vị nhỏ nhất để tính toán công và lương là **Work Unit (Số nguyên)**. Tuyệt đối không lưu trữ hoặc tính toán số giờ dưới dạng số thập phân (float) ở tầng Service/Database.
2.  **Calculation Rule**: 
    - `Minutes / work_unit_minutes = Units`. 
    - Phải áp dụng quy tắc làm tròn (`rounding_policy`) của Workspace ngay tại thời điểm chuyển đổi sang Unit.
3.  **Temporal Integrity (Snapshotting)**: 
    - Khi chốt lương, **PHẢI** snapshot số lượng Unit (`standard_units`, `overtime_units`) vào bảng `payslips`. 
    - Tuyệt đối không tính toán lại số unit từ log thô khi xem lại phiếu lương cũ, vì cấu hình Workspace (độ dài unit, quy tắc làm tròn) có thể đã thay đổi theo thời gian.
4.  **UI Representation**: 
    - Frontend hiển thị số giờ (`Decimal Hours`) phục vụ UX: `Hours = Units * (work_unit_minutes / 60)`.
    - Tuy nhiên, khi nhập liệu hoặc hiển thị chi tiết, nên hiển thị kèm số lượng Unit thô để người dùng đối soát.

---

## ⚛️ React (Frontend)

### 1. Development
- **Vite**: Sử dụng Vite làm build tool.
- **Server State (React Query)**: Sử dụng `@tanstack/react-query` để quản lý Server State. Toàn bộ logic fetching, caching và mutation phải nằm trong **Domain-specific Custom Hooks** (ví dụ: `useAgents`, `useDevices`).
- **Component Design**: Component chỉ nên chịu trách nhiệm hiển thị (UI) và nhận sự kiện từ người dùng. Tránh sử dụng `useEffect` và `useState` thủ công để fetch dữ liệu.
- **Tailwind CSS 4**: Sử dụng các tiện ích của Tailwind 4 để styling.
- **Lucide React**: Thư viện icon chuẩn.

### 📅 Logic Lịch trình & Ca làm việc (Effective Schedule)

Khi tính toán ca làm việc hiệu lực của nhân viên tại một thời điểm, **PHẢI** tuân thủ thứ tự ưu tiên 5 cấp sau:

1.  **Mức 1: Lịch trực tuần (Manual Roster)** - `shift_rosters`. Bản ghi gán cho một ngày cụ thể. Nếu gán là NULL, nghĩa là nhân viên ĐƯỢC NGHỈ (Day Off) bất chấp các mức dưới.
2.  **Mức 2: Mẫu lịch cá nhân (Individual Working Pattern)** - `employee_working_patterns`. Mẫu lịch lặp lại gán cho nhân viên trong một khoảng thời gian.
3.  **Mức 3: Mẫu lịch phòng ban (Department Working Pattern)** - Mẫu lịch mặc định của phòng ban mà nhân viên đang thuộc về.
4.  **Mức 4: Gán cố định (Fixed Assignment - Legacy)** - `employee_shifts`. Lấy bản ghi có ngày hiệu lực (`start_date`) thỏa mãn và có thời điểm tạo (`created_at`) mới nhất.
5.  **Mức 5: Ca mặc định phòng ban (Department Default Shift)** - Ca làm việc mặc định cuối cùng của phòng ban.

**Lưu ý**: Tuyệt đối không tính toán logic này ở Frontend. Luôn sử dụng API `/api/workspaces/:id/schedules` để lấy kết quả đã tính toán từ Backend.

---

### 🔍 Tìm kiếm nhân viên (Unified Search Pattern)

Để đảm bảo trải nghiệm người dùng nhất quán, việc tìm kiếm nhân viên phải tuân thủ mẫu thiết kế sau:

1.  **Component**: Luôn sử dụng component `EmployeeSearch` (Combobox) thay vì các input text đơn giản.
2.  **Behavior**:
    - **Debounced Remote Search**: Tìm kiếm server-side với delay 400ms để tối ưu hiệu năng.
    - **Offline Result Shadowing**: Tắt tính năng tự động filter của thư viện client (như `cmdk`) để hiển thị chính xác kết quả từ Server (đặc biệt khi tìm theo mã nhân viên).
    - **Label Persistence**: Phải capture và lưu trữ label (tên nhân viên) của item đã chọn để hiển thị đúng ngay cả khi item đó không còn nằm trong tập kết quả search hiện tại.
3.  **Backend API**: Luôn sử dụng endpoint `/workspaces/:ws_id/employees?q=...` hỗ trợ tìm kiếm không phân biệt hoa thường (Case-insensitive) trên Tên, Họ, và Mã nhân viên.

---

### 🌐 Định danh Workspace (Path Priority)

Hệ thống ưu tiên định danh tổ chức dựa trên URI để tăng tính tường minh và khả năng tương thích:

1.  **Path over Header**: Ưu tiên sử dụng tham số `:ws_id` hoặc `:id` trong URL.
2.  **Client Responsibility**: Frontend phải đảm bảo truyền ID vào path của request. Utility `apiFetch` hỗ trợ việc này một cách tự động khi kết hợp với `WorkspaceGuard`.
3.  **Optional Header**: Header `X-Workspace-Id` chỉ mang tính dự phòng và không được coi là bắt buộc cho các routes đã có ID trong path.

---

### 🎨 Frontend: Thông báo (Toasts)
- **No Browser Dialogs**: Tuyệt đối **KHÔNG** sử dụng `window.alert()`, `window.confirm()` hoặc `window.prompt()` cho các thông báo hành động thông thường.
- **Global Toast**: Sử dụng custom hook `useToast()` để hiển thị thông báo.
  - `toast.success()`: Khi hành động thành công (Lưu, Cập nhật).
  - `toast.error()`: Khi có lỗi API hoặc lỗi logic nghiêm trọng.
  - `toast.warning()`: Khi người dùng nhập thiếu dữ liệu.
- **Transitions**: Sử dụng Tailwind `animate-in` cho tất cả các Modals và Toast items để tạo cảm giác mượt mà.

### 🕒 Date & Time Display
- **Standardized Formatter**: Tuyệt đối **KHÔNG** sử dụng `toLocaleDateString()`, `toLocaleTimeString()` hoặc các hàm format cứng của thư viện bên ngoài trực tiếp trong Component.
- **Unified Hook**: Luôn sử dụng custom hook `useDateTimeFormatter()` để hiển thị thời gian. Hook này tự động lấy cấu hình `date_format` và `time_format` của Workspace hiện tại.
  - `formatDate(date)`: Hiển thị ngày (ví dụ: 14/01/2026).
  - `formatTime(date)`: Hiển thị giờ (ví dụ: 08:30 hoặc 08:30 AM).
  - `formatDateTime(date)`: Hiển thị cả ngày và giờ.
  - `formatMonthYear(date)`: Hiển thị tháng và năm (ví dụ: 01/2026 hoặc 2026-01) tùy theo cấu hình workspace.

- **Internal/API Format**: 
  - **BẮT BUỘC** sử dụng `format(date, 'yyyy-MM-dd')` từ `date-fns` khi cần lấy chuỗi ngày cho API hoặc internal state. 
  - **CẤM** sử dụng `date.toISOString().split('T')[0]` vì phương thức này chuyển về UTC, gây lệch ngày ở các múi giờ dương (như Việt Nam UTC+7) vào thời điểm rạng sáng.

---

## 🛡️ Security

- **Multi-Tenancy**: Luôn kiểm tra `workspace_id` trong mọi query.
- **RBAC**: Sử dụng `MemberContext` để xác thực quyền hạn. Không hard-code check role ở tầng handler nếu có thể dùng chung logic kiểm tra quyền.

### Security Anti-Patterns (Rust Backend)

The following patterns are **FORBIDDEN** and enforced by Clippy lints in `Cargo.toml`:

| Anti-Pattern | Why Dangerous | Correct Alternative |
|---|---|---|
| `.unwrap()` in handler/service/repo | Panics crash the server | Use `?` or `.map_err()` |
| `let _ = db_operation.await` | Silently swallows DB errors | Use `?` or `if let Err(e) = ... { tracing::error!(...) }` |
| Query without `workspace_id` | Cross-tenant data leak | Always include `WHERE workspace_id = $N` |
| `limit.unwrap_or(20)` without clamp | Memory DoS with limit=999999 | `limit.unwrap_or(20).clamp(1, 100)` |
| Password without length check | bcrypt DoS with >1MB input | Validate 8-128 chars before hashing |
| `format!()` with user input in SQL | SQL injection | Use `sqlx::query!` with `$N` bind params |
| Check-then-act without row lock | Race condition / double-processing | Use `FOR UPDATE NOWAIT` in transaction |
| `(StatusCode, String)` error return | Bypasses centralized error handling | Return `AppResult<T>` with `AppError` |

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

