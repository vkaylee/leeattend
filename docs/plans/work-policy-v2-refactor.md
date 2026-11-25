# Work Policy v2 — Pre-Go-Live HRM Standardization Refactor

Status: Completed & verified (working tree ready for review/commit)
Date: 2026-11-25
Owner: LeeAttend HRM, Attendance & Payroll domain
Prerequisites: Employee Tenures v2 (`87d6c4d02`, `ff7cf2419` — completed on `main`)
Supersedes transitional pieces of: `docs/plans/work-policy-configuration-plan.md`, `docs/decisions/2026-11-16-work-policy-activation-boundary.md`

---

## Confirmed Decisions (Pre-Go-Live)

1. **Single source of truth on `work_policy_*`:** App chưa go-live → gỡ toàn bộ nguồn cấu hình cạnh tranh/fallback (`work_credit_policies`, `pay_basis_policies`, `attendance_policies`, `attendance_policy_versions`, `attendance_policy_assignments`, `overtime_policy_assignments`, và các payroll/OT policy keys trong `config_entries`).
2. **Anchor membership to `employment_relationships`:** Theo mô hình 4 tầng của Tenures v2 (`employees → employee_tenures → employment_relationships → employments`), membership gắn vào `employment_relationship_id` thay vì `employments.id` để chuyển phòng ban trong cùng relationship không làm gãy policy, đồng thời hỗ trợ nhiều employer song song trong cùng 1 tenure.
3. **Built-in lifecycle `draft → published → archived`:** Không tạo cờ `activated` rời. `work_policy_versions.status = 'published'` là 1 cổng duy nhất cho mọi consumer (Attendance, Reports, Payroll, Simulation). Draft chỉnh sửa tự do có optimistic lock (`row_version`); publish nguyên tử kèm revalidation và `Idempotency-Key`.
4. **Immutable versioned OT policy (lean 1-table model):** Dùng `overtime_policy_versions` với cột `rules jsonb` bất biến + `daily_cap_minutes`, `monthly_cap_hours`, `yearly_cap_hours` (chỉ tách bảng `overtime_policy_version_rules` nếu sau này cần truy vấn SQL theo từng rule OT).
5. **Stable route prefix & Pure Payroll Engine:** Giữ prefix `/api/v1/payroll/work-policy-groups` để giảm phạm vi sửa không cần thiết; tách `repository/payroll.go` thành Batch Loader + Pure Engine + Persistence Writer để bỏ hack write-then-rollback của simulation và loại bỏ N+1 ở batch payroll.

---

## Target Data Model & DB Invariants

```text
employment_relationships (1 tenure × 1 employer)
  └── 1:N work_policy_memberships [effective_from, effective_to)
        └── N:1 work_policy_groups (code, name, description, archived_at)
              └── 1:N work_policy_versions (draft | published | archived, row_version)
                    ├── Attendance & Work Credit rules (hợp nhất attendance_policies + work_credit_policies)
                    ├── Pay Basis rules (monthly[fixed|schedule] | hourly | shift + paid_leave_credit_mode)
                    ├── working_pattern_id (nullable FK → working_patterns)
                    └── overtime_policy_version_id → overtime_policy_versions (immutable caps + rules jsonb)
```

### Mandatory PostgreSQL Tables & Constraints

1. **`overtime_policy_versions`** (gắn `overtime_policy_definitions`):
   - Cột: `id`, `policy_id` (FK `overtime_policy_definitions`), `version_no`, `status` (`draft|published|archived`), `row_version`, `daily_cap_minutes`, `monthly_cap_hours numeric(8,2)`, `yearly_cap_hours numeric(8,2)`, `rules jsonb NOT NULL`, `created_by`, `created_at`, `updated_at`, `published_by`, `published_at`.
   - `UNIQUE (policy_id, version_no)`.
   - `CHECK`: non-negative caps, `jsonb_typeof(rules) = 'array'`, status enum.
   - Mỗi phần tử trong `rules`: `{ priority, day_type ('weekday'|'weekend'|'holiday'|'any'), days_of_week (bitmask 0..127), time_start?, time_end?, multiplier (>0) }`.
   - Trigger immutability: chặn `UPDATE`/`DELETE` nội dung khi `status = 'published'` (chỉ cho chuyển sang `archived`).

2. **`work_policy_groups`**:
   - Cột: `id`, `code text UNIQUE`, `name text`, `description text`, `archived_at timestamptz`, `created_by`, `created_at`, `updated_at`.
   - `CHECK`: `length(btrim(code)) BETWEEN 1 AND 64`, `length(btrim(name)) BETWEEN 1 AND 255`.

