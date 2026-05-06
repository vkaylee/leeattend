# Kế hoạch Refactor (Làm sạch) Kiến trúc Salary Components

## Bối cảnh
Bảng `salary_component_definitions` đang chứa các cột boolean (`is_taxable`, `is_base_for_ot`, `is_insurance_base`) gây trùng lặp logic với hệ thống `salary_base_groups`. Việc loại bỏ các cột này sẽ đưa hệ thống về chuẩn Enterprise (Mô hình Accumulators), tận dụng tối đa bảng trung gian `salary_component_base_members` để cấu hình động.

---

## 🎯 Mục tiêu
1. **Dọn dẹp Database**: Xóa bỏ các cột boolean dư thừa trong `salary_component_definitions`.
2. **Cập nhật Backend API**: Sửa API liên quan đến thao tác tạo và sửa đổi Salary Component để sử dụng danh sách `base_group_ids` thay vì các giá trị boolean. Đảm bảo toàn vẹn dữ liệu trong quá trình lưu trữ (Transaction).
3. **Cập nhật Frontend UI**: Điều chỉnh màn hình Thêm mới / Chỉnh sửa Salary Component. Thay thế các checkbox (Taxable, BHXH, OT Base) bằng một giao diện Select / Multi-select động lấy danh sách từ `salary_base_groups`.
4. **Kiểm thử & Rà soát (QA)**: Chạy lại Unit / Integration test để đảm bảo engine tính lương và các luồng liên quan không bị phá vỡ.

---

## 📋 Chi tiết các công việc (Task Breakdown)

### Phase 1: Database Migration
- [ ] **Tạo file Migration**: Dùng `sqlx` để sinh ra file migration mới.
- [ ] **Viết lệnh DROP**: Viết lệnh `ALTER TABLE salary_component_definitions DROP COLUMN is_taxable, DROP COLUMN is_base_for_ot, DROP COLUMN is_insurance_base;`.
- [ ] **Cập nhật Model Rust**: Vào `apiserver/src/models/` hoặc `apiserver/src/repositories/` và xóa các field tương ứng khỏi các Struct của Rust. Xóa bỏ những râu ria liên quan đến những field này trong repository query.

### Phase 2: Cập nhật Backend API (Rust)
- [ ] **Sửa Payload Model**: Trong `apiserver/src/handlers/`, cập nhật `CreateComponentRequest` và `UpdateComponentRequest`. Thêm mảng `base_group_ids: Option<Vec<Uuid>>`.
- [ ] **Sửa luồng Xử lý (Transaction)**:
  - Khi tạo Component: Insert Component -> Lấy `id` -> Lặp qua mảng `base_group_ids` và insert các dòng tương ứng vào `salary_component_base_members`. Đưa tất cả vào một `sqlx::Transaction`.
- [ ] **Chỉnh sửa Engine/Dịch vụ**: Nếu engine tính lương hiện tại vẫn còn lỡ đọc `c.is_taxable` thì phải chuyển sang đọc dựa theo Base Group Mapping.
- [ ] **Cập nhật / Viết mới Integration Test**: Sửa các file test hiện tại vì request payload đã thay đổi. Đảm bảo API POST/PUT pass 100%.

### Phase 3: Cập nhật Frontend (Web Dashboard)
- [ ] **Cập nhật API Client**: Sửa type/interface của `SalaryComponentDefinition` trong React Query/Axios type definition. Xóa các field boolean, thêm `base_group_ids`.
- [ ] **Sửa Form UI**:
  - Tại Form tạo / sửa Component: Gỡ bỏ 3 checkbox hardcode.
  - Thay thế bằng giao diện `MultiSelect` hoặc `Combobox`, kết nối vào React Query fetch `salary_base_groups`.
- [ ] **Kiểm tra UX**: Đảm bảo hiển thị tốt, có dịch i18n đầy đủ cho "Nhóm cấu hình lương" (Salary Bases).

### Phase 4: Kiểm thử, Đánh giá và Triển khai
- [ ] **Run Linter**: Chạy `cargo clippy` và `npm run lint`.
- [ ] **Run Test Pipeline**: Chạy `./test.sh api`.
- [ ] **Deploy Check**: Khởi động Local server và test trực tiếp trên UI luồng: Tạo Component -> Assign vào Base -> Xem DB.

---

## 🛠 Phân bổ Agent / Kỹ năng
- **Backend / DB Specialist**: Thực hiện Phase 1 & 2 (Cần đảm bảo Rust / SQLx / Transaction).
- **Frontend Specialist**: Thực hiện Phase 3 (React, Multi-select, i18n).
- **QA / DevOps**: Chạy test và rà soát Phase 4.
