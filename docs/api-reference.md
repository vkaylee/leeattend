# API Reference

## 🚀 API Server (Central - Rust)
Base URL: `http://localhost:3000`

### 0. Authentication & Context (Updated)

Hệ thống sử dụng JWT (JSON Web Token) để xác thực người dùng.

- **Access Token**: Gửi trong header `Authorization: Bearer <token>`. Hết hạn sau 1 giờ.
- **Refresh Token**: Dùng để lấy Access Token mới. Hết hạn sau 7 ngày.
- **X-Workspace-Id**: Header bắt buộc cho các yêu cầu liên quan đến tài nguyên cụ thể của tổ chức.

#### 🌐 API URL Resolution Strategy (Client-side)
Đối với các ứng dụng Web (đặc biệt trong môi trường Development với Vite), client sẽ ưu tiên gọi API thông qua đường dẫn tương đối `/api`. Vite sẽ tự động proxy các request này đến backend server đang chạy trên một port ngẫu nhiên.
- `VITE_API_URL` trong `.env` chỉ được sử dụng cho các môi trường yêu cầu URL tuyệt đối (ví dụ: Tauri, hoặc Production khi API đặt ở một domain khác và không có proxy). Trong môi trường phát triển web tiêu chuẩn, việc set `VITE_API_URL` sẽ khiến client bỏ qua proxy và gọi thẳng vào địa chỉ hardcode, gây ra lỗi `ERR_CONNECTION_REFUSED`.

#### POST /auth/register
Đăng ký tài khoản người dùng mới.
- **Request**: `{ "email": "...", "password": "...", "full_name": "..." }`
- **Response (200)**: `{ "access_token": "...", "refresh_token": "...", "user_id": "..." }`

#### POST /auth/login
Đăng nhập để lấy token.
- **Request**: `{ "email": "...", "password": "..." }`
- **Response (200)**: `{ "access_token": "...", "refresh_token": "...", "user": { ... } }`

#### GET /dev/super-user (Debug Mode Only)
Tự động đăng nhập với tư cách Superuser (admin@leeattend.com).
- **Mục đích**: Phục vụ phát triển nhanh (Onboarding) và Testing.
- **Response (200)**: `{ "access_token": "...", "workspace_id": "...", "user_id": "..." }`

#### POST /auth/refresh
Làm mới Access Token.
- **Request**: `{ "refresh_token": "..." }`
- **Response (200)**: `{ "access_token": "...", "refresh_token": "..." }`

#### POST /auth/change-password
Đổi mật khẩu người dùng hiện tại.
- **Headers**: `Authorization: Bearer <token>`
- **Request**: `{ "old_password": "...", "new_password": "..." }`
- **Response (200 OK)**: "Password changed successfully"

---

### 1. User Profile (New)

Quản lý thông tin định danh toàn cầu.

#### GET /me
Lấy thông tin hồ sơ của người dùng hiện tại.
- **Headers**: `X-User-Id`
- **Response (200)**: `User` object (`id`, `email`, `full_name`, `avatar_url`, `bio`, `created_at`)

#### PATCH /me
Cập nhật thông tin hồ sơ.
- **Headers**: `X-User-Id`
- **Request**: `{ "full_name": "...", "avatar_url": "...", "bio": "..." }`
- **Response (200 OK)**

#### GET /users (Dev Only)
Lấy danh sách tất cả người dùng trong hệ thống (phục vụ giả lập login).
- **Response (200)**: `Array<User>`

---

### 1. Workspaces

Quản lý không gian làm việc và cấu hình hệ thống.

#### Workspace Model
```json
{
  "id": "uuid",
  "name": "string",
  "work_unit_minutes": 30, // 1, 15, 30, 60
  "rounding_policy": "floor", // none, floor, ceil, round
  "date_format": "DD/MM/YYYY", // DD/MM/YYYY, MM/DD/YYYY, YYYY-MM-DD
  "time_format": "24h", // 24h, 12h
  "timezone": "Asia/Ho_Chi_Minh", // IANA Timezone ID
  "created_at": "datetime"
}
```

