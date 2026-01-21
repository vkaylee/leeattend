# API Reference

## 🚀 API Server (Central - Rust)
Base URL: `http://localhost:3000`

### 0. Authentication & Context (Updated)

Hệ thống sử dụng JWT (JSON Web Token) để xác thực người dùng.

- **Access Token**: Gửi trong header `Authorization: Bearer <token>`. Hết hạn sau 1 giờ.
- **Refresh Token**: Dùng để lấy Access Token mới. Hết hạn sau 7 ngày.
- **X-Workspace-Id**: Header dùng để xác định tổ chức (Workspace). 
    - **Lưu ý quan trọng**: Đối với hầu hết các yêu cầu có chứa `:ws_id` hoặc `:id` trong đường dẫn (Path), backend sẽ ưu tiên trích xuất ID từ URI. Trong trường hợp này, header `X-Workspace-Id` là không bắt buộc (redundant).
    - Header này vẫn cần thiết cho các endpoint "vùng đệm" (ví dụ: ESS portal) nơi ID không xuất hiện trong đường dẫn.

#### 🌐 API URL Resolution Strategy (Client-side)
Đối với các ứng dụng Web (đặc biệt trong môi trường Development với Vite), client sẽ ưu tiên gọi API thông qua đường dẫn tương đối `/api`. Vite sẽ tự động proxy các request này đến backend server đang chạy trên một port ngẫu nhiên.
- `VITE_API_URL` trong `.env` chỉ được sử dụng cho các môi trường yêu cầu URL tuyệt đối (ví dụ: Tauri, hoặc Production khi API đặt ở một domain khác và không có proxy). Trong môi trường phát triển web tiêu chuẩn, việc set `VITE_API_URL` sẽ khiến client bỏ qua proxy và gọi thẳng vào địa chỉ hardcode, gây ra lỗi `ERR_CONNECTION_REFUSED`.
- **Resilient URL Construction**: Tiện ích `apiFetch` được thiết kế để tự động xử lý các định dạng path khác nhau, ngăn chặn lỗi "double-prefixing" khi truyền vào các path đã có sẵn tiền tố hoặc URL tuyệt đối.

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
- **Response (200)**: `User` object (`id`, `email`, `full_name`, `avatar_url`, `bio`, `preferred_language`, `created_at`)

#### PATCH /me
Cập nhật thông tin hồ sơ.
- **Headers**: `X-User-Id`
- **Request**: `{ "full_name": "...", "avatar_url": "...", "bio": "...", "preferred_language": "vi|en|zh" }`
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
  "default_language": "vi", // vi, en, zh
  "theme_primary_color": "#7c3aed", // HEX color for UI branding
  "pairing_algorithm": "anchor_first", // anchor_first (Primary), fsm (Finite State Machine)
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
    "timezone": "Asia/Ho_Chi_Minh",
    "default_language": "vi|en|zh",
    "theme_primary_color": "#059669",
    "pairing_algorithm": "anchor_first|fsm"
  }
  ```
- **Response (200)**: `Workspace` (updated)

---

### 2. Workspace Members

Hệ thống quản lý thành viên và phân quyền trong tổ chức. Một thành viên có một **Role chính** (Primary Role) và có thể được gán thêm các **Role phụ** (Extra Roles) có thời hạn.

#### GET /workspaces/:ws_id/members
Lấy danh sách tất cả thành viên trong workspace.
- **Response (200)**: `Array<WorkspaceMember>`
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > People`.

#### POST /workspaces/:ws_id/members
Mời hoặc thêm thành viên mới vào workspace.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "user_id": "...", "role": "admin|member" }`
- **Response (201)**: `WorkspaceMember` object

#### PATCH /workspaces/:ws_id/members/:user_id
Cập nhật **Role chính** của thành viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "role": "admin|member" }`
- **Response (200 OK)**

#### DELETE /workspaces/:ws_id/members/:user_id
Xóa thành viên khỏi workspace.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`. Không cho phép tự xóa chính mình hoặc xóa `owner`.
- **Response (204)**: No Content

#### GET /workspaces/:ws_id/members/:user_id/roles
Lấy danh sách tất cả các role hiện tại của một thành viên (bao gồm role chính và các role phụ đang hiệu lực).
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

#### POST /workspaces/:ws_id/members/:user_id/roles
Gán một **Role phụ** có thời hạn cho thành viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "role": "admin",
    "expires_at": "2026-01-10T00:00:00Z"
  }
  ```
