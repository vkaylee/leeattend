# Employee Tenures v2 — Pre-Go-Live HRM Standardization Refactor

Status: Approved decisions, ready to execute
Date: 2026-11-25
Owner: LeeAttend HRM domain
Supersedes transitional compatibility pieces of: `docs/plans/employee-tenures.md`

## Confirmed Decisions (Pre-Go-Live)
1. **Permanent single-person code ownership:** Một mã nhân viên (`employee_number_normalized = UPPER(BTRIM(employee_number))`) đã từng cấp cho Person A thì **cấm vĩnh viễn** cấp cho Person B (kể cả sau khi Person A nghỉ việc). Person A có thể dùng lại mã cũ của chính mình ở đợt rehire mới hoặc nhận mã mới chưa ai sở hữu.
2. **Single rehire/tenure endpoint:** Xóa hẳn `POST /api/v1/employees/{id}/rehire`. Dùng duy nhất `POST /api/v1/employees/{employee_id}/tenures` (không gate sau `ff_employee_identity_match`; chỉ giữ gate cho `POST /api/v1/employees/match`).
3. **Seniority-ready tenure metadata:** Thêm trực tiếp vào `employee_tenures`:
   - `hire_type varchar(20) NOT NULL DEFAULT 'initial' CHECK (hire_type IN ('initial', 'rehire'))`
   - `prior_service_credit_days integer NOT NULL DEFAULT 0 CHECK (prior_service_credit_days >= 0)`
   - `termination_reason text`
   - `rehire_eligible boolean NOT NULL DEFAULT true`
   - `notes text`
4. **No legacy dual-write debt:** App chưa go-live → gỡ 6 cột legacy trên `employees` (`user_id`, `employee_id_number`, `hire_date`, `original_hire_date`, `termination_date`, `is_active`) và xóa bảng transitional `employee_tenure_reconciliation`.

---

## Target 4-Layer HRM Data Model & DB Invariants

```
employees (Person)
  └── 1:N employee_tenures (Hiring episode: HR code, HRM user_id, hire_type, prior_service_credit_days, dates)
        ├── 1:N employment_relationships (Legal employer relationship; multi-employer overlap allowed inside 1 tenure)
        │     ├── 1:N employments (Dated assignment segment: dept, location, job title, manager)
        │     └── 1:N employee_contracts (Legal contract)
        └── 1:N employee_documents / file_acknowledgements (ESS-scoped by tenure_id)
```

### Mandatory PostgreSQL Constraints
1. **`employees` (pure Person):** `id`, `full_name`, `tags`, `recruitment_source_id`, `created_at`, `updated_at`, `deleted_at`.
2. **`employee_tenures`:**
   - `employee_number_normalized text GENERATED ALWAYS AS (UPPER(BTRIM(employee_number))) STORED`
   - `employee_tenures_no_overlap_person`: `EXCLUDE USING gist (employee_id WITH =, daterange(start_date, COALESCE(end_date + 1, 'infinity'::date), '[)') WITH &&) WHERE (deleted_at IS NULL)`
   - `employee_tenures_permanent_code_owner`: `EXCLUDE USING gist (employee_number_normalized WITH =, employee_id WITH <>)` (khóa vĩnh viễn chủ sở hữu mã theo Person; fallback bằng bảng B-tree ownership nếu `btree_gist` không hỗ trợ `uuid WITH <>`).
   - `idx_employee_tenures_active_user`: `UNIQUE (user_id) WHERE user_id IS NOT NULL AND end_date IS NULL AND deleted_at IS NULL` (cho phép giữ `user_id` lịch sử trên tenure đã đóng).
   - `hire_type IN ('initial', 'rehire')`, `prior_service_credit_days >= 0`, `end_date IS NULL OR start_date <= end_date`.
3. **Strict parent links (no orphans):**
   - `employment_relationships.tenure_id NOT NULL` + composite FK `(tenure_id, employee_id) REFERENCES employee_tenures(id, employee_id)`.
   - `employments.relationship_id NOT NULL` + composite FK `(relationship_id, employee_id, employer_id) REFERENCES employment_relationships(id, employee_id, employer_id)`.
