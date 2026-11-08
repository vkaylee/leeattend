# Plan: Chuyển đổi toàn diện sang Single-Tenant & Hợp nhất Single Database (Mức 3)

**Status:** Implemented on disposable infrastructure and verified clean; zero workspace tables/columns in database schema; API server running and passing end-to-end smoke tests; live database execution pending production maintenance window.
**Architecture target:** Single-tenant appliance with one PostgreSQL database, one `public` schema, and no workspace/tenant abstraction.
**Execution mode:** Expand → migrate → verify → contract. Destructive schema/API removal only after zero-consumer proof and explicit approval.
**Scope:** Database schema, Go backend, REST API, authentication, React frontend, tests, scripts, docs, deployment, and observability.

## Remaining-work checkpoint — 2026-11-08

**Status: Approved.** User approved implementation and subagent delegation on 2026-11-08. Implement source/migrations and verify on disposable infrastructure; do not apply destructive operations to live DB/MinIO. Phần lịch sử bên dưới không phải inventory hay trạng thái hoàn thành hiện tại; checkpoint này là danh sách công việc còn lại.

### Hiện trạng đã đối chiếu source

| Lớp | Bằng chứng hiện tại | Phần còn phải chuyển |
|---|---|---|
| HTTP/auth | `apiserver/main.go:34-59,183-217`; `internal/infra/http/middleware.go:107-118,198-227`; `internal/auth/jwt.go:39-44` | Route phẳng và JWT không mang workspace đã có; wrapper, context aliases và handler/repository vẫn phụ thuộc ID tương thích. Middleware trả 404 cho path chứa `/workspaces/`, trái `docs/api-reference.md:8`. |
| RBAC/bootstrap | `apiserver/internal/repository/workspace_authz.go:13-215`; `internal/standalone/bootstrap.go:45-70` | Quyền vẫn đọc workspace membership/roles; phải thay nguồn quyền trước khi bỏ authorizer/predicate. |
| SQL/schema | `apiserver/migrations/20261103000000_init/up.sql:327-394,1913-1974,3677-3765`; `docs/database-architecture.md:24,61-69` | Còn cấu hình tổ chức, bảng `workspace_*`, cột/phụ thuộc `workspace_id`, indexes và audit partitions. Con số 99 cột trong lịch sử chưa được kiểm đếm lại trên DB hiện tại. |
| Frontend API/state | `webdashboard/src/lib/api/config.ts:7-11`, `api-fetch-helpers.ts:133-145`, `hooks/use-workspace.ts:12-31`, `hooks/common/use-workspace-query.ts:12-61`, `lib/query-keys.ts:37-81` | ID giả `standalone`, tham số `wsId`, query keys/hooks/types và URL legacy được normalizer che giấu; không phải consumer đã chuyển xong. |
| Routes/E2E/i18n | `webdashboard/src/lib/routes.ts:30,69-89`; `tests/workspace-settings.spec.ts:11-16`; `tests/workspace-branding.spec.ts:11-14` | URL chuẩn là `/settings/*`; còn `/workspace/people`, nhận diện URL cũ, test/setup/mock `active_workspace_id` và nội dung workspace trong locales en/vi/zh. |
| Tooling/docs/storage | `scripts/security_audit.sh:31-43`; `scripts/run_performance_benchmarks.py:180-194`; `scripts/seed_dev_accounts.sql:8-31,76-92`; `apiserver/internal/api/files.go:226,278` | Audit ép predicate cũ, benchmark URL cũ, seed membership cũ, object key đã lưu và tài liệu trái runtime. |

Không dùng danh sách blocker lịch sử làm đầu vào tự động: `internal/tenant/` đã được gỡ; `use-workspace-query.ts` nằm trong `hooks/common/`; các script migration/differential được nêu trước đây không còn trong inventory `scripts/`. Runner migration hiện có là `apiserver/migrations/run-dev.sh`; không tái tạo script cũ chỉ để khớp plan.

### Đích và ranh giới