- **Response (201)**: `MemberExtraRole` object

#### DELETE /workspaces/:ws_id/members/:user_id/roles/:role_id
Thu hồi một role phụ trước thời hạn.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Response (204)**: No Content

---

### 4. Device Management (CRUD)

Quản lý thiết bị phần cứng.
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > Connectivity`.

#### GET /devices
Lấy danh sách tất cả các thiết bị trong workspace hiện tại (Xác định qua header `X-Workspace-Id`).
- **Response (200)**: `Array<Device>`

#### POST /devices
Thêm thiết bị mới vào workspace.
- **Request**: `{ "agent_id": "...", "name": "...", "ip": "...", "port": 4370 }`
- **Response (201)**: `{ "id": "uuid-v4" }`

---

### 4. Agent Management

Quản lý các Edge Agents.
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > Connectivity`.

#### GET /authorized-agents
Lấy danh sách Agent thuộc workspace hiện tại (Xác định qua header `X-Workspace-Id`).
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
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > People`.

#### GET /workspaces/:ws_id/employees
Lấy danh sách tất cả nhân viên.
- **Query Params**:
    - `q`: string (Tìm kiếm theo tên, họ, mã nhân viên hoặc tên đầy đủ. Case-insensitive).
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

#### GET /workspaces/:ws_id/leave/balances (New - Admin)
Lấy bảng tổng hợp số dư phép của tất cả nhân viên trong workspace.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Response (200)**: 
  ```json
  [
    {
      "employee_id": "uuid",
      "employee_name": "Nguyen Van A",
      "leave_type": "vacation",
      "entitlement": 12,
      "accrued_amount": 3.0,
      "carried_over": 0,
      "used": 2,
      "remaining": 13.0
    }
  ]
  ```

#### GET /workspaces/:ws_id/employees/:id/leave/balances
Lấy thông tin hạn mức và số dư phép của một nhân viên cụ thể.
- **Response (200)**: 
  ```json
  [
    {
      "leave_type": "vacation",
      "entitlement": 12,
      "accrued_amount": 3.0,
      "carried_over": 0,
      "used": 2,
      "remaining": 13.0
    }
  ]
  ```

#### GET /workspaces/:ws_id/employees/:id/leave/history (New)
Lấy lịch sử chi tiết các biến động số dư (cộng phép định kỳ, sử dụng, điều chỉnh thủ công).
- **Response (200)**: `Array<LeaveHistoryItem>`
  ```json
  [
    {
      "id": "uuid",
      "date": "2026-01-01",
      "transaction_type": "accrual|usage|adjustment|carry_over",
      "amount": 1.0,
      "reason": "...",
      "reference_id": "optional-uuid",
      "created_at": "datetime"
    }
  ]
  ```

#### POST /workspaces/:ws_id/employees/:id/leave/adjust (New)
Thực hiện điều chỉnh số dư phép thủ công cho nhân viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "leave_type": "vacation",
    "amount": 1.5, // Dương để cộng, âm để trừ
    "reason": "Thưởng thêm ngày phép dự án",
    "transaction_date": "2026-01-15"
  }
  ```
- **Response (200 OK)**

---

### 8. Dynamic Leave Policies (New)

