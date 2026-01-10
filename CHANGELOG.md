# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

## [Unreleased]

### Added
- **Dynamic Leave Accrual Policies**: Implemented flexible leave accrual rules (Monthly, Quarterly, Yearly) with temporal assignment capabilities.
    - Added `leave_policy_definitions` table to store parametric rules (JSON).
    - Added `leave_policy_assignments` table to manage policy application over time (Effective Dating).
    - Added `leave_accrual_logs` to audit all system-generated leave additions.
    - Implemented `LeaveAccrualService` in Rust with support for seniority bonuses and pro-rating logic.
    - Added new API endpoints:
        - `POST /workspaces/:ws_id/leave/policies`: Create new policy definitions.
        - `POST /workspaces/:ws_id/leave/policies/assign`: Assign policies to employees/departments.
        - `POST /workspaces/:ws_id/leave/accrual/run`: Manually trigger accrual calculation (for Cron/Debug).
- **Testing Infrastructure:** Dynamic Test Database helper for Rust `apiserver`.
- **Auto-Cleanup:** Background thread logic to drop test databases after execution.
- **Test Factory:** Improved `TestFactory` with more builders for Payroll and Shifts.

### Fixed
- **Flaky Tests:** Resolved database contention issues in `shift_break_config_tests` and `work_unit_rounding_tests`.
- **Test Isolation:** Integration tests now run in complete isolation from the main development database.
- **Cleanup:** Fixed database "leak" where test databases were not being deleted.


### Changed
- **Payroll Service**: Switched from UTC-based grouping to Local-date grouping for better compliance with business workday definitions.
- **Reporting UI**: Integrated a Tab-based interface to toggle between Monthly Summary and Daily Detail reports.

### Fixed
- **Timezone Offsets**: Resolved a bug where test data generated in UTC caused inconsistent daily rollups in the Payroll Engine.
- **Unused Assignments**: Cleared compiler warnings in `PayrollService` to ensure clean build logs.

## [Unreleased] - 2026-01-10

### Added
- **RBAC Enforcement**: Secured all administrative endpoints (Employee/Shift/Roster management) with strict `MemberContext` role checks. Only `owner` and `admin` roles can now modify workspace resources.
- **Comprehensive Integration Tests**: Added 5 new test suites covering:
    - **Leave Management**: Full lifecycle from submission to approval and balance tracking.
    - **Attendance Adjustments**: Request submission, approval flows, and time entry generation.
    - **Employee Management**: CRUD operations and Department transfer history.
    - **Department Management**: Creation and hierarchy logic.
    - **RBAC Verification**: Explicit tests ensuring non-admins receive `403 Forbidden` on protected routes.
- **Test Infrastructure**: Added `Serialize` support to key request models (`SubmitLeaveRequest`, `CreateAttendanceAdjustmentRequest`) to facilitate integration testing.

### Fixed
- **API Path Extraction**: Fixed a critical bug in `apiserver` where multi-parameter routes (e.g., `/workspaces/:ws_id/members/:id`) were failing to extract the second parameter correctly using Axum's `Path`.
- **Employee Service**: Implemented missing `get_department_history` and `transfer_employee` methods in `EmployeeService` that were causing compilation errors.
- **SQL Query Syntax**: Corrected invalid column aliasing syntax (removed `as "name!"`) in `LeaveRequestRepository` to be compatible with `sqlx::query_as`.

### Changed
- **Dev Rules Enforcement**: Updated `docs/code-standards.md` to mandate that 100% of tests must pass before any commit or merge.
- **Automated Test Database Cleanup**: Implemented a robust mechanism for managing temporary Postgres databases during integration tests.
    - Added `Drop` trait implementation for `TestDatabase` with detached background shell cleanup.
    - Added `SIGINT`/`SIGTERM` signal traps to prevent database leaks on interrupted test runs.
    - Added `tests/db_trap_test.rs` to verify the reliability of the cleanup infrastructure.
    - Added session termination logic to ensure successful `DROP DATABASE` even with active connections.
- **Database Safety Guard**: Added a strict check in `apiserver` test suite to prevent accidental connection to production-named databases during testing.