3. **`work_policy_versions`**:
   - Cột định danh & vòng đời: `id`, `group_id`, `version_no`, `status` (`draft|published|archived`), `row_version bigint DEFAULT 1`, `effective_from date NOT NULL`, `effective_to date` (exclusive `[from, to)`), `created_by`, `created_at`, `updated_at`, `published_by`, `published_at`.
   - Cột chấm công & quy đổi công: `minutes_per_credit > 0`, `work_unit_minutes > 0 DEFAULT 1`, `rounding_policy IN ('none','floor','round','ceil')`, `early_arrival_strategy IN ('allow','clamp','block')`, `late_arrival_grace_minutes >= 0`, `early_ot_enabled boolean`, `early_ot_qualification_grace_minutes >= 0`, `early_ot_max_minutes >= 0` (nullable), `break_auto_deduct boolean`, `break_is_overtime boolean`, `ot_aggregation_mode IN ('aggregate','per_segment')`.
   - Cột lịch: `working_pattern_id uuid NULL REFERENCES working_patterns(id) ON DELETE RESTRICT`.
   - Cột căn cứ trả lương: `pay_method IN ('monthly','hourly','shift')`, `monthly_denominator IN ('fixed','schedule')`, `monthly_standard_days numeric(12,4)`, `rate_component_definition_id uuid REFERENCES salary_component_definitions(id)`, `paid_leave_credit_mode text NOT NULL DEFAULT 'standard_day'`.
   - Cột OT: `overtime_policy_version_id uuid NOT NULL REFERENCES overtime_policy_versions(id) ON DELETE RESTRICT`.
   - Constraints:
     - `UNIQUE (group_id, version_no)`
     - `CHECK (effective_to IS NULL OR effective_to > effective_from)`
     - `CHECK` chặt chẽ cho `monthly` (fixed cần `monthly_standard_days > 0`, schedule yêu cầu `NULL`, không có `rate_component_definition_id`) vs `hourly|shift` (bắt buộc `rate_component_definition_id`, không có monthly fields).
     - `EXCLUDE USING gist (group_id WITH =, daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)') WITH &&) WHERE (status = 'published')`.
   - Trigger immutability: khi `status = 'published'`, cấm sửa mọi trường rule hoặc xóa; chỉ cho phép cập nhật `effective_to` (khi publish version mới nối tiếp) hoặc chuyển `status = 'archived'`.

4. **`work_policy_memberships`**:
   - Cột: `id`, `employment_relationship_id uuid NOT NULL REFERENCES employment_relationships(id) ON DELETE RESTRICT`, `group_id uuid NOT NULL REFERENCES work_policy_groups(id) ON DELETE RESTRICT`, `effective_from date NOT NULL`, `effective_to date` (exclusive `[from, to)`), `created_by`, `created_at`.
   - `CHECK (effective_to IS NULL OR effective_to > effective_from)`.
   - `EXCLUDE USING gist (employment_relationship_id WITH =, daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)') WITH &&)`.
   - Temporal containment triggers (đồng bộ Tenures v2):
     - `trg_work_policy_memberships_rel_bounds`: `membership.effective_from >= rel.start_date` và nếu `rel.end_date IS NOT NULL` thì `membership.effective_to IS NOT NULL AND membership.effective_to <= (rel.end_date + 1)`.
     - `trg_employment_relationships_policy_bounds_guard`: chặn thu hẹp `rel [start_date, end_date]` khiến membership nằm ngoài khoảng.

5. **Legacy Removals (Pre-Go-Live):**
   - Drop tables: `labor_policy_memberships`, `labor_policy_group_versions`, `labor_policy_groups`, `work_credit_policies`, `pay_basis_policies`, `attendance_policy_assignments`, `attendance_policy_versions`, `attendance_policies`, `overtime_policy_assignments`, `overtime_rules` (sau khi chuyển sang `overtime_policy_versions`).
   - Update FK references: `attendance_daily_summaries.work_policy_version_id` và `payslip_segments.work_policy_version_id` (thay `labor_policy_group_version_id`) trỏ tới `work_policy_versions(id)`.
   - Clean `config_entries` keys: `payroll.std_working_day_mode`, `payroll.fixed_std_days`, `payroll.working_days_per_week`, `payroll.hours_per_day`, `payroll.max_ot_hours_per_day`, `payroll.max_ot_hours_per_month`, `overtime.max_hours_per_month`.
   - Remove legacy routes: `/payroll-settings*`, `/employees/{id}/payroll-config`, `/departments/{id}/payroll-config`, `/departments/payroll-reconciliation`, `/attendance/rules*`, `/attendance/policies/assignments*`, `/overtime/policies/assign`. *(Giữ `/attendance/adjustments/behalf-assignments*`.)*

