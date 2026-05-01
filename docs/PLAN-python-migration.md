# Phân tích & Kế hoạch Chuyển đổi Shell sang Python (Infrastructure Automation)

## 1. Phân tích: Có cần thiết chuyển sang Python không?

**Câu trả lời là: CÓ, RẤT ĐÁNG GIÁ.** 

Mặc dù các giải pháp Bash hiện tại (`safe-run.sh`, `trap`, `setsid`) đã giải quyết được vấn đề PTY Leak, nhưng chúng giống như việc "đắp vá" (patching) lên một nền tảng vốn không được thiết kế cho việc quản lý tiến trình phức tạp. Dưới góc độ của DevOps và System Architecture, Python mang lại những ưu điểm vượt trội:

| Tính năng / Vấn đề | Bash / Shell Script | Python (subprocess / psutil) |
| :--- | :--- | :--- |
| **Quản lý File Descriptor (FD)** | Dễ rò rỉ, cú pháp khó đọc (`> /dev/null 2>&1 < /dev/null`) | Kiểm soát tuyệt đối với `stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, close_fds=True` |
| **Quản lý Process Group (Session)** | Phụ thuộc vào `setsid`, `pkill -P`. Dễ sót "tiến trình mồ côi" (zombie/orphans) nếu bị reparent. | Module `psutil` cho phép duyệt toàn bộ cây tiến trình (Process Tree). Giết sạch sẽ từ gốc rễ. |
| **Bắt lỗi & Cleanup (Graceful Shutdown)** | Dùng `trap` rất phức tạp, dễ xảy ra race-condition. | Khối `try...except...finally` bảo đảm cleanup chạy 100% trong mọi tình huống. |
| **Đọc Output Real-time (Non-blocking)** | Phải dùng mẹo `tail --pid` hoặc file tạm. | Dễ dàng dùng Thread hoặc Asyncio để stream output trực tiếp mà không chiếm PTY. |
| **Mở rộng đa nền tảng** | Khó chạy trên Windows (PowerShell/CMD). | Đa nền tảng (Linux, macOS, Windows). |

## 2. Phạm vi chuyển đổi (Scope)

Không cần thay thế toàn bộ. Chúng ta chỉ tập trung chuyển đổi các script **"điều phối vòng đời" (Lifecycle & Orchestration)**:
- `scripts/lifecycle.sh` ➡️ `scripts/lifecycle.py` (Quản lý dọn dẹp hệ thống, process sweep).
- `scripts/safe-run.sh` ➡️ `scripts/safe_run.py` (Wrapper chạy lệnh an toàn cho AI).
- `run-tests.sh` ➡️ `run_tests.py` (Orchestrator test tự động).

Các script đơn giản như `_cargo.sh`, `_npm.sh` vẫn có thể giữ lại dưới dạng shell script gọi vào Python Core.

## 3. Lộ trình thực thi (4-Phase Implementation)

### Giai đoạn 1: Foundation (Xây dựng thư viện lõi Python)
- Tạo module `scripts/lib/process_manager.py`.
- Triển khai class `SafeProcess` với khả năng: Detach PTY, quản lý FD, streaming log non-blocking, và thu dọn Process Tree qua `psutil`.

### Giai đoạn 2: Migration (Chuyển đổi các script nguy hiểm nhất)
- Viết lại `safe-run.sh` bằng Python. Đảm bảo input/output real-time nhưng PTY độc lập 100%.
- Viết lại module dọn dẹp `lifecycle.sh` (Sử dụng `psutil` thay thế cho `/proc` sweep).

### Giai đoạn 3: Tích hợp (Integration)
- Thay đổi `run-tests.sh` thành file Python hoặc cấu hình để nó sử dụng bộ quản lý Python mới.
- Cập nhật `AGENTS.md` về tiêu chuẩn Python mới.

### Giai đoạn 4: Xác thực & Cắt PTY (Verification)
- Cố tình chạy lệnh sinh Zombie/Orphans.
- Đo lường PTY FDs bằng script tự động kiểm tra lỗi rò rỉ.

---
**Trạng thái Kế hoạch:** CHỜ PHÊ DUYỆT