### Fixed
- **Frontend Runtime Error**: Resolved "deleteRoster is not defined" error in `ShiftsPage.tsx` caused by missing import.
- **Frontend Test Suite**: Fixed broken imports (`userEvent`, `fireEvent`, `waitFor`) in `WorkspaceSettingsPage.test.tsx`.
- **Backend Test Isolation**: Refactored multiple integration tests (`agent_token_update_test.rs`, `heartbeat_tests.rs`, `shift_versioning_tests.rs`) to use isolated `TestDatabase` and fixed premature drop issues.
- **Test Configuration**: Corrected `apiserver/.env.test` to ensure test environment uses the intended local test database.
- **Database Cleanup Reliability**: Improved background cleanup worker in `TestDatabase` to prevent database leaks and handle concurrent drops more robustly.
- **Python Test Suite**: Temporarily ignored legacy broken tests in `agent-zk-worker` to maintain CI/CD stability during Rust transition.

## [0.1.0] - 2026-01-09

### Added
- **Early Clock-in Grace Period**: Implemented a configurable grace period for shifts (e.g., 30 mins). Punch-ins within this window are treated as being exactly on time, while punch-ins before the window count as overtime for the entire duration.
- **Automatic Dev Data Seeding**: Added `dev_initialization` module to automatically upsert a default admin user and workspace upon server startup in debug mode.
- **Enhanced Test Builders**: Extended `TestFactory` with builders for `Employee`, `Shift`, and `TimeEntry` to support complex integration testing scenarios.

### Changed
- **Shift Configuration**: Added `early_punch_in_grace_period_minutes` to `shifts` table and API models.
- **Agent Database Reliability**: Refactored `agent-main` to use standard `sqlx` query methods instead of macros, eliminating compile-time database dependencies and improving build reliability.

### Fixed
- **Agent Build Issues**: Resolved `openssl-sys` compilation errors by documenting `libssl-dev` requirement.
- **Integration Test Pathing**: Corrected Python `venv` path in `agent-main` integration tests.
- **Test Linter Warnings**: Cleared `dead_code` warnings in the test suite by properly annotating unused builder methods.

### Added (Previous)
- **Dynamic Break-to-Overtime Logic**: Implemented a flexible system to handle cases where employees work through their lunch breaks. This can now be tracked per-entry rather than being a fixed shift setting.
- **Attendance Adjustment Workflow**: Introduced a formal process for handling forgotten punches. Employees can now submit adjustment requests (Attendance Confirmation Sheets) for past dates.
- **Automated Time Entry Updates**: Approved adjustment requests automatically create or update the corresponding official `time_entries` records, ensuring accurate payroll data.
- **Manager Approval Dashboard**: A dedicated UI for administrators to review, approve, or reject attendance adjustment requests with historical context and reasons.
- **Backend Adjustment Service**: New business logic layer to handle the complex state transitions between requests and official attendance records.
- **Database Schema Expansion**: Added `attendance_adjustment_requests` table to track the full lifecycle of adjustment claims.

### Changed
- **Sidecar Navigation Update**: Added "Xác nhận công" link to the main dashboard sidebar for quick access to the adjustment module.
- **API Client Expansion**: Updated TypeScript API client with types and methods for attendance adjustments.
- **Time Tracking UI Enhancement**: Integrated "Forget to punch?" link and manual edit buttons directly into the employee's time history view.

### Fixed
- **Type Compatibility**: Resolved a TypeScript mismatch in `LeaveRequestsPage` and `api.ts` regarding approval payload structures.
- **Backend Cleanup**: Fixed an unused variable warning in `handlers.rs` for the attendance adjustment endpoints.

## [Unreleased] (Previous)

### Added
- **Robust WebSocket Client**: Implemented a state-of-the-art connection manager in Rust with **Exponential Backoff** and **Smart Heartbeat** (30s interval).
- **Stability Logic**: Added a stability detection window (10s) and message-based reset to prevent rapid reconnection loops.
- **Unit Tested Reliability**: Integrated comprehensive tests for `RetryManager` covering initial failures, premature drops, and stability resets.
- **Stateless Thin Driver**: Refactored the Python ZK Worker into a stateless driver. Removed 100% of business logic, database management, and cloud sync code from Python.
- **Unified Persistence**: Centralized all local storage logic into Rust using SQLite (`attendance.db`) via `sqlx`.

### Changed
- **Lighter Dependencies**: Reduced Python worker footprint by removing `sqlalchemy`, `alembic`, `websockets`, and `psycopg2`.
- **Improved Fault Isolation**: JSON-RPC over TCP (Localhost:5000) now strictly separates hardware communication from core business orchestration.

