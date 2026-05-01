# Kế hoạch Refactor Architecture Debt (crud.rs)

## 📌 Tổng quan
File `apiserver/src/handlers/workspaces/crud.rs` hiện đang là một "God File" chứa hàng trăm dòng code truy vấn raw SQL kết hợp với logic xử lý múi giờ và mức độ ưu tiên ca làm việc (priority shifts). Nhiệm vụ này nhằm tách lớp truy cập dữ liệu (Data Access Layer) ra khỏi tầng điều hướng (Routing/Handler Layer), tuân thủ nguyên tắc Clean Architecture mà không làm thay đổi hành vi hiện tại của các API liên quan đến thống kê (Workspace Stats).

**Người chịu trách nhiệm (Agent):** `backend-specialist`, `technical-debt-management`

---

## 🛠 Các giai đoạn thực hiện

### Giai đoạn 1: 🛡 Protect (Bảo vệ)
Trước khi chạm vào mã nguồn hiện tại, chúng ta cần đảm bảo có lưới an toàn (safety net).

- [ ] **1.1. Kiểm tra Test Coverage:** Chạy `./run-tests.sh --api` để xem các bài test hiện tại có cover các API `/api/workspaces/:ws_id/stats`, `attendance-trend`, và `department-attendance` không.
- [ ] **1.2. Bổ sung Integration Tests (Nếu thiếu):** Viết thêm các test cases tại thư mục `tests/integration/workspaces/` cho 3 endpoints bị ảnh hưởng chính:
  - `get_attendance_trend`
  - `get_department_attendance`
  - `get_unexcused_absences`
- [ ] **1.3. Đánh giá Baseline:** Chạy lại toàn bộ test suite. Phải đảm bảo đang ở trạng thái 100% Passed.

### Giai đoạn 2: 🏗 Refactor (Cải tạo Layer)
Di chuyển logic xử lý DB sang một Repository riêng biệt, hoàn toàn không sửa đổi business logic của chức năng.

- [ ] **2.1. Tạo Repository:** 
  - Tạo file mới: `apiserver/src/repositories/workspace_stats_repository.rs`
  - Định nghĩa trait `WorkspaceStatsRepository` bao gồm các method: `fetch_attendance_trend`, `fetch_department_attendance`, `fetch_overtime_trend`, v.v.
  - Tạo struct `SqlxWorkspaceStatsRepository` (có chứa DbPool) để triển khai (implement) trait này.
- [ ] **2.2. Di dời SQL Queries:** 
  - Cắt toàn bộ các string raw SQL khổng lồ trong `crud.rs` và dán sang các method tương ứng ở repository mới.
  - Chuyển đổi các magic numbers (như khoảng thời gian `29 days`) thành tham số truyền vào hàm (e.g. `start_date`, `end_date`).
- [ ] **2.3. Cập nhật AppState:**
  - Khai báo và khởi tạo `WorkspaceStatsRepository` trong struct `AppState` hoặc module quản lý Dependency Injection tương ứng của hệ thống.
- [ ] **2.4. Refactor Handlers:**
  - Cập nhật các hàm xử lý API trong `crud.rs` để gọi tới logic vừa tạo thông qua `state.repo.workspace_stats.fetch_...()` thay vì gọi trực tiếp bằng `sqlx::query!`.

### Giai đoạn 3: ✅ Verify (Kiểm chứng)
Đảm bảo mã nguồn sau refactor an toàn, chặt chẽ và không phá vỡ bất kỳ luồng tính năng nào.

- [ ] **3.1. Type Safety Check:** Chạy lệnh `./scripts/_cargo.sh clippy`. Chỉnh sửa toàn bộ các lỗi tham chiếu (lifetime) hay type mismatch. Kết quả yêu cầu: 0 Errors, 0 Warnings.
- [ ] **3.2. Regression Test:** Chạy lệnh `./run-tests.sh --api`. Tất cả test (bao gồm test vừa bổ sung ở bước 1.2) phải vượt qua thành công. Kết quả JSON trả về phải khớp với API chuẩn.
- [ ] **3.3. Check Code Format:** Chạy lệnh `./scripts/_cargo.sh fmt` để đồng nhất code style của Rust.

---

## 🚦 Chấp nhận hoàn thành (Definition of Done)
- Kích thước của `crud.rs` phải giảm tối thiểu 50%.
- Tầng Handlers không chứa trực tiếp các câu `sqlx::query!` dài phức tạp (Ngoại trừ các query CRUD cơ bản 1-2 dòng nếu chưa tiện gỡ).
- Lệnh chạy test `./run-tests.sh --api` báo xanh 100%.
- `./scripts/_cargo.sh clippy` vượt qua mà không cần tắt cảnh báo (`#[allow(clippy...)]`).
