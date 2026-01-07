# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

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