#### GET /workspaces
Lấy danh sách các workspace mà người dùng là thành viên.
- **Response (200)**: `Array<Workspace>`

#### GET /workspaces/:ws_id
Lấy thông tin chi tiết của một workspace cụ thể.
- **Response (200)**: `Workspace`

#### POST /workspaces
Tạo mới một workspace (Người tạo sẽ trở thành `owner`).
- **Request**: `{ "name": "..." }`
- **Response (201)**: `Workspace`

#### PATCH /workspaces/:ws_id
Cập nhật thông tin hoặc cấu hình hệ thống của workspace.
- **Request**: 
  ```json
  { 
    "name": "Tên mới", 
    "work_unit_minutes": 30, 
    "rounding_policy": "floor",
    "date_format": "MM/DD/YYYY",
    "time_format": "12h",
    "timezone": "Asia/Ho_Chi_Minh"
  }
  ```
- **Response (200)**: `Workspace` (updated)

---

### 2. Workspace Members

Hệ thống cho phép gán thêm các Role phụ có thời hạn cho thành viên.

#### GET /workspaces/:id/members/:user_id/roles
Lấy danh sách tất cả các role của một thành viên (bao gồm role chính và các role phụ).
- **Response (200)**: 
  ```json
  {
    "primary": "member",
    "extra": [
      {
        "id": "...",
        "role": "admin",
        "assigned_at": "...",
        "expires_at": "..."
      }
    ]
  }
  ```

#### POST /workspaces/:id/members/:user_id/roles
Gán một role phụ có thời hạn cho thành viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "role": "admin",
    "expires_at": "2026-01-10T00:00:00Z"
  }
  ```
- **Response (201)**: `MemberExtraRole` object

#### DELETE /workspaces/:id/members/:user_id/roles/:role_id
Thu hồi một role phụ trước thời hạn.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Response (204)**: No Content

---

### 4. Device Management (CRUD)

**Yêu cầu headers: `X-User-Id`, `X-Workspace-Id`**.

#### GET /devices
Lấy danh sách tất cả các thiết bị trong workspace hiện tại.
- **Response (200)**: `Array<Device>`

#### POST /devices
Thêm thiết bị mới vào workspace.
- **Request**: `{ "agent_id": "...", "name": "...", "ip": "...", "port": 4370 }`
- **Response (201)**: `{ "id": "uuid-v4" }`

---

### 4. Agent Management

**Yêu cầu headers: `X-User-Id`, `X-Workspace-Id`**.

#### GET /authorized-agents
Lấy danh sách Agent thuộc workspace hiện tại.
- **Query Params**: `?include_inactive=true`
- **Response (200)**: `Array<AuthorizedAgent>`

#### POST /authorized-agents
Khai báo một Agent mới vào workspace.
- **Request**:
  ```json
  {
    "id": "AGENT_01",
    "name": "Chi nhánh Hà Nội",
    "token": "secret_token_123"
  }
  ```
- **Response (201)**: `{ "id": "AGENT_01" }`

#### PATCH /authorized-agents/:id (New)
Cập nhật thông tin Agent (tên, token, trạng thái active/trusted).
- **Response (200 OK)**

#### DELETE /authorized-agents/:id (Soft Delete)
Đánh dấu Agent là không hoạt động (`is_active = false`). Agent sẽ bị ngắt kết nối WebSocket và ẩn khỏi danh sách mặc định.
- **Response (204)**: No Content

#### PUT /authorized-agents/:id/trust
Phê duyệt (Trust) hoặc Thu hồi quyền của một Agent. Khi thu hồi, kết nối WebSocket hiện tại sẽ bị ngắt.
- **Request**:
  ```json
  { "is_trusted": true }
  ```
- **Response (200 OK)**

---

### 5. Employee Management

Quản lý hồ sơ nhân viên trong tổ chức.

#### GET /workspaces/:ws_id/employees
Lấy danh sách tất cả nhân viên.
- **Response (200)**: `Array<Employee>`

#### POST /workspaces/:ws_id/employees
Tạo hồ sơ nhân viên mới.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: 
  ```json
  {
    "user_id": "optional-uuid",
    "first_name": "Nguyen",
    "last_name": "Van A",
    "employee_id_number": "EMP001",
    "hire_date": "2026-01-01"
  }
  ```
- **Response (201)**: `Employee` object

#### PUT /workspaces/:ws_id/employees/:id
Cập nhật thông tin nhân viên (họ tên, trạng thái nghỉ việc). Nếu cập nhật `termination_date`, phiên làm việc hiện tại sẽ được đóng lại.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Response (200 OK)**

#### GET /workspaces/:ws_id/employees/:id/history (New)
Lấy toàn bộ lịch sử các giai đoạn làm việc (Employment Periods) của nhân viên.
- **Response (200)**: `Array<EmploymentPeriod>`
  ```json
  [
    {
      "id": "uuid",
      "employee_id": "uuid",
      "hire_date": "2024-01-01",
      "termination_date": "2025-12-31",
      "is_rehire": false,
      "created_at": "datetime",
      "updated_at": "datetime"
    }
  ]
  ```

#### POST /workspaces/:ws_id/employees/:id/rehire (New)
Thực hiện tái tuyển dụng (re-hire) một nhân viên đã nghỉ việc. Sẽ tạo một `EmploymentPeriod` mới với cờ `is_rehire: true`.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "hire_date": "2026-01-14"
  }
  ```
