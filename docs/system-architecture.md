# System Architecture

## 🛡️ Reliability & Sync Logic

### 1. Guaranteed Delivery (ACK Protocol)
- Agent chỉ đánh dấu log là `is_synced = 1` sau khi nhận được bản tin `type: "ack"` từ Server.
- Nếu không nhận được ACK trong vòng 10 giây, Agent sẽ giữ log ở trạng thái chưa gửi và thử lại ở batch sau.

### 2. Batch Sync Optimization
- Agent gom tối đa 100 bản ghi vào một message WebSocket duy nhất.
- Sử dụng `SYNC_INTERVAL=0.1s` để đẩy dữ liệu liên tục khi có backlog lớn, giúp giảm thiểu độ trễ.

### 3. Config Push & Smart Rebuild
- Khi Admin thay đổi thiết bị trên Web Dashboard, Server sẽ gửi message `config_push` tới Agent tương ứng.
- **Delta Check**: Agent so sánh chữ ký (signature) của cấu hình mới và cũ. Chỉ thực hiện `stop` và `restart` các monitor nếu có sự khác biệt về IP, Port hoặc danh sách ID.
- **Empty State**: Nếu danh sách thiết bị nhận được là rỗng (`[]`), Agent coi đây là lệnh dừng tất cả giám sát và xóa sạch bộ nhớ tạm.
- **Persistence**: Mọi cấu hình nhận được đều được lưu vào `devices.json` để phục vụ chế độ Offline-first.

### 4. Optimized Boot Sequence
Quy trình khởi động của Agent:
1. **Local Init**: Nạp cache log và cấu hình `devices.json`.
2. **Immediate Monitor**: Bắt đầu giám sát các thiết bị local ngay lập tức (Downtime ~ 0).
3. **Cloud Connect**: Kết nối WebSocket tới API Server.
4. **Sync & Update**: Gửi trạng thái hiện tại và nhận cấu hình mới nhất (nếu có).

### 5. Agent Trust & Whitelisting
Hệ thống sử dụng cơ chế phê duyệt danh tính chủ động:
1. **Registration**: Admin khai báo `agent_id` và `token` trên Dashboard.
2. **Connection**: Agent kết nối WebSocket và gửi `token` trong bản tin Handshake.
3. **Verification**: 
   - Nếu Token sai -> Trả về `INVALID_TOKEN` -> Ngắt.
   - Nếu chưa được phê duyệt (`is_trusted=false`) -> Trả về `AGENT_NOT_TRUSTED` -> Ngắt.
4. **Active State**: Chỉ Agent đã được Trust mới nhận được Config Push và có quyền đẩy Attendance Logs.

### 6. Memory Safety & Resource Protection
Agent được thiết kế để chạy ổn định 24/7 trên phần cứng giới hạn (Raspberry Pi/Mini PC):
- **Bounded Queue**: Hàng đợi xử lý log được giới hạn (Max 1000 items). Khi hàng đợi đầy (do mất mạng hoặc DB chậm), quy trình đọc dữ liệu từ thiết bị sẽ tạm dừng (Backpressure), ngăn chặn tràn RAM.
- **Smart Polling**: Tần suất đọc dữ liệu tự động điều chỉnh dựa trên tải:
  - Default: 10s
  - Logs > 5000: 60s
  - Logs > 20000: 300s
- **Rolling Deduplication**: Bộ nhớ đệm chống trùng lặp sử dụng cấu trúc `deque` + `set` với cửa sổ trượt cố định (20,000 items), đảm bảo bộ nhớ không tăng theo thời gian.

### 7. Multi-Tenancy (Workspaces)
Hệ thống hỗ trợ phân tách dữ liệu hoàn toàn giữa các tổ chức thông qua mô hình Workspace:
- **Global Identity**: Người dùng có một tài khoản duy nhất trên toàn hệ thống (Global User), cho phép họ tham gia vào nhiều Workspace khác nhau mà không cần tạo nhiều tài khoản.
- **Isolation**: Mọi tài nguyên nghiệp vụ (Agent, Thiết bị, Log) đều gắn liền với một `workspace_id`. Một Workspace đại diện cho một tổ chức hoặc khách hàng độc lập.
- **MemberContext Isolation**: API Server thực thi kiểm tra bảo mật thông qua cặp header `X-User-Id` và `X-Workspace-Id`. Extractor `MemberContext` sẽ xác thực tính hợp lệ của người dùng trong workspace trước khi cho phép truy cập tài nguyên.
- **RBAC (Role-Based Access Control)**: Phân quyền chi tiết trong từng Workspace (Owner, Admin, Member):
  - `Owner`: Quyền cao nhất, được phép xóa workspace.
  - `Admin`: Được phép quản lý thành viên (invite/remove) và cấu hình thiết bị/agent.
  - `Member`: Quyền xem dữ liệu và tương tác cơ bản.
- **Handshake Verification**: Khi Agent kết nối WebSocket, Server tự động xác định Workspace tương ứng từ `agent_id` và thực thi cách ly luồng dữ liệu ngay tại tầng kết nối.

## Infrastructure Diagram
```ascii
[ Web Dashboard ] <--- REST API ---> [ API Server ] <--- WebSocket (Outbound Only) ---> [ Agent ] <---> [ ZKTeco Devices ]
 (Tauri Desktop)                        (Rust)                                           (Python CLI)
```

## 1. Agent (Client Side)
... (giữ nguyên) ...

## 2. API Server (Server Side)
... (giữ nguyên) ...

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

## Technology Stack

### Agent
- **Language**: Python 3.12+
- **Architecture**: **Single Responsibility Principle (SRP)**, Factory Pattern.
- **Runtime**: asyncio (Non-blocking I/O)
- **Library**: pyzk2 (Hardware Protocol)
- **Framework**: **None (Pure CLI)** - FastAPI removed for performance.

### API Server
- **Language**: Rust (1.75+)
- **Framework**: Axum (Web), Tokio (Runtime)
- **Database**: PostgreSQL
- **Logging**: Tracing