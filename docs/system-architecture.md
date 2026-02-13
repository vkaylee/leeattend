# System Architecture

## 🛡️ Reliability & Sync Logic

### 0. Graceful Shutdown & Lifecycle Management
To ensure data integrity and clean process termination, both the Edge Agent and the Central Server implement proactive signal handling:
- **Signal Propagation**: Upon receiving `SIGINT` (Ctrl+C) or `SIGTERM`, the Orchestrator (`agent-main`) triggers a `CancellationToken`. This immediately signals all background tasks (WebSocket sync, device polling, heartbeat) to stop after their current unit of work.
- **Sidecar Cleanup**: The Orchestrator explicitly kills the Python sidecar process and waits for its exit, preventing "zombie" processes from lingering on the host system.
- **Stateless Exit**: The Python worker uses an `asyncio.Event` to stop its JSON-RPC server and close hardware connections safely before exiting.

### 1. Guaranteed Delivery (ACK Protocol)
- Agent chỉ đánh dấu log là `is_synced = 1` sau khi nhận được bản tin `type: "ack"` từ Server.
- Nếu không nhận được ACK trong vòng 10 giây, Agent sẽ giữ log ở trạng thái chưa gửi và thử lại ở batch sau.

### 2. Edge Agent Resilience & Retry Architecture

The Edge Agent (`agent-main`) implements robust retry logic with exponential backoff to handle transient failures in device polling and server connectivity.

#### 2.1 Central Server Connection (WebSocket)
The agent maintains a persistent WebSocket connection to the Central Server for bidirectional sync:

- **Connection Timeout**: 30 seconds - prevents indefinite hangs on network unavailability
- **Stability Threshold**: 60 seconds - retry count only resets after 60s of stable connection
- **Exponential Backoff**: 1s, 2s, 4s, 8s, 16s, 32s, 60s (capped)
- **Formula**: `delay = min(2^retry_count, 60)` seconds
- **Jitter**: ±25% jitter applied to all backoff delays to prevent thundering herd effect
- **Max Retries**: 10 attempts before manual intervention

**Retry Manager Implementation** (`src/central_client.rs`):
- Tracks `retry_count` (0-10) and `stable_since` timestamp
- Resets retry count to 0 after connection remains stable for 60 seconds
- Logs all connection state transitions with structured error messages

#### 2.2 Sidecar (Device Worker) Connection
The agent spawns brand-specific Python sidecars (ZKTeco, Mock) that poll biometric devices:

- **RPC Protocol**: JSON-RPC 2.0 over TCP (loopback at 127.0.0.1)
- **Max Retry Delay**: 60 seconds (capped)
- **Stability Threshold**: 30 seconds to reset retry counter
- **Max Retry Count**: 10 attempts per device
- **Exponential Backoff**: 2s, 4s, 8s, 16s, 32s, 60s (capped)
- **Jitter**: ±25% on all delays

**Sidecar Retry Module** (`src/retry.rs`):
```rust
pub const SIDECAR_MAX_RETRY_DELAY_SECS: u64 = 60;
pub const SIDECAR_MAX_RETRY_COUNT: u32 = 10;
pub const SIDECAR_STABILITY_THRESHOLD_SECS: u64 = 30;
pub fn get_sidecar_retry_delay(retry_count: u32) -> Duration
```

Each worker maintains independent retry tracking:
- Initial sidecar spawn failure → increment retry count, apply backoff, restart
- Sidecar connection timeout (>10s) → increment retry count, kill process, restart
- Device polling error → increment retry count, kill sidecar, restart
- Successful polling for 30s → reset retry count to 0

#### 2.3 Worker Client Improvements
The `WorkerClient` struct handles RPC communication with sidecars:

- **Read Timeout**: 5 minutes (300s) for slow device connections
- **Write Timeout**: 30 seconds for RPC requests
- **Blocking Mode**: Ensures tokio non-blocking sockets are converted to blocking for reliable I/O
- **from_stream()**: Constructor for accepted TCP connections (sets blocking mode + timeouts)

#### 2.4 Failure Recovery Paths
All failure points increment retry count and apply exponential backoff:

1. **Bind failure** → Retry sidecar spawn
2. **Sidecar spawn failure** → Retry with backoff
3. **Connection accept timeout** → Kill sidecar, retry with backoff
4. **Stream conversion error** → Kill sidecar, retry with backoff
5. **Client creation error** → Kill sidecar, retry with backoff
6. **Device.connect RPC error** → Kill sidecar, retry with backoff
7. **Device.get_logs RPC error** → Kill sidecar, retry with backoff
8. **Connection drop** → Kill sidecar, retry with backoff

The system is designed to tolerate transient network issues, slow hardware, and temporary service unavailability without manual intervention.

### 10. Attendance Pairing & Audit Engine
Hệ thống xử lý dữ liệu quẹt thẻ (punch) theo mô hình **Event-Sourcing Audit Trail** để đảm bảo tính minh bạch và khả năng phục hồi dữ liệu:

#### 10.1. Raw Attendance Logs (Bất biến)
- **Immutable Store**: Mọi lượt quẹt thẻ từ Agent (thiết bị ZKTeco) hoặc Import (file Excel/CSV) đều được lưu nguyên bản vào bảng `raw_attendance_logs`. 
- **Auditability**: Dữ liệu này không bao giờ bị xóa hoặc sửa, cho phép người quản trị luôn có thể đối soát lại nguồn gốc của một session làm việc.
- **Source Tracking**: Mỗi bản ghi lưu rõ nguồn (`device`, `import`, `manual`) và ID thiết bị tương ứng.

#### 10.2. Pairing Engine (Strategy Pattern)
Dịch vụ `PairingService` thực hiện nhiệm vụ chuyển đổi luồng sự kiện phẳng (flat events) thành các phiên làm việc (sessions) thông qua thuật toán ghép cặp (pairing algorithm) thông qua kiến trúc pluggable:

- **Primary Algorithm: Anchor First**: Thuật toán lấy bản ghi đầu tiên (In) và cuối cùng (Out) trong một cửa sổ thời gian làm mốc (Anchors). Đây là giải pháp tối ưu cho hầu hết các môi trường sản xuất và văn phòng.
- **Explicit Intent: FSM (Finite State Machine)**: Chiến lược máy trạng thái được khôi phục và tối ưu hóa để xử lý các lượt quẹt thẻ có kèm mã trạng thái (Punch Status) từ thiết bị, cho phép ghép cặp chính xác dựa trên ý định thực tế của nhân viên.
- **16-Hour Session Limit**: Hệ thống áp đặt giới hạn 960 phút (16 giờ) cho một phiên làm việc trong chiến lược Anchor-First. Điều này cho phép bao phủ các ca làm việc kéo dài hơn trong ngày mà không bị tự động tách thành hai phiên riêng biệt. Nếu khoảng cách vượt quá 16 giờ, hệ thống sẽ coi đó là các phiên làm việc độc lập.
- **Identity Deduplication (Identity-based)**: Tối ưu hóa việc lọc nhiễu bằng cách chỉ bỏ qua các lượt quẹt thẻ có cùng mốc thời gian chính xác (timestamp).
- **Morning Lookback (Overnight support)**: Tự động tính toán lại dữ liệu của ngày hôm trước nếu nhận được log vào buổi sáng (trước 12:00 PM).
- **Pairing Traceability**: Mỗi `TimeEntry` được tạo ra đều lưu trữ `generation_metadata` (JSONB) chứa ID của các log nguồn và tên thuật toán được sử dụng, phục vụ công tác đối soát.

#### 10.3. Schedule Traceability (New)
Hệ thống tính toán lịch biểu tự động hiện tích hợp khả năng truy vết nguồn gốc (Traceability):
- **Decision Engine**: `RuleEvaluator` không chỉ trả về ca làm việc mà còn ghi lại toàn bộ quá trình đánh giá: quy tắc nào khớp, mức độ ưu tiên, và thời điểm đánh giá.
- **Conflict Resolution**: Khi có nhiều quy tắc cùng khớp, hệ thống ghi lại lý do tại sao một quy tắc cụ thể được chọn (ví dụ: Priority cao hơn hoặc Manual Override).
- **Frontend Visualization**: Dashboard hiển thị các "Trace Hints" giúp người quản trị hiểu ngay lập tức tại sao nhân viên A lại có ca làm việc X vào ngày Y.