4. **Temporal containment triggers (DB-enforced):**
   - `trg_employment_relationships_tenure_bounds`: `relationship.start_date >= tenure.start_date` và nếu `tenure.end_date IS NOT NULL` thì `relationship.end_date IS NOT NULL AND relationship.end_date <= tenure.end_date`.
   - `trg_employments_relationship_bounds`: `segment.start_date >= relationship.start_date` và nếu `relationship.end_date IS NOT NULL` thì `segment.end_date IS NOT NULL AND segment.end_date <= relationship.end_date`.
   - `trg_employee_tenures_bounds_guard`: chặn thu hẹp `tenure [start_date, end_date]` khiến child relationship nằm ngoài khoảng.
5. **ESS document scoping:**
   - Thêm `tenure_id uuid` + composite FK `(tenure_id, employee_id) REFERENCES employee_tenures(id, employee_id)` cho `employee_documents` và `file_acknowledgements`.

---

## Ordered Execution Plan

### Phase 1 — Clean Schema, Phased Baseline & Seeds
- [x] **1.1** Cập nhật `apiserver/migrations/phases/hr_core/up.sql` & `down.sql`, `apiserver/migrations/phases/foundation/up.sql` (`file_acknowledgements.tenure_id`), và tạo incremental migration mới `apiserver/migrations/20261126010000_employee_tenures_v2_standardization/{up,down}.sql` để giữ đồng bộ cả hai nhánh migration.
- [x] **1.2** Gỡ `employee_tenure_reconciliation` khỏi `phases/hr_core`, `testdata/schema_contract.sql`, và các test schema contract liên quan; cập nhật contract check cho 4-layer invariants mới.
- [x] **1.3** Cập nhật `scripts/seed_dev_accounts.sql` và `apiserver/migrations/phases/runtime_seeds.up.sql` (nếu có) bỏ ghi 6 cột legacy trên `employees`, seed đầy đủ `employee_tenures` (`hire_type = 'initial'`), `employment_relationships.tenure_id`, `employments.relationship_id`.
- [x] **Check:** Chạy migration & `schema_contract.sql` trên disposable DB, verify `btree_gist` constraint `employee_tenures_permanent_code_owner` và triggers containment.

### Phase 2 — Unified Transactional Lifecycle (Backend Writers)
- [x] **2.1 Hire (`CreateEmployee`):** `apiserver/internal/repository/employees.go` — 1 transaction tạo `employees` (Person) + `employee_profiles` (nếu có) + `employee_tenures` (`hire_type = 'initial'`) + `employment_relationships` (bắt buộc `employer_id` khi tạo assignment) + `employments`; advisory lock theo `emp_code` và identity hashes; không còn dual-write vào `employees`.
- [x] **2.2 Unified Rehire (`CreateEmployeeTenure`):** `apiserver/internal/repository/employee_tenures.go`, `apiserver/internal/api/employee_tenures.go`, `apiserver/routes_employees.go`:
  - Xóa `Rehire` handler, `RehireEmployee` repo method, `RehireEmployeeRequest`, route `POST /api/v1/employees/{id}/rehire`.
  - Đưa `POST /api/v1/employees/{employee_id}/tenures` ra khỏi `identityMatchGate` (chạy trên `router` chuẩn, vẫn require `employees:rehire` scoped permission).
  - Mở rộng `CreateEmployeeTenureRequest`: hỗ trợ `employee_number` tùy chọn (bỏ trống = kế thừa mã tenure gần nhất; nhập mới = validate & lock `emp_code` mới), `prior_service_credit_days`, `notes`, `expected_employee_code`, `expected_status`.
  - Bắt buộc `hire_type = 'rehire'`, chặn `deleted_at IS NOT NULL`, chặn `rehire_eligible = false` (trừ khi có cờ override tường minh nếu cần — mặc định chặn `409`), chặn tenure/relationship/segment mở hoặc overlap (`prev.end_date < start_date`), luôn khởi tạo `user_id = NULL`.