---

## Core Contracts

### 1. Shared `WorkPolicyResolver` (`apiserver/internal/repository/work_policy_resolver.go`)
Một resolver duy nhất cho Attendance Reprocess, Attendance Summaries, Reports, Payroll, Simulation và Impact Staleness:
- `ResolveAt(ctx, db, relationshipID, asOf) (ResolvedPolicy, error)`
- `ResolveRange(ctx, db, relationshipIDs, start, end) (map[uuid.UUID][]ResolvedPolicy, error)` (batch `ANY($1::uuid[])`, lọc `v.status = 'published'`, hỗ trợ map từ `employment_id` qua `employments.relationship_id`).

### 2. Work Policy & Overtime API
- `GET /api/v1/payroll/work-policy-groups` (`payroll:read`) — danh sách nhóm + current published + latest draft.
- `POST /api/v1/payroll/work-policy-groups` (`payroll:manage`) — tạo nhóm + draft v1.
- `GET /api/v1/payroll/work-policy-groups/{group_id}/versions` (`payroll:read`) — lịch sử version (phân trang, filter `status`).
- `GET /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}` (`payroll:read`) — chi tiết version.
- `POST /api/v1/payroll/work-policy-groups/{group_id}/versions` (`payroll:manage`) — tạo draft mới (tùy chọn `clone_from_version_id`).
- `PATCH /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}` (`payroll:manage`) — sửa draft, bắt buộc `expected_row_version` (sai → `409`).
- `POST /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/validate` (`payroll:read`) — trả `blockers[]` & `warnings[]`.
- `POST /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/publish` (`payroll:manage`) — atomic TX: lock group, check `expected_row_version` + `Idempotency-Key`, revalidate blockers, chặn chạm kỳ payroll đã khóa, hỗ trợ `auto_close_previous`, set `status='published'`, emit audit không PII/lương.
- `GET /api/v1/payroll/work-policy-groups/{group_id}/memberships` (`payroll:read`) & `POST .../memberships` (`payroll:manage`) — nhận `employment_relationship_id`.
- `POST /api/v1/payroll/work-policy-memberships/transfer` (`payroll:manage`) — atomic TX đóng membership cũ tại `effective_date` + mở membership mới tại `effective_date`, chặn `409` nếu đụng kỳ đã khóa.
- `GET /api/v1/employment-relationships/{relationship_id}/work-policy?as_of=YYYY-MM-DD` (`payroll:read`) — point-in-time resolution.
- `POST /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/simulate` (`payroll:sensitive:read`) — batch ≤ 100 `employment_relationship_id`, chạy Pure Engine so sánh current published vs draft trên cùng `PeriodInput`, **0 DB write**, chạy được cả trên kỳ đã khóa; trả `total/computed/affected/uncomputable`, `input_fingerprint`, `evaluated_at`, `expires_at`, `items[]` kèm `trace`.
- Overtime policy versions: `GET/POST /api/v1/overtime/policies`, `GET/POST /api/v1/overtime/policies/{policy_id}/versions`, `PATCH/POST .../publish` (bất biến khi published).

### 3. Pure Payroll Engine
Tách `apiserver/internal/repository/payroll.go` thành 3 lớp:
1. **Batch Period Loader:** prefetch salary components, attendance summaries, leave, approved OT, schedule workdays, KPI inputs và `WorkPolicyResolver.ResolveRange` bằng `ANY($1::uuid[])` (loại bỏ N+1).
2. **Pure Domain Engine (`internal/domain/payroll`):** `CalculatePayslip(PeriodInput, []ResolvedPolicy) (CalculationResult, error)` — 0 DB call, 0 side effect, tính đúng OT theo `day_type`/khung giờ/ca đêm + caps, sinh `Trace` chi tiết + `InputFingerprint`.
3. **Persistence Writer:** chỉ ghi payslip/segments/calculation_run cho calculate thật; simulation không đụng DB write hay `FOR UPDATE` lock.

---

## Ordered Execution Plan