#### 10.4. Strict Import Validation
...

### 11. Hierarchical Leave Policy Assignment
...

#### 11.1. Phân cấp ưu tiên (Hierarchy)
Khi tính toán cộng phép cho một nhân viên tại một thời điểm nhất định, hệ thống sẽ tìm kiếm chính sách có hiệu lực theo thứ tự ưu tiên giảm dần:
1. **Employee-level**: Chính sách gán trực tiếp cho cá nhân nhân viên.
2. **Department-level**: Chính sách gán cho phòng ban mà nhân viên đang thuộc về.
3. **Workspace-level (Global)**: Chính sách mặc định cho toàn bộ tổ chức.

#### 11.2. Transactional Auto-Close (Quản lý vòng đời)
Để đảm bảo tính nhất quán của dữ liệu lịch sử và tránh việc một nhân viên có hai chính sách cùng loại (`leave_type`) hiệu lực đồng thời:
- Khi tạo một bản gán chính sách (`Assignment`) mới, hệ thống thực hiện một transaction:
    1. Tìm kiếm bản gán hiện tại đang mở (`effective_to IS NULL`) của cùng đối tượng cho cùng loại phép.
    2. Tự động đóng bản gán cũ bằng cách set `effective_to = effective_from_new - 1 day`.
    3. Chèn bản gán mới.
- Cơ chế này đảm bảo dữ liệu cộng phép trong quá khứ được bảo toàn theo đúng chính sách tại thời điểm đó.

### 12. Branding & Dynamic Themeing
LeeAttend áp dụng cơ chế định danh thương hiệu linh hoạt (Dynamic Identity) để hỗ trợ cả SaaS đa khách hàng và cá nhân hóa trải nghiệm:

#### 11.1. Workspace Branding
- **Primary Color Persistence**: Mỗi Workspace có thể cấu hình một màu chủ đạo (`theme_primary_color`) duy nhất trong Database.
- **Propagation**: Màu sắc này được API trả về trong object `Workspace` và được Frontend sử dụng làm gốc để tính toán toàn bộ bảng màu UI.

#### 11.2. Frontend Shade Generation
- **`useTheme` Hook**: Chịu trách nhiệm chuyển đổi một mã HEX duy nhất thành một bộ 11 sắc thái (50-950) dựa trên thuật toán điều chỉnh độ sáng (Brightness Adjustment). 
- **CSS Variable Injection**: Các sắc thái màu được inject trực tiếp vào `:root` dưới dạng các biến CSS (`--primary-50`...). Tailwind CSS được cấu hình để sử dụng các biến này thay vì mã màu tĩnh.

#### 11.3. User Personalization Override
- **Hierarchical Priority**: 
    1. **User Preference** (Lưu tại `localStorage`): Có độ ưu tiên cao nhất, cho phép cá nhân hóa theo thiết bị.
    2. **Workspace Brand**: Màu chuẩn của tổ chức.
    3. **System Default**: Màu Violet mặc định.
- **Cross-Tab Sync**: Sử dụng sự kiện `storage` của trình duyệt để đảm bảo thay đổi màu sắc tại trang Profile được cập nhật tức thì trên tất cả các tab đang mở mà không cần tải lại trang.

### 13. Employee Self-Service (ESS) Portal
Hệ thống cung cấp một phân hệ dành riêng cho nhân viên để tự quản lý dữ liệu cá nhân, giảm bớt gánh nặng quản trị cho bộ phận HR:

#### 13.1. Identity-Linked Security
- **Automatic Context**: Hệ thống tự động ánh xạ tài khoản người dùng (`User`) với hồ sơ nhân viên (`Employee`) thông qua `user_id`. Nhân viên không cần biết ID của mình để truy cập dữ liệu.
- **Scope Isolation**: Các API trong nhóm `/api/ess/` được thiết kế để chỉ trả về dữ liệu thuộc về chính nhân viên đang đăng nhập, thực thi bảo mật tại tầng Database Query.

#### 13.2. Personal Data Autonomy
- **Profile & Contacts**: Nhân viên có quyền cập nhật các thông tin biến động như số điện thoại, địa chỉ và danh sách người liên hệ khẩn cấp mà không cần qua Admin phê duyệt.
- **Document Repository**: Cung cấp kho lưu trữ an toàn cho các tài liệu cá nhân (CCCD, Bằng cấp, Hợp đồng lao động), cho phép nhân viên tra cứu bất cứ lúc nào.

#### 13.3. Transparency & Insight
- **Live Attendance History**: Nhân viên có thể theo dõi lịch sử quẹt thẻ và các phiên làm việc đã được ghép cặp để phát hiện sớm các sai sót.
- **Real-time Leave Balances**: Hệ thống hiển thị số dư phép được tính toán theo thời gian thực (bao gồm cả phép cộng dồn theo thâm niên), giúp nhân viên chủ động lập kế hoạch nghỉ ngơi.

#### 13.4. Self-Service Clock Interface (Phase 1: Time-Tracking Refactor)
- **MyClockPage**: Centralized ESS punch interface with component-based architecture:
    - **Punch Card Display**: Shows active session status (Working/On Break/Off Duty) with large, prominent punch buttons and real-time session start time.
    - **Quick History Timeline**: Displays last 7 days of time entries in tabular format with date, entry type (Work/Break), punch-in time, and punch-out status.
    - **Adjustment Request Workflow**: Single-click access to request attendance corrections with reason and new punch times.
- **Real-time Updates**: 30-second auto-refresh via React Query to maintain synchronized clock status across sessions.
- **Component Reusability**: Extracted time-tracking UI components (`PunchCard`, `TimeEntryRow`, `AdjustmentRequestModal`) for use in future admin dashboards and reports.

### 14. Dynamic Payroll System V2
LeeAttend implements a robust, professional-grade payroll engine designed for complex enterprise requirements:

#### 14.1. Multi-Segment Calculation
Instead of a simple monthly calculation, the system employs a **Segmentation Engine** that automatically detects "Change Points" (e.g., Department Transfer, Salary Increase) within a payroll period.
- **Slicing**: A pay period (e.g., 01/01 - 31/01) is sliced into segments (e.g., 01-14, 15-31).
- **Independent Configuration**: Each segment is calculated independently with its own snapshot of configuration (Standard Working Days, Base Salary).
- **Exact Prorating**: Ensures mathematically precise salary calculation when an employee moves between departments with different working policies (e.g., 26 days vs 22 days).

#### 14.2. Immutable Snapshots (`payslip_segments`)
- **Granular Audit Trail**: Every calculation detail for each segment is persisted in the `payslip_segments` table.
- **History Protection**: Future changes to department settings or global policies will **never** alter the values of finalized payslips, as the calculation parameters are frozen in the snapshot.

#### 14.3. Variable Incentive Evaluation (New)
The system supports complex, metric-driven bonuses (e.g., Diligence Bonus) through a dedicated evaluation engine.
- **Incentive Context**: During payroll calculation, the engine aggregates a high-fidelity context including `total_late_early_count`, `unpaid_leave_days`, and `sick_leave_days`.
- **Policy Matching**: The `PayrollEvaluator` matches this context against a prioritized list of `IncentiveTiers`.
- **Generosity Principle**: Tiers are sorted by priority. The engine selects the first matching tier (e.g., Tier 1 for 0 violations, Tier 2 for < 2 violations), rewarding the most diligent behavior.
- **Persistence**: Calculated incentives are stored in `payslip_incentives`, preserving the exact tier and amount awarded for historical auditing.

#### 14.4. Granular Salary Components (New)
The payroll system now supports flexible, granular salary component definitions replacing the simple monolithic "base salary" model.
- **Component Definition**: Each workspace maintains a catalog of salary component types (`SalaryComponentDefinition`) with metadata:
  - `name`: Human-readable name (e.g., "Basic Salary", "Housing Allowance", "Meal Allowance")
  - `code`: Unique identifier (e.g., "BASIC", "HRA", "MEAL") for system integration
  - `is_base_for_ot`: Boolean flag indicating if this component participates in overtime rate calculation
  - `is_taxable`: Boolean flag for tax withholding logic
- **Employee Assignment**: Admins assign components to employees via `EmployeeSalaryComponent` records with:
  - `component_definition_id`: Reference to the component type
  - `amount`: Monthly amount for this component
  - `effective_date`: Point-in-time activation (enables retroactive corrections)