Quản lý cấu hình cộng phép động (Dynamic Accrual) và phân bổ chính sách theo thời gian.
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > Time`.

#### POST /workspaces/:ws_id/leave/policies
Tạo một định nghĩa chính sách mới (Rule Definition).
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "name": "Standard Monthly",
    "description": "Cộng 1 ngày/tháng",
    "leave_type": "vacation", // vacation, sick, personal, etc.
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
- **Transactional Auto-Close**: Hệ thống sẽ tự động tìm các bản gán (assignment) hiện tại của cùng đối tượng cho cùng loại phép (`leave_type`) và cập nhật ngày kết thúc (`effective_to`) của chúng thành `effective_from - 1 day` để đảm bảo tính liên tục của dữ liệu.
- **Request**:
  ```json
  {
    "assignee_type": "employee", // workspace, department, employee
    "assignee_id": "uuid-or-null", // NULL if assignee_type is 'workspace'
    "policy_id": "uuid",
    "effective_from": "2026-01-01"
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
    "requested_punch_in_time": "2026-01-08T08:00:00Z", // UTC ISO string
    "requested_punch_out_time": "2026-01-08T17:00:00Z", // UTC ISO string
    "is_break_overtime": true,
    "reason": "Làm thông trưa"
  }
  ```
  *Lưu ý: Nếu chuỗi thời gian không có offset (Naive), Backend sẽ tự động parse theo múi giờ đã cài đặt của Workspace.*
- **Response (201)**: `AttendanceAdjustmentRequest` object

#### GET /workspaces/:ws_id/attendance/adjustments
Lấy danh sách các yêu cầu điều chỉnh trong workspace.
- **Query Params**: `?status=pending|approved|rejected`
- **Response (200)**: `Array<AttendanceAdjustmentRequest>`

#### POST /workspaces/:ws_id/employees/upload (Updated)
Upload file Excel (.xlsx, .xls) hoặc CSV. Trả về Job ID để theo dõi tiến độ bất đồng bộ.
- **Silent Skip**: Hệ thống sẽ tự động bỏ qua các bản ghi có mã nhân viên (Employee ID) đã tồn tại mà không làm gián đoạn quá trình nhập liệu.
- **Job Status**: Trường `processed_rows` trong Job Status sẽ chỉ đếm số lượng bản ghi *thực sự được thêm mới*. Log lỗi sẽ ghi nhận số lượng bản ghi bị bỏ qua.
- **Request**: `multipart/form-data` (file)
- **Response (200)**: `{ "job_id": "uuid" }`

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
  ```json
  [
    {
      "id": "uuid",
      "employee_id": "uuid",
      "punch_in": "2026-01-14T08:00:00Z",
      "punch_out": "2026-01-14T17:00:00Z",
      "work_units": 8.0,
      "generation_metadata": {
        "algorithm": "anchor_first",
        "source_logs": ["log-uuid-1", "log-uuid-2"],
        "strategy_params": { "gap_limit_minutes": 720 }
      },
      "created_at": "datetime"
    }
  ]
  ```

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

#### POST /workspaces/:ws_id/attendance/reprocess/:employee_id (New)
Kích hoạt tính toán lại (reprocess) dữ liệu chấm công cho một nhân viên trong khoảng thời gian cụ thể.
- **Query Params**:
    - `start_date`: YYYY-MM-DD
    - `end_date`: YYYY-MM-DD
- **Mục đích**: Chạy lại thuật toán ghép cặp (pairing algorithm) dựa trên logs thô hiện có. Thường dùng khi thay đổi cấu hình thuật toán của Workspace hoặc sửa lỗi ca làm việc.
- **Response (200 OK)**

---

### 11. Payroll & Shifts

Quản lý lương, ca làm việc và báo cáo công. Hệ thống hỗ trợ cấu hình lương động dựa trên các thành phần thu nhập (Components) và chính sách công chuẩn của từng tổ chức.
- **UI Note**: Trong WebDashboard, cài đặt lương đã được hợp nhất vào `Settings > Payroll`.

#### Payroll Settings Model
Lưu cấu hình tính toán lương cấp Workspace.
```json
{
  "workspace_id": "uuid",
  "std_working_day_mode": "fixed|actual_month",
  "fixed_std_days": 26.0,
  "working_days_per_week": 6, // 5, 6, 7
  "hours_per_day": 8.0,
  "updated_at": "datetime"
}
```

#### Payslip Model (New)
Lưu thông tin chi tiết phiếu lương hàng tháng.
```json
{
  "id": "uuid",
  "employee_id": "uuid",
  "period_start": "YYYY-MM-DD",
  "period_end": "YYYY-MM-DD",
  "base_pay_amount": "decimal_string",
  "overtime_pay_amount": "decimal_string",
  "deductions_amount": "decimal_string",
  "net_pay_amount": "decimal_string",
  "standard_units": 160, // Số Work Unit chuẩn (Số nguyên)
  "overtime_units": 10,  // Số Work Unit tăng ca (Số nguyên)
  "segments": [
    {
      "period_start": "YYYY-MM-DD",
      "period_end": "YYYY-MM-DD",
      "standard_days": 26.0,
      "hours_per_day": 8.0,
      "base_salary": "decimal_string"
    }
  ],
  "status": "draft|published|paid",
  "created_at": "datetime",
  "updated_at": "datetime"
}
```
- **Note**: `segments` chứa thông tin chi tiết về việc chia nhỏ kỳ lương khi có thay đổi chính sách (Proration).

#### GET /workspaces/:ws_id/payroll-settings
Lấy cấu hình lương hiện tại của workspace.
- **Response (200)**: `PayrollSettings`

#### GET /workspaces/:ws_id/payroll-settings/history (New)
Lấy toàn bộ lịch sử thay đổi cấu hình lương của workspace.
- **Response (200)**: `Array<PayrollSettings>`

#### PATCH /workspaces/:ws_id/payroll-settings
Cập nhật hoặc lên lịch thay đổi cấu hình tính công chuẩn.
- **Request**:
  ```json
  {
    "std_working_day_mode": "actual_month",
    "fixed_std_days": 26.0,
    "working_days_per_week": 5,
    "hours_per_day": 8.0,
    "effective_date": "2026-02-01" // Optional point-in-time scheduled change
  }
  ```
- **Response (200 OK)**

#### GET /workspaces/:ws_id/departments/:id/payroll-config (New)
Lấy lịch sử cấu hình lương đặc thù của phòng ban.
- **Response (200)**: `Array<DepartmentPayrollConfig>`

#### POST /workspaces/:ws_id/departments/:id/payroll-config (New)
Thiết lập hoặc cập nhật đặc cách ngày công/giờ công cho phòng ban.
- **Request**:
  ```json
  {
    "std_working_days": 22.0,
    "hours_per_day": 8.0,
    "effective_date": "2026-02-01"
  }
  ```
- **Response (201)**: `DepartmentPayrollConfig` object

#### GET /workspaces/:ws_id/employees/:id/payslips
Lấy danh sách các phiếu lương của một nhân viên cụ thể.
- **Response (200)**: `Array<PayslipResponse>`

#### POST /workspaces/:ws_id/employees/:id/calculate-payroll (New)
Tính toán phiếu lương cho một nhân viên cụ thể.
- **Query Params**: `?year=2026&month=1`
- **Response (200)**: `PayslipResponse`

#### GET /workspaces/:ws_id/payroll/payslips (New)
Lấy danh sách toàn bộ phiếu lương của workspace trong một kỳ cụ thể (Tháng/Năm).
- **Query Params**:
    - `year`: number (ví dụ: 2026)
    - `month`: number (1-12)
- **Response (200)**: `Array<Payslip>`

#### POST /workspaces/:ws_id/payroll/calculate-batch (New)
Kích hoạt tính toán lương hàng loạt cho nhiều nhân viên cùng lúc. Xử lý song song để tối ưu hiệu năng.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "employee_ids": ["uuid-1", "uuid-2", "..."],
    "year": 2026,
    "month": 1
  }
  ```
