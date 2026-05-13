# Kế hoạch chuyển đổi SQLx sang Diesel - Giai đoạn 5 (Giai đoạn cuối)

Dựa trên việc rà soát toàn bộ dự án, mặc dù chúng ta đã thực hiện chuyển đổi Diesel cho nhiều repository (sử dụng Hybrid pattern), mã nguồn vẫn còn tồn tại một lượng lớn các truy vấn `sqlx::query` nằm rải rác ở các repositories, handlers, và background workers. 

Dưới đây là kế hoạch chi tiết để loại bỏ hoàn toàn SQLx (ngoại trừ những query phức tạp như `WorkspaceStatsRepository` đã được quyết định giữ lại) và chuyển sang Diesel 100%.

## 1. Dọn dẹp các Legacy SQLx Repositories (Đã có Diesel nhưng dùng Hybrid Pattern)
Nhiều repository đã được tạo phiên bản `Diesel...Repository` và được wire vào `app_builder.rs`. Tuy nhiên, khi kiểm tra kỹ, mình phát hiện ra rằng **các Diesel Repositories này đang ủy quyền (delegate) ngược lại cho SqlxRepository** đối với các câu truy vấn phức tạp (ví dụ `get_department_by_name`, `transfer_employee`, `get_workspace_swaps`). Do đó, bản thân file SQLx cũ chưa thể xóa được ngay.

**Mục tiêu:**
1. Viết lại các phương thức còn đang "vay mượn" bằng `diesel::sql_query` hoặc Diesel DSL bên trong các struct `Diesel...Repository`.
2. Sau khi đã tự chủ hoàn toàn 100%, tiến hành xóa bỏ struct `Sqlx...Repository` và tất cả các macro `sqlx::query` trong các file repository tương ứng.

* [ ] `DepartmentRepository` (`diesel_department_repository.rs` đang gọi về sqlx)
* [ ] `ShiftSwapRepository` (`diesel_shift_swap_repository.rs` đang gọi về sqlx)
* [ ] `ImportJobRepository` (`import_job_repository.rs`) - **Repo này đã hoàn toàn dùng diesel::sql_query, có thể xóa sqlx ngay.**
* [ ] `SalaryBaseRepository` (`salary_base_repository.rs`)
* [ ] `EmploymentPhaseRepository` (`employment_phase_repository.rs`)
* [ ] `FeatureFlagRepository` (`feature_flag_repository.rs`)
* [ ] `AssignmentRuleRepository` (`assignment_rule_repository.rs`)
* [ ] `StagingLogRepository` (`staging_log_repository.rs`)
* [ ] `RawAttendanceLogRepository` (`raw_attendance_log_repository.rs`)

## 2. Refactor và chuyển đổi Inline SQL trong Handlers / Services
Rất nhiều logic truy vấn cơ sở dữ liệu đang được viết trực tiếp bên trong Handlers (inline SQL). Đây là vi phạm quy tắc thiết kế (DIP) và cần được chuyển vào các Repositories trước khi chuyển sang Diesel.

**Mục tiêu:** Đưa các câu `sqlx::query` vào trait Repository, sau đó implement bằng Diesel.
* [ ] **Workspaces Handlers:** 
  * `data_management.rs` (~30 queries)
  * `roles.rs` (~10 queries)
  * `crud.rs`, `members.rs`, `permissions.rs`
* [ ] **Employee Self-Service (ESS):**
  * `ess.rs` (~13 queries)
* [ ] **Attendance Handlers:**
  * `policies.rs` (~9 queries)
  * `policy_assignments.rs`, `overtime.rs`, `logs.rs`, `paper_forms.rs`, `staging.rs`
* [ ] **Payroll Handlers:**
  * `incentives.rs`, `configs.rs`, `settings.rs`
* [ ] **System Admin / Auth / Tenants:**
  * `system_admin.rs` (~7 queries), `system_auth.rs`, `system_tenants.rs`
* [ ] **Khác:**
  * `agents.rs`, `dashboard_preferences.rs`, `leave_types.rs`, `debug.rs`, `notifications.rs`

## 3. Chuyển đổi các Repositories chưa có Diesel
Đây là danh sách các Repositories hiện tại vẫn đang sử dụng thuần túy `sqlx::query` và chưa có bản implement Diesel tương ứng.

**Mục tiêu:** Áp dụng Hybrid pattern (hoặc chuyển thẳng sang Diesel) và wire vào AppState.
* [ ] **Core Business:** `workspace_repository`, `department_repository` (nếu còn sót), `agent_repository`, `user_repository`
* [ ] **Employee Module:** `employee/core_queries.rs`, `employee/recruitment_queries.rs`, `employee/analytics_queries.rs`
* [ ] **Payroll Module:** `payroll_repository`, `payroll_rule_repository`
* [ ] **Leave Module:** `leave_policy_repository`, `leave_request_repository`, `leave_accrual_log_repository`, `leave_balance_repository`, `holiday_repository`
* [ ] **Attendance Module:** `attendance_daily_summary_repository`, `attendance_policy_repository`, `attendance_log_error_repository`, `time_entry_repository`
* [ ] **Shift / Roster Module:** `working_pattern_repository`, `overtime_policy_repository`, `roster/roster_crud.rs`, `roster/schedule_queries.rs`
* [ ] **System / Logs:** `audit_log_repository`

## 4. Chuyển đổi Background Workers và Domain Logic
Các workers hiện đang chạy logic ngầm và gọi trực tiếp `sqlx::query`.

**Mục tiêu:** Sử dụng AppState / Repositories trong workers hoặc viết lại query bằng Diesel.
* [ ] `worker/handlers/expiring_contracts.rs` (~5 queries)
* [ ] `worker/handlers/sync_hire_date.rs` (~3 queries)
* [ ] `worker/handlers/reprocess_attendance.rs`
* [ ] `worker/handlers/merge.rs`
* [ ] `worker/handlers/attendance.rs`
* [ ] `domain/policy/resolver.rs` (Domain logic không nên chứa macro `sqlx::query`)

## Ưu tiên thực hiện (Thứ tự đề xuất)
1. **Bước 1:** Dọn dẹp sạch sẽ các `SqlxRepository` đã được migrate (Phần 1) để giảm số lượng "nhiễu" của `sqlx` trong dự án.
2. **Bước 2:** Chuyển đổi các Repositories cốt lõi chưa có Diesel (Phần 3) như Employee, Roster, Payroll.
3. **Bước 3:** Bắt tay vào xử lý Handlers (Phần 2) - đây là phần rủi ro vì phải thay đổi cả kiến trúc (đưa logic vào repo).
4. **Bước 4:** Xử lý Workers và Domain logic (Phần 4).

> **Ghi chú:** `WorkspaceStatsRepository` vẫn được giữ nguyên là `sqlx` do chứa các câu truy vấn CTE phức tạp, theo như quyết định từ Phase 4.