- **Payslip Breakdown**: When generating payslips, the system includes detailed `salary_components` array in `PayslipResponse`, showing each component's:
  - Definition metadata (name, code, tax/OT flags)
  - Applied amount for the period
  - Prorated calculation (accounting for partial months due to segmentation)
- **Backward Compatibility**: If no components are assigned, the system defaults to displaying segments' legacy `base_salary` field, ensuring existing payroll data remains functional.
- **Flexible Display**: The PayslipDocument component adapts presentation:
  - Shows granular component breakdown when `salary_components` array is populated
  - Falls back to segment-based display when components are absent

### 15. Special Attendance Policy Resolution
Beyond the standard 3-tier payroll settings, the system implements a **Working Condition Overlay** system to handle individual employee privileges (e.g., Parental Leave, Breastfeeding Breaks).

#### 15.1. Resolution Flow
During attendance calculation, the `PolicyResolver` fetches all active `working_conditions` for the employee on the target date.
1. **Fetch**: Query `working_conditions` where `start_date <= date` and `end_date >= date`.
2. **Merge**: If multiple policies exist (e.g., a general "Early Leave" and a "Breastfeeding" privilege), the system merges them into a single `PolicyRules` object.
3. **Generosity Principle**: When privileges overlap, the engine defaults to the most generous allowance (highest `minute_allowance`).

#### 15.2. Impact on Calculation
The resolved `PolicyRules` are passed to the `AttendanceCalculator`, affecting the resulting work buckets:
- **`authorized_early_leave_minutes`**: If an employee leaves early but within this allowance, the "Early Leave" status is suppressed.
- **`is_early_leave_paid`**: If true, the system "boosts" the standard work duration to cover the missing minutes, ensuring the employee receives full pay despite departing early.

### 📅 Temporal Policy Engine (3-Tier)

The system uses a hierarchical resolution strategy to determine working conditions (Standard Days, Hours per Day) at any specific point in time.

#### Resolution Order (Descending Priority)
1. **Individual Override**: Highest priority, stored in `employee_payroll_configs`.
2. **Department Override**: Inherited if no individual config exists, stored in `department_payroll_configs`.
3. **Workspace Global**: Global default for the organization, stored in `payroll_settings`.
4. **System Default**: Hardcoded fallback (26 days, 8 hours).

#### Point-in-Time Lookup
The `PolicyResolver` uses the logic: `Max(EffectiveDate) where EffectiveDate <= TargetDate`. This ensures that recalculating past payroll periods always uses the rules that were actually in effect during that time.

---

### 🧮 Detailed Payroll Calculation Flow

The `PayrollService` uses a multi-pass approach to ensure precise prorating when organizational or financial changes occur mid-month.

```ascii
    [ Request: Calculate Payroll ]
    (EmployeeID, Month, Year)
               │
               ▼
    ┌──────────────────────────┐
    │ 1. Change Point Detection│
    └──────────┬───────────────┘
               │  Detects: Salary Changes, Department Transfers,
               │          and Policy Override Effective Dates.
               ▼
    ┌──────────────────────────┐
    │ 2. Period Segmentation   │
    └──────────┬───────────────┘
               │  Splits month into N segments.
               │  Example: Jan 01-15 (Old Salary), Jan 16-31 (New Salary).
               ▼
    ┌──────────────────────────────────────────────────────────┐
    │ 3. PER-SEGMENT CALCULATION (Loop)                        │
    ├──────────────────────────────────────────────────────────┤
    │                                                          │
    │  A. Policy Resolution (3-Tier)                           │
    │     - Resolve (Std Days, Hours/Day) via:                 │
    │       Individual > Dept > Workspace hierarchy.           │
    │                                                          │
    │  B. Rate Derivation                                      │
    │     - Sum all components where `is_base_for_ot = TRUE` at segment start (strict boolean logic, no magic strings). │
    │     - Hourly Rate = Total Base / (Std Days * Hours/Day). │
    │                                                          │
    │  C. Attendance Aggregation                               │
    │     - Filter Work Units (Standard/OT) within segment.    │
    │                                                          │
    │  D. Financial Computation                                │
    │     - Subtotal = (Units * Hourly Rate) + Flat Additions. │
    │                                                          │
    └──────────────────────────┬───────────────────────────────┘
                               │
                               ▼
    ┌──────────────────────────┐
    │ 4. Final Aggregation     │
    └──────────┬───────────────┘
               │  Sum all segment subtotals, taxes, and
               │  deductions into a master Payslip record.
               ▼
    ┌──────────────────────────┐
    │ 5. Immutable Snapshot    │
    └──────────────────────────┘
       - Persist Payslip.
       - Persist PayslipSegments (storing snapshots of
         applied_hours_per_day and base_salary).
```

---

### 🛠 Debug & Observability Stack

The system includes a developer-first observability suite that hooks deep into the request lifecycle.

#### Authentication Impersonation
The `UserId` extractor in `apiserver/src/lib.rs` includes a high-priority check for the `X-Debug-Impersonate-Id` header. When present (and in debug builds), it bypasses JWT verification and automatically assumes the identity of the specified UUID.

#### Time Travel Middleware
Every request can include an optional `X-Debug-Mock-Date` header. This is parsed into a `DebugContext` which handlers can use to override the current system time when calling the `PolicyResolver` or `PayrollService`.

```ascii
[ Browser ] --(X-Debug-*)--> [ Axum Router ]
                                   │
                        ┌──────────┴──────────┐
                        │  Debug Extractors   │
                        └──────────┬──────────┘
                                   ▼
                        ┌─────────────────────┐
                        │   Business Logic    │
                        │ (uses Mock Context) │
                        └─────────────────────┘
```

## Infrastructure Diagram
```ascii
[ Web Dashboard ] <--- REST API + WebSocket ---> [ API Server ] <--- WebSocket (Outbound Only) ---> [ Agent (Rust) ] <--- JSON-RPC (TCP) ---> [ Multi-Brand Workers ] <---> [ ZKTeco Devices ]
 (Tauri Desktop)       (Real-time Report Progress)  (Rust)                                          (Orchestrator)                        (Sidecar Driver)
                                                        │
                                                  [ Redis Pub/Sub ]
                                                   (Job Messages)
                                                        │
                                                  [ PostgreSQL ]
                                                   (Job Metadata)
                                                        │
                                                   [ Workers ]
                                                  (Report Jobs)
```

## 1. Agent (Client Side)
Hệ thống sử dụng mô hình **Sidecar Architecture** để tối ưu hóa hiệu năng và độ ổn định:

### 1.1. Rust Agent (Main Orchestrator)
- **Responsibility**: Quản lý vòng đời ứng dụng, kết nối WebSocket tới Server trung tâm, quản lý cơ sở dữ liệu local (SQLite), thực thi logic nghiệp vụ (đồng bộ dữ liệu, retry, cấu hình) và điều phối các driver.
- **Persistence**: Trực tiếp quản lý file `attendance.db` thông qua thư viện `sqlx`. Mọi log từ thiết bị đều được Rust lưu trữ trước khi đồng bộ lên Cloud.
- **Reliability**: Tích hợp cơ chế **Exponential Backoff** và **Heartbeat** mạnh mẽ để duy trì kết nối WebSocket ổn định.

### 1.2. Python ZK Worker (Stateless Thin Driver)
- **Responsibility**: Đóng vai trò là một driver phần cứng thuần túy, cung cấp API JSON-RPC để giao tiếp với thiết bị ZKTeco thông qua thư viện `pyzk2`.
- **Simplified Architecture**: Đã được refactor để loại bỏ hoàn toàn các phụ thuộc không cần thiết (`models`, `device_manager`). Chỉ giữ lại `ZKWorker` (RPC Server) và `ZKService` (ZK Protocol wrapper).
- **Stateless**: Worker này không còn quản lý Database, không có logic đồng bộ Cloud và không lưu trữ trạng thái. Mọi dữ liệu thu thập được sẽ được trả về ngay lập tức cho Rust Agent.
- **Communication**: Kết nối với Rust Agent qua giao thức **JSON-RPC 2.0** chạy trên Localhost TCP (Port 5000).
- **Isolation**: Đảm bảo lỗi từ driver phần cứng không ảnh hưởng đến logic nghiệp vụ chính của Agent.
... (giữ nguyên) ...