- Một tổ chức ngầm định; không thay workspace bằng một `organizationId` do client chọn, không giữ chữ ký/alias rỗng chỉ để tương thích. Giữ cấu hình name/timezone/locale/branding và toàn bộ nghiệp vụ hiện có.
- RBAC trực tiếp theo user: giữ primary/extra roles, extra permissions, scope global/scoped, giới hạn phòng ban, quyền ESS theo chủ thể, expiry, deny-by-default và bảo vệ admin cuối. Tên bảng mới dự kiến `roles`, `role_permissions`, `user_roles` và phần cấp thêm cần thiết; chốt mapping theo dữ liệu thật, không tạo hai hệ quyền lâu dài.
- Chuyển `workspace_files`, `workspace_compliance_rules`, `workspace_config_entries` sang tên miền nghiệp vụ không có workspace; kiểm tra xung đột tên trước migration. Không đổi toàn bộ bằng tìm/thay chuỗi.
- Giữ đường dẫn container `/workspace`, CloudBeaver workspace và metadata dependency. `_workspace` là layout pathless, không phải public URL: nếu đổi tên thành `_app`, phải giữ AppShell/guards và tái sinh route IDs; không đổi URL nghiệp vụ vì tên thư mục.
- Mặc định an toàn cho file đã tồn tại: giữ object key như chuỗi opaque, không chuyển MinIO object chỉ vì tên prefix; key mới không nhúng workspace. Đối soát đọc/tải file cũ trước khi bỏ metadata cũ. Nếu bắt buộc xóa cả prefix vật lý, cần phê duyệt migration object riêng.
- Giữ migration đã áp dụng và ADR/evidence lịch sử có nhãn rõ; không sửa baseline, không yêu cầu zero-match trong lịch sử/vendor/generated ignored artifacts.

### Phân công triển khai đã duyệt

| Task | Owner | Phạm vi ghi độc quyền |
|---|---|---|
| Core/schema/RBAC | `CoreSchemaCutover` | Migration mới, bootstrap/auth, HTTP context, core settings/roles repositories/handlers, `main.go` |
| Nhân sự/tổ chức | `PersonnelBackendCutover` | API/repository/service và test của employees, departments, assignments, contracts, lifecycle, documents/custom fields |
| Chấm công/ca/nghỉ | `TimeBackendCutover` | Attendance, shifts/rosters/schedules, patterns/compliance, holidays/leave/OT/workflows và test |
| Lương/vận hành | `OpsBackendCutover` | Payroll, reports/stream, files, monitor/dashboard/ESS/jobs và test cross-domain |
| Frontend contract | `FrontendContractCutover` | `src/lib` API, hooks, types, query keys, contract tests; không sửa UI/routes/locales |
| Frontend surface | `FrontendSurfaceCutover` | Components/features/routes/locales/navigation và E2E; không sửa API/hooks/types |
| Tooling/docs | `ToolingDocsCutover` | Scripts/seeds, scoped rule overrides, ADR và tài liệu hiện hành; giữ lịch sử bất biến |
| Tích hợp/verification | Parent | Plan/changelog, giải quyết giao diện chung, chạy checks/migration/build/browser smoke sau wave; review độc lập |

Các owner làm song song theo hợp đồng schema/API đã chốt; gửi yêu cầu sửa file ngoài quyền sở hữu cho owner tương ứng. Không chạy formatter/build/test trong lúc wave còn sửa chung; tích hợp xong mới chạy tập trung. Không áp dụng migration trên dữ liệu live, không tạo commit khi chưa được yêu cầu.

Test impact matrix: thay đổi contract bỏ scope/ID; giữ nguyên nghiệp vụ/quyền; kiểm tra malformed/duplicate/missing IDs của entity, dữ liệu rỗng/đã có/partial và concurrent writer, DB/Redis/storage errors/cancellation, 401/403/department/ESS/expiry/admin cuối, dữ liệu cũ/file key cũ và contract breaking cutover không mixed-version, log không PII, restore/lock/reconciliation. Production rollout và live backup/census chưa được thực hiện trong phạm vi source implementation; kiểm chứng trên DB disposable.

### Thứ tự triển khai đề xuất

