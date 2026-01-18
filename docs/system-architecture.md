# System Architecture

## 🛡️ Reliability & Sync Logic

### 1. Guaranteed Delivery (ACK Protocol)
- Agent chỉ đánh dấu log là `is_synced = 1` sau khi nhận được bản tin `type: "ack"` từ Server.
- Nếu không nhận được ACK trong vòng 10 giây, Agent sẽ giữ log ở trạng thái chưa gửi và thử lại ở batch sau.

### 10. Attendance Pairing & Audit Engine
Hệ thống xử lý dữ liệu quẹt thẻ (punch) theo mô hình **Event-Sourcing Audit Trail** để đảm bảo tính minh bạch và khả năng phục hồi dữ liệu:

#### 10.1. Raw Attendance Logs (Bất biến)
- **Immutable Store**: Mọi lượt quẹt thẻ từ Agent (thiết bị ZKTeco) hoặc Import (file Excel/CSV) đều được lưu nguyên bản vào bảng `raw_attendance_logs`. 
- **Auditability**: Dữ liệu này không bao giờ bị xóa hoặc sửa, cho phép người quản trị luôn có thể đối soát lại nguồn gốc của một session làm việc.
- **Source Tracking**: Mỗi bản ghi lưu rõ nguồn (`device`, `import`, `manual`) và ID thiết bị tương ứng.

#### 10.2. Pairing Engine (Strategy Pattern)
Dịch vụ `PairingService` thực hiện nhiệm vụ chuyển đổi luồng sự kiện phẳng (flat events) thành các phiên làm việc (sessions) thông qua các thuật toán có thể cấu hình tùy theo đặc thù doanh nghiệp:

- **Deduplication (1-minute window)**: Tự động bỏ qua các lượt quẹt thẻ liên tiếp của cùng một nhân viên trong vòng 60 giây để chống nhiễu.
- **Configurable Strategies**:
    - **Pattern Matching (DP)**: Sử dụng quy hoạch động (Dynamic Programming) để khớp các logs với các mẫu ca làm việc (Shift Patterns) có sẵn. Tốt nhất cho môi trường văn phòng có ca cố định và cần theo dõi giờ nghỉ giữa ca.
    - **First & Last (Anchor)**: Thuật toán đơn giản chỉ lấy bản ghi đầu tiên (In) và cuối cùng (Out) trong ngày. Tốt nhất cho môi trường sản xuất (nhà máy) có lượng Overtime lớn và không cần theo dõi chi tiết giờ nghỉ.
    - **Sequential (Retail)**: Ghép cặp các bản ghi theo thứ tự thời gian (Log 1+2, 3+4...). Tốt nhất cho môi trường bán lẻ hoặc y tế nơi nhân viên làm ca gãy (split shifts) hoặc ra vào nhiều lần trong ngày.
    - **State-based (FSM)**: Sử dụng máy trạng thái hữu hạn để xử lý logs dựa trên metadata `punch_type` từ thiết bị (Vào, Ra, Nghỉ...). Có độ chính xác cao nhất đối với các thiết bị phần cứng hỗ trợ phím chức năng.
    - **Chronological (Fallback)**: Tự động ghép cặp theo thứ tự thời gian khi không có lịch trình (shift) được gán.
- **Metadata Awareness**: Engine hiện tại sử dụng cấu trúc `LogEvent` thay vì chỉ lưu mốc thời gian, cho phép các thuật toán truy cập vào trạng thái nguyên bản từ thiết bị để đưa ra quyết định ghép cặp chính xác hơn.
- **Enhanced Reliability**: Thuật toán DP được cấu hình với độ trễ (penalty) lên tới 300 phút, đảm bảo không bỏ sót các trường hợp làm thêm giờ (overtime) muộn.
- **Timezone Safety**: Mọi mốc thời gian được lưu trữ dưới dạng UTC và chỉ được chuyển đổi sang múi giờ địa phương (ICT) tại tầng hiển thị.

#### 10.3. Strict Import Validation
Quy trình nhập dữ liệu từ file được thực thi với tiêu chuẩn cao nhất về độ chính xác:
- **Workspace-Aware Parsing**: Engine phân tích ngày tháng nghiêm ngặt dựa trên cài đặt của từng Workspace (`DD/MM/YYYY`, `MM/DD/YYYY`, v.v.). Không tự động "đoán" định dạng để tránh sai lệch dữ liệu.
- **Instructional Feedback**: Khi có dòng lỗi, hệ thống cung cấp thông báo chi tiết: **Số dòng**, **Mã nhân viên**, **Giá trị sai thực tế** và **Định dạng mong đợi** để người dùng sửa file nhanh nhất.

### 11. Hierarchical Leave Policy Assignment
Hệ thống gán chính sách nghỉ phép được thiết kế theo mô hình phân cấp linh hoạt, cho phép áp dụng các quy tắc cộng phép khác nhau cho từng nhóm đối tượng:

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
    │     - Sum all "Base for OT" Components at segment start. │
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
[ Web Dashboard ] <--- REST API ---> [ API Server ] <--- WebSocket (Outbound Only) ---> [ Agent (Rust) ] <--- JSON-RPC (TCP) ---> [ ZK Worker (Python) ] <---> [ ZKTeco Devices ]
 (Tauri Desktop)                        (Rust)                                           (Orchestrator)                        (Sidecar Driver)
                                                                                               │
                                                                                         [ attendance.db ]
                                                                                             (SQLite)
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