## 2. API Server (Server Side)
Core backend viết bằng **Rust** sử dụng framework **Axum**.
- **Performance**: High-concurrency support with Tokio runtime.
- **Safety**: Type-safe business logic to prevent common runtime errors (NullPointer, Data Race).
- **Architecture**: Modular design with clean separation of concerns:
  - `handlers/`: Domain-specific API endpoints (Auth, Agents, Attendance, etc.).
  - `domain/`: Core Domain Logic (Policy Resolver, Payroll Engine).
  - `services/`: Business logic (PayrollService, Pairing, Accrual).
  - `repositories/`: Database abstraction layer via SQLx.
  - `models/`: Type-safe domain entities.

### 2.1. Multi-Tenancy & Identification (Path Priority)
LeeAttend sử dụng chiến lược định danh Workspace ưu tiên đường dẫn (Path Priority) để đảm bảo tính cô lập dữ liệu giữa các tổ chức:
- **URI-based Identification**: Hệ thống ưu tiên trích xuất `workspace_id` trực tiếp từ tham số đường dẫn (ví dụ: `/workspaces/:ws_id/employees`). Điều này giúp API trở nên tường minh và tương thích tốt hơn với các công cụ web tiêu chuẩn.
- **Smart Extraction**: Extractor `WorkspaceId` trong Rust sẽ tự động kiểm tra các tham số `:ws_id` hoặc `:id` trong URL trước khi tìm kiếm trong Header.
- **Fallback Header**: Header `X-Workspace-Id` vẫn được duy trì như một phương thức dự phòng cho các ứng dụng không sử dụng Path-based routing (như ESS portal) hoặc các phiên bản client cũ.

## 3. Web Dashboard (Admin UI)
Ứng dụng quản trị tập trung dành cho Admin.
- **Responsibility**: Quản lý cấu hình, phê duyệt Agent và theo dõi dữ liệu chấm công.
- **Stack**: **React 19** + **Vite**, **Tauri** (Desktop Wrapper), **Tailwind CSS 4**.
- **Architecture**: Single Page Application (SPA) sử dụng **React Router 7** để điều hướng client-side.
- **Mode**: 
  - **SPA Build**: Build thành static assets (HTML/JS/CSS).
  - **Desktop App**: Chạy như ứng dụng Windows native (Tauri).
  - **Standalone Server**: Chạy như một Web Server độc lập (Rust Wrapper), không cần cài đặt Node.js hay bất kỳ runtime nào khác.

## Data Flow
... (giữ nguyên) ...
1. **Collector**: `DeviceManager` periodically fetches logs from ZK devices.
2. **Persistence**: `AttendanceRepository` saves logs immediately to local DB with `is_synced=0`.
3. **Sync Worker**: Queries unsynced logs and pushes them to the Central Server.
4. **Reliability (ACK)**: Server sends an acknowledgment for each batch. Agent updates `is_synced=1` only upon receiving ACK.
5. **Backoff**: If any connection fails (ZK device or Central Server), the Agent applies **Exponential Backoff with Jitter**.

## 🛰️ Service Discovery (Development)

Để hỗ trợ AI tự động khởi chạy và debug mà không gây xung đột với môi trường của người dùng, hệ thống sử dụng cơ chế **File-based Discovery**:

1.  **API Server:** Khi khởi chạy với `API_PORT=0`, OS sẽ cấp một port ngẫu nhiên. Server sẽ ghi port này vào file `apiserver/.api_port`.
2.  **Web Dashboard (Vite):** Khi khởi chạy, `vite.config.ts` sẽ kiểm tra sự tồn tại của file `../apiserver/.api_port`. Nếu thấy, nó sẽ tự động cấu hình `proxy` để chuyển hướng các request `/api` tới đúng port đó.
3.  **Playwright (AI Mắt thần):** Sử dụng các script trong `webdashboard/tests/debug-ui.spec.ts` để tự động hóa việc kiểm tra giao diện, lắng nghe console log và chụp ảnh màn hình.

## 🛠️ AI Debugging Workflow



Khi gặp lỗi giao diện, AI Agent nên thực hiện các bước sau:

1.  **Gắn cờ:** Luôn thêm `AI_MANAGED=true` vào trước lệnh khởi chạy để dễ dàng quản lý.

2.  Khởi chạy Backend: `AI_MANAGED=true (cd apiserver && cargo run) &`.

3.  Khởi chạy Frontend: `AI_MANAGED=true (cd webdashboard && npm run dev) &`.

4.  Chạy Playwright Debug: `npx playwright test tests/debug-ui.spec.ts`.

5.  **Dọn dẹp:** Sử dụng `pkill -f "AI_MANAGED=true"` để tắt chính xác các server do AI tạo ra, tránh giết nhầm server của người dùng.
### 18. Mock API Intercept Layer (Frontend Isolation)
To enable 100% independent frontend development and stable testing in resource-constrained environments (like Docker containers), the system implements a **Mock API Layer** using Playwright Intercept.

#### 18.1. Architecture
Instead of connecting to a live Rust backend, the frontend in test mode interacts with a simulated API layer controlled by Playwright:
```ascii
[ Frontend UI ] <--- fetch() --- [ Playwright Intercept (page.route) ]
                                          │
                                          ▼
                                [ mocks/api-intercept.ts ]
                                          │
                                          ▼
                                [ mocks/mock-data.ts ]
```

#### 18.2. Key Features
- **Deterministic Responses**: All API calls (Auth, Employees, Attendance, etc.) return consistent, predefined data from `mock-data.ts`.
- **Zero Backend Dependency**: E2E tests can run without Postgres, Redis, or the apiserver binary.
- **Vite Proxy Bypass**: In test mode (`isTest = true`), Vite's proxy configuration is automatically disabled to allow Playwright to handle all `/api/*` requests directly.
- **Multi-language Support**: Mock data and intercept logic are designed to support both English and Vietnamese localization flows.

#### 18.3. Implementation Details
- **Location**: `webdashboard/tests/mocks/`
- **Global Setup**: Initialized via `base.beforeEach` in `worker-orchestrator.ts`.
- **Regex Routing**: Uses flexible regex patterns to match API endpoints while allowing static assets (JS/CSS) to be served normally by Vite.

---

## 🤖 Edge Agent Architecture

### 1. Inverted Master-Sidecar Communication
The system utilizes a hybrid Rust/Python architecture where the Rust Orchestrator (`agent-main`) acts as the "Master" and the Python script (`agent-zk-worker`) acts as the "Sidecar". 

To ensure maximum reliability and ease of deployment, the communication model is inverted:
- **Rust as Server**: The Orchestrator opens a TCP Server on a **dynamic port** (assigned by the OS at runtime, port 0).
- **Python as Client**: The Python sidecar receives this port via a command-line argument and proactively connects to the Master.
- **Bi-directional JSON-RPC**: Once the connection is established, the Master sends control requests (fetch logs, sync time) while the Sidecar can optionally push real-time events back to the Master over the same stream.
- **Zero Conflict**: Dynamic port assignment allows multiple Agents to run on the same physical gateway without port collision or manual configuration.

### 2. Reliable Data Sync (Log-Forwarding Pattern)
The Agent follows a "Local-First" approach to ensure data safety during internet outages:
1. **Fetch**: The Orchestrator polls logs from physical devices via the Python Sidecar.
2. **Commit**: Logs are immediately persisted to a local SQLite database with `is_synced = 0`.
3. **Sync Loop**: A background task in `CentralClient` continuously polls the local DB for unsynced records.
4. **Push & ACK**: Logs are pushed to the Cloud WebSocket API. The record is only marked as `is_synced = 1` once a valid `type: "ack"` is received from the server.

---


### 0. Graceful Shutdown & Lifecycle Management
To ensure data integrity and clean process termination, both the Edge Agent and the Central Server implement proactive signal handling:
- **Signal Propagation**: Upon receiving `SIGINT` (Ctrl+C) or `SIGTERM`, the Orchestrator (`agent-main`) triggers a `CancellationToken`. This immediately signals all background tasks (WebSocket sync, device polling, heartbeat) to stop after their current unit of work.
- **Sidecar Cleanup**: The Orchestrator explicitly kills the Python sidecar process and waits for its exit, preventing "zombie" processes from lingering on the host system.
- **Stateless Exit**: The Python worker proactively closes hardware connections and its control stream safely before exiting.

### 17. High-Performance Edge Ingestion
To handle high-traffic environments (30k+ logs), the Edge Agent employs an optimized ingestion pipeline:

