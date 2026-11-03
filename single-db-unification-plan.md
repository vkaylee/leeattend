# Plan: Chuyển đổi toàn diện sang Single-Tenant & Hợp nhất Single Database (Mức 3)

**Status:** Phase 1 complete; Phase 2 pending
**Architecture Target:** 100% Single-Tenant Standalone Appliance (Zero-SaaS, Zero-Workspace Abstraction)
**Estimated Scope:** Database (140 tables) + Backend DAL (~50 repos) + API (~200 endpoints) + Frontend (120+ components/hooks) + Test Suites + Docs + Scripts

---

## 💎 Tuân thủ quy tắc kiến trúc (Rule Compliance)
- **`database-rules.md#3` (Transactions & Data Integrity):** Duy trì toàn vẹn dữ liệu, các ràng buộc UNIQUE và Foreign Keys được chuẩn hóa khi bỏ `workspace_id`. Tận dụng việc hợp nhất vào 1 DB để tạo trực tiếp Foreign Key giữa `employees.user_id` và `users.id`.
- **`database-rules.md#4` (Schema Migrations):** Đảm bảo migration gộp duy nhất có đầy đủ `up.sql` và `down.sql` rollback sạch sẽ (`DROP SCHEMA public CASCADE; CREATE SCHEMA public;`).
- **`coding-standards.md#Architecture` (Clean Code & Boy Scout):** Loại bỏ toàn bộ dead code, boilerplate `TenantDB` closure, authorization wrappers thừa thãi.
- **`change-management.md` (Documentation Synchronization):** Cập nhật tài liệu kiến trúc đồng thời tại mỗi phase (`docs/database-architecture.md`, `docs/standalone-deployment.md`, `docs/api-reference.md`).

---

## 🎯 Mục tiêu (Goal)
Loại bỏ hoàn toàn kiến trúc multi-tenant/workspace trong toàn bộ hệ thống (Database, Go Backend, REST API, React Frontend, Scripts, Docs), biến LeeAttend thành một ứng dụng doanh nghiệp On-Premise Single-Tenant thuần túy: 1 Database, 1 Schema, không còn bảng `workspaces`, không còn cột `workspace_id`, không còn path parameter `{ws_id}` trong API.

---

## 🔍 Phân tích các điểm then chốt dễ bị bỏ sót (Deep Audit Findings)
1. **Audit Logs Partitions (13 bảng partition):**
   - Bảng `audit_logs` có 12 partition tháng (`audit_logs_y2026m01`..`m12`) + `audit_logs_default`. Tất cả đều đang chứa cột `workspace_id NOT NULL` và index composite `(workspace_id, created_at DESC)`.
   - Bảng `system_audit_logs` (từ system DB cũ): Cần quyết định gộp vào `audit_logs` hay giữ riêng.
2. **Auth & JWT Claims (`auth.Claims`, `MemberContext`, `EmployeeContext`):**
   - `auth.Claims` trong `jwt.go` hiện chứa trường `WorkspaceID *uuid.UUID`.
   - Hàm `Login` hiện truy vấn qua bảng `workspace_members`. Khi bỏ `workspace_members`, cần xác thực role trực tiếp từ `user_roles`.
   - `httpinfra.WorkspaceIDFromContext` cần được dọn dẹp hoặc gỡ bỏ.
3. **Các bảng có tiền tố `workspace_`:**
   - `workspace_files` (lưu metadata tệp đính kèm)
   - `workspace_compliance_rules` (luật tuân thủ chấm công)
   - `workspace_config_entries` (cấu hình động)
   - *Quyết định kỹ thuật:* Giữ nguyên tên bảng `workspace_files` / `workspace_compliance_rules` hoặc đổi tên thành `files`, `compliance_rules` (đổi tên sẽ yêu cầu sửa hàng trăm câu truy vấn raw SQL trong các repositories). Khuyến nghị: Giữ tên bảng ở Phase 1, chỉ drop cột `workspace_id`, để giảm thiểu diff không cần thiết.
4. **Redis Cache Keys (`apiserver/internal/infra/redis/keys.go`):**
   - Các key builder: `KeyCacheWorkspace(wsID)`, `KeyCacheShifts(wsID)`, `KeyCacheAuthPermissions(wsID, userID)`. Cần chuyển sang dạng đơn giản không cần `wsID` (ví dụ: `cache:shifts`, `cache:auth:permissions:%s`).
5. **Scripts CI/CD & Security Invariant:**
   - `scripts/security_audit.sh`: Check 1 hiện đang ép mọi query repo vào bảng `employees` phải có `workspace_id`. Cần cập nhật script này, nếu không CI sẽ đánh fail.
   - `scripts/run_migration.sh`: Đang hardcode 2 phase `system/` và `tenant/`.
   - `scripts/run_performance_benchmarks.py`: Đang chứa hardcode path `/api/v1/workspaces/...`.
