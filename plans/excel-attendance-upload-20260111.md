# Implementation Plan: Excel Attendance Upload (via Auto-Approved Adjustments)

### Approach
- **Giải pháp:** Sử dụng thư viện `calamine` để parse file Excel (.xlsx) trực tiếp tại Backend.
- **Quy trình:** 
    1. Admin upload file lên endpoint `/api/workspaces/:ws_id/attendance/upload`.
    2. Backend parse file, ánh xạ `employee_id_number` sang `employee_id`.
    3. Với mỗi dòng hợp lệ, tạo một `AttendanceAdjustmentRequest` với trạng thái `approved`.
    4. Hệ thống tự động kích hoạt logic của `AttendanceAdjustmentService` để tạo/cập nhật `TimeEntry`.
- **Tại sao chọn phương án này:** Tận dụng tối đa logic validation và audit trail đã có, đảm bảo tính nhất quán dữ liệu mà không cần viết lại logic tính toán thời gian phức tạp.

---

### Steps

1. **Install Dependencies**
   Thêm thư viện xử lý Excel và multipart form:
   ```bash
   # In apiserver directory
   cargo add calamine
   ```
   *Lưu ý: axum đã có multipart nếu được enable.*

2. **Backend: Implement Upload Handler**
   - Sửa `apiserver/src/handlers.rs`: Thêm `upload_attendance_excel`.
   - Sửa `apiserver/src/services/attendance_adjustment_service.rs`: Thêm logic xử lý hàng loạt.
   - Sửa `apiserver/src/lib.rs`: Đăng ký route mới.

3. **Frontend: Integration**
   - Sửa `webdashboard/src/lib/api.ts`: Thêm function `uploadAttendanceExcel`.
   - Sửa `webdashboard/src/pages/AttendanceAdjustmentsPage.tsx`: Thêm nút "Import Excel".

4. **Testing**
   - Tạo file test `apiserver/tests/attendance_excel_upload_tests.rs`.
   - Kiểm tra việc tạo TimeEntry sau khi upload.

---

### Timeline
| Phase | Duration |
|-------|----------|
| Dependencies | 5 min |
| Backend Implementation | 40 min |
| Frontend UI/API | 30 min |
| Testing & Refinement | 30 min |
| **Total** | **1 hour 45 min** |

---

### Security Checklist
- [x] **Auth checks:** Chỉ Admin/Owner mới có quyền upload.
- [x] **Workspace Guard:** Kiểm tra nhân viên trong file có thuộc Workspace hiện tại không.
- [x] **File validation:** Giới hạn kích thước file upload và kiểm tra định dạng extension.