- **Response (200)**: 
  ```json
  {
    "results": [
      {
        "employee_id": "uuid-1",
        "status": "success",
        "payslip": { ... },
        "error_message": null
      },
      {
        "employee_id": "uuid-2",
        "status": "error",
        "payslip": null,
        "error_message": "HTTP Error: 404 Not Found"
      }
    ]
  }
  ```

#### GET /workspaces/:ws_id/employees/:id/payroll-config (New)
Lấy lịch sử đặc cách lương cá nhân của nhân viên.
- **Response (200)**: `Array<EmployeePayrollConfig>`

#### POST /workspaces/:ws_id/employees/:id/payroll-config (New)
Thiết lập đặc cách ngày công/giờ công cho riêng một nhân viên.
- **Request**:
  ```json
  {
    "std_working_days": 20.0,
    "hours_per_day": 4.0, // Chế độ part-time
    "effective_date": "2026-02-01"
  }
  ```
- **Response (201)**: `EmployeePayrollConfig` object

#### Salary Component Definition
Định nghĩa các loại khoản thu nhập (Lương cơ bản, Phụ cấp, Thưởng...).
```json
{
  "id": "uuid",
  "workspace_id": "uuid",
  "name": "Phụ cấp ăn trưa",
  "code": "MEAL",
  "is_base_for_ot": false, // Nếu TRUE, khoản này sẽ được cộng vào gốc tính lương tăng ca
  "is_taxable": true
}
```

#### GET /workspaces/:ws_id/salary-components
Lấy danh mục các loại thành phần lương của workspace.
- **Response (200)**: `Array<SalaryComponentDefinition>`

#### POST /workspaces/:ws_id/salary-components
Tạo loại thành phần lương mới.
- **Request**: `{ "name": "...", "code": "...", "is_base_for_ot": true, "is_taxable": true }`