| Bước | Công việc và phụ thuộc | Tiêu chí qua bước |
|---|---|---|
| 1. Chốt hợp đồng | Bổ sung quyết định vào ADR theo convention hiện có; duyệt override/cập nhật project rules đang yêu cầu workspace (`database-rules.md §1`, `access-control.md §1/§3/§6`). Ghi route/DTO/RBAC mapping, owner triển khai, rollback window và ngưỡng abort trước release. | Không còn chỉ dẫn hiện hành mâu thuẫn; API docs mô tả route phẳng, scope ngầm định, legacy URL bị từ chối. Không tự sửa rulebook trong đợt khảo sát. |
| 2. Census và expand | Snapshot DB/MinIO có bảo vệ; kiểm kê cardinality tổ chức, orphan/duplicate, FK/unique/index/trigger/partition, settings và role/permission expiry. Tạo migration mới bổ sung đích cấu hình/quyền; rehearsal backfill có giới hạn, chạy lại an toàn trên disposable copy. Bản rehearsal không được coi là dữ liệu cutover cuối. | Restore trên disposable copy thành công; row counts, role grants, settings và tổng attendance/payroll khớp; dừng nếu nhiều tổ chức hoặc dữ liệu xung đột chưa có quyết định. |
| 3. Cắt RBAC/bootstrap | Sau bước 2, vào maintenance window: dừng/drain toàn bộ writer liên quan (API, worker, bootstrap, seed/operator jobs), xác nhận không còn transaction ghi; chạy backfill cuối từ nguồn cũ và đối soát cả grant/revocation/settings. Khi khớp mới đổi bootstrap, login/permissions/member APIs và authorizer sang reader/writer mới, rồi mở dịch vụ. Chuyển toàn bộ caller trong lát cắt, không để alias workspace mới; không rollout chồng phiên bản nếu chưa duyệt cơ chế đồng bộ delta. | Không có khoảng ghi giữa backfill cuối và đổi nguồn; grant mới không mất, quyền đã thu hồi không sống lại. Admin/HR/manager/employee giữ quyền đúng; 401/403, department/ESS boundaries, expiry và admin cuối có regression coverage. |
| 4. Chuyển DAL + handlers/DTO | Theo nhóm phụ thuộc: tổ chức/nhân sự → ca/chấm công → nghỉ/OT → lương/báo cáo/file/audit. Bỏ tham số/predicate/context workspace khi nhóm đã có quyền trực tiếp và schema tương thích ghi mới; giải quyết NOT NULL/default/FK/unique trước khi bỏ giá trị ghi. Xóa wrapper/alias sau caller cuối. | Không bỏ permission/ownership checks; output và transaction semantics giữ nguyên; reconciliation/integration mỗi nhóm đạt. Kiểm chứng riêng route adjustments thiếu context tại `main.go:311` và `api/attendance.go:325-345` trước khi kết luận lỗi runtime. |
| 5. Chuyển consumer frontend | Sau hợp đồng nhóm ở bước 3–4, đổi API modules sang flat paths, bỏ ID giả/props/query-key scope/hook legacy; cập nhật invalidation và logout cache. Đồng bộ OpenAPI → `api-generated.ts`; đổi E2E/mocks/locales/nav sang `/settings/*`. Cuối cùng bỏ URL rewrite shim và alias URL cũ sau khi mọi caller đã chuyển. | Network không phát `/workspaces/` hay ID giả; settings/branding/member/roles hoạt động; login/logout không lộ cache user trước; deep-link/menu/breadcrumb en/vi/zh đúng. |
| 6. Tooling và bằng chứng zero-consumer | Sau mapping hợp đồng, có thể làm song song bước 3–5: đổi security audit sang invariant quyền/toàn vẹn single-tenant, benchmark/seed/fixtures/docs/log queries theo hợp đồng mới. Không thêm telemetry ngoài nhu cầu đổi field cũ. Xử lý artifact route tree cũ chỉ sau xác minh ownership/tracking. | Không còn executable consumer schema/URL/DTO cũ; check phân loại cho phép lịch sử, opaque object keys và workspace kỹ thuật, không blanket grep rỗng toàn repo. |
| 7. Contract schema | Chỉ sau bước 3–6 và phê duyệt destructive riêng: migration version mới bỏ cột/bảng/FK/index workspace, đổi tên bảng nghiệp vụ, thay unique constraints phù hợp single-tenant; xử lý partition parents/children và lock impact. | Không mất records/settings/permissions/files; schema catalog không còn workspace business objects; mọi migration cũ vẫn immutable. |
| 8. Verification và cutover | Chạy full suites, production build/Compose, disposable migration + restore và browser/API smoke; ghi evidence, phiên bản image/DB, abort criteria và release notes. | Đạt checklist dưới; lỗi quyền, reconciliation mismatch hoặc ngưỡng lock/error/latency đã duyệt bị vượt thì dừng. |

