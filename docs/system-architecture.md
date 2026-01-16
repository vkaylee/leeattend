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

#### 10.2. Pairing Engine (Logic ghép cặp)
Dịch vụ `PairingService` thực hiện nhiệm vụ chuyển đổi luồng sự kiện phẳng (flat events) thành các phiên làm việc (sessions) có ý nghĩa:
- **Deduplication (1-minute window)**: Tự động bỏ qua các lượt quẹt thẻ liên tiếp của cùng một nhân viên trong vòng 60 giây để chống nhiễu (ví dụ: nhân viên quẹt thẻ 2 lần do lo lắng máy chưa nhận).
- **Auto-Session Matching**:
    - Khi có log mới, Engine tìm kiếm phiên làm việc (Time Entry) gần nhất chưa đóng (`punch_out_time IS NULL`).
    - **Session Recovery (14-hour window)**: Nếu log mới xảy ra trong vòng 14 giờ kể từ lúc bắt đầu phiên, nó sẽ được coi là giờ **Ra (Punch Out)** và đóng phiên đó lại.
    - **New Session**: Nếu không tìm thấy phiên cũ hoặc khoảng cách quá 14 giờ, Engine tự động khởi tạo một phiên làm việc mới (giờ **Vào - Punch In**).
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
  - `services/`: Pure business logic (Payroll, Pairing, Accrual).
  - `repositories/`: Database abstraction layer via SQLx.
  - `models/`: Type-safe domain entities.

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


