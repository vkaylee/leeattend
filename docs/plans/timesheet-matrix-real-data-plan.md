# Ke Hoach Noi Du Lieu That Cho Bang Cong Thang (Timesheet Matrix Real Data Integration)

Status: Approved

## Mục Tiêu
Thay thế toàn bộ thuật toán sinh mã công giả lập (mock deterministic hash) trong `TimesheetMatrixGrid` bằng dữ liệu chấm công thực tế từ Backend API `GET /v1/workspaces/{ws_id}/reports/attendance?type=daily`, hiển thị chính xác trạng thái quẹt thẻ, đi làm đủ ca, nửa ca, nghỉ phép, nghỉ lễ, và nghỉ tuần cho từng nhân sự trong tháng.

## Rule Compliance
- **Rule ID `PERF-001` (Performance & Memoization):** Sử dụng `Map<"${employeeId}-${date}", DailyAttendanceReport>` với độ phức tạp $O(1)$ khi truy xuất từng ô trên lưới ma trận 31 ngày $\times$ $N$ nhân viên.
- **Rule ID `CODE-001` (Single Responsibility & OCP):** Tách bộ phân giải mã công `resolveAttendanceCode` thành hàm thuần túy (pure function) tái sử dụng `ATTENDANCE_CODE_REGISTRY`, độc lập với tầng hiển thị UI.
- **Rule ID `TEST-001` (Hermetic Verification):** Viết unit tests kiểm tra chính xác bộ phân giải mã công và kiểm thử tích hợp truy xuất API báo cáo hàng ngày.

---

## Chi Tiết Kế Hoạch Triển Khai

### Phase 1: Xây Dựng Bộ Phân Giải Mã Chấm Công Thực Tế (Pure Resolver Engine)
- **Tập tin:** `webdashboard/src/components/features/attendance/attendance-code-resolver.ts`
- **Nội dung:**
  - Xây dựng hàm `resolveDailyAttendanceCode(report?: DailyAttendanceReport, isWeekend?: boolean): AttendanceCodeConfig`.
  - Quy tắc phân giải chuẩn Luật Lao động VN:
    - Nếu `is_holiday === true` $\rightarrow$ Ký hiệu `L` (Nghỉ lễ/Tết).
    - Nếu `status === "leave"`: Phân loại theo `is_leave_paid` $\rightarrow$ Ký hiệu `P` (Phép năm hưởng nguyên lương) hoặc `Ô`/`TS` (Ốm đau/Thai sản) hoặc `Ro` (Nghỉ không lương).
    - Nếu có giờ công thực tế (`computed_std_hours`):
      - $\ge 7.0$ giờ $\rightarrow$ `X` (Đủ công 1.0).
      - $3.5 \le \text{hours} < 7.0$ $\rightarrow$ `X/2` (Nửa công 0.5).
      - $> 0$ và $< 3.5$ giờ $\rightarrow$ `X/2` kèm cảnh báo thiếu giờ.
    - Nếu không có quẹt thẻ / vắng mặt:
      - Cuối tuần (`isWeekend === true`) $\rightarrow$ `Ro` (Nghỉ hàng tuần).
      - Ngày thường $\rightarrow$ `V` (Vắng không phép).
  - Viết unit tests cho `resolveDailyAttendanceCode` trong `attendance-code-resolver.test.ts`.

### Phase 2: Hook Tải & Chỉ Mục Dữ Liệu Chấm Công Tháng (Data Fetching & O(1) Indexing)
- **Tập tin:** `webdashboard/src/pages/reports/hooks/use-monthly-timesheet-data.ts`
- **Nội dung:**
  - Tạo hook `useMonthlyTimesheetData({ wsId, month, year, enabled })` gọi `fetchAttendanceReport` với `type: "daily"`, `start_date` ngày đầu tháng và `end_date` ngày cuối tháng.
  - Sử dụng TanStack Query với caching (`staleTime: 5 * 60 * 1000`).
  - Chuyển mảng kết quả thành `Map<string, DailyAttendanceReport>` với key dạng `${employee_id}:${date_str}` để truy xuất tức thời.
  - Tính toán tổng kết thực tế cho từng nhân viên: Tổng công thực (`workDays`), Tổng phép (`leaveDays`), Tổng công hưởng lương (`payableDays`).

### Phase 3: Tích Hợp Lưới Ma Trận & Xuất CSV Dữ Liệu Thật
- **Tập tin:** `webdashboard/src/components/features/attendance/timesheet-matrix-grid.tsx`
- **Nội dung:**
  - Nhận `attendanceMap` và `isLoadingAttendance` từ props.
  - Thay thế hàm mock `getEmployeeDayCode` cũ bằng `resolveDailyAttendanceCode(attendanceMap.get(...))`.
  - Hiển thị Skeleton loader khi đang tải dữ liệu tháng.
  - Cập nhật hàm `handleExportMatrixCSV` để xuất đúng dữ liệu thật và các cột tổng công tính lương từ Backend.

### Phase 4: Đồng Bộ Bộ Lọc Tại `reports-page.tsx`
- **Tập tin:** `webdashboard/src/pages/reports-page.tsx`
- **Nội dung:**
  - Kết nối `useMonthlyTimesheetData` với `specificMonth` hiện tại của trang báo cáo.
  - Tự động kích hoạt tải khi người dùng chuyển sang `viewMode === "matrix"`.
  - Truyền dữ liệu và trạng thái tải vào `TimesheetMatrixGrid`.

### Phase 5: Kiểm Thử Tự Động & Hồi Quy (Verification)
- Chạy unit tests frontend: `leedevkit test web --unit-only --pattern "reports-page|timesheet-matrix|attendance-code"`.
- Chạy toàn bộ unit test backend: `leedevkit test api --unit-only`.
- Xác minh HTTP endpoints và kiểm tra không có lỗi build/types/lint.