- [x] **2.3 Atomic Terminate & Close Tenure:**
  - Thêm `POST /api/v1/employees/{employee_id}/employments/{employment_id}/terminate` (`apiserver/internal/api/contracts_employments.go`, `repository/contracts_employments.go`): khóa `employees` row → kiểm tra locked payroll period → đóng `employments` & `employment_relationships` → nếu là relationship mở cuối cùng của tenure thì tự động đóng `employee_tenures` (`end_date`, `termination_reason`, `rehire_eligible`) + **revoke toàn bộ HRM sessions & Redis `user:revoked_at`** (fail-closed trước commit hoặc roll back nếu revocation lỗi).
  - Thêm `POST /api/v1/employees/{employee_id}/tenures/{tenure_id}/close`: đóng toàn bộ active relationships/segments của tenure + đóng tenure + revoke sessions atomically.
  - Chặn `PATCH /api/v1/employees/{employee_id}/employments/{employment_id}` không cho bypass đổi `status='terminated'` mà không qua flow terminate (trả `409`).
  - Cập nhật `TerminateEmploymentsTransactional` (`apiserver/internal/repository/employee_imports.go`) dùng chung logic đóng tenure khi relationship cuối kết thúc + revoke sessions.
- [x] **2.4 Relationship & Segment Integrity:**
  - `CreateEmploymentRelationship` (`repository/contracts_employments.go`): bắt buộc giải mã/nhận `tenure_id` hiệu lực thuộc `employee_id`, chặn gắn vào tenure đã đóng hoặc ngoài khoảng ngày.
  - `CreateEmployment` (`repository/contracts_employments.go`): bắt buộc có `relationship_id` hợp lệ thuộc `employee_id` và `employer_id` (hoặc tạo relationship gắn tenure hiệu lực).
  - `TransferEmployee` (`repository/employee_departments.go`): giữ nguyên `relationship_id` và `tenure_id`, kiểm tra locked payroll period.

### Phase 3 — Centralized Read Model, ESS `TenureContext` & Account Link
- [x] **3.1 Centralized Tenure Resolver Helper:** tạo `apiserver/internal/repository/tenure_sql.go` giải mã tenure theo `orgTodaySQL` (theo org timezone, không hardcode `CURRENT_DATE`) và chuẩn hóa 3 trạng thái `planned`, `active`, `closed`.
- [x] **3.2 Migrate toàn bộ legacy `employees.*` readers:**
  - `employees.go` (`ListEmployees`, `GetEmployee`, `UpdateEmployee`, `SoftDeleteEmployee`): hiển thị cả NV `planned` (future-dated hire/rehire), search khớp cả mã tenure hiện tại và mã lịch sử, bỏ hoàn toàn cột legacy.
  - `attendance.go`, `schedule_evidence.go`, `organization_authz.go`, `files.go`, `overtime.go`, `leaves.go`: đổi `employees.user_id` sang `employee_tenures` hiệu lực.
  - `pattern_compliance.go`, `lifecycle.go`, `employee_identity_match.go`, `dashboard_analytics.go`: đọc mã NV, ngày vào làm, ngày nghỉ việc, trạng thái hoạt động từ `employee_tenures`.
  - Sửa bug `LEFT JOIN LATERAL ... ON TRUE AND em.status = 'active'` tại `attendance_policies.go:394`.
- [x] **3.3 ESS `TenureContext` Isolation:**
  - Scope `GetPendingDocuments`, `AcknowledgeDocument`, `GetEmployeeDocuments` (`repository/ess.go`) và `DistributeDocument` (`repository/documents.go`) theo `tenure_id` (và `file_acknowledgements.tenure_id`).
  - Scope `shift_swaps.go:283` ràng buộc `swap_date` nằm trong `[tenure.start_date, COALESCE(tenure.end_date, 'infinity')]`.
- [x] **3.4 Account Link/Unlink (`organization_rbac.go`):**
  - `LinkMemberEmployee`: gắn `user_id` duy nhất vào tenure đang mở (`end_date IS NULL`), không còn ghi `employees.user_id`.
  - `UnlinkMemberEmployee`: chỉ `SET user_id = NULL` trên tenure đang mở (`end_date IS NULL`) — **giữ nguyên `user_id` trên các tenure đã đóng để audit** — và **revoke HRM sessions/tokens** của user vừa gỡ (fail-closed).
- [x] **3.5 Tenure History Read API:** thêm `GET /api/v1/employees/{employee_id}/tenures`.