- **Response (200)**: `Employee` object (updated)

---

### 6. Time Tracking

Theo dõi dữ liệu chấm công thực tế.

#### POST /workspaces/:ws_id/time/punch-in
Ghi nhận giờ vào làm hoặc bắt đầu nghỉ giải lao.
- **Request**: 
  ```json
  { 
    "employee_id": "uuid", 
    "entry_type": "work|break", 
    "is_break_overtime": false, 
    "note": "Ghi chú tùy chọn" 
  }
  ```
- **Response (201)**: `TimeEntry` object

#### POST /workspaces/:ws_id/time/punch-out/:employee_id
Ghi nhận giờ ra về hoặc kết thúc nghỉ giải lao cho lần punch-in gần nhất chưa đóng.
- **Request**: `{ "note": "..." }`
- **Response (200 OK)**

#### GET /workspaces/:ws_id/time/history/:employee_id
Lấy lịch sử chấm công trong khoảng thời gian.
- **Query Params**: `?start=ISO-8601&end=ISO-8601`
- **Response (200)**: `Array<TimeEntry>`

---

### 7. Leave Management

Quản lý đơn xin nghỉ phép.

#### POST /workspaces/:ws_id/leave/submit
Gửi đơn xin nghỉ phép.
- **Request**:
  ```json
  {
    "employee_id": "...",
    "leave_type": "vacation|sick|personal",
    "start_date": "2026-02-01",
    "end_date": "2026-02-03",
    "reason": "..."
  }
  ```
- **Response (201)**: `LeaveRequest` object

#### GET /workspaces/:ws_id/leave/requests
Lấy danh sách đơn nghỉ phép trong workspace.
- **Query Params**: `?status=pending|approved|rejected`
- **Response (200)**: `Array<LeaveRequest>`

#### POST /workspaces/:ws_id/leave/requests/:id/approve
Phê duyệt hoặc từ chối đơn nghỉ phép.
- **Request**: `{ "status": "approved|rejected" }`
- **Response (200 OK)**

#### GET /workspaces/:ws_id/employees/:id/leave/balances
Lấy thông tin hạn mức và số dư phép năm của nhân viên.
- **Response (200)**: 
  ```json
  [
    {
      "id": "uuid",
      "leave_type": "annual",
      "year": 2026,
      "entitlement": 12,
      "taken": 2.5,
      "remaining": 9.5,
      "carried_over": 0
    }
  ]
  ```

---

### 8. Dynamic Leave Policies (New)

Quản lý cấu hình cộng phép động (Dynamic Accrual) và phân bổ chính sách theo thời gian.