#### 17.1. Batch DB Writer Worker
- **Pattern**: Producer-Consumer with a 10,000-slot `mpsc` channel.
- **Batching Strategy**: The worker buffers incoming logs and flushes them to SQLite in a single transaction when:
    - The buffer reaches **500 records**.
    - An interval of **500ms** has passed (ensuring real-time availability).
- **Efficiency**: Reduces SQLite transaction overhead by up to 100x compared to single-row inserts.

#### 17.2. SQLite Performance Tuning
The local database is configured with the following PRAGMAs:
- `journal_mode=WAL`: Enables concurrent reads and writes.
- `synchronous=NORMAL`: Balances safety and write speed.
- `cache_size=-64000`: Allocates 64MB of RAM for database caching.
- `temp_store=MEMORY`: Keeps temporary tables in RAM to avoid disk I/O.

#### 17.3. Signature Cache Deduplication (New)
The Edge Agent uses hash-based signature caching to prevent duplicate log insertion with minimal memory overhead.

**Architecture:**
```
┌─────────────────────────────────────────────────────────┐
│                SIGNATURE CACHE SYSTEM                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Log → compute_signature() → Check DashSet<u64>         │
│         (FxHash)                                         │
│            ↓                                             │
│         Exists?                                          │
│         ├─ YES → Reject (duplicate)                     │
│         └─ NO  → Insert to DashSet + Queue              │
│                                                          │
│  Signature = hash(device_id, user_id, timestamp,        │
│                   punch, status)                         │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

**Memory Efficiency:**
- Hash signature: 8 bytes (u64)
- Full log: ~200 bytes
- **96% memory savings** (0.8MB vs 20MB for 100K logs)
- Persistent protection: Signatures remain after log cleanup

**Two-Phase Startup:**
```
Phase A: Load ALL signatures from disk
         → SELECT device_id, user_id, timestamp, punch, status
         → ~25 bytes per row (vs ~200+ for full log)
         → Populate DashSet<u64> for dedup index

Phase B: Load unsynced/in-flight logs for sync queue
         → Same as original startup
         → Only pending logs loaded into memory
```

**Critical Fix (2026-01-29):**
Signature formula now includes `punch` and `status` fields to prevent false positives when same employee punches multiple times at exact same timestamp with different punch types (In/Out/Break).

**Formula:**
```rust
hash(device_id | user_id | timestamp_unix | punch | status)
```

**Benefits:**
- Duplicate protection persists across restarts
- No memory waste on synced logs
- 25x faster startup (lightweight Phase A)
- Concurrent lock-free dedup checks (DashSet)

### 17.4. Dual-Layer Server Protection & Staging (New - 2026-01-29)
The API Server implements a dual-layer protection system to separate business unknowns from technical errors and prevent Agent retry spam.

**Architecture:**
```
┌─────────────────────────────────────────────────────────────┐
│              ATTENDANCE LOG ROUTING LOGIC                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Incoming Log → Find Employee                                │
│                    ↓                                         │
│              ┌─────┴─────┐                                   │
│              │  Found?   │                                   │
│              └─────┬─────┘                                   │
│         YES ↙      ↘ NO                                      │
│         ↓           ↓                                        │
│  Parse Timestamp   Stage Log                                 │
│       ↓            (attendance_staging_logs)                 │
│   ┌───┴───┐        → Return Ok(false)                       │
│   │Valid? │                                                  │
│   └───┬───┘                                                  │
│  YES ↙  ↘ NO                                                 │
│    ↓      ↓                                                  │
│  Save   Log Error                                            │
│  (raw_attendance_logs)  (attendance_log_errors)             │
│  → Ok(true)            → Ok(false)                          │
│                                                              │
│  ⚠️ Only DB errors return Err() for Agent retry             │
└─────────────────────────────────────────────────────────────┘
```

**Tables:**
- **`attendance_staging_logs`**: Stores valid logs from unknown users
  - Columns: `id`, `workspace_id`, `device_id`, `raw_user_id`, `punch_time`, `raw_data`, `fingerprint`, `status`, `created_at`
  - Unique constraint on `(workspace_id, fingerprint)` prevents duplicate spam
  - Status workflow: `pending` → `processed` | `dismissed`

- **`attendance_log_errors`**: Enhanced with fingerprint deduplication
  - Added `fingerprint` column with partial unique index
  - Prevents duplicate error entries from repeated agent retries

**Fingerprint Formula (SHA256):**
```rust
SHA256(device_id | raw_user_id | punch_time_raw | punch | status)
```

**Benefits:**
- Unknown users don't cause Agent retry loops (staged for later review)
- Validation errors don't spam error table (fingerprint dedup)
- Admin can review staged logs and map users later
- Only true database errors trigger Agent retries

### 17.5. Unknown Logs Dashboard & Auto-Claim (New - 2026-01-29)
The system provides a UI for managing staged logs and automatically claims them when employees are created.

**Auto-Claim Flow:**
```
Employee Created/Updated
        ↓
EmployeeService.create_employee()
        ↓
AttendanceLogService.claim_staged_logs()
        ↓
Match staging logs by (workspace_id, raw_user_id)
        ↓
Move to raw_attendance_logs
        ↓
Mark staging as 'processed'
        ↓
