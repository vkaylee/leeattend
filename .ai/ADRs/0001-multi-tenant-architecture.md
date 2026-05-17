# ADR 001: Multi-Tenant Architecture (System DB + Tenant DBs)

## Status
**Accepted** (Date: 2026-05-17)

## Context
LeeAttend định hướng là một hệ thống HRM & Attendance chuẩn Enterprise (Premium Pro Max). Với số lượng khách hàng B2B lớn, cần có một chiến lược rõ ràng về mức độ cô lập dữ liệu (Data Isolation) giữa các workspace để đảm bảo bảo mật, tuân thủ pháp lý (Compliance), và khả năng mở rộng (Scalability) dài hạn.

## Decision
Hệ thống sẽ áp dụng kiến trúc **Tách biệt: System DB (Global) + Các Tenant DBs**.
- **System DB**: Là database trung tâm, lưu trữ thông tin có tính chất Global như `Users` (tài khoản đăng nhập), `Workspaces` (thông tin metadata của tenant), `Billing`, `System Configurations`.
- **Tenant DBs**: Mỗi workspace sẽ được cấp phát một Database vật lý riêng biệt (hoặc Schema độc lập hoàn toàn). Dữ liệu nghiệp vụ như `Attendance`, `Payroll`, `Shifts`, `Employees` sẽ được lưu trữ hoàn toàn trong Tenant DB này.
- **Connection Pooling**: Sử dụng **Supavisor** làm trung tâm định tuyến (router) và quản lý Connection Pool cho các Tenant DBs.

## Consequences (Hệ quả)

### ✅ Positive (Ưu điểm)
1. **Bảo mật & Cô lập tuyệt đối:** Dữ liệu của công ty A không bao giờ có thể bị lộ sang công ty B do lỗi logic code (SQL Injection / quên check `workspace_id`).
2. **Khả năng mở rộng (Physical Scalability):** Có thể dễ dàng chuyển một Tenant DB của khách hàng VIP sang một server vật lý khác mạnh hơn mà không ảnh hưởng hệ thống chung.
3. **Backup & Restore:** Có thể backup/restore hoặc xóa toàn bộ dữ liệu của một công ty cực kỳ dễ dàng mà không cần phải chạy script rà soát từng dòng (Row-level deletion).

### ⚠️ Negative (Thách thức kỹ thuật phải giải quyết)
1. **Application-Level Joins:** Bắt buộc phải xử lý map dữ liệu bằng code Rust khi cần thông tin từ cả 2 DB (VD: Lấy danh sách nhân viên trong Tenant DB, sau đó query lấy Email từ System DB).
2. **Dynamic Connections:** Backend (Axum) phải lấy connection tương ứng với `workspace_id` thông qua Supavisor thay vì dùng chung 1 `PgPool` duy nhất.
3. **Migration Orchestration:** Phải phát triển một công cụ/luồng CI/CD để tự động chạy database migrations qua tất cả các Tenant DBs khi có phiên bản mới. Không thể chạy 1 lệnh `sqlx migrate` đơn thuần nữa.
4. **Provisioning:** Khi tạo mới Workspace, cần có luồng tự động tạo DB vật lý -> Chạy schema init -> Đăng ký Supavisor.