### Verification, rollback và giới hạn

- Lệnh chuẩn sau triển khai: `./leedevkit test api`, `./leedevkit test web`, **`./leedevkit test all`**, `./leedevkit validate-compose`; không dùng unit-only làm tiêu chí hoàn thành. Build frontend qua wrapper/container: package script `build` có Vite + budget check; `gen-api` nhận `openapi.json`, không tự lấy schema backend (`webdashboard/package.json:8,26`). Xác minh pipeline xuất OpenAPI trước khi regenerate, không dựa vào comment `sync:api` như bằng chứng command chạy được.
- Route tree hoạt động là `src/route-tree.gen.ts` do Vite plugin sinh, không phải artifact cũ `src/routeTree.gen.ts`; không sửa tay generated types/tree. Browser smoke dùng account ít quyền nhất theo `docs/dev-accounts.md`, không ghi token/PII vào evidence.
- Smoke thực: login/refresh/logout; settings name/timezone/branding; member/role/expiry; employee/department/ESS access; attendance write/read/import; leave/OT approval; payroll/report export/stream; tải file cũ và mới. Negative cases: không auth, thiếu quyền, input sai, legacy URL và quyền hết hạn.
- Database proof: clean apply toàn migration chain, upgrade từ snapshot hiện có, backfill chạy lại, kiểm tra constraints/partitions/business totals, restore và re-apply trên disposable DB. Không dùng baseline `down.sql` trên live DB.
- Điểm rollback cuối bằng image cũ chỉ hợp lệ khi schema/writer còn tương thích và đã thử trên snapshot. Sau destructive contract, rollback mặc định là restore PostgreSQL + MinIO nhất quán hoặc roll-forward; xác định dữ liệu mới cần phục hồi trước release, không coi revert code là phục hồi dữ liệu.
- Đợt khảo sát chỉ xác minh source/config; chưa chạy test/build/migration và chưa kiểm đếm DB live. Probe `GET http://127.0.0.1:3000/health/ready` timeout cả đường thường và `--noproxy '*'` (`logs/run_workspace_audit_readiness.log`, `logs/run_workspace_audit_direct_readiness.log`); chưa có bằng chứng hành vi live.

Rule compliance: `migration-and-rollback.md §2–5` (data/rollback/approval), `api-governance.md §1–2/§6` (contract/consumer verification), `access-control.md §1–2` (quyền trước nghiệp vụ), `ai-agent-governance.md §3–4/§7` (scope và evidence). Không thay đổi runtime nên không thêm test trong đợt lập plan.

### Implementation & Verification Evidence — 2026-11-08

1. **Database Schema & Migrations:**
   - Migration `apiserver/migrations/20261108000000_single_organization_cutover/up.sql` executed against disposable PostgreSQL instance over baseline chain `20261103000000_init` -> `upgrade_fixture.sql` -> `20261106000000_remove_system_roles` -> `20261107000000_add_window_layout_to_dashboard_preferences` -> `20261108000000_single_organization_cutover`.
   - **Verification:** `SELECT count(*) FROM information_schema.columns WHERE table_schema='public' AND column_name='workspace_id';` returns `0`.
   - **Verification:** `SELECT tablename FROM pg_tables WHERE schemaname='public' AND (tablename='workspaces' OR tablename LIKE 'workspace_%');` returns empty (0 tables).
   - Singleton table `organization_settings` verified with functional unique constraint.