### Fixed
- **1s Retry Loop**: Resolved a critical bug where the agent would reconnect every 1s upon handshake failure instead of increasing the delay.
- **Resource Leakage**: Ensured proper cleanup of child processes and WebSocket streams on the Rust side.

## [Unreleased] (Previous)

### Added
- **Sidecar Architecture**: Migrated Agent core to Rust while retaining Python for hardware communication.
- **Rust-Python JSON-RPC Protocol**: Implemented a lightweight JSON-RPC 2.0 interface over TCP for communication between Rust Orchestrator and Python Worker.
- **Enhanced Test Reliability**: 
    - Implemented **Timeout Wrapping** across all test factories (`AsyncTestFactory` in Python, `TestFactory` in Rust) to prevent hanging tests.
    - Added unit tests for ZK Worker and Client with built-in timeout protection.
- **Prominent Auth Notifications**: Added high-visibility console output for critical authentication errors in the Edge Agent.
- **WebSocket Heartbeat**: Refactored heartbeat mechanism from redundant HTTP requests to native WebSocket messages, significantly reducing overhead and improving reliability.
- **Shared UI Components**: Implemented a common UI library starting with a highly reusable, accessible, and animated **DropdownMenu** component based on Radix UI.
- **Archive/Restore Agents**: UI now supports toggling "Archived" agents and restoring them directly from the dashboard.
- **Agent Token Editing**: Re-enabled the ability to update agent security tokens via the dashboard (fixed backend model and handler).
- **Health Check Endpoint**: Properly registered `/health` route in the API server for monitoring.
- **Centralized Status Logic**: Extracted `isAgentOnline` utility for consistent status calculation across the dashboard.

### Changed
- **UI Standardization**: Refactored `WorkspaceSwitcher` and `DevicesPage` to use the new shared `DropdownMenu` component.
- **Backend Clean-up**: Removed obsolete `/agents/heartbeat` HTTP endpoint and associated auth extractors.
- **Agent Optimization**: Removed `httpx` dependency from the Python Agent as HTTP calls are no longer needed for heartbeats.
- **Repository Pattern**: Refined `AgentRepository` to use Rust-side timestamps for `last_seen_at` updates to ensure cross-platform consistency.

### Fixed
- **404 Heartbeat Errors**: Resolved the persistent 404 errors in agent logs by eliminating redundant HTTP heartbeat calls.
- **White Screen Fix**: Restored missing imports in `AgentsPage.tsx` that caused component crashes.
- **Async Test Races**: Fixed flaky hook tests by properly waiting for React Query re-fetches.
- **Linter Warnings**: Cleared `dead_code` and `unused_variables` warnings across the Rust test suite.

### Added (Previous Changes - Jan 8)
- **Heartbeat Mechanism (Initial)**: Implemented time-based tracking for Agent and Device online status.
- **Member Management**: Added APIs and UI to update member roles and remove members.
- **Create Workspace UI**: New dialog in Workspace Switcher for easy organization creation.
- **Real-time Status Badges**: UI indicators for Online/Offline status with a 90s threshold.
- **Rust Repository Refactoring**: Reorganized backend repositories for SRP compliance.
- **Workspace Management**: Create organizations, invite members, and manage roles.
- **RBAC Support**: Basic role-based access control (Owner, Admin, Member) and custom roles.
- **Dynamic Port Selection**: Backend and Frontend now automatically find available ports (3000-3100 for API, 5173-5273 for Web) to avoid `EADDRINUSE` errors.
- **Port Discovery**: Automated `.api_port` and `.web_port` files for tool integration (Playwright, CLI).
- **Onboarding Flow**: New user experience for creating their first workspace.
- **Profile Page**: User details management and security settings.

### Changed
- Refactored Frontend `LoginPage` to use actual backend authentication.
- Updated Playwright configuration to dynamically detect target ports.
- Enhanced `AuthGuard` and `WorkspaceGuard` for more robust navigation.

### Fixed
- Fixed port collision issues in development environments.
- Corrected API path prefixing in testing environments.


## [3.3.0] - 2026-01-07
### Improved
- **Windows Compatibility**: Optimized `agent/main.py` signal handling for graceful shutdown on Windows (where `add_signal_handler` is limited).
- **Service Installation**: Enhanced `install_service.bat` to auto-detect `venv`, handle `.env` creation, and reliably register the service via NSSM.
- **Database Flexibility**: `agent/database.py` now supports SQLite via `DATABASE_URL` as the default fallback, eliminating the hard dependency on PostgreSQL for local agents.