### Phase 1 — Clean Schema, Phased Baseline & Seeds
- [x] **1.1** Tạo incremental migration `apiserver/migrations/20261127000000_work_policy_v2_standardization/{up,down}.sql` và đồng bộ `apiserver/migrations/phases/{payroll,attendance_schedule,hr_core}/{up,down}.sql`.
- [x] **1.2** Cập nhật `apiserver/migrations/testdata/{schema_contract.sql,phased_schema_contract.sql}` kiểm tra constraints, immutability triggers, membership containment triggers và vắng mặt các bảng legacy.
- [x] **1.3** Cập nhật `scripts/seed_dev_accounts.sql`, `scripts/seed_schedule_lifecycle_dev.sql` và `apiserver/migrations/phases/runtime_seeds.up.sql` seed sẵn `overtime_policy_versions` (`published`), `work_policy_groups` + `work_policy_versions` (`published`) và `work_policy_memberships` theo `employment_relationship_id`.
- [x] **Check:** Chạy migration up/down/reapply + `schema_contract.sql` + `phased_schema_contract.sql` trên disposable PostgreSQL.

### Phase 2 — Shared `WorkPolicyResolver`, Versioned OT & Lifecycle API
- [x] **2.1** Viết `apiserver/internal/domain/workpolicy` (version rules, OT rules/caps validation, cross-policy validation rules) và `apiserver/internal/repository/work_policy_resolver.go` (`ResolveAt`, `ResolveRange`).
- [x] **2.2** Thay toàn bộ 6 câu SQL join lặp trong `work_policy.go`, `attendance.go`, `reports.go`, `payroll.go` bằng `WorkPolicyResolver`; fix `currentImpactPolicies` kiểm tra toàn kỳ thay vì chỉ ngày mùng 1.
- [x] **2.3** Nâng cấp Overtime Policy (`routes_overtime.go`, `api/overtime.go`, `repository/overtime.go`) sang `overtime_policy_versions` draft/publish bất biến; bỏ `POST /overtime/policies/assign`.
- [x] **2.4** Triển khai Work Policy Lifecycle API (`routes_work_policy.go`, `api/work_policy.go`, `repository/work_policy.go`): create group+draft, create/clone draft, patch draft (`expected_row_version` → `409`), validate (cross-policy blockers/warnings), atomic publish (`Idempotency-Key` + locked period guard + `auto_close_previous` + audit không PII/lương), membership CRUD + atomic transfer.
- [x] **2.5** Gỡ các legacy routes/handlers/repositories/tests đã liệt kê ở mục Legacy Removals và xóa toàn bộ fallback `config_entries` cho payroll/OT policy.
- [x] **Check:** Focused unit + PostgreSQL integration tests cho resolver, publish concurrency (`409`), idempotency, immutability trigger, membership containment & atomic transfer.

### Phase 3 — Pure Payroll Engine & Batch Simulation thật
- [x] **3.1** Tạo Batch Period Loader (`apiserver/internal/repository/payroll_loader.go`) prefetch dữ liệu theo `employment_relationship_ids[]` bằng `ANY($1::uuid[])` — loại bỏ N+1 trong `CalculateBatchPayroll`.
- [x] **3.2** Chuyển logic tính lương sang Pure Engine (`apiserver/internal/domain/payroll/engine.go`): monthly (fixed/schedule), hourly, shift, mid-period policy switch, OT multiplier theo `day_type` (weekday/weekend/holiday) + khung giờ/ca đêm + daily/monthly/yearly caps, `paid_leave_credit_mode`, sinh `Trace` chi tiết + `InputFingerprint`.
- [x] **3.3** Tách Persistence Writer (`apiserver/internal/repository/payroll_writer.go`); cập nhật `SimulateEmployeePayroll` gọi trực tiếp Pure Engine (xóa hack `errSimulationRollback` và lock `payroll_periods`).
- [x] **3.4** Thêm `POST /api/v1/payroll/work-policy-groups/{group_id}/versions/{version_id}/simulate` (batch ≤ 100 relationships, current vs draft trên cùng input, 0 DB write, chạy được trên kỳ đã khóa, `payroll:sensitive:read`).
- [x] **3.5** Thêm pagination limit (≤ 100) cho `ListPayrollCalculationRuns` và request context deadline cho batch calculate/simulate.
- [x] **Check:** Pure engine unit matrix + PostgreSQL integration tests cho batch calculate (đo giảm N+1), locked-period simulation, 0 DB writes, RBAC `403`, oversized batch `400`.