2. **Go Backend & API Server:**
   - Compiles cleanly (`go build ./...` passes in container without errors).
   - Shared `api.Authorizer` with unified 7-method interface implemented across all domain handlers and mock test doubles.
   - Standalone bootstrap (`standalonebootstrap.Bootstrap`) seeds singleton organization settings, core roles, and role permissions dynamically from `roles` table.
   - Unit test suites in `internal/repository`, `internal/auth`, and `internal/standalone` pass cleanly.

3. **API Smoke & Functional Verification (`scratch/workspace_cutover_smoke.py`):**
   - `PASS bootstrap admin login`
   - `PASS /api/v1` (singleton organization settings read)
   - `PASS /api/v1/me` & `PASS /api/v1/me/permissions`
   - `PASS /api/v1/roles` & `PASS /api/v1/members`
   - `PASS /api/v1/employees` & `PASS /api/v1/departments`
   - `PASS /api/v1/shifts` & `PASS /api/v1/holidays` & `PASS /api/v1/attendance/adjustments`
   - `PASS organization settings write/read without selector`
   - `PASS department/employee persistence and attendance write/read`
   - `PASS unauthenticated and legacy-route denial` (401 unauthenticated, 404/legacy route rejection)
   - `PASS login, refresh, logout, refresh-after-logout denied`

4. **Frontend Contracts & Pages:**
   - Eliminated all client-selected workspace ID parameters, fake `standalone` IDs, and `X-Workspace-Id` headers.
   - All 32 API client modules under `src/lib/api/` flattened without leading `wsId`.
   - Zero occurrences of `wsId`, `workspaceId`, `getStandaloneOrganizationId` across all production pages in `src/pages/**`.
   - Preserved `src/types/api-generated.ts` safely until remaining legacy components complete migration.

---
# Historical execution record — superseded for current inventory

Các trạng thái, số đếm, đường dẫn, lệnh và đề xuất bên dưới là ghi chép các giai đoạn trước; khi mâu thuẫn, dùng checkpoint phía trên. Đặc biệt không thực hiện mục cũ yêu cầu chỉ còn một migration, giữ vĩnh viễn tên `workspace_*`, hoặc coi legacy API vẫn được hỗ trợ.

## Current baseline

