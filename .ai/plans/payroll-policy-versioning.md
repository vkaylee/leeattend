# Clone & Versioning cho Payroll Incentive Policies

## Goal
Bảo vệ tính toàn vẹn của dữ liệu tính lương lịch sử bằng cách hỗ trợ nhân bản (Clone) chính sách nhanh chóng và áp dụng quản lý theo phiên bản/thời gian (Versioning/Temporal) để đảm bảo khi tính lại lương tháng cũ sẽ dùng đúng luật cũ.

## Tasks

- [ ] Task 1: **Cập nhật Database Schema (Versioning)** 
  → Thêm các cột `effective_from` (DATE) và `effective_to` (DATE) vào bảng `payroll_incentive_policies` hoặc đổi mô hình sang parent-child (1 Policy có nhiều Version).
  → Verify: Run `sqlx migrate run` thành công.

- [ ] Task 2: **Điều chỉnh Backend API (Fetch/Calculate)** 
  → Cập nhật `fetch_incentive_policies` và hàm tính lương `calculate_incentives` để chỉ query các chính sách có hiệu lực (active) trong khoảng thời gian `start_date` -> `end_date` của kỳ lương.
  → Verify: Tính lương cho tháng trước không bị ảnh hưởng bởi chính sách mới tạo hôm nay.

- [ ] Task 3: **Xây dựng API Clone Policy**
  → Tạo endpoint `POST /api/workspaces/:wsId/payroll/incentive-policies/:id/clone` trên Rust Backend, thực hiện copy toàn bộ policy và `payroll_incentive_tiers` với tên mới (vd: "Tên cũ (Copy)") và trạng thái `is_active = false`.
  → Verify: Chạy CURL hoặc Swagger thấy trả về policy mới với các rules y hệt.

- [ ] Task 4: **Cập nhật UI/UX (Clone Button)**
  → Thêm nút "Clone" (icon Copy) vào `BonusPolicyCard` trên giao diện web. Gắn hàm gọi API Clone và tự động refetch lại danh sách.
  → Verify: Bấm nút Clone, một thẻ policy mới xuất hiện ngay lập tức với chữ "(Copy)".

- [ ] Task 5: **Cập nhật UI/UX (Versioning / Date Picker)**
  → Thêm trường chọn Ngày áp dụng (`effective_from`) vào form tạo/sửa chính sách `BonusPolicyForm`. 
  → Verify: Khi lưu chính sách, dữ liệu ngày hiệu lực được gửi đúng định dạng xuống API.

## Done When
- [ ] Tính lương lại cho tháng cũ sẽ không bị ảnh hưởng bởi những sửa đổi chính sách của tháng hiện tại.
- [ ] Người dùng có thể dễ dàng nhân bản (Clone) các cấu hình phức tạp với 1 click thay vì phải tạo lại từ đầu.
- [ ] `./test.sh all` pass với cả backend và frontend.