#### POST /workspaces/:ws_id/leave/policies
Tạo một định nghĩa chính sách mới (Rule Definition).
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "name": "Standard Monthly",
    "description": "Cộng 1 ngày/tháng",
    "rule_config": {
      "frequency": "monthly", // monthly, quarterly, yearly
      "accrual_amount": 1.0,
      "accrual_timing": "start_of_period", // start_of_period, end_of_period
      "seniority_bonus": [
        { "min_years": 5, "extra_days": 1.0 }
      ]
    }
  }
  ```
- **Response (201)**: `LeavePolicyDefinition` object

#### GET /workspaces/:ws_id/leave/policies
Lấy danh sách các định nghĩa chính sách đã tạo.
- **Response (200)**: `Array<LeavePolicyDefinition>`

#### POST /workspaces/:ws_id/leave/policies/assign
Gán chính sách cho đối tượng (Global, Dept, Employee) trong khoảng thời gian cụ thể (Temporal Assignment).
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "assignee_type": "employee", // workspace, department, employee
    "assignee_id": "uuid-or-null",
    "policy_id": "uuid",
    "effective_from": "2026-01-01",
    "effective_to": "2026-12-31" // Optional, null = forever
  }
  ```
- **Response (201)**: `LeavePolicyAssignment` object

#### POST /workspaces/:ws_id/leave/accrual/run (System/Cron)
Kích hoạt thủ công quá trình tính toán cộng phép cho một ngày cụ thể (thường dùng cho Cron Job hoặc Debug).
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "date": "2026-02-01" }`
- **Response (200)**: Danh sách các bản ghi `LeaveAccrualLog` vừa được tạo.

---

### 9. Attendance Adjustments & Bulk Import

Quy trình xử lý các trường hợp quên chấm công hoặc cần điều chỉnh dữ liệu chấm công quá khứ, hỗ trợ nhập liệu hàng loạt từ file.

#### POST /workspaces/:ws_id/attendance/adjustments
Gửi yêu cầu xác nhận/điều chỉnh công mới.
- **Request**:
  ```json
  {
    "employee_id": "...",
    "time_entry_id": "optional-uuid-if-editing",
    "requested_punch_in_time": "2026-01-08T08:00:00Z",
    "requested_punch_out_time": "2026-01-08T17:00:00Z",
    "is_break_overtime": true,
    "reason": "Làm thông trưa"
  }
  ```
- **Response (201)**: `AttendanceAdjustmentRequest` object

#### GET /workspaces/:ws_id/attendance/adjustments
Lấy danh sách các yêu cầu điều chỉnh trong workspace.
- **Query Params**: `?status=pending|approved|rejected`
- **Response (200)**: `Array<AttendanceAdjustmentRequest>`

#### POST /workspaces/:ws_id/attendance/upload
Upload file Excel (.xlsx, .xls) hoặc CSV. Trả về Job ID để theo dõi tiến độ bất đồng bộ.
- **Request**: `multipart/form-data` (file)
- **Response (200)**: `{ "job_id": "uuid" }`

#### GET /workspaces/:ws_id/attendance/import-jobs/latest
Lấy thông tin Job import gần nhất của workspace.
- **Response (200)**: `ImportJob` object

#### GET /workspaces/:ws_id/attendance/import-jobs/:id
Lấy trạng thái chi tiết của một Job cụ thể.
- **Response (200)**: `ImportJob` object

---

### 10. Attendance Logs (New)

Xem nhật ký quẹt thẻ thô từ các thiết bị ngoại vi (Edge Agents).

#### GET /workspaces/:ws_id/attendance/logs
Lấy danh sách các phiên làm việc (sessions) đã được ghép cặp.
- **Query Params**:
    - `employee_id`: uuid (lọc theo nhân viên)
    - `from`: ISO-8601 (từ ngày)
    - `to`: ISO-8601 (đến ngày)
    - `limit`: number (mặc định 20)
    - `page`: number (mặc định 1)
- **Response (200)**: `Array<TimeEntry>`

#### GET /workspaces/:ws_id/attendance/raw-logs (New)
Lấy danh sách nhật ký quẹt thẻ thô (nguyên bản) từ thiết bị hoặc file import.
- **Query Params**: Giống `/attendance/logs`.
- **Response (200)**: `Array<RawAttendanceLog>`
  ```json
  [
    {
      "id": "uuid",
      "employee_id": "uuid",
      "punch_time": "2026-01-14T08:00:00Z",
      "source": "device|import|manual",
      "device_id": "string-or-null",
      "created_at": "datetime"
    }
  ]
  ```

---

### 11. Payroll & Shifts

Quản lý lương, ca làm việc và báo cáo công.

#### GET /workspaces/:ws_id/shifts
Lấy danh sách các ca làm việc đã định nghĩa (bao gồm cả các phiên bản cũ).
- **Response (200)**: `Array<Shift>`

#### POST /workspaces/:ws_id/shifts
Tạo mới hoặc cập nhật một loại ca làm việc. Nếu cập nhật, hệ thống sẽ tạo một version mới (`version_id`) để bảo toàn dữ liệu chấm công quá khứ.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.

**Body:**
```json
{
  "name": "Ca hành chính",
  "start_time": "08:00:00",
  "end_time": "17:00:00",
  "break_duration_minutes": 60,
  "break_start": "12:00:00",
  "break_end": "13:00:00",
  "is_break_auto_deduct": true,
  "early_punch_in_grace_period_minutes": 15
}
```

#### GET /workspaces/:ws_id/reports/attendance
Xuất báo cáo chấm công của workspace.
- **Query Params**: 
    - `start_date`: YYYY-MM-DD
    - `end_date`: YYYY-MM-DD
    - `type`: `summary` (mặc định) hoặc `daily`
- **Response (type=summary)**: Báo cáo tổng hợp số giờ, số ngày công theo từng nhân viên.
- **Response (type=daily)**: Báo cáo chi tiết từng lượt quẹt thẻ (Work/Break) theo ngày của từng nhân viên.
    - **Fields**: `employee_id`, `employee_name`, `date`, `scheduled_start`, `scheduled_end`, `break_duration_minutes`, `break_start`, `break_end`, `is_break_auto_deduct`, `actual_punch_in`, `actual_punch_out`, `status`.

---

### 12. Departments & Rostering (New)

Quản lý cơ cấu tổ chức và lịch trình làm việc linh hoạt.

#### GET /workspaces/:ws_id/departments
Lấy danh sách các phòng ban.
- **Response (200)**: `Array<Department>`

#### POST /workspaces/:ws_id/departments
Tạo phòng ban mới.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "name": "IT", "description": "...", "manager_id": "uuid" }`
- **Response (201)**: `Department` object