#### GET /workspaces/:ws_id/employees/:id/salary-components
Lấy các khoản lương đang được áp dụng cho một nhân viên cụ thể (Lấy bản ghi mới nhất tính đến hiện tại).
- **Response (200)**: 
  ```json
  [
    {
      "id": "uuid",
      "name": "Lương cơ bản",
      "code": "BASE",
      "amount": 10000000,
      "is_base_for_ot": true,
      "effective_date": "2026-01-01"
    }
  ]
  ```

#### POST /workspaces/:ws_id/employees/:id/salary-components
Gán một khoản lương/phụ cấp mới cho nhân viên.
- **Request**: 
  ```json
  {
    "component_definition_id": "uuid",
    "amount": 500000,
    "effective_date": "2026-01-15"
  }
  ```

#### GET /workspaces/:ws_id/shifts
Lấy danh sách các ca làm việc đã định nghĩa (bao gồm cả các phiên bản cũ).
- **Response (200)**: `Array<Shift>`

#### POST /workspaces/:ws_id/shifts
Tạo mới hoặc cập nhật một loại ca làm việc. Nếu cập nhật, hệ thống sẽ tạo một version mới (`version_id`) để bảo toàn dữ liệu chấm công quá khứ trừ khi cờ `is_correction` được bật.
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
  "early_punch_in_grace_period_minutes": 15,
  "allow_early_ot": false,
  "is_correction": false,
  "effective_date": "2026-01-16"
}
```
- `allow_early_ot`: Nếu `true`, thời gian vào ca sớm hơn `start_time` (sau khi trừ grace period) sẽ được tính là Overtime.
- `is_correction`: (Chỉ dành cho update) Nếu `true`, ghi đè trực tiếp lên bản ghi hiện tại thay vì tạo version mới.
- `effective_date`: (Chỉ dành cho update) Ngày bắt đầu hiệu lực của version mới. Mặc định là hôm nay.

#### GET /workspaces/:ws_id/reports/attendance
Xuất báo cáo chấm công của workspace.
- **Query Params**: 
    - `start_date`: YYYY-MM-DD
    - `end_date`: YYYY-MM-DD
    - `type`: `summary` (mặc định) hoặc `daily`
- **Response (type=summary)**: Báo cáo tổng hợp số Work Unit, số ngày công theo từng nhân viên.
    - **Fields**: `employee_id`, `employee_name`, `total_work_units` (Số lượng đơn vị công nguyên), `total_leave_days`.
- **Response (type=daily)**: Báo cáo chi tiết từng lượt quẹt thẻ (Work/Break) theo ngày của từng nhân viên.
    - **Fields**: `employee_id`, `employee_name`, `date`, `scheduled_start`, `scheduled_end`, `break_duration_minutes`, `break_start`, `break_end`, `is_break_auto_deduct`, `actual_punch_in`, `actual_punch_out`, `work_units` (Số lượng đơn vị công nguyên sau khi làm tròn), `status`.

---

### 12. Departments & Rostering (New)

Quản lý cơ cấu tổ chức và lịch trình làm việc linh hoạt.
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > People`.

#### GET /workspaces/:ws_id/departments
Lấy danh sách các phòng ban.
- **Response (200)**: `Array<Department>`

#### POST /workspaces/:ws_id/departments
Tạo phòng ban mới.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "name": "IT", "description": "...", "manager_id": "uuid" }`
- **Response (201)**: `Department` object

#### PATCH /workspaces/:ws_id/departments/:id
Cập nhật thông tin phòng ban. Hỗ trợ cập nhật từng phần (Partial Update).
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: 
  ```json
  { 
    "name": "...", 
    "description": "...", 
    "manager_id": "uuid",
    "default_working_pattern_id": "uuid-or-null" 
  }
  ```
- **Ghi chú**: Nếu `default_working_pattern_id` được truyền là `null`, mẫu lịch trình mặc định của phòng ban sẽ bị xóa. Nếu không truyền trường này, giá trị cũ sẽ được giữ nguyên.
- **Response (200)**: `Department` object (updated)

#### DELETE /workspaces/:ws_id/departments/:id
Xóa phòng ban.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Response (204)**: No Content

#### POST /workspaces/:ws_id/employees/:id/departments
Điều chuyển phòng ban cho nhân viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "department_id": "...", "effective_date": "...", "notes": "..." }`
- **Response (200)**: `EmployeeDepartmentHistory` object