6. **Foreign Key liên bảng (Cross-Entity FKs - Cơ hội tối ưu):**
   - Trước đây do phân tách 2 DB, `employees.user_id` và `behalf_assignments.assignee_user_id` không thể có FK tham chiếu tới `users(id)`.
   - Khi gộp vào 1 DB, có thể thiết lập FK trực tiếp: `ALTER TABLE employees ADD CONSTRAINT fk_employees_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL`.
7. **Test Suites đồ sộ (Hơn 130 files test):**
   - Go: ~35 file `_test.go` trong `internal/api/` mock `req.SetPathValue("ws_id", ...)`.
   - React: ~100 file `.test.tsx` trong `webdashboard/` mock `localStorage.setItem("active_workspace_id", ...)`.
8. **File Storage S3 / Local Key Format:**
   - Kiểm tra đường dẫn lưu tệp trên S3/MinIO (`apiserver/internal/infra/external/s3.go` và `repository/files.go`).

---

## 🛡️ Ràng buộc & Giao ước hệ thống (Constraints & Interfaces)
- **Database Engine:** PostgreSQL 16+, 1 database duy nhất (`leeattend`), 1 schema duy nhất (`public`), 1 migration duy nhất (`apiserver/migrations/20261103000000_init`).
- **Data Access:** Loại bỏ hoàn toàn abstraction `TenantDB(ctx, wsID)` và `GetTenantTxPool`. Toàn bộ repository chỉ nhận trực tiếp `db DBTX` (pgxpool / pgx.Tx).
- **API Contract:** Mọi route có prefix `/api/v1/workspaces/{ws_id}/...` được rút gọn thành `/api/v1/...`.
- **UI State:** Bỏ `active_workspace_id` trong localStorage, bỏ workspace switcher dialog, bỏ context workspace.
- **Zero Breaking Runtime:** Sau mỗi phase, test suite tương ứng (`go test`, lint, typecheck) phải PASS 100%.

---

## 📋 Kế hoạch thực hiện theo 7 Giai đoạn chi tiết (Phased Execution)

### Phase 1: Hợp nhất Migration & Tinh giản Database Schema (Database Layer)
- [x] **Task 1.1:** Consolidate `system/` and `tenant/` baseline SQL into `apiserver/migrations/20261103000000_init/up.sql`, preserving system-first ordering and one migration history.
  - Workspace-compatible tables and columns remain temporarily because current Go auth/DAL/API contracts still require them.
  - Final removal is coupled to the Phase 2–5 consumer changes; no runtime-breaking SQL-only cutover is allowed.
- [x] **Task 1.2:** Audit schema objects, partitions, indexes, triggers, and FKs containing `workspace_id`; record the dependency boundary for later removal.
- [x] **Task 1.3:** Add destructive baseline `down.sql` with documented backup requirement.
- [x] **Task 1.4:** Update `docs/database-architecture.md` and `docs/standalone-deployment.md` to document one physical database and one baseline migration.
  - **Deferred by design:** dropping `workspace_id`, replacing workspace role tables, and adding organization-level tables require backend/API compatibility changes and are tracked in later phases.
- **Verification:**
  - Disposable PostgreSQL clean apply passed: 140 public tables, partitioned parents/children, functions, triggers, indexes, and FKs created.
  - Disposable rollback passed: `down.sql` removed application objects.
  - Disposable re-apply passed; `fn_set_updated_at=1`, `workspace_id_columns=99` (expected temporary compatibility state).
  - `./leedevkit test api` passed: Go unit, integration, format, vet, generated-diff, mod-verify, govulncheck, and staticcheck.

---

### Phase 2: Refactor Connection Pool, Config & Bootstrap (Backend Core)
- [x] **Task 2.1:** Removed `GetTenantTxPool(ctx, workspaceID)` and `WorkspaceID` from pool config. API and worker pools remain separate connections to one database.
- [x] **Task 2.2:** Removed `StandaloneWorkspaceID` and `StandaloneWorkspaceName`; added required `OrganizationName`.
- [x] **Task 2.3:** Removed workspace claims from JWTs and workspace identity fields from authorization contexts. Repository/API workspace compatibility remains for Phase 3–4.
- [x] **Task 2.4:** Bootstrap now generates or reads the single compatibility organization ID from the database; configuration no longer supplies it.
- [x] **Task 2.5:** Redis cache keys no longer include workspace namespaces.
- [x] **Task 2.6:** Updated standalone deployment environment examples and documentation to use `ORGANIZATION_NAME`.
- **Verification:**
  - `./scripts/_go-gates.sh --unit-only --package './internal/db/pool/... ./internal/auth/... ./internal/config/... ./internal/standalone/... ./internal/infra/redis/... ./internal/infra/http/...'` passed.
  - `./scripts/_go-gates.sh` passed: gofmt, vet, staticcheck, govulncheck, module verification, and full race tests.

---