#### POST /workspaces/:ws_id/employees/:id/departments
Điều chuyển phòng ban cho nhân viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "department_id": "...", "effective_date": "...", "notes": "..." }`
- **Response (201)**: `EmployeeDepartmentHistory` object

#### GET /workspaces/:ws_id/employees/:id/departments
Xem lịch sử điều chuyển phòng ban của một nhân viên.
- **Response (200)**: `Array<EmployeeDepartmentHistory>`

#### GET /workspaces/:ws_id/employees/:id/rosters
Lấy lịch biểu (ca làm việc thực tế) của nhân viên trong một khoảng thời gian.
- **Query Params**: `?start_date=2026-01-01&end_date=2026-01-07`
- **Response (200)**: `Array<ShiftRoster>`

#### GET /api/workspaces/:id/schedules
Tính toán lịch biểu hiệu lực (Effective Schedule) cho tất cả nhân viên trong workspace dựa trên thứ tự ưu tiên (Lịch tuần > Lịch cố định > Lịch phòng ban).
- **Query Params**: `?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD`
- **Response (200)**: 
  ```json
  [
    {
      "employee_id": "uuid",
      "first_name": "...",
      "last_name": "...",
      "schedules": [
        {
          "date": "2026-01-10",
          "shift_id": "uuid-or-null",
          "source": "roster|fixed|department|none",
          "shift_name": "Morning Shift",
          "start_time": "08:00:00",
          "end_time": "17:00:00"
        }
      ]
    }
  ]
  ```

#### GET /api/workspaces/:id/assignment-history
Lấy lịch sử tất cả các thay đổi về gán ca (cả cố định và linh hoạt) của workspace.
- **Response (200)**: 
  ```json
  [
    {
      "id": "uuid",
      "employee_id": "uuid",
      "employee_name": "...",
      "assignment_type": "roster|fixed",
      "shift_name": "Night Shift",
      "date_range": "2026-01-10",
      "created_at": "2026-01-09T10:00:00Z"
    }
  ]
  ```

#### POST /workspaces/:ws_id/employees/:id/rosters
Cập nhật hoặc gán ca làm việc cho một ngày cụ thể (Upsert).
- **Request**:
  ```json
  {
    "employee_id": "...",
    "date": "2026-01-01",
    "shift_id": "shift-uuid-or-null",
    "notes": "..."
  }
  ```
- **Response (200 OK)**

---

---

### 13. Debug & Development (Debug Mode Only)

Các công cụ hỗ trợ phát triển nhanh và dọn dẹp dữ liệu thử nghiệm. Chỉ khả dụng khi server chạy ở chế độ Debug.

#### POST /api/dev/workspaces/:ws_id/clear
Dọn dẹp nhanh dữ liệu theo nhóm trong một workspace.
- **Query Params**: `?group=attendance|schedules|leave|payroll|full_reset`
- **Mục đích**: Reset dữ liệu để thử nghiệm lại các kịch bản import/tính toán mà không làm ảnh hưởng đến cấu hình chung của workspace.
- **Các nhóm dữ liệu**:
    - `attendance`: Xóa logs thô, time entries, yêu cầu điều chỉnh công và lịch sử import.
    - `schedules`: Xóa lịch biểu (rosters) và gán ca linh hoạt.
    - `leave`: Xóa đơn nghỉ phép, lịch sử cộng phép và số dư phép.
    - `payroll`: Xóa phiếu lương (payslips) và lịch sử lương.
    - `full_reset`: Xóa toàn bộ dữ liệu trên cộng với danh sách nhân viên, phòng ban, ca làm việc và chính sách phép. (Giữ lại thông tin Workspace và Members).
- **Response (200 OK)**

#### GET /dev/super-user
Tự động đăng nhập với tư cách Superuser (admin@leeattend.com).
- **Mục đích**: Phục vụ phát triển nhanh (Onboarding) và Testing.
- **Response (200)**: `{ "access_token": "...", "workspace_id": "...", "user_id": "..." }`

---

### WebSocket Giao thức (Agent <-> Server)

Agent gửi dữ liệu chấm công thô và trạng thái sức khỏe lên Server qua một kết nối duy nhất.

#### 1. Handshake (Agent -> Server)
Bản tin đầu tiên sau khi kết nối để xác thực định danh.
```json
{
  "type": "handshake",
  "agent_id": "AGENT_01",
  "token": "secret_token_abc_123",
  "timestamp": "2026-01-07T10:00:00Z"
}
```

#### 2. Heartbeat (Agent -> Server)
Agent gửi định kỳ (mỗi 30s) để duy trì trạng thái "Online" trong Database.
```json
{
  "type": "heartbeat",
  "agent_id": "AGENT_01",
  "timestamp": "ISO-8601"
}
```

#### 3. Status Update (Agent -> Server)
Cập nhật chi tiết trạng thái của các máy chấm công đang được giám sát.
```json
{
  "type": "status_update",
  "agent_id": "AGENT_01",
  "devices": [
    {
      "device_id": "DEV_01",
      "status": "online",
      "last_sync": "...",
      "error": null
    }
  ]
}
```

#### 4. Attendance Log (Agent -> Server)
... (giữ nguyên cấu trúc cũ) ...

---

### WebSocket Error Codes
Khi có lỗi xảy ra trong quá trình Handshake hoặc kết nối, Server sẽ gửi message `type: "error"` và đóng kết nối.

| Code | Message | Mô tả |
|------|---------|-------|
| `UNAUTHORIZED_AGENT` | Agent ID không nằm trong danh sách... | ID này chưa được khai báo trong bảng `authorized_agents`. |
| `INVALID_TOKEN` | Token không chính xác. | ID đúng nhưng chuỗi bí mật `token` không khớp. |
| `AGENT_NOT_TRUSTED` | Agent chưa được phê duyệt... | Agent đã khai báo nhưng cờ `is_trusted` đang là `false`. |
| `DUPLICATE_ID` | Agent ID ... đã có kết nối hoạt động. | ID này hiện đang có một WebSocket session khác đang chạy. |

---



### POST /agent/:agent_id/command
Gửi lệnh điều khiển đến một Agent đang online.

**Request Body:**
```json
{
  "method": "sync_time",
  "device_id": "OFFICE_01",
  "data": {}
}
```

**Supported Methods:**
- `sync_time`: Đồng bộ thời gian cho máy chấm công.
- `get_users`: Lấy danh sách nhân viên từ máy.
- `ping`: Kiểm tra kết nối đến Agent.

**Response (200 OK):**
```text
Command sent
```

**Errors:**
- `404 Not Found`: Agent không online hoặc không tồn tại.

---

## 🤖 Agent Remote Control (WebSocket Tunnels)

Agent không cung cấp HTTP API local. Mọi tương tác với Agent được thực hiện gián tiếp thông qua Central Server WebSocket. 

Admin gửi lệnh tới API Server, và API Server sẽ định tuyến lệnh đó tới Agent tương ứng qua kết nối WebSocket đang hoạt động.

### POST /agent/:agent_id/command (Central Server API)
Gửi lệnh điều khiển đến một Agent đang online.

**Request Body:**
```json
{
  "method": "sync_time",
  "device_id": "OFFICE_01",
  "data": {}
}
```

**Supported Methods:**
- `sync_time`: Đồng bộ thời gian cho máy chấm công cụ thể.
- `get_users`: Lấy danh sách nhân viên từ máy.
- `ping`: Kiểm tra kết nối và độ trễ tới Agent.

**Response (200 OK):**
```json
{
  "success": true,
  "data": { ... result of method ... },
  "error": null
}
```

---

## 🌐 Agent Outbound Giao thức (WebSocket)

Agent chủ động kết nối tới `CENTRAL_SERVER_URL` để thiết lập kênh giao tiếp 2 chiều.

### 1. Handshake (Agent -> Server)
Bản tin đầu tiên để xác thực.
```json
{
  "type": "handshake",
  "agent_id": "AGENT_01",
  "token": "secret_token_abc",
  "timestamp": "ISO-8601"
}
```

### 2. Status Update (Agent -> Server)
Agent định kỳ (mỗi 30s) gửi trạng thái chi tiết của các máy chấm công local.
```json
{
  "type": "status_update",
  "agent_id": "AGENT_01",
  "devices": [
    {
      "device_id": "DEV_01",
      "status": "online",
      "last_sync": "2026-01-07T18:00:00",
      "error": null
    }
  ]
}
```

### 3. Attendance Batch (Agent -> Server)
Gửi dữ liệu chấm công theo lô (batch) để tối ưu hiệu năng.
```json
{
  "type": "attendance",
  "msg_id": "uuid-v4",
  "agent_id": "AGENT_01",
  "device_id": "BATCH",
  "data": [
    { "user_id": "101", "timestamp": "2026-01-07 18:00:00", "status": 1, "punch": 0 },
    { "user_id": "102", "timestamp": "2026-01-07 18:01:00", "status": 1, "punch": 0 }
  ]
}
```

---

## 🛠 Định dạng Log (Agent Console Output)

Dù không có HTTP server, Agent vẫn in thông tin hoạt động ra Console:
```text
[18:00:00] [SYSTEM] ℹ️ INFO : Agent đã khởi động và đang giám sát thiết bị local.
[18:05:00] [OFFICE_01] ⚠️ WARN : Lỗi kết nối máy chấm công (ZKError). Thử lại sau 15.0s.
[18:10:00] [OFFICE_01] ℹ️ INFO : Kết nối máy chấm công đã được phục hồi.
```