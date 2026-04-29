# Kế hoạch Refactor: Enterprise Early Arrival Policy (Zero Tech Debt)

## 🎯 Mục tiêu
Đưa hệ thống đạt chuẩn "Enterprise Architecture" thông qua việc tập trung hóa (Centralize) cấu hình chấm công. Chuyển hoàn toàn `EarlyArrivalStrategy` từ bảng `workspaces` sang JSONB `rules` của bảng `attendance_policy_versions`. 
Triệt tiêu Nợ kỹ thuật (Technical Debt) bằng cách duy nhất một Nguồn sự thật (Single Source of Truth) cho các quy tắc chấm công.

## 🏗 Kiến trúc phân cấp mới (Resolution Hierarchy)
1. **Shift Overrides:** Ghi đè trực tiếp trên ca làm việc (đã có).
2. **Assigned Policy:** Lấy từ `AttendanceRules` của chính sách đang được gán (có thể là Default Policy của công ty hoặc Policy riêng của phòng ban).
*(Loại bỏ hoàn toàn việc đọc cấu hình từ Workspace)*

---

## 📋 Các giai đoạn triển khai (Phases)

### Phase 1: Database Migration (Sạch sẽ Dữ liệu)
- [ ] Chạy lệnh tạo migration: `./scripts/_sqlx.sh migrate add move_early_arrival_to_policy`
- [ ] **`up.sql`:**
  1. Cập nhật cột `rules` (JSONB) trong bảng `attendance_policy_versions`, chèn thêm field `"early_arrival_strategy"` bằng cách lấy giá trị từ bảng `workspaces` (thông qua `policy_id` -> `workspace_id`).
  2. `ALTER TABLE workspaces DROP COLUMN early_arrival_strategy;`
- [ ] **`down.sql` (Rollback an toàn):**
  1. Thêm lại cột `early_arrival_strategy` vào bảng `workspaces`.
  2. Phục hồi dữ liệu từ Default Policy quay ngược lại bảng `workspaces`.

### Phase 2: Nâng cấp Backend Models
- [ ] **Xóa cấu hình cũ (`apiserver/src/models/workspace.rs`):**
  - Loại bỏ trường `early_arrival_strategy` khỏi tất cả các struct liên quan đến Workspace (Ví dụ: `Workspace`, `CreateWorkspaceRequest`, `UpdateWorkspaceRequest`).
- [ ] **Thêm cấu hình mới (`apiserver/src/models/attendance.rs`):**
  - Thêm trường `pub early_arrival_strategy: EarlyArrivalStrategy` vào struct `AttendanceRules`. (Sử dụng `#[serde(default)]` nếu cần thiết cho an toàn deserialize).
- [ ] **Cập nhật DTOs (`apiserver/src/handlers/attendance/policies.rs`):**
  - Thêm trường `early_arrival_strategy` vào struct `UpdateAttendanceRulesRequest` và `AttendancePolicyVersionResponse`.

### Phase 3: Cập nhật Domain Logic (Resolver)
- [ ] **Cấu trúc lại `PolicyResolver` (`apiserver/src/domain/policy/resolver.rs`):**
  - Sửa đổi `resolve_attendance_rules`: Thay vì lấy mặc định từ `Workspace`, hàm này cần gọi `resolve_attendance_policy` để lấy `AttendanceRules` hiện hành của nhân viên.
  - Sử dụng `early_arrival_strategy` từ `AttendanceRules` để build ra `PolicyRules`.

### Phase 4: Cập nhật API Handlers & Services
- [ ] **Sửa Handlers `policies.rs`:** Cập nhật logic tạo/sửa policy để nhận và lưu field mới.
- [ ] **Dọn dẹp Workspace Handlers (`workspaces/crud.rs`):** Xóa các đoạn mã đọc/ghi `early_arrival_strategy` khi tạo mới hoặc cập nhật workspace.
- [ ] **Kiểm tra Services tính toán:** Đảm bảo `attendance_calculator.rs` và `time_tracking_service.rs` vẫn hoạt động trơn tru sau khi `PolicyResolver` được refactor.

### Phase 5: Frontend & Tích hợp (UI/UX)
- [ ] **API Client Types:** Cập nhật TypeScript interface tương ứng với thay đổi của Backend.
- [ ] **Gỡ bỏ UI cũ:** Xóa trường cấu hình "Early Arrival Strategy" khỏi màn hình Workspace Settings chung.
- [ ] **Thêm UI mới:** Thêm Dropdown Field cho "Early Arrival Strategy" vào màn hình Policy Settings (Thời gian làm việc & Nghỉ ngơi), đảm bảo nó nằm cùng chỗ với "Rounding Policy" và "Work Unit".

### Phase 6: Testing & Verification (Bắt buộc)
- [ ] Chạy `./scripts/_cargo.sh clippy` để rà soát lỗi type do việc xóa column gây ra.
- [ ] Chạy `./scripts/_sqlx.sh migrate run` để kiểm tra migration script.
- [ ] Chạy `./run-tests.sh --api` để đảm bảo toàn bộ hệ thống test pass.
- [ ] Tạo ADR (Architecture Decision Record) lưu lại kiến trúc mới này.
