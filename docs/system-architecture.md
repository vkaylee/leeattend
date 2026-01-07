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

## Infrastructure Diagram
```ascii
[ Web Dashboard ] <--- REST API ---> [ API Server ] <--- WebSocket ---> [ Agent ] <---> [ ZKTeco Devices ]
      (Next.js)                         (Rust)                           (Python)
```

## 1. Agent (Client Side)
Running on the local machine where ZKTeco devices are located.
- **Responsibility**: 
    - Connects to ZK devices.
    - Buffers logs locally (Safety).
    - Syncs logs to Central Server via WebSocket.
    - **Executes remote commands** (sync-time, reboot, etc.) received from Central Server.
- **Stack**: Python 3.12, FastAPI, PyZK2.

## 2. API Server (Server Side)
Central processing unit for multi-site management.
- **Responsibility**:
    - Receives attendance logs from multiple Agents.
    - **Manages active WebSocket connections** from Agents.
    - **Routes remote commands** to specific Agents.
    - Provides Reporting APIs for Dashboard.
- **Stack**: **Rust**, **Axum**, Tokio, PostgreSQL.

## Data Flow
1. **Collector**: Agent periodically fetches logs from ZK devices.
2. **Persistence**: Logs are saved immediately to local PostgreSQL with `is_synced=0`.
3. **Sync Worker**: A dedicated background task queries unsynced logs and pushes them to the Central Server.
4. **Reliability (ACK)**: Server sends an acknowledgment for each log. Agent updates `is_synced=1` only upon receiving ACK.
5. **Backoff**: If the connection fails, the Central Client uses an **Exponential Backoff** strategy to reconnect safely.
6. **Storage**: Central Server stores logs in the Cloud Database.

## Technology Stack

### Agent
- **Language**: Python 3.12+
- **Framework**: FastAPI (Async)
- **Library**: pyzk2 (Protocol)

### API Server
- **Language**: Rust (1.75+)
- **Framework**: Axum (Web), Tokio (Runtime)
- **Database**: PostgreSQL
- **Logging**: Tracing