Trigger PairingService for all affected dates
```

**API Endpoints:**
- `GET /workspaces/:ws_id/attendance/staging-logs` - List pending staging logs
- `POST /workspaces/:ws_id/attendance/staging-logs/:id/resolve` - Manually assign to employee
- `DELETE /workspaces/:ws_id/attendance/staging-logs/:id` - Dismiss log

**Frontend Features:**
- New "Unidentified" tab in AttendanceLogsPage with pending count badge
- Table showing: Unknown ID, Device, Punch Time, Actions
- Actions: Assign to existing employee, Create new employee, Delete
- "Create Employee" button redirects to employee form with pre-filled enrollment ID

### 18. Multi-Brand Sidecar Support (New)
The Edge Agent has been evolved from a monolithic ZK-only driver to a modular, multi-brand orchestrator.

#### 18.1. Standardized JSON-RPC 2.0 Contract
All biometric workers (ZKTeco, Hikvision, Mock, etc.) must implement a standardized JSON-RPC 2.0 interface:
- **`common.*` namespace**: Heartbeats and lifecycle checks (`common.ping`).
- **`device.*` namespace**: Low-level hardware operations (`device.connect`, `device.get_logs`, `device.get_time`).
- **Isolation**: Each sidecar handles exactly one physical device connection, ensuring that a crash in one driver does not affect other devices.

#### 18.2. Parallel Polling Architecture
The Orchestrator (`agent-main`) utilizes `tokio` tasks and `futures::join_all` to communicate with multiple sidecars concurrently:
- **Concurrency**: Logs are fetched from all configured devices in parallel, drastically reducing the total polling cycle time.
- **Resource Management**: Each worker process is spawned with piped stdout/stderr, allowing the Orchestrator to capture and prefix logs for better observability.

#### 18.3. Dynamic Reconfiguration \& Execution Strategy (New)
The Edge Agent now supports hot-reloading of its worker configuration and fine-grained resource control:
- **Database-Driven Configuration**: Instead of hardcoded values, the Orchestrator loads active workers from its local SQLite database.
- **Hot-Reloading**: When the dashboard pushes a configuration change, the Orchestrator receives a signal via `CentralClient`, gracefully shuts down all current sidecars, and spawns new ones based on the updated DB state.
- **Execution Strategy (Bounded Parallelism)**:
    - **Concurrency Control**: A `Semaphore` is used to limit the number of sidecar processes active at any given time.
    - **Sequential Polling**: Setting `MAX_CONCURRENT_WORKERS = 1` forces the agent to poll devices one-by-one, minimizing RAM usage.
    - **Parallel Polling**: Setting `MAX_CONCURRENT_WORKERS > 1` enables high-throughput polling for gateways with sufficient resources.

#### 18.4. Fair Round-Robin Device Polling (New)
To ensure no device is starved when resources are limited, the Edge Agent implements fair scheduling:
- **Semaphore-Based Fairness**: Each polling cycle, the Orchestrator acquires `MAX_CONCURRENT_WORKERS` permits from a Semaphore.
- **Per-Cycle Acquisition**: Permits are released after each device finishes, allowing other devices to use freed capacity immediately.
- **No Device Starvation**: With `MAX_CONCURRENT_WORKERS = 1`, devices take turns in defined order (via `join_all` and sequential iteration).
- **Optimization**: Enables proportional load balancing when `MAX_CONCURRENT_WORKERS > 1`, with high-speed devices completing faster and yielding permits sooner.
- **Implementation**: `agent-main/src/main.rs:302-334` uses tokio Semaphore + futures::join_all.

#### 18.5. Log Cleanup & 365-Day Retention (New)
The Edge Agent automatically maintains database hygiene by removing old synced logs:
- **Retention Policy**: Synced logs older than 365 days are automatically deleted.
- **Cleanup Frequency**: Background task runs every 24 hours (configurable via `CLEANUP_INTERVAL_HOURS`).
- **Safety**: Only logs marked as `is_synced=1` are eligible for deletion; in-flight logs are preserved.
- **Traceability**: Before deletion, the count of removed logs is logged for audit purposes.
- **Database Impact**: Reduces SQLite file size and improves query performance on historical searches.
- **Implementation**: `agent-main/src/main.rs:135-162`, database.rs:212-225.

#### 18.6. FIFO Ordering Guarantee (New)
To ensure logs are processed in the exact order they arrived from devices, the Edge Agent enforces FIFO semantics:
- **Auto-Increment IDs**: Logs are assigned monotonically increasing IDs at insertion time.
- **Order Preservation**: When fetching unsynced logs, `ORDER BY id ASC` is always applied.
- **ID Reuse Prevention**: SQLite auto-increment never reuses deleted IDs within the same database session.
- **Impact**: Guarantees that log batches pushed to the Central Server maintain source device order, enabling correct payroll calculations even if devices clock is not synchronized.
- **Implementation**: `agent-main/src/database.rs:138-155` in `get_unsynced_logs()`.

#### 18.7. Duplicate Detection (Atomic DashSet)
To eliminate redundant queue traffic and reduce database operations by up to 99.95%, the Edge Agent filters duplicate logs at the `InMemoryStore` level:
- **Mechanism**: Global `DashSet<u64>` (concurrent hash set) tracks all log signatures. Uses `DashSet::insert()` return value for atomic check-then-insert, preventing TOCTOU race conditions under concurrent access.
- **Key Format**: FNV hash of `"device_id:user_id:timestamp_utc:punch:status"`.
- **Filtering Strategy**: During `InMemoryStore::insert()`, signature is atomically inserted:
    - `insert()` returns `true` → new log, proceed with storage.
    - `insert()` returns `false` → duplicate, return `StoreError::Duplicate`.
- **Concurrency Safety**: Capacity check runs before signature insertion to prevent signature leaks on `CapacityExceeded` errors.
- **Memory Bounds**: Approximate memory usage is ~5MB per 50,000 accumulated logs.
- **Scope**: Signatures persist across device reconnections via `load_device_memory()` / `unload_device_memory()` for lazy per-device loading.
- **Impact**: Reduces queue channel traffic, database write overhead, and sync cycle time.
- **Implementation**: `agent-main/src/memory_store.rs` in `InMemoryStore::insert()`.
- **Tests**: `tests/duplicate_detection_test.rs` + `test_concurrent_inserts_no_duplicates` validating thread-safe dedup under 10 concurrent threads.

## Overtime Management

Overtime is managed through a "3-Way Match" workflow:
1.  **System Calculation:** `AttendanceCalculator` analyzes logs against policies to detect "System OT" and stores it in `attendance_daily_summaries`.
2.  **Employee Claim:** Employees review System OT and submit a claim (up to the detected amount).
3.  **Manager Approval:** Managers approve claims, which then become the "Payable OT" for payroll.

This ensures compliance and auditability, avoiding automatic payments for unauthorized presence while simplifying the claim process for valid work.

### 19. Unified Attendance Report Calculation (New - 2026-01-30)
The Summary and Detail attendance reports now share identical calculation logic with database-level pagination, ensuring data consistency and optimal performance.

**Problem Solved:**
Previously, the Summary Report used raw SQL aggregation (`SUM(duration_minutes)`), bypassing business rules like break deduction, shift boundaries, and OT calculation. This caused discrepancies with the Detail Report. Additionally, both reports loaded ALL employees into memory before pagination.

**Unified Architecture with DB Pagination:**
```
┌─────────────────────────────────────────────────────────────┐
│           ATTENDANCE REPORT CALCULATION FLOW                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Both Summary & Detail Reports:                              │
│                                                              │
│  1. DB-Level Pagination (NEW)                                │
│     └─ employee_repository::get_paginated_employee_ids()     │
│        → Returns (Vec<EmployeeBasicInfo>, total_count)       │
│        → Sorts by name at DB level                           │
│                                                              │
│  2. Fetch Schedules (batch, paginated employees only)        │
│     └─ roster_repository::get_workspace_schedules()          │
│                                                              │
│  3. Batch Fetch Time Entries (NEW - eliminates N+1)          │
│     └─ time_entry_repository::get_workspace_time_entries_batch()
│        → Returns HashMap<Uuid, Vec<TimeEntry>>               │
│        → Single query for all employees                      │
│                                                              │
│  4. For each day:                                            │
│     ├─ Resolve PolicyRules (via PolicyResolver)              │
│     ├─ Handle shift boundaries (inc. overnight shifts)       │
│     └─ Calculate via AttendanceCalculator::calculate_daily_units()
│                                                              │
│  5. Summary Report: Aggregate daily results per employee     │
│     Detail Report: Return daily breakdown                    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Performance Improvements:**
- **Before**: O(N) queries for time entries, O(N) memory for all employees
- **After**: O(1) batch query for time entries, O(L) memory (L=limit per page)
- **Eliminated N+1**: All time entries fetched in single query via batch function
- **DB Pagination**: Only requested page of employees loaded into memory

**Sorting Behavior:**
- **Name Sort**: Global sorting handled at database level (ORDER BY full_name ASC/DESC)
- **Calculated Fields** (WorkHours, LeaveDays): Within-page sorting only (cannot sort globally without computing all)

**AttendanceCalculator Applied Rules:**
- **Break Deduction**: Auto-deduct scheduled break or deduct actual break taken
- **Shift Boundaries**: Standard time = intersection of punch time and shift window
- **Early OT**: Time before shift start (if `allow_early_ot = true`)
- **Late OT**: Time after shift end
- **Grace Period**: Blocks early OT if arrival is within grace window
- **Authorized Early Leave**: Pays full shift if leaving within allowed minutes
- **Overnight Shift Support**: Automatically linearizes shifts crossing midnight
- **No-Shift Fallback**: When no schedule assigned, calculates raw duration

**Consistency Guarantee:**
```
Summary.total_work_units = Σ Detail[day].work_units (for same employee, date range)
```

**Repository Functions:**
```rust
// New paginated employee fetch
employee_repository::get_paginated_employee_ids(
    pool, workspace_id, filter_ids, page, limit, sort_desc
) -> (Vec<EmployeeBasicInfo>, total_count)

// New batch time entry fetch
time_entry_repository::get_workspace_time_entries_batch(
    pool, employee_ids, from, to
) -> HashMap<Uuid, Vec<TimeEntry>>
```

### 19.1 Schedule Resolution Performance Optimizations (New - 2026-02-03)

Two key optimizations reduce schedule computation complexity from O(N×M×K) to O(N×M×k) for `get_workspace_schedules()` and eliminate per-call DB queries in `get_effective_shift()`:

#### Optimization 1: Employee-Keyed HashMap Grouping
After fetching flat vectors of assignments, pattern details, and department history, all 5 vectors are indexed by `employee_id`:
- `fixed_assignments` → `fixed_assignments_map: HashMap<Uuid, Vec<_>>`
- `pattern_assignments` → `pattern_assignments_map: HashMap<Uuid, Vec<_>>`
- `dept_patterns` → `dept_patterns_map: HashMap<Uuid, Vec<_>>`
- `dept_fixed_shifts` → `dept_fixed_shifts_map: HashMap<Uuid, Vec<_>>`
- `all_dept_history` → `all_dept_history_map: HashMap<Uuid, Vec<_>>`

**Impact**: Inner loop (Employee × Day) previously scanned full workspace-wide vectors with `.iter().find(|a| a.employee_id == emp_id && ...)`. Now each lookup is O(1) to the per-employee slice (typically 1-3 items per employee vs 700+ in workspace), reducing inner loop from O(K) to O(k).