### Fixed
- **Blocking I/O**: Resolved critical event loop blocking on Windows by wrapping synchronous `pyzk2` calls in `asyncio.to_thread` within `DeviceManager`.
- **Test Stability**: Updated test suite (`conftest.py`, `test_utils.py`) to use `AsyncMock`, ensuring full compatibility with the new async architecture.

## [3.2.0] - 2026-01-07
### Added
- **API Server Windows Service**: Integrated NSSM wrapper for `apiserver` (Rust), allowing it to run as a reliable background service on Windows.
- **Tauri Desktop App**: Packaged `webdashboard` as a native Windows application (`.exe`/`.msi`) using Tauri 2.0.
- **Static Export Flow**: Optimized Web Dashboard for static export (`output: 'export'`), removing Node.js runtime dependency for client-side distribution.

### Changed
- **Dashboard Architecture**: Refactored `page.tsx` into a Client Component to support static site generation and Tauri compatibility.

## [3.1.0] - 2026-01-07
### Added
- **Memory Safety Guard**: Implemented bounded queues with backpressure to prevent RAM saturation during network outages.
- **Smart Polling**: Adaptive polling interval (10s -> 60s -> 300s) based on device log volume to reduce CPU stress.
- **Rolling Window Deduplication**: Optimized log cache using a `deque` + `set` hybrid to maintain a fixed 20k window, preventing memory leaks while ensuring accurate deduplication.

### Improved
- **Resource Monitoring**: Enhanced internal status reporting to include queue size and cache utilization metrics.

## [3.0.0] - 2026-01-07
### Added
- **SRP Architecture in Agent**: Separated concerns into `AttendanceRepository` (DB), `DeviceManager` (Hardware), and `AttendanceAgent` (Container/Mediator).
- **Async Test Factory**: New robust testing environment with loop breaking and absolute state isolation for reliable async tests.
- **Robust Connection Handling**: Implemented singleton connection tasks and explicit error propagation in `CentralClient`.

### Changed
- **Pure CLI Mode**: Removed FastAPI, Uvicorn, and HTTPX from the agent. It now runs as a lightweight background service.
- **Outbound-Only Communication**: Agent now only connects to Central Server via Websocket; no listening ports are opened, enhancing security.
- **Exponential Backoff with Jitter**: Applied consistent retry logic across all components (Central Server, Sync Worker, ZK Devices).

### Fixed
- **Memory Safety**: Resolved memory leaks caused by stale `asyncio.Future` objects in ACK tracking.
- **Hanging Tests**: Fixed deadlocks and infinite busy-loops in the test suite by ensuring proper task cancellation and loop breaking.
- **State Contamination**: Eliminated test interference by resetting global instances between test runs.

## [2.3.1] - 2026-01-07
### Fixed
- **Filtered Synchronization**: SyncWorker hiện chỉ đồng bộ dữ liệu của các thiết bị có trong cấu hình hiện tại, ngăn chặn việc đẩy log của các thiết bị đã bị xóa.
- **Smart Idle Sync**: SyncWorker sẽ tạm dừng hoạt động hoàn toàn khi không có thiết bị nào được cấu hình, giúp tối ưu hóa tài nguyên hệ thống.

## [2.3.0] - 2026-01-07
### Added
- **Agent Trust Management**: API Server hiện đã hỗ trợ phê duyệt danh tính Agent trước khi cho phép hoạt động chính thức.
- **WebSocket Gatekeeper**: Thực thi xác thực `token` và trạng thái `is_trusted` ngay trong bước Handshake của WebSocket.
- **Agent Approval UI**: Trang quản lý mới trên Dashboard cho phép Admin khai báo, phê duyệt hoặc thu hồi quyền truy cập của bất kỳ Agent nào.
- **Unified Navigation**: Bổ sung Sidebar cố định vào Dashboard giúp quản lý Agent và Thiết bị tập trung.
- **Security Whitelist**: Chống giả mạo Agent ID bằng cơ chế so khớp Token bí mật.

