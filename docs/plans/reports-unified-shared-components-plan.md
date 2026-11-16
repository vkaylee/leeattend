# Ke Hoach Hop Nhat & Toi Uu Component Dung Chung Giua Bang Cong Thang & Tong Hop Gio

Status: Approved

## Mục Tiêu
Tái cấu trúc kiến trúc trang `/reports` theo nguyên lý DRY & Single Responsibility: đưa thanh bộ lọc (`ReportFilters`) lên tầng trang dùng chung xuyên suốt cho cả **Bảng công tháng (Matrix Grid)** và **Tổng hợp số giờ (Summary Table)**, chuẩn hóa hiển thị nhân viên bằng component dùng chung `EmployeeIdentityCell`, hợp nhất nút xuất dữ liệu thông minh trên `ReportsHeader`, và loại bỏ hoàn toàn các thành phần UI bị phân mảnh/trùng lặp.

## Rule Compliance
- **Rule ID `CODE-001` (Single Responsibility & DRY):** Loại bỏ logic tìm kiếm và lọc dữ liệu bị nhân bản bên trong `TimesheetMatrixGrid`; tập trung quản lý trạng thái lọc tại `useReportsPage`.
- **Rule ID `A11Y-001` (WCAG 2.1 AA Compliance):** Đảm bảo component định danh nhân sự `EmployeeIdentityCell` có nhãn `aria-label`, phím Tab điều hướng chuẩn xác và độ tương phản đạt chuẩn $\ge 4.5:1$.
- **Rule ID `PERF-001` (Performance & Debounce Optimization):** Sử dụng chung kết quả tìm kiếm đã debounce (`debouncedSearch`) và bộ lọc phòng ban cho cả 2 chế độ hiển thị mà không gây re-render thừa.

---

## Chi Tiết Kế Hoạch Triển Khai

### Phase 1: Xây Dựng Component Dùng Chung `EmployeeIdentityCell`
- **Tập tin:** `webdashboard/src/components/common/employee-identity-cell.tsx`
- **Nội dung:**
  - Thiết kế component định danh nhân sự chuẩn Enterprise:
    - Avatar vuông bo góc với 2 chữ cái viết tắt (Initials Badge) với màu sắc thương hiệu.
    - Họ và tên in đậm (hỗ trợ hover underline và click mở Drawer phân tích).
    - Mã nhân viên (`EMPxxx`) font monospace với kích thước chữ nhỏ gọn, tương phản cao.
    - Tên tổ chức/công ty phụ (nếu có).
  - Viết unit test cho `EmployeeIdentityCell` trong `employee-identity-cell.test.tsx`.

### Phase 2: Đưa `ReportFilters` Lên Tầng Trang Dùng Chung (`reports-page.tsx`)
- **Tập tin:** `webdashboard/src/pages/reports-page.tsx` và `webdashboard/src/pages/reports/components/reports-table.tsx`
- **Nội dung:**
  - Di chuyển `ResponsiveToolbar` và `ReportFilters` từ bên trong `ReportsTable` ra ngoài `reports-page.tsx`.
  - Đặt bộ lọc ngay bên dưới `ReportsHeader` để áp dụng chung cho cả 2 chế độ xem:
    - Khi lọc theo Tháng/Năm: Cả bảng ma trận 31 ngày và bảng tổng hợp giờ đều tự động nhảy theo.
    - Khi lọc theo Phòng ban hoặc Trạng thái nhân viên: Danh sách nhân sự được đồng bộ tự động cho cả 2 view.
    - Ô tìm kiếm nhân viên trên thanh lọc dùng chung cho cả 2 view.
  - Tinh gọn `ReportsTable.tsx`: chỉ tập trung hiển thị `NavigableList` và bảng tổng hợp số giờ.

### Phase 3: Tinh Gọn & Tái Cấu Trúc `TimesheetMatrixGrid`
- **Tập tin:** `webdashboard/src/components/features/attendance/timesheet-matrix-grid.tsx`
- **Nội dung:**
  - Nhận danh sách nhân viên đã được lọc từ page (`filteredEmployees`).
  - Xóa bỏ ô `input` tìm kiếm nội bộ trùng lặp và bộ đếm nhân viên thừa trong grid header.
  - Thay thế phần hiển thị Mã NV và Họ tên ở các cột cố định (sticky columns) bằng component `EmployeeIdentityCell`.
  - Cung cấp phương thức `exportMatrixCSV` ra ngoài qua React ref (`useImperativeHandle`) để Header có thể kích hoạt trực tiếp.

### Phase 4: Nút Xuất Dữ Liệu Thông Minh Trên `ReportsHeader` (Context-Aware Export)
- **Tập tin:** `webdashboard/src/pages/reports/components/reports-header.tsx` và `webdashboard/src/pages/reports-page.tsx`
- **Nội dung:**
  - Loại bỏ nút "Xuất ma trận" bên trong grid để giao diện thoáng sạch.
  - Nút "Xuất dữ liệu" trên `ReportsHeader` tự nhận diện:
    - Khi ở chế độ **Bảng công tháng:** Click trực tiếp tải xuống file CSV Ma trận 31 ngày chuẩn UTF-8 BOM.
    - Khi ở chế độ **Tổng hợp giờ:** Click mở `ReportExportDialog` xuất báo cáo chi tiết hoặc bảng tổng hợp.

### Phase 5: Kiểm Thử Tự Động & Xác Minh Trực Quan (Verification)
- **Kiểm thử tự động:**
  - Chạy toàn bộ unit test web: `./leedevkit test web --unit-only --pattern "reports|employee-identity|timesheet-matrix"`.
  - Chạy toàn bộ backend API test: `./leedevkit test api --unit-only`.
- **Xác minh trực quan:**
  - Dùng Playwright script kiểm tra giao diện thực tế khi đổi qua lại giữa 2 chế độ xem: kiểm tra đồng bộ bộ lọc phòng ban, tìm kiếm nhân viên và nút xuất file.