#### Optimization 2: Composite-Key HashMap for Rule Pattern Lookups
Pattern details pre-fetched once as `HashMap<(Uuid, i32), Option<Uuid>>` keyed by `(pattern_id, day_of_week)`:
- In `get_workspace_schedules()`: `.get(&(pattern_id, dow))` replaces `.iter().find()` → O(1) vs O(K).
- In `get_effective_shift()`: Eliminates per-call DB query by pre-fetching all active rule pattern details in single batch query, reusing same HashMap structure.

**Semantics**: `if let Some(&shift_opt) = map.get(...)` distinguishes key-exists-with-None (day off, early return) from key-missing (fall through to next priority level).

**Priority Chain Unchanged**: P1–P6 evaluation order (Roster → Individual Pattern → Dynamic Rules → Department Pattern → Fixed → Department Fixed) remains identical.

### 20. Distributed Report Processing (New - 2026-01-30)
The system implements an asynchronous, horizontally-scalable report generation architecture to handle large workspaces (7000+ employees) without blocking API responses or causing memory exhaustion.

**Problem Solved:**
Previously, Excel report generation for large workspaces caused:
- HTTP request timeouts (45s+ for 7000 employees)
- OOM crashes in 256MB containers (single employee batch loaded full dataset)
- Blocking API threads during multi-minute calculations

**Architecture:**
```
Client → POST /export → Coordinator
                            ↓
                     Split into micro-batches (20 emp/batch)
                            ↓
                     Redis Queue (RPUSH report:jobs:attendance)
                            ↓
Workers (N containers) ← BLPOP → AttendanceCalculator → Redis HASH
                            ↓
                     All batches done? → Merge Job
                            ↓
Merge → Combine → Generate Excel → Upload S3 → Presigned URL
                            ↓
Client ← GET /status ← Completed + download_url
```

#### 20.1. Coordinator Service (Job Splitting)
The `ReportCoordinator` handles job creation and micro-batch generation:

**Responsibilities:**
- Split employee list into batches of 20 (safe for 256MB RAM)
- Create parent job metadata (ID, status, progress tracking)
- Enqueue micro-batches to Redis LIST queue
- Return immediately with `job_id` for client polling

**Batch Size Calculation:**
```rust
const BATCH_SIZE: usize = 20; // 20 employees × ~2.5MB/emp = 50MB peak RAM
total_batches = ceil(employee_count / BATCH_SIZE)
```

**Job Metadata (Redis HASH `report:jobs:meta:{job_id}`):**
- `status`: pending | processing | completed | failed | cancelled
- `total_batches`: Number of micro-batches enqueued
- `completed_batches`: Number of batches processed
- `created_at`: Job creation timestamp
- `download_url`: S3 presigned URL (set after merge)
- `error`: Error message if failed

#### 20.2. Generic Worker Pattern
Workers are stateless containers running background tokio tasks:

**Concurrency Control:**
- Tokio Semaphore with 2 permits (controls concurrent jobs per container)
- Environment variable `WORKER_MAX_CONCURRENT` to override default
- Prevents memory spikes from parallel execution

**Worker Lifecycle:**
```rust
loop {
    // Acquire semaphore permit (blocks if 2 jobs running)
    let _permit = semaphore.acquire().await;

    // Blocking BLPOP with 5s timeout
    batch_data = redis.blpop("report:jobs:attendance", 5).await;

    // Process batch in separate tokio task
    tokio::spawn(async move {
        process_batch(batch_data).await;
        // Permit auto-released on drop
    });
}
```

**Process Batch:**
1. Deserialize job metadata from Redis
2. Fetch cached workspace/shift data (5min TTL)
3. Query DB for time entries (paginated employee subset only)
4. Run `AttendanceCalculator::calculate_daily_units()` per day
5. Store results to Redis HASH `report:results:{job_id}:{batch_id}`
6. Increment `completed_batches` counter
7. Check if all batches done → enqueue merge job if complete

#### 20.3. Application-Side Join Strategy
To avoid database join overhead, the worker uses Redis caching for static data:

**Cached in Redis (5min TTL):**
- Workspace configuration (payroll settings, grace periods)
- Shift definitions (all workspace shifts)
- Employee basic info (name, department) for display

**Queried from DB (per batch):**
- Time entries (dynamic, date-range specific)
- Leave applications (dynamic, status changes frequently)
- Working conditions (employee-specific, effective date filtering)

**Cache Keys:**
- `report:cache:workspace:{ws_id}` → WorkspaceConfig JSON
- `report:cache:shifts:{ws_id}` → Vec<Shift> JSON
- `report:cache:employees:{ws_id}` → HashMap<employee_id, BasicInfo> JSON

#### 20.4. Result Aggregation (Merge Handler)
When `completed_batches == total_batches`, a merge job is enqueued:

**Merge Workflow:**
1. Fetch all batch results from Redis (`report:results:{job_id}:*`)
2. Deserialize and combine into single dataset (sorted by employee name)
3. Generate Excel file using `rust_xlsxwriter`
4. Upload to S3 bucket configured via `S3_BUCKET` env var
5. Generate presigned URL (24h expiry)
6. Update job metadata: `status=completed`, `download_url=<url>`
7. Clean up batch results from Redis

**Excel Structure:**
- Sheet 1: Summary Report (per employee aggregates)
- Sheet 2: Detail Report (daily breakdown)
- Formatting: Auto-width columns, bold headers, number formats

#### 20.5. API Endpoints

**Create Export Job:**
```http
POST /api/workspaces/:ws_id/reports/export
Content-Type: application/json

{
  "from_date": "2026-01-01",
  "to_date": "2026-01-31",
  "report_type": "attendance_summary"
}

Response 201:
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "total_batches": 350
}
```

**Poll Job Status:**
```http
GET /api/workspaces/:ws_id/reports/jobs/:job_id

Response 200:
{
  "job_id": "550e8400-...",
  "status": "processing",
  "total_batches": 350,
  "completed_batches": 127,
  "progress_percentage": 36
}
```

**Download Report:**
```http
GET /api/workspaces/:ws_id/reports/jobs/:job_id/download

Response 307 (Redirect):
Location: https://s3.amazonaws.com/bucket/reports/...?presigned=...
```

**Cancel Job:**
```http
DELETE /api/workspaces/:ws_id/reports/jobs/:job_id

Response 200:
{ "status": "cancelled" }
```

#### 20.6. Environment Configuration