### Phase 4 — OpenAPI, Generated Types & Policy Management UI
- [x] **4.1** Cập nhật `apiserver/openapi.yaml` (xóa legacy endpoints, thêm v2 work-policy & overtime-version endpoints), chạy `scripts/validate_openapi_contract.py` và regenerate `webdashboard/src/types/api-generated.ts`.
- [x] **4.2** Cập nhật `webdashboard/src/lib/api/payroll.ts` và `webdashboard/src/lib/query-keys.ts`; gỡ client/UI legacy (`/payroll-settings`, `/attendance/rules`, `/departments/{id}/payroll-config`, `/overtime/policies/assign`).
- [x] **4.3** Tách `webdashboard/src/components/payroll/payroll-settings-section.tsx` thành các component rõ trách nhiệm:
  - `work-policy-group-list.tsx` (nhóm + badge published/draft + hành động tạo/clone).
  - `work-policy-version-editor.tsx` (4 phần, ẩn trường không áp dụng, lưu draft có `row_version` + xử lý `409`, validate blockers/warnings, publish).
  - `work-policy-memberships-panel.tsx` (gán theo `employment_relationship` + modal atomic transfer).
  - `work-policy-simulation-panel.tsx` (batch simulate current vs draft, hiện uncomputable/expired/fingerprint + trace viewer).
- [x] **4.4** Đồng bộ i18n (`en`, `vi`, `zh`), loading/empty/error/409 states, keyboard & a11y, route unsaved-draft guard.
- [x] **Check:** `leedevkit test web` (vitest, typecheck, i18n, lint, playwright E2E).

### Phase 5 — Full Verification & Docs Sync
- [x] **5.1** Chạy full gates (không `--unit-only`): `leedevkit test api`, `leedevkit test web`, `leedevkit test all`; đọc per-phase logs trong `.test_logs/` và chạy `git diff --check`.
- [x] **5.2** Playwright MCP route × role audit trên đủ 5 role (`employee`, `team_lead`, `shift_manager`, `hr_manager`, `admin`) × 4 desktop viewports (`1920×1080`, `1440×900`, `1366×768`, `1280×800`) với dữ liệu policy group thật: create draft → validate → simulate → publish → atomic transfer → verify trace, 0 console/network errors bất thường.
- [x] **5.3** Cập nhật `docs/plans/work-policy-configuration-plan.md` và `docs/decisions/2026-11-16-work-policy-activation-boundary.md` (đánh dấu superseded bởi `work_policy_versions.status = 'published'` + `WorkPolicyResolver`).

---

## Test Impact Matrix

| Dimension | Required Coverage |
|---|---|
| Changed behavior | Draft/patch/validate/publish lifecycle, versioned OT rules/caps, relationship-scoped memberships, atomic transfer, pure-engine batch simulation |
| Existing behavior | Finalized/locked payroll immutability, approved-OT-only payout, attendance reprocessing break/grace/OT aggregation, KPI formula runs |
| Inputs & boundaries | Empty/malformed UUID/dates, exclusive `effective_to`, `0`/negative minutes/caps, `monthly_denominator` fixed vs schedule, batch size `100` vs `101`, duplicate IDs, overnight OT windows |
| State | Draft vs published vs archived, stale `row_version` (`409`), publish overlap (`409`), membership outside relationship bounds, mid-period transfer, locked payroll period |
| Failures | DB rollback on publish/transfer failure, missing policy fail-closed, uncomputable simulation items (never zero/unaffected), context deadline exceeded |
| Security | RBAC `payroll:read`, `payroll:manage`, `payroll:sensitive:read` (`403` for unauthorized roles), audit events without PII/salary/secrets |
| Compatibility | Phased + incremental migration parity (`schema_contract.sql`, `phased_schema_contract.sql`), OpenAPI + generated TS contract parity |
| Operations | Bounded queries (`ANY($1::uuid[])`, no N+1 in batch calculate/resolver), paginated calculation runs, zero DB writes during simulation |

## Rule Compliance
- `database-rules.md §3–4`: transactional integrity, PostgreSQL GIST/trigger invariants, incremental + phased baseline parity, no N+1.
- `access-control.md §1–3`: handler-level RBAC enforcement before business logic, default deny `403`, masked audit.
- `api-governance.md §1–2`: single canonical OpenAPI contract, bounded pagination, UUID/date validation, `AppError`.
- `reliability-engineering.md §2–3`: bounded batch sizes, idempotent publish, request deadlines, read-only lock-free simulation.
- `development-workflow.md §2,§7` & `testing-standards.md §1–2`: test-impact matrix, full unit + PostgreSQL integration + web/E2E + Playwright MCP 5-role × 4-viewport verification.
- `data-governance.md`: zero credentials, tokens, PII, or salary values in logs, audit details, or artifacts.