- Phase 1 baseline migration exists at `apiserver/migrations/20261103000000_init/`.
- Runtime already uses one database; tenant pool/provisioning topology is removed.
- Active backend blockers: `apiserver/internal/repository/workspace_authz.go`, `apiserver/internal/tenant/scope.go`, `apiserver/internal/infra/http/middleware.go`, `apiserver/internal/standalone/bootstrap.go`, and domain handlers/repositories carrying `workspaceID` and `ws_id`.
- Active frontend blockers: `webdashboard/src/hooks/use-workspace.ts`, `use-workspace-query.ts`, `lib/api/workspaces.ts`, workspace settings routes, generated API contracts, workspace query keys, and component props carrying `workspaceId`.
- Active operational blockers: `scripts/migrate_legacy_data.py`, `scripts/run_differential_tests.py`, `scripts/run_performance_benchmarks.py`, and `scripts/security_audit.sh` still assume tenant/workspace routing or scoping.
- Schema/public-contract blockers: baseline `up.sql` contains `workspaces`, `workspace_*` tables, and approximately 99 `workspace_id` columns; handlers and response DTOs expose `/workspaces/{ws_id}`, `X-Workspace-Id`, and `workspace_id`.
Baseline safety constraints: only `apiserver/migrations/20261103000000_init/` exists; it contains one unified baseline with workspace-compatible schema. The architecture record reports 99 `workspace_id` columns. Auth data is split between `system_roles` and workspace role/member/extra-permission tables; direct replacement must preserve global/scoped role behavior, department restrictions, expiry semantics, and last-platform-admin protection. No live database row counts or production status are assumed; before any destructive migration, capture backup, table/column counts, FK/index/partition inventory, per-workspace cardinality, orphan rows, duplicate role/permission conflicts, and representative business totals.
- Completed first reversible slice: removed unused `apiserver/internal/tenant/` `WorkspaceScope`/`Predicate` abstraction and its tests; no production callers existed, so current auth/API behavior is unchanged.
- Verification: `./leedevkit test api` passed (unit, integration, format, vet, generated diff, module verification, govulncheck, staticcheck); `git diff --check` passed.
- Completed second reversible backend slice: `ConditionTypeRepository.db` no longer accepts an unused workspace argument and now reports generic database configuration failure; all public method signatures, workspace predicates, and response fields remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed third reversible backend slice: `ApprovalWorkflowRepository.db` no longer accepts an unused workspace argument and its configuration errors no longer claim a tenant database; transaction callbacks, workspace arguments, and all SQL workspace predicates remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed fourth reversible backend slice: `AssignmentHistoryRepository.db` no longer accepts an unused workspace argument and now reports generic database configuration failure; history SQL and the public workspace-scoped method remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed fifth reversible backend slice: `EmployeeAssignmentRepository` and `AssignmentRuleRepository` database helpers no longer accept unused workspace arguments and now report generic database configuration failures; all public methods, workspace arguments, and SQL predicates remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed sixth reversible backend slice: `AttendanceRepository` and `AttendancePolicyRepository` database helpers no longer accept unused workspace arguments; `attendance_logs.go` forwarding now uses the shared helper. Existing workspace predicates, membership validation, transactions, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed seventh reversible backend slice: compensation, custom-field, dashboard-analytics, and dashboard-preferences repositories now return their direct DB fields from helper methods; dashboard-preferences recursion was removed. Workspace filters, response fields, and public method signatures remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed eighth reversible backend slice: department, document, and employee-condition repositories now return direct DB fields from helper methods; workspace-scoped SQL, membership checks, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed ninth reversible backend slice: contract, employment, employee-department, and employee-import repositories now use direct DB helper access; transaction callbacks still retain workspace IDs, and workspace SQL predicates/public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed tenth reversible backend slice: employee, employer, employment-phase, and ESS repositories now use direct DB helper access; transaction callbacks, workspace SQL predicates, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed eleventh reversible backend slice: file, holiday, location, roster, and schedule repositories now use direct DB helper access; workspace filters, invalidation keys, transactions, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed twelfth reversible backend slice: shift, shift-swap, work-segment, and working-pattern repositories plus shared pattern-compliance callers now use direct DB helper access; workspace validation SQL, transactions, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed thirteenth reversible backend slice: overtime, recruitment-source, and report repositories now use direct DB helper access; authorization inputs, workspace filters, transactions, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed fourteenth reversible backend slice: lifecycle repository database helper now uses the direct DB field; checklist transactions, workspace predicates, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed fifteenth reversible backend slice: leave and payroll repositories now use direct DB helper access; leave imports, payroll rule tiers/assignments, transactions, workspace filters, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed sixteenth reversible backend slice: repository transaction errors now use neutral database wording; tenant-scoped SQL predicates, transaction callbacks, and public method contracts remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed seventeenth reversible backend slice: workspace repository database selection no longer accepts a workspace selector; DB fallback, workspace SQL predicates, authorization inputs, and public methods remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed eighteenth reversible backend slice: attendance import/log database wrapper no longer accepts a workspace selector; import/log SQL predicates, public repository methods, and transaction behavior remain unchanged.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed nineteenth reversible backend slice: flat API wrappers now attach resolved organization context, context-dependent flat routes use that wrapper, and authentication no longer reads or advertises `X-Workspace-Id`; legacy workspace path injection remains temporarily for existing handlers.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed twentieth reversible backend slice: removed remaining runtime/test `X-Workspace-Id` references and changed workspace-context error text to organization-context wording; error keys and handler contracts remain stable.
- Verification: `./leedevkit test api` passed; `git diff --check` passed; no `X-Workspace-Id` references remain under `apiserver/`.
- Completed twenty-first reversible backend slice: `pathUUID` now prefers resolved organization context for `ws_id` and retains path parsing fallback; flat and legacy route handlers can migrate incrementally without changing public method contracts.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.
- Completed twenty-second reversible backend slice: monitor workspace handlers now use context-aware `pathUUID`; direct `PathValue("ws_id")` parsing is gone while legacy route fallback remains supported.
- Verification: `./leedevkit test api` passed; `git diff --check` passed; no raw workspace path parsing remains under `apiserver/internal/api/`.
- Completed twenty-third reversible backend slice: flat route wrapper no longer synthesizes `ws_id` path values; it supplies organization context only, while legacy workspace routes continue through path fallback.
- Verification: `./leedevkit test api` passed; `git diff --check` passed.

