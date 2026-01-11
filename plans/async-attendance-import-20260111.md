# Implementation Plan: Async Attendance Import with Status Polling

### Approach
- **Giải pháp:** Chuyển đổi luồng Upload từ đồng bộ sang bất đồng bộ (Asynchronous). Sử dụng Database để quản lý trạng thái Jobs, cho phép người dùng theo dõi tiến độ ngay cả khi refresh trang.
- **Luồng xử lý:**
    1. Admin upload file.
    2. Backend lưu file tạm (hoặc giữ trong memory nếu nhỏ) -> Tạo bản ghi Job với trạng thái `processing` -> Trả về `job_id`.
    3. Backend khởi chạy `tokio::spawn` để xử lý ngầm.
    4. Frontend nhận `job_id` và bắt đầu polling API mỗi 2s để cập nhật progress bar.

---

### Steps

1. **Database Migration**
   Tạo bảng theo dõi Jobs: `attendance_import_jobs`.

2. **Backend implementation**
   - **Model & Repo:** Định nghĩa `ImportJob` và các hàm CRUD.
   - **Service:** Refactor `batch_import_from_excel` để chạy trong background và cập nhật progress định kỳ.
   - **Handlers:** Thêm các endpoint kiểm tra trạng thái job.

3. **Frontend implementation**
   - **API:** Thêm hàm `fetchLatestImportJob` và `fetchImportJobStatus`.
   - **UI:** Thêm component `ImportStatusBanner` hiển thị tiến độ xử lý (X/Y dòng).
   - **Polling:** Sử dụng `useQuery` với `refetchInterval`.

4. **Testing**
   - Test việc refresh trang khi đang upload.
   - Test việc xử lý file lớn (> 1000 dòng).

---

### Rollback Plan
1. Revert handlers và service về trạng thái xử lý đồng bộ.
2. Xóa các route và bảng DB mới.

---

### Security Checklist
- [x] **Workspace Isolation:** Đảm bảo người dùng chỉ xem được trạng thái job của workspace mình.
- [x] **Panic Safety:** Sử dụng `catch_unwind` hoặc đảm bảo cập nhật trạng thái `failed` nếu task background gặp lỗi nghiêm trọng.