### Phase 3: Refactor Data Access Layer (Repositories)
- [x] **Task 3.1:** Converted repository structs in `apiserver/internal/repository/` from `TenantDB` closures to direct `DB DBTX` fields. Updated main wiring, workspace authorization to use the shared DB, and repository tests.
- [ ] **Task 3.2:** Remove `workspaceID uuid.UUID` method parameters and `workspace_id` predicates from repository SQL. Retain schema columns only until API contracts are migrated.
- [ ] **Task 3.3:** Remove `WorkspaceAuthorizer` and workspace authorization dependencies after API handlers/routes are flattened in Phase 4.
- **Verification:** `./scripts/_go-gates.sh` passed: gofmt, vet, staticcheck, govulncheck, module verification, and full race tests.

---

### Phase 4: Refactor REST API Routing & Handlers (HTTP Layer)
- [x] **Task 4.1:** Added flat `/api/v1/...` route aliases for every existing `/api/v1/workspaces/{ws_id}/...` route. Flat aliases resolve the single organization from the shared database and inject compatibility `ws_id` for unchanged handlers.
- [ ] **Task 4.2:** Remove `ws_id` parsing and `Authorizer` dependencies from handlers after repository SQL and API contracts are fully migrated. Current handlers preserve compatibility for existing clients.
- [ ] **Task 4.3:** Update API tests to exercise flat routes directly and remove workspace-path fixtures.
- [ ] **Task 4.4:** Update `docs/api-reference.md` to make flat routes canonical.
- **Verification:** `./scripts/_go-gates.sh` passed: gofmt, vet, staticcheck, govulncheck, module verification, and full race tests.

---

### Phase 5: Refactor Frontend API Client & State (React Webdashboard)
- [ ] **Task 5.1:** Cập nhật API client trong `webdashboard/src/lib/api/`:
  - Bỏ tham số `wsId` trong tất cả các hàm gọi API (`employees.ts`, `departments.ts`, `attendance-core.ts`, `shifts.ts`, `leaves.ts`, `payroll.ts`,...).
  - Sửa URL endpoint template literal: `/api/v1/workspaces/${wsId}/...` → `/api/v1/...`.
- [ ] **Task 5.2:** Xóa bỏ Workspace state/context:
  - Xóa `getCurrentWorkspaceId()`, `localStorage.getItem("active_workspace_id")`.
  - Xóa component `create-workspace-dialog.tsx` và workspace selector trong layout navigation.
- [ ] **Task 5.3:** Cập nhật TypeScript types & Mock factories trong `webdashboard/`:
  - Loại bỏ trường `workspace_id` khỏi các data interfaces và test fixtures.
- [ ] **Task 5.4:** Cập nhật frontend test suite (`src/**/*.test.tsx`, `src/**/*.test.ts`).
- **Xác minh (Verify):**
  - Chạy `npm run typecheck` / `./scripts/_npm.sh test` trong webdashboard.

---

### Phase 6: Refactor Scripts & CI/CD Invariants
- [ ] **Task 6.1:** Cập nhật `scripts/security_audit.sh`:
  - Điều chỉnh Check 1: Thay thế kiểm tra `workspace_id` scoping bằng kiểm tra Single-Tenant Transaction & PII invariants.
- [ ] **Task 6.2:** Cập nhật script migration:
  - Sửa `scripts/run_migration.sh`, `scripts/migrate_legacy_data.py`: chạy duy nhất 1 thư mục migration `apiserver/migrations`.
- [ ] **Task 6.3:** Cập nhật các script test & benchmark:
  - `scripts/run_differential_tests.py`, `scripts/run_performance_benchmarks.py`.

---

### Phase 7: Tổng kết & Đồng bộ hóa toàn bộ Tài liệu (Final Verification & Docs Sync)
- [ ] **Task 7.1:** Kiểm tra toàn diện scripts deployment & compose:
  - `docker-compose.yml`, `.env.example`, `.env.test`.
- [ ] **Task 7.2:** Cập nhật file Rulebook dự án:
  - Cập nhật `.agent/rules/database-rules.md`: chuyển quy tắc Multi-Tenancy sang Single-Tenant Data Integrity.
  - Cập nhật `AGENTS.md` (đồng bộ context).
- [ ] **Task 7.3:** Chạy E2E & Smoke test toàn hệ thống:
  - `./leedevkit test api`
  - Build frontend React production bundle.

---

## 🎯 Tiêu chí hoàn thành (Done When)
1. Thư mục `apiserver/migrations/` chỉ chứa 1 migration khởi tạo duy nhất cho toàn bộ hệ thống.
2. Không còn bất kỳ bảng nào trong database chứa cột `workspace_id`.
3. Không còn bất kỳ route nào trong `apiserver` chứa `/workspaces/{ws_id}`.
4. Toàn bộ `go-gates`, unit test, integration test của Go API pass 100%.
5. Frontend React build và typecheck pass 100%.
6. Tất cả các script trong `scripts/` và tài liệu trong `docs/` được cập nhật đồng bộ.
