# Kế Hoạch Tối Ưu Bảng Công Tháng (Timesheet Matrix) Vào Trung Tâm Báo Cáo /reports

Status: Approved

## Mục Tiêu
Chuyển đổi Bảng công tháng 31 ngày (Timesheet Matrix Grid) từ trang Nhật ký quẹt thẻ (`/attendance-logs`) sang trang Báo cáo chấm công (`/reports`), biến `/reports` thành Trung tâm Tổng hợp & Báo cáo Chấm công (Attendance & Timesheets Hub). Trang bị thanh chuyển đổi chế độ xem linh hoạt giữa **Bảng ma trận 31 ngày (Grid View)** và **Báo cáo tổng hợp số giờ (Summary View)**, đồng thời đưa `/attendance-logs` trở về đúng bản chất giám sát nhật ký vi mô từ máy chấm công.

---

## Kiến Trúc & Các Ràng Buộc (Constraints & Interfaces)
- **Single Responsibility (SRP):**
  - `/attendance-logs`: Chuyên trách dữ liệu vi mô từ phần cứng chấm công (`Recorded Logs`, `Unknown Staging`, `No-Show Shifts`).
  - `/reports`: Chuyên trách dữ liệu vĩ mô và kết quả nghiệp vụ C&B (`Monthly Timesheet Grid`, `Summary Hours Report`).
- **DRY (Don't Repeat Yourself):**
  - Tái sử dụng `TimesheetMatrixGrid` và `attendance-code-registry.ts` đã xây dựng.
  - Dùng chung bộ lọc kỳ tháng, phòng ban, tìm kiếm nhân viên trên trang `/reports`.
- **Open/Closed (OCP):** Chế độ xem mở rộng thông qua enum `ReportViewMode: "summary" | "matrix"`, hỗ trợ lưu trạng thái vào `localStorage` và query param URL.
- **Trợ năng (WCAG 2.1 AA):** Các nút chuyển đổi chế độ xem có `aria-pressed`, `aria-label` và focus ring rõ ràng.

---

## Các Bước Triển Khai (Tasks)

1. **Dọn dẹp trang `/attendance-logs`:**
   - Xóa bỏ tab `matrix` khỏi `attendance-logs-page.tsx`, `attendance-logs-tabs.tsx`, và `search-schema.ts`.
   - Đảm bảo trang log giữ nguyên 3 tab chuẩn: `recorded`, `unknown`, `unrecorded`.

2. **Mở rộng trạng thái chế độ xem trên `/reports` (`use-reports-page.ts`):**
   - Thêm trạng thái `viewMode: "summary" | "matrix"`.
   - Tự động đồng bộ và lưu trữ tùy chọn vào `localStorage` (`leeattend_reports_view_mode`).

3. **Cập nhật `ReportsHeader`:**
   - Bổ sung `ViewModeSwitcher` (gồm 2 nút: `Lưới 31 ngày` - icon `LayoutGrid`, `Tổng hợp giờ` - icon `Table`).

4. **Tích hợp `TimesheetMatrixGrid` vào `ReportsPage`:**
   - Khi `viewMode === "matrix"`, hiển thị `TimesheetMatrixGrid` với tháng/năm đang chọn và danh sách nhân viên của workspace.
   - Khi `viewMode === "summary"`, hiển thị `ReportsTable` như hiện tại.

5. **Kiểm thử tự động & Xác minh trực quan Playwright MCP:**
   - Chạy toàn bộ unit test webdashboard.
   - Dùng Playwright MCP xác minh trực tiếp trải nghiệm chuyển đổi chế độ xem trên `/reports`.