## [2.2.1] - 2026-01-07
### Fixed
- **Empty Config Handling**: Sửa lỗi Agent bỏ qua cấu hình khi danh sách thiết bị trống (`[]`). Agent hiện đã có thể dừng tất cả monitors một cách chính xác khi Admin xóa sạch thiết bị trên Dashboard.
- **CORS Multiple Origins**: Sửa lỗi chỉ nhận diện origin cuối cùng trong danh sách `ALLOWED_ORIGINS`.

### Improved
- **Dashboard UI Layout**: Bổ sung Sidebar cố định, tối ưu hóa không gian hiển thị và hệ thống điều hướng tập trung.
- **Config Delta Logic**: Phân biệt rõ ràng giữa dữ liệu rỗng hợp lệ (Stop all) và dữ liệu lỗi `None` (Keep current).

## [2.2.0] - 2026-01-07
### Added
- **Smart Rebuild (Delta Check)**: Agent giờ đây chỉ khởi động lại các monitor khi cấu hình thực sự thay đổi (IP, Port, hoặc danh sách thiết bị), giúp duy trì kết nối ổn định.
- **Config Persistence (Offline-first)**: Agent tự động lưu cache cấu hình từ Server vào `devices.json`. Hỗ trợ khởi động và làm việc ngay cả khi mất kết nối với Server trung tâm.
- **Optimized Boot Sequence**: Quy trình khởi động mới giúp giảm thiểu downtime, ưu tiên nạp cấu hình local trước khi kết nối WebSocket.
- **Safety Guard**: Cơ chế bảo vệ ngăn chặn việc áp dụng cấu hình rỗng hoặc lỗi từ Server, giữ cho Agent luôn ở trạng thái ổn định nhất.

## [2.1.0] - 2026-01-07
### Added
- **Graceful Shutdown**: API Server hiện đã hỗ trợ dừng an toàn (SIGINT/SIGTERM).
- **Agent Notification**: Server tự động thông báo cho tất cả Agent đang kết nối trước khi đóng WebSocket.
- **Resource Cleanup**: Đảm bảo đóng kết nối Database (SQLx Pool) sạch sẽ khi tắt server.

## [2.0.0] - 2026-01-07
### Added
- **Web Dashboard (Next.js)**: Giao diện quản trị hiện đại, tích hợp quản lý thiết bị và theo dõi Agent thời gian thực.
- **Centralized Device Management**: Lưu trữ cấu hình máy chấm công trong Postgres với UUID.
- **Config Push Mechanism**: Tự động cập nhật cấu hình cho Agent qua WebSocket khi có thay đổi trên Dashboard.
- **CORS Support**: API Server hỗ trợ cấu hình nguồn cho phép từ file `.env`.
- **Batch Sync Optimization**: Agent hỗ trợ gửi batch 100 logs với interval 0.1s.

## [1.9.0] - 2026-01-07
### Added
- **Guaranteed Delivery Protocol**: Triển khai cơ chế ACK (Acknowledgment) cho dữ liệu chấm công.
- **Reliable Sync Worker**: Worker chạy ngầm tại Agent đảm bảo mọi bản ghi trong DB đều được đẩy lên Cloud thành công.
- **Exponential Backoff**: Cơ chế tự động điều chỉnh thời gian kết nối lại linh hoạt (5s -> 300s) khi Server gặp sự cố.
- **Message Tracking**: Hỗ trợ `msg_id` (UUID) để theo dõi trạng thái từng bản ghi quẹt thẻ qua WebSocket.

### Changed
- Refactor `CentralClient` để hỗ trợ xử lý phản hồi bất đồng bộ (Async ACK tracking).
- Cập nhật Database Schema: Thêm cột `is_synced` vào bảng `zkt_raw_attendance`.

## [1.8.0] - 2026-01-07
### Added
- **Duplicate ID Prevention**: Server từ chối kết nối nếu Agent ID đã tồn tại trong hệ thống.
- **Heartbeat Status Tracking**: Cơ chế theo dõi trạng thái online/offline của máy chấm công và Agent theo thời gian thực.
- **Dynamic Device Monitoring API**: Endpoint `GET /agents` cung cấp trạng thái chi tiết của toàn bộ hạ tầng.
- **Random Port Allocation**: Agent tự động chọn port trống (Port 0) để khởi chạy API local, tránh xung đột trên máy trạm.

### Changed
- Cải tiến giao thức WebSocket: Hỗ trợ mã lỗi `DUPLICATE_ID`.
- Refactor `AgentManager` để cache trạng thái thiết bị và timestamp `last_seen`.

