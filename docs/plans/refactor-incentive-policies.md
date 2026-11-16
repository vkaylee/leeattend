# Refactor Incentive & Bonus Policies Schema

## 🛡️ Rule Compliance Check
1. **Rule 5.3 Database Rules (Postgres / SQLx)**: Every migration MUST include a corresponding `down.sql` script for Disaster Recovery. We will ensure the migration cleanly moves from hardcoded columns to JSONB and adds the assignment table, while providing a rollback script.
2. **Rule 4.11 Enterprise-Grade Premium Standard**: The architecture must be production-ready, meaning strict validation of the JSONB rules in Rust and explicit error handling mapped to `AppError`.
3. **Rule 4.2 Clean Code Standards (SOLID & DRY)**: Moving from hardcoded columns to a JSONB `rule_config` follows the Open-Closed Principle, allowing future KPI additions without database schema changes.

## 1. Mục Tiêu (Objective)
Cấu trúc lại thiết kế cơ sở dữ liệu cho phần Incentive & Bonus Policies để:
1. Chuyển đổi từ các cột hardcode (như `max_sick_leave_days`) sang cấu trúc Rule-Engine bằng JSONB (`rule_config`), giải quyết triệt để sự bất đồng bộ với `leave_types` động.
2. Hỗ trợ gán chính sách chi tiết tới từng phòng ban, nhân viên thông qua bảng `payroll_incentive_assignments` (Targeted Assignment).

## 2. Thiết Kế Database (Database Schema Changes)

### 2.1. Cập nhật bảng `payroll_incentive_tiers`
Xóa các cột hardcode và thay bằng `rule_config` (JSONB).

```sql
-- Thêm cột JSONB
ALTER TABLE public.payroll_incentive_tiers
ADD COLUMN rule_config jsonb DEFAULT '{}'::jsonb NOT NULL;

-- Xóa các cột cũ
ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_late_early_count;
ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_unpaid_leave_days;
ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_sick_leave_days;
ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_total_leave_days;
```

**Cấu trúc JSONB đề xuất cho `rule_config`:**
```json
{
  "conditions": [
    { "type": "attendance_late_early_max", "value": 0 },
    { "type": "leave_days_total_max", "value": 1.5 },
    { "type": "leave_days_specific_max", "leave_type_id": "uuid-of-unpaid-leave", "value": 0 }
  ]
}
```

### 2.2. Tạo bảng mới `payroll_incentive_assignments`
Cho phép gán chính sách thưởng theo phạm vi (Workspace, Department, Employee).

```sql
CREATE TABLE public.payroll_incentive_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    workspace_id uuid NOT NULL,
    policy_id uuid NOT NULL REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_incentive_assignments_assignee_type_check
        CHECK (assignee_type IN ('workspace', 'department', 'employee'))
);

CREATE INDEX idx_payroll_incentive_assignments_lookup
ON public.payroll_incentive_assignments (workspace_id, assignee_type, assignee_id);
```

## 3. Các bước thực hiện (Implementation Steps)

- [ ] **Phase 1: Database Migration**:
  - Viết `up.sql` để tạo bảng `payroll_incentive_assignments`, thêm `rule_config` vào `payroll_incentive_tiers` và drop các cột cũ.
  - Viết `down.sql` để rollback (trả lại các cột cũ và xóa bảng assignment).
  - Chạy migrate test bằng `./scripts/_sqlx.sh migrate run`.
- [ ] **Phase 2: Backend Rust Models & Handlers**:
  - Cập nhật struct `PayrollIncentiveTier` trong Rust (sử dụng `serde_json::Value` cho `rule_config`).
  - Cập nhật DTOs cho API Tạo/Sửa Tier.
  - Tạo các structs, repository và endpoints cho `payroll_incentive_assignments`.
- [ ] **Phase 3: Cập nhật Evaluator (Logic tính lương)**:
  - Cập nhật `services/payroll_evaluator.rs` để phân tích `rule_config` thay vì đọc cột cứng. Lấy `leave_type_id` từ JSON để check với bảng `leave_accrual_logs` hoặc `leave_requests`.
- [ ] **Phase 4: Frontend UI (Tùy chọn)**:
  - Cập nhật `BonusPolicyManager` UI để hỗ trợ Rule Builder động (có thể tạo JSON config).
  - Thêm tab "Assignment" để gán policy.

## 4. Rủi ro & Cân nhắc
- Do `rule_config` là JSONB, validation phải được thực hiện cực kỳ chặt chẽ ở tầng API (Backend) trước khi insert xuống DB để tránh rác data.
- Truy vấn tính lương sẽ phải parse JSON trên Rust thay vì phụ thuộc hoàn toàn vào DB.