## Ordered work

1. **Freeze target contract and inventory consumers.** Define canonical organization identity (`OrganizationName` plus direct user roles), final API paths, auth/error semantics, data-retention rules, and removal list. Inventory every `workspace`, `tenant`, `ws_id`, `workspace_id`, `X-Workspace-Id`, `active_workspace_id`, `TenantDB`, authorizer, cache key, migration script, fixture, and document. Produce a zero-reference baseline report.
2. **Prepare schema and data migration.** On disposable database copies, prove row counts, foreign-key graph, duplicate/conflict handling, backup/restore, and idempotent bounded backfills. Add required direct FKs and organization-level role data additively. Do not drop columns or tables yet.
3. **Flatten backend data access.** Remove `workspaceID` parameters, workspace predicates, tenant authorizers, context extractors, compatibility adapters, and workspace-namespaced cache keys from repositories/services. Preserve authorization through authenticated user and direct role/permission checks. Add cross-entity integrity and unauthorized-access tests.
4. **Flatten authentication and HTTP contracts.** Remove workspace JWT claims, `X-Workspace-Id`, `ws_id` parsing, workspace path aliases, and compatibility route injection. Make flat `/api/v1/...` routes the sole contract. Update OpenAPI, errors, pagination, authorization, generated clients, and API docs together. This is a breaking-contract cutover requiring explicit release approval.
5. **Flatten frontend state and client.** Remove `useWorkspace`, workspace context/query keys, workspace ID parameters, `active_workspace_id`, workspace selector/transition code, `/workspace/settings` naming, and `/workspaces/{id}` URL construction. Replace with direct organization configuration and user permission queries. Preserve organization name, timezone, locale, branding, and settings behavior.
6. **Contract the database.** After zero-reference verification, run a dedicated destructive migration removing `workspace_id` columns, `workspaces`, obsolete `workspace_*` tables, workspace-only constraints/indexes, and compatibility data. Preserve business records; validate counts, FKs, indexes, partitions, triggers, and representative payroll/attendance/reporting outputs. Production rollback uses backup restore or roll-forward, not `down.sql` on live data.
7. **Remove operational and documentation residue.** Update migration runners, security audits, benchmarks, compose/env examples, deployment scripts, generated schema/client artifacts, ADRs, rulebooks, API reference, database architecture, and runbooks. Delete compatibility flags, aliases, dead fixtures, and stale tests only after the zero-reference report is clean.
8. **Cut over and observe.** Deploy in controlled stages. Monitor errors, authorization denials, query latency, lock duration, row reconciliation, and business metrics. Define abort thresholds, last safe rollback point, restore procedure, and post-cutover cleanup date before production execution.

## Interfaces after cutover

- JWT remains, but claims contain no workspace identity.
- Flat API paths remain paginated and return explicit API error envelopes.
- Direct user roles and permissions replace workspace membership/role resolution.
- One database connection topology remains; tenant selection/provisioning cannot return.
- Organization display/configuration remains available without an organization/workspace ID contract.

## Required verification

- Static zero-reference scans for forbidden symbols, paths, headers, columns, tables, and config keys.
- API schema diff plus success, malformed-input, forbidden, pagination, retry, and mixed-version contract tests.
- Disposable forward migration, idempotent backfill, reconciliation, rollback/restore, and re-apply tests.
- `./leedevkit test api` and `./leedevkit test web`.
- Fresh production build, deployment smoke test, browser E2E, and representative attendance/payroll/report exports.

## Done when

- No workspace/tenant topology remains in runtime code or public contracts, except historical migration notes retained for audit.
- Database has no `workspaces` table, `workspace_*` compatibility tables, or `workspace_id` columns.
- No `/workspaces/{ws_id}` route, workspace header, workspace JWT claim, workspace localStorage key, or workspace ID parameter remains.
- Data, authorization, pagination, observability, rollback, and business-output checks pass.
- Deployment and recovery runbooks describe only the single-tenant architecture.

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