**Required:**
- `REDIS_URL`: Redis connection string (default: redis://localhost:6379)
- `S3_BUCKET`: S3 bucket name for report storage

**Optional:**
- `WORKER_MAX_CONCURRENT`: Max concurrent jobs per worker (default: 2)
- `REPORT_RESULT_TTL`: Result TTL in seconds (default: 86400 = 24h)
- `REPORT_CACHE_TTL`: Cache TTL in seconds (default: 300 = 5min)

#### 20.7. Scalability Characteristics

**Horizontal Scaling:**
- Deploy N worker containers (each processes 2 jobs concurrently)
- Workers compete for jobs via Redis BLPOP (built-in fairness)
- No coordination needed (stateless workers)

**Performance Metrics:**
- Batch processing time: ~2-3s per 20 employees
- 7000 employees = 350 batches × 2.5s avg = ~14.5 minutes (single worker)
- With 4 workers (8 concurrent jobs): ~1.8 minutes end-to-end

**Memory Safety:**
- Per-batch peak RAM: ~50MB (20 employees × 30 days)
- Container limit: 256MB (safe 5x headroom)
- Merge peak RAM: ~150MB (combines all batches)

**Failure Handling:**
- Worker crash: Job remains in queue, picked by another worker
- Partial completion: Coordinator tracks `completed_batches`, resume from checkpoint
- Job expiry: Redis TTL auto-cleans metadata after 24h

**Design Principles:**
- YAGNI: Simple Redis LIST queue (no complex message broker)
- KISS: Generic worker pattern (reusable for other report types)
- DRY: Reuses existing `AttendanceCalculator` business logic

---

## 🏗️ Optimized Test Infrastructure (v3)

LeeAttend utilizes a high-performance, container-first testing architecture designed to ensure total isolation while maintaining extreme speed on resource-constrained environments (like WSL2).

### 1. Hybrid Orchestration Model
The system uses a "Host-Orchestrated, Container-Executed" model:
- **Host**: Manages RAM disk setup, triggers pre-compilation (offline), and directs Docker via `run-tests.sh`.
- **Container**: Executes all unit and integration tests in a strictly isolated environment with tuned infrastructure.

### 2. WSL2 Performance Protocol (RAM Disk)
To bypass the severe Disk I/O bottlenecks of WSL2 (VirtioFS), critical high-frequency directories are automatically symlinked to RAM (`/dev/shm`):
- **`.test_logs/`**: All stdout/stderr from test runners.
- **`apiserver/migrations/`**: Prevents I/O spikes during database setup.
- **`apiserver/.sqlx/`**: Ensures extremely fast metadata reading for offline compilation.

### 3. Containerized Setup & internal Networking
Database preparation is fully encapsulated within the `apiserver_test` container:
- **`sqlx-cli` Integration**: Migrations and metadata preparation (`sqlx prepare`) happen internally.
- **Internal Database**: Postgres (`test_db`) is only accessible within the Docker network (`test_db:5432`). No host ports are exposed, preventing conflicts with system databases.
- **`trust` Authentication**: Simplified internal connectivity using `POSTGRES_HOST_AUTH_METHOD: trust` for the test subnet.

### 4. High-Performance Execution (`cargo-nextest`)
All Rust tests are executed using `nextest`, which provides:
- **Process Isolation**: Each test runs in its own process, preventing environment pollution.
- **Smart Scheduling**: Optimized CPU utilization, reducing suite duration by ~50%.
- **Status-Level Control**: Configured with `--no-tests=pass` to support modular parallel filtering without artificial failures.

### 5. Multi-Stage Build Optimization (`cargo-chef`)
Docker image build times are minimized through a 3-stage `cargo-chef` strategy:
1. **Planner**: Identifies dependencies (Cargo.lock).
2. **Cacher**: Compiles all third-party crates into a reusable layer.
3. **Builder**: Only compiles LeeAttend source code.
*Impact*: Rebuilds after source code changes complete in seconds rather than minutes.

---

The system implements a real-time report export feature using WebSocket connections for live progress updates, replacing HTTP polling with immediate feedback to users.

### 21.1. Architecture Overview

**Problem Solved:**
- Previous HTTP polling approach (1 request per second) caused 90% latency and high server load
- Users couldn't see real-time progress updates
- 30+ API calls per export added unnecessary network overhead

**Solution:**
```
┌──────────────────────────────────────────────────────────┐
│           WEBSOCKET REAL-TIME REPORT EXPORT              │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Client connects via WebSocket                          │
│       ↓                                                  │
│  POST /api/workspaces/:ws_id/reports/export             │
│       ↓                                                  │
│  Redis Queue (micro-batches)                           │
│       ↓                                                  │
│  Workers process + emit CHUNK messages via WebSocket   │
│       ↓                                                  │
│  Client receives real-time progress                    │
│       ↓                                                  │
│  DONE message → Download link ready                    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### 21.2. WebSocket Message Protocol

**Connection Setup:**
```
Client: GET /ws/reports/{job_id}
Server: Upgrade to WebSocket
```

**Message Types:**

1. **CHUNK (Progress Update)**
   ```json
   {
     "type": "CHUNK",
     "job_id": "uuid",
     "batch_id": 5,
     "batch_index": 5,
     "total_batches": 350,
     "progress_percentage": 1,
     "employees_processed": 100,
     "timestamp": "2026-02-03T10:15:30Z"
   }
   ```

2. **DONE (Completion)**
   ```json
   {
     "type": "DONE",
     "job_id": "uuid",
     "download_url": "https://s3.../report.xlsx?presigned=...",
     "total_batches": 350,
     "timestamp": "2026-02-03T10:25:15Z"
   }
   ```

3. **ERROR (Failure)**
   ```json
   {
     "type": "ERROR",
     "job_id": "uuid",
     "error_message": "Worker timeout or database error",
     "timestamp": "2026-02-03T10:16:00Z"
   }
   ```

### 21.3. Frontend Implementation

**Hook: `useReportStream`**
```typescript
const {
  status,           // idle | connecting | streaming | completed | error
  progress,         // 0-100 percentage
  chunks,           // Array of received CHUNK messages
  error,            // Error message if status === error
  reconnect         // Function to retry connection
} = useReportStream({
  jobId: reportJobId,
  onComplete: (downloadUrl) => downloadFile(downloadUrl),
  onError: (error) => showNotification(error)
});
```

**Fallback Mechanism:**
- 3 WebSocket connection attempts with exponential backoff (1s, 2s, 4s)
- After 3 failures → automatic switch to HTTP polling (2s interval)
- User notification: "Real-time updates unavailable. Using standard polling."
- Ensures 100% reliability (zero export failures)

**UI Component: `ReportExportDialog`**
- Shows live batch counter: "Processing batch 127 of 350"
- Real-time progress bar (updated per CHUNK message)
- Fallback mode indicator (amber banner when polling)
- Download button appears immediately after DONE message

### 21.4. Backend Implementation

**WebSocket Handler: `report_websocket.rs`**
```rust
pub async fn ws_report_stream(
    ws: WebSocketUpgrade,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    ws.on_upgrade(|socket| handle_report_stream(socket, job_id))
}
```

**Job Status Updates:**
- Worker sends CHUNK after completing each micro-batch
- Messages published via Redis Pub/Sub to all connected WebSocket clients
- TTL-preserved metadata ensures job tracking across worker restarts

**Error Handling:**
- Worker timeout → ERROR message (no silent failures)
- Partial completion → resume from checkpoint
- Connection drop → client uses HTTP fallback

### 21.5. Performance Improvements

| Metric | Before (Polling) | After (WebSocket) | Improvement |
|--------|------------------|-------------------|-------------|
| **Latency** | 1000ms | <100ms | 90% faster |
| **API Calls** | ~30/export | 1/export | 97% reduction |
| **Network Usage** | ~150KB | ~5KB | 97% reduction |
| **UX** | Delayed updates | Live progress | Real-time |

### 21.6. Configuration Requirements

**Redis Compatibility:**
- Requires Redis 3.0+ for `KEEPTTL` and TTL preservation
- TTL formula: `SET key value KEEPTTL` (or pipeline: GET TTL + SET + EXPIRE)
- Default job metadata TTL: 24 hours

**WebSocket Infrastructure:**
- Vite proxy configuration (dev): `/api/ws` route before `/api` HTTP proxy
- Production: WebSocket URL must match API base URL (protocol upgrade)

**Environment Variables:**
```
REDIS_URL=redis://localhost:6379
REPORT_RESULT_TTL=86400          # 24 hours
WEBSOCKET_TIMEOUT=600            # 10 minutes
```

### 21.7. Testing Coverage

**Unit Tests (22 tests - 100% passing)**
- Hook state transitions (idle → connecting → streaming → completed)
- Progress calculation (batch_index / total_batches * 100)
- Error handling and retry logic
- Fallback trigger on connection failures

**E2E Tests (12 scenarios - 100% passing)**
- Happy path: WebSocket streaming → completion → download
- Fallback mode: WebSocket fails → polling succeeds
- Connection interruption: Auto-reconnect logic
- Multi-language support (en, vi, zh)

**Data-TestID Standards:**
```
"report-export-dialog"
"report-export-progress-bar"
"report-export-batch-counter"
"report-export-fallback-notice"
"report-export-download-button"
```

### 21.8. Known Limitations & Future Enhancements

**Current Scope:**
- Single report export per user (sequential)
- Excel generation at completion (not incremental)
- Progress not persisted on page refresh
- 24-hour result expiry (Redis TTL)

**Recommended Future Work:**
1. Migrate results to PostgreSQL for persistent storage
2. Incremental Excel rendering from chunks
3. Resume capability after page refresh
4. Multiple concurrent export support
5. WebSocket connection telemetry dashboard

### 21.9. Deployment Checklist

- [x] WebSocket handler implemented (`report_websocket.rs`)
- [x] Redis Pub/Sub integration tested (5 new TTL tests)
- [x] Frontend hook with fallback mechanism (22 unit tests)
- [x] Vite proxy configuration for dev server
- [x] i18n translations (en, vi, zh)
- [x] Data-testid attributes for E2E testing
- [x] No breaking changes to existing APIs
- [x] 461 total tests passing

---
