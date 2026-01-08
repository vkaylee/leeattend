# Implementation Plan: Shift Rostering System

## Subtasks
1.  **Database Migration**:
    *   Tạo bảng `shift_rosters` (lịch làm việc cụ thể ngày).
    *   Tạo bảng `department_shifts` (gán ca mặc định cho phòng ban).
    *   Tạo bảng `employee_default_shifts` (gán ca mặc định cho cá nhân).
2.  **Backend (Rust)**:
    *   Cập nhật `models.rs` với các struct mới.
    *   Tạo `roster_repository.rs` xử lý CRUD và Bulk Insert.
    *   Cập nhật `payroll_service.rs` để tính lương dựa trên thứ tự ưu tiên: Roster > Employee Default > Department Default.
    *   Đăng ký API handlers cho Roster.
3.  **Frontend (React)**:
    *   Cập nhật `api.ts` với các hàm fetch/post mới.
    *   Tạo trang `RosterPage.tsx` với giao diện bảng/lịch.
4.  **Verification**:
    *   Chạy `cargo check` và `npm run build`.
    *   Viết test case kiểm tra logic ưu tiên ca.

## Challenges
*   Xử lý va chạm lịch (Overlap) khi gán ca.
*   Tính toán công cho ca qua đêm trong Roster.

## Success Criteria
*   Có thể gán ca cho nhân viên theo ngày cụ thể.
*   Hệ thống tự động nhận diện ca làm việc khi nhân viên chấm công dựa trên Roster.
*   Payroll tính đúng lương dựa trên ca trong Roster.
