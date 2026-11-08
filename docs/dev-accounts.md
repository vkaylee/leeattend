# Danh Sách Tài Khoản Thử Nghiệm Chuẩn (On-Premise Appliance)

Mô hình hệ thống đã chuyển đổi từ SaaS đa workspace sang kiến trúc **Single-Tenant On-Premise**. Toàn bộ các vai trò SaaS thừa (`owner`, `platform_admin`, `support`, `billing_admin`) đã được loại bỏ và hợp nhất.

Hệ thống tinh gọn chuẩn gồm **5 vai trò duy nhất**:
1. **`admin`**: Quản trị viên tối cao (kế thừa `platform_admin` và `owner`, toàn quyền hệ thống, hạ tầng, worker monitor, cấu hình & nhân sự).
2. **`hr_manager`**: Quản lý Nhân sự, hồ sơ, hợp đồng, chấm công, nghỉ phép, tính lương.
3. **`shift_manager`**: Quản lý Ca kíp, phân ca, đổi ca, mẫu lịch làm việc (Working Patterns).
4. **`team_lead`**: Trưởng nhóm / Quản lý trực tiếp, duyệt giải trình công, duyệt đơn tăng ca (OT).
5. **`employee`**: Nhân viên tiêu chuẩn, tự phục vụ (ESS: clock in/out, xin nghỉ, xem bảng công).

**Mật khẩu dùng chung cho tất cả tài khoản (development-only fixture):** `development-password`

---

## 1. Bảng Chi Tiết 5 Tài Khoản Thử Nghiệm

| STT | Email | Vai Trò (Role) | System Role | Mã Nhân Viên | Phòng Ban | Chức Danh (Job Title) | Phạm Vi Quyền Hạn |
|:---:|:---|:---|:---:|:---|:---|:---|:---|
| 1 | `admin@example.invalid` | **`admin`** | **`admin`** | `EMP001` | Ban Giám Đốc | System Administrator | Toàn quyền cao nhất: Hạ tầng, Giám sát Worker & Queue (`/infra/monitor`), cấu hình tổ chức, phân quyền. |
| 2 | `hr.manager@example.invalid` | **`hr_manager`** | *none* | `EMP005` | Phòng Nhân Sự | HR Director | Quản lý Nhân sự, hồ sơ lao động, hợp đồng, phê duyệt đơn nghỉ phép, tính lương, import chấm công. |
| 3 | `shift.manager@example.invalid` | **`shift_manager`** | *none* | `EMP006` | Phòng Kỹ Thuật | Shift Operations Manager | Quản lý Ca kíp: Lịch làm việc tuần/tháng, mẫu lịch xoay ca, điều phối đổi ca. |
| 4 | `team.lead@example.invalid` | **`team_lead`** | *none* | `EMP007` | Phòng Kỹ Thuật | Technical Team Lead | Trưởng nhóm: Duyệt giải trình công (Attendance Adjustments), duyệt đơn làm thêm giờ (OT Approvals). |
| 5 | `employee@example.invalid` | **`employee`** | *none* | `EMP008` | Phòng Kỹ Thuật | Standard Staff Member | Nhân viên tự phục vụ (ESS): Máy chấm công (Clock in/out), xem lịch sử quẹt thẻ, nộp đơn xin nghỉ, xin làm OT. |

---

## 2. Lệnh Nạp Lại Dữ Liệu & Kiểm Tra Tự Động

```bash
# Nạp tài khoản chỉ trong disposable dev/smoke DB đã xác nhận. Không dùng live DB.
# Chạy SQL từ psql client đã cấu hình kết nối tới DB disposable:
psql "$DISPOSABLE_DATABASE_URL" -v ON_ERROR_STOP=1 -f scripts/seed_dev_accounts.sql

# Kiểm tra tự động đăng nhập và quyền hạn. BASE_URL mặc định
# http://127.0.0.1:8088/api/v1; đặt biến BASE_URL để trỏ tới môi trường smoke disposable.
# Thông tin đăng nhập lấy từ .secrets/playwright-credentials.env (đã gitignore)
# hoặc biến môi trường PLAYWRIGHT_<ROLE>_EMAIL / PLAYWRIGHT_<ROLE>_PASSWORD.
BASE_URL="http://127.0.0.1:8088/api/v1" ./scripts/test_all_dev_accounts.sh
```