## [1.7.0] - 2026-01-07
### Added
- **WebSocket Command Routing**: Triển khai hoàn thiện cơ chế gửi lệnh từ API Server (Rust) xuống Agent (Python) qua WebSocket.
- **Bi-directional Communication**: Handshake protocol và command/response tracking giữa Agent và Central Server.
- **API Server Configuration**: Hỗ trợ cấu hình `API_HOST` và `API_PORT` qua biến môi trường (ENV).
- **Comprehensive Test Suite**: 
    - Unit tests cho `AgentManager` trong Rust.
    - Integration tests cho WebSocket flow sử dụng `tokio-tungstenite`.
    - Async tests cho Remote Command handler trong Python Agent.
- **Agent Dependencies**: Thêm `requirements.txt` để chuẩn hóa việc cài đặt Agent.

### Changed
- **Rust Server Refactoring**: Tách logic ra `lib.rs` để tăng khả năng test (Testability).
- **Root Documentation**: Cập nhật Deployment Guide và API Reference bao hàm cả 2 thành phần.

## [1.6.0] - 2026-01-07
### Added
- **New Rust API Server**: Khởi tạo `apiserver` sử dụng Rust + Axum.
- **API Server Health Check**: Endpoint `GET /health` cho server trung tâm.
- **Project Restructuring**: Tổ chức lại thư mục thành Monorepo (Agent + API Server).
- **Global Documentation**: Di chuyển tài liệu ra thư mục gốc để bao quát toàn bộ hệ thống.

## [1.5.0] - 2026-01-07
### Added
- **Central Server Bridge**: Tích hợp WebSocket client để kết nối với Server trung tâm (Outbound).
- **Dynamic Configuration**: Khả năng nhận và áp dụng cấu hình máy chấm công từ xa mà không cần restart.
- **Remote Commands**: Agent hỗ trợ lắng nghe và xử lý lệnh từ Cloud.
- **Reliable Messaging**: Cơ chế queue nội bộ đảm bảo không mất dữ liệu khi mất kết nối Cloud tạm thời.
- **Auto-reconnect**: Tự động phục hồi kết nối WebSocket với cơ chế retry.

### Changed
- Cải tiến `lifespan` để quản lý task giám sát thiết bị một cách động.

## [1.4.0] - 2026-01-07
### Added
- **PostgreSQL Persistence**: Tích hợp lưu trữ dữ liệu chấm công thô vào PostgreSQL sử dụng SQLAlchemy.
- **Raw Attendance Table**: Bảng `zkt_raw_attendance` chuyên biệt để lưu log thô từ máy ZKT.
- **Query API**: Endpoint `GET /attendance/raw` hỗ trợ lọc theo User, Device, Khoảng thời gian và phân trang.
- **Agent Health & Status API**: Endpoint `/health` và `/status` để giám sát vận hành thời gian thực.
- **Operational Metrics**: Theo dõi uptime, tổng số bản ghi xử lý, và tình trạng từng máy chấm công.
- **Database Migrations**: Tích hợp Alembic để quản lý phiên bản cơ sở dữ liệu.
- **Model Validation**: Thêm Pydantic schemas để validate dữ liệu đầu vào/đầu ra.

### Fixed
- Xử lý mật khẩu DB chứa ký tự đặc biệt (URL-encoding).
- Bổ sung Index cho các cột quan trọng để tối ưu truy vấn.

## [1.3.0] - 2026-01-06
### Added
- **Centralized Event System**: Chuyển đổi toàn bộ log hệ thống sang mô hình tin nhắn (system events).
- **Silent Runners**: Các task monitor không in log trực tiếp, giúp Console sạch sẽ và đồng nhất.
- **Enhanced UI**: Runner chính hỗ trợ định dạng log đẹp hơn với emojis (ℹ️, ⚠️, 🔔).
- **Graceful Shutdown**: Cải thiện việc dừng các task và runner khi tắt ứng dụng.

### Changed
- Refactor `main_runner` để xử lý đa dạng loại thông điệp (`type: attendance` vs `type: system`).

## [1.2.0] - 2026-01-06

---

## [1.0.0] - 2026-01-06
### Added
- Khởi tạo dự án với FastAPI.
- Kết nối đơn máy ZKTeco XFace100.
- Live console monitoring.