#### GET /workspaces/:ws_id/employees/:id/departments
Xem lịch sử điều chuyển phòng ban của một nhân viên.
- **Response (200)**: `Array<EmployeeDepartmentHistory>`

#### GET /workspaces/:ws_id/employees/:id/rosters
Lấy lịch biểu (ca làm việc thực tế) của nhân viên trong một khoảng thời gian.
- **Query Params**: `?start_date=2026-01-01&end_date=2026-01-07`
- **Response (200)**: `Array<ShiftRoster>`

#### GET /api/workspaces/:id/schedules
Tính toán lịch biểu hiệu lực (Effective Schedule) cho tất cả nhân viên trong workspace dựa trên thứ tự ưu tiên (Lịch tuần > Lịch mẫu cá nhân > Lịch mẫu phòng ban > Lịch cố định).
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
          "source": "roster|pattern|department_pattern|fixed|department_fixed|none",
          "traceability": {
            "rule_id": "uuid-or-null",
            "evaluated_at": "datetime",
            "decision_path": "manual_override -> pattern -> default"
          },
          "shift_name": "Morning Shift",
          "start_time": "08:00:00",
          "end_time": "17:00:00",
          "allow_early_ot": false
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

### 13. Working Patterns (New)

Quản lý các mẫu lịch làm việc định kỳ (Templates) để gán cho hàng loạt nhân viên.
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > Time`.

#### Working Pattern Model
```json
{
  "id": "uuid",
  "workspace_id": "uuid",
  "name": "Văn phòng chuẩn",
  "description": "Thứ 2 - Thứ 6: 8:00 - 17:00",
  "is_active": true,
  "created_at": "datetime"
}
```

#### GET /workspaces/:ws_id/working-patterns
Lấy danh sách các mẫu lịch làm việc của workspace.
- **Response (200)**: `Array<WorkingPattern>`

#### POST /workspaces/:ws_id/working-patterns
Tạo mẫu lịch làm việc mới kèm chi tiết các ngày trong tuần.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "name": "Ca hành chính",
    "description": "Làm việc từ T2 đến T6",
    "details": [
      { "day_of_week": 1, "shift_id": "uuid-ca-sang" },
      { "day_of_week": 2, "shift_id": "uuid-ca-sang" },
      { "day_of_week": 3, "shift_id": "uuid-ca-sang" },
      { "day_of_week": 4, "shift_id": "uuid-ca-sang" },
      { "day_of_week": 5, "shift_id": "uuid-ca-sang" },
      { "day_of_week": 6, "shift_id": null }, // Nghỉ thứ 7
      { "day_of_week": 0, "shift_id": null }  // Nghỉ Chủ nhật
    ]
  }
  ```
- **Response (201)**: `WorkingPattern` object

#### GET /workspaces/:ws_id/working-patterns/:pattern_id
Lấy chi tiết ca làm việc của từng ngày trong một mẫu pattern.
- **Response (200)**: `Array<WorkingPatternDetail>`

#### POST /workspaces/:ws_id/working-patterns/bulk-assign (New)
Gán mẫu lịch làm việc cho hàng loạt nhân viên cùng lúc.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "employee_ids": ["uuid-1", "uuid-2"],
    "pattern_id": "uuid",
    "start_date": "2026-01-01",
    "end_date": null
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success_count": 2,
    "failed_employees": [] // List of employee_ids that caused conflicts (if any)
  }
  ```

#### POST /workspaces/:ws_id/employees/:id/working-patterns
Gán một mẫu lịch làm việc cho nhân viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "pattern_id": "uuid",
    "start_date": "2026-01-01",
    "end_date": null // NULL nếu gán vĩnh viễn
  }
  ```
- **Response (200 OK)**

#### GET /workspaces/:ws_id/employees/:id/working-patterns
Xem lịch sử gán pattern của một nhân viên.
- **Response (200)**: `Array<EmployeeWorkingPattern>`

---

### 14. Employee Self-Service (ESS) (New)

Các endpoint dành riêng cho nhân viên tự quản lý thông tin cá nhân. **Yêu cầu header: `Authorization: Bearer <token>`**. Hệ thống tự động xác định `employee_id` dựa trên User đang đăng nhập.