### Phase 4 — API Contract & Frontend Integration
- [x] **4.1 OpenAPI & Generated Types:** cập nhật `apiserver/openapi.yaml` (xóa `/employees/{id}/rehire`, cập nhật `POST /employees/{employee_id}/tenures`, thêm `GET /employees/{employee_id}/tenures`, `POST /employees/{employee_id}/tenures/{tenure_id}/close`, `POST /employees/{employee_id}/employments/{employment_id}/terminate`) và regenerate `webdashboard/src/types/api-generated.ts`.
- [x] **4.2 Frontend API Client & Modals:**
  - `webdashboard/src/lib/api/employees.ts`: thay `rehireEmployee` bằng `createEmployeeTenure`, `fetchEmployeeTenures`, `closeEmployeeTenure`, `terminateEmployeeEmployment`.
  - `webdashboard/src/components/features/employees/terminate-employee-modal.tsx`: gọi `POST /api/v1/employees/{employee_id}/employments/{employment_id}/terminate` kèm `termination_reason` và `rehire_eligible`.
  - `webdashboard/src/components/features/employees/rehire-employee-modal.tsx`: gọi `POST /api/v1/employees/{employee_id}/tenures` (hỗ trợ giữ mã cũ hoặc nhập mã mới, nhập `prior_service_credit_days`).
  - `webdashboard/src/components/features/employees/profile-tab.tsx` & `profile-sections.tsx`: hiển thị `TenureHistorySection` từ `GET /api/v1/employees/{employee_id}/tenures`.
  - Bổ sung i18n keys đồng bộ cho `en`, `vi`, `zh`.

### Phase 5 — Verification, Docs & Definition of Done
- [x] **5.1 Integration Tests (PostgreSQL + Redis):**
  - DB constraints: permanent cross-person code ban (kể cả sau khi Person A nghỉ việc), case/whitespace normalization (`" emp-01 "` vs `"EMP-01"`), temporal containment triggers (relationship ngoài tenure, segment ngoài relationship, thu hẹp tenure), multi-employer overlap trong 1 tenure.
  - Lifecycle: terminate 1/2 employer (tenure giữ mở) vs terminate employer cuối (tenure tự đóng + session bị revoke ngay); explicit close tenure; rehire giữ mã & rehire đổi mã (`hire_type = 'rehire'`, `prior_service_credit_days`); chặn rehire khi `rehire_eligible = false` hoặc `deleted_at IS NOT NULL`; future-dated hire/rehire (`planned`); concurrent double-rehire / locked payroll period rejection.
  - ESS & RBAC: unlink giữ nguyên `user_id` trên tenure đã đóng + revoke session; user mới link vào tenure 2 bị chặn direct-ID đọc payslip/attendance/leave/shift_swap/document của tenure 1.
- [x] **5.2 Full Automated Gate:** Go lint/race unit (`./scripts/_go-gates.sh --package ./...`), Go PostgreSQL integration (`./scripts/_go-gates.sh --integration --package ./...`), OpenAPI contract (`scripts/validate_openapi_contract.py`), frontend build/i18n/lint/vitest (`330/330` files, `2949/2949` tests) — all PASS.
- [x] **5.3 Disposable UI Audit (Playwright MCP):** 5 role (`employee`, `team_lead`, `shift_manager`, `hr_manager`, `admin`) × 4 desktop viewports (`1920×1080`, `1440×900`, `1366×768`, `1280×800`), bật `ff_ess=true` cho kịch bản ESS + HR lifecycle (create `EMP-V2-UI-01` → terminate via `/terminate` → rehire via `/tenures` đổi mã `EMP-V2-UI-02` + `prior_service_credit_days=167` → search mã cũ & mã mới) — all PASS, 0 console errors, 0 overflow.
- [x] **5.4 Documentation Sync:** cập nhật `docs/decisions/2026-11-12-employment-relationship-and-contract-boundaries.md`, `docs/operations/data-inventory.md` (phân loại `T3` cho `employee_tenures`), và đóng `docs/plans/employee-tenures.md`.

## Rule Compliance
- `database-rules.md §3–4`: transactional safety, PostgreSQL constraints, incremental + phased baseline parity.
- `access-control.md §1–3`: tenure-scoped ESS authorization, fail-closed session revocation, RBAC scoped permissions.
- `api-governance.md §1–2`: single canonical OpenAPI contract, UUID/date/code validation, `AppError`.
- `testing-standards.md §1–2`: full unit + PostgreSQL/Redis integration + Playwright MCP 5-role × 4-viewport verification.
- `data-governance.md`: PII masking in logs/audit, T3 classification for HR numbers/tenure metadata in `docs/operations/data-inventory.md`.