#### GET /ess/profile
Lấy toàn bộ thông tin hồ sơ cá nhân, bao gồm cả các giai đoạn làm việc và liên hệ khẩn cấp.
- **Response (200)**: `MyProfileResponse`
  ```json
  {
    "employee": { "id": "...", "first_name": "...", "hire_date": "..." },
    "profile": { "phone": "...", "address": "...", "bank_name": "..." },
    "emergency_contacts": [ ... ]
  }
  ```

#### PATCH /ess/profile
Cập nhật thông tin hồ sơ (số điện thoại, địa chỉ, ngày sinh, thông tin ngân hàng).
- **Request**: `UpdateEmployeeProfileRequest`
- **Response (200)**: `EmployeeProfile` object

#### GET /ess/emergency-contacts
Lấy danh sách người liên hệ khẩn cấp.
- **Response (200)**: `Array<EmployeeEmergencyContact>`

#### POST /ess/emergency-contacts
Thêm mới người liên hệ khẩn cấp.
- **Request**: `{ "contact_name": "...", "relationship": "...", "phone": "...", "is_primary": true }`
- **Response (201)**: `EmployeeEmergencyContact` object

#### DELETE /ess/emergency-contacts/:id
Xóa người liên hệ khẩn cấp.
- **Response (204)**: No Content

#### GET /ess/documents
Lấy danh sách các tài liệu cá nhân đã tải lên (CCCD, Hợp đồng, Bằng cấp).
- **Response (200)**: `Array<EmployeeDocument>`

#### GET /ess/attendance
Xem lịch sử chấm công cá nhân.
- **Query Params**: `?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD`
- **Response (200)**: `Array<TimeEntry>`
- **Note**: Bao gồm `generation_metadata` để nhân viên tự đối soát cách tính công.

#### GET /ess/leave/balances
Xem số dư phép hiện tại trong năm.
- **Response (200)**: 
  ```json
  [
    {
      "leave_type": "vacation",
      "entitlement": 12,
      "accrued_amount": 3.0,
      "used": 2,
      "remaining": 13.0
    }
  ]
  ```

#### GET /ess/leave/requests
Xem lịch sử các đơn xin nghỉ phép cá nhân.
- **Response (200)**: `Array<LeaveRequest>`

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
    - `payroll`: Xóa phiếu lương (payslips) và cấu hình lương nhân viên (salary components).
    - `full_reset`: Xóa toàn bộ dữ liệu trên cộng với danh sách nhân viên, phòng ban, ca làm việc và chính sách phép. (Giữ lại thông tin Workspace và Members).
- **Response (200 OK)**

#### GET /dev/super-user
Tự động đăng nhập với tư cách Superuser (admin@leeattend.com).
- **Mục đích**: Phục vụ phát triển nhanh (Onboarding) và Testing.
- **Response (200)**: `{ "access_token": "...", "workspace_id": "...", "user_id": "..." }`

---

### 15. Assignment Rules (New)

Quản lý các quy tắc gán ca tự động dựa trên điều kiện (Ngày trong tuần, Vị trí, Phòng ban).
- **UI Note**: Trong WebDashboard, trang này đã được hợp nhất vào `Settings > Time`.

#### Assignment Rule Model
```json
{
  "id": "uuid",
  "workspace_id": "uuid",
  "pattern_id": "uuid",
  "priority": 100, // Độ ưu tiên cao nhất sẽ được áp dụng (số nhỏ hơn = ưu tiên cao hơn)
  "conditions": {
    "days_of_week": [1, 2, 3, 4, 5],
    "department_ids": ["uuid-1"],
    "employee_ids": [],
    "locations": []
  },
  "is_active": true,
  "created_at": "datetime"
}
```

#### GET /workspaces/:ws_id/assignment-rules
Lấy danh sách các quy tắc gán ca tự động.
- **Response (200)**: `Array<AssignmentRule>`

#### POST /workspaces/:ws_id/assignment-rules
Tạo quy tắc mới.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "pattern_id": "uuid",
    "priority": 10,
    "conditions": { ... },
    "is_active": true
  }
  ```
- **Response (201)**: `AssignmentRule` object

#### PATCH /workspaces/:ws_id/assignment-rules/:rule_id
Cập nhật quy tắc.
- **Response (200 OK)**

#### DELETE /workspaces/:ws_id/assignment-rules/:rule_id
Xóa quy tắc.
- **Response (204)**: No Content

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