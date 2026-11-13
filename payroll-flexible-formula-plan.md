# Payroll Flexible Formula & KPI Foundation

Status: In progress — implementation authorized by user (2026-11-13)

## Goal

Xây nền payroll mở rộng theo mô hình `source metric → payroll input → versioned rule → pay component → payroll result`, bắt đầu bằng production KPI → `KPI_BONUS`, không cho chạy arbitrary code và không làm thay đổi kết quả payroll lịch sử.

## Current context

- Backend runtime hiện tại là Go `net/http` + PostgreSQL; các ADR cũ còn mô tả Rust, cần cập nhật trước khi dùng làm source of truth.
- Đã có salary components, payroll rules/tiers/assignments, incentive policies và payroll period `open/locked`.
- `routes_payroll.go` đã có các endpoint cho salary components, incentive policies, payroll rules, calculate và period lock/reopen.
- Không thay thế toàn bộ payroll engine hiện tại trong phase đầu; thêm lớp input/rule/snapshot theo vertical slice rồi migrate từng component.

## Constraints and interfaces

- **Rules:** `ARCH-01` ADR cho thay đổi kiến trúc; `API-01` contract-first/backward compatibility; `DB-01` migration có `down.sql`; `SEC-01` validate input + fail closed; `TEST-01` unit + integration/E2E; `DATA-01` salary/KPI result không lọt log/artifact.
- **Ownership:** production gửi metric; HR cấu hình policy/rule; payroll tính tiền; finance nhận kết quả/export. Production không gửi trực tiếp tiền thưởng.
- **Core contract:** `MetricDefinition`, `PayrollInput`, `PolicyParameter`, `PayComponent`, `CalculationRule`, `CalculationRun`, `CalculationTrace`.
- **Scope:** một organization hiện tại; mọi ID là UUID; employment là identity tính lương; mọi ngày hiệu lực dùng ngày/timezone của organization.
- **Safety:** decimal/fixed precision cho tiền; unit/type validation; dependency DAG; giới hạn expression depth/length/steps; không SQL, JavaScript, HTTP hoặc arbitrary code trong formula.
- **Lifecycle:** `draft → submitted → approved → locked`; rule `draft → approved → published → retired`; payroll snapshot giữ input/policy/rule/trace đã dùng.
- **Out of scope phase 1:** thuế/BHXH đầy đủ, live integration với ERP/production, team allocation phức tạp, drag-and-drop builder, arbitrary custom code, sửa trực tiếp payroll đã finalized.

## Tasks

### 1. Chốt domain vocabulary và cập nhật ADR

- [x] Inventory schema/API/domain hiện tại; map bảng cũ vào 6 khái niệm mới, ưu tiên reuse `salary_component_definitions`, `payroll_rule_definitions`, incentive policy và period thay vì tạo bản sao.
- [ ] Cập nhật/tách ADR formula engine và lifecycle theo runtime Go/PostgreSQL thực tế; ghi rõ component pipeline, immutable versions, provider boundary và migration/rollback. Deferred until calculation versioning lands.
- [x] Chốt code convention và canonical examples: `production.output`, `production.quality_rate`, `kpi.output_score`, `KPI_BONUS`.
- **Verify:** review schema hiện tại, route/handler/repository callers, ADR consistency; không còn quyết định Rust/schema workspace trái với runtime thực tế.

### 2. Thiết kế registry và migration tương thích

- [x] Thiết kế tối thiểu các entity: metric definitions, input records/staging, policy parameters, rule versions, calculation runs/traces; xác định entity nào mở rộng bảng hiện có. Phase slice adds metric definitions, input batches and inputs; rule/run tables remain next slice.
- [x] Mỗi metric/parameter/rule/component có type, unit, scope, source, effective dates, status, owner/audit metadata. Initial metric registry has type/unit/scope/source.
- [x] Thêm unique/index/foreign-key/overlap constraints cần thiết; chống duplicate theo source + period + employment + metric + idempotency key.
- [x] Viết `up.sql` và `down.sql`; dùng expand → backfill nếu cần; không drop/rename destructive.
- **Verify:** migration forward/down trên schema test; constraint, index, row-count và mixed-version compatibility tests.

### 3. Xây Payroll Input contract và staging flow

- [x] API/import contract nhận `period`, `employment_id`, `metric_code`, `value`, `unit`, `scope`, `source_reference`, `idempotency_key`.
- [x] Validate UUID, kỳ lương, employment tồn tại, unit/type, range, effective date, period status và duplicate; không query trong loop.
- [ ] Implement trạng thái `draft/submitted/approved/locked`, accepted/rejected/warning row result và correction path. Current slice persists `submitted`; approval/lock is next slice.
- [x] Batch import atomic theo policy đã chọn; retry cùng idempotency key không tạo bản ghi mới.
- [x] Enforce quyền ở handler trước repository: production submit/own source, HR approve/lock, payroll calculate; default deny.
- **Verify:** API contract + integration tests cho success, malformed input, duplicate, wrong unit, unknown employment, closed/locked period, retry, partial rejection, unauthorized/forbidden và rollback.

### 4. Xây safe calculation model và rule evaluator

- [ ] Định nghĩa typed value/units và dependency graph; reject unknown variable, type mismatch, unit mismatch, cycle, missing required input.
- [ ] Phase đầu chỉ hỗ trợ `+ - * /`, `min`, `max`, `round`, `if`, `clamp`; decimal rounding policy phải là dữ liệu của rule.
- [x] Tách KPI evaluator pure khỏi DB/HTTP/repository; batch input path validates before persistence.
- [ ] Rule version immutable sau `published`; publish tạo version mới, không mutate version đang được payslip tham chiếu.
- [ ] Trả calculation trace có masked/safe metadata, không log raw salary/PII.
- **Verify:** unit/self-check cho arithmetic, rounding, zero denominator, null/missing, cap/floor, invalid expression, cycle và resource limits; property tests nếu phù hợp.

### 5. Tích hợp versioning, period lifecycle và snapshot

- [ ] Resolve rule/policy theo `effective_from/effective_to` và payroll period; không dùng cấu hình hiện tại cho kỳ cũ.
- [ ] Tạo `CalculationRun` với actor, period, rule/policy version; lưu immutable input snapshot, trace và result reference.
- [ ] Gate calculate/approve/lock theo payroll period hiện có; conflict dùng optimistic version; calculator/approver separation theo permission đã có hoặc ghi rõ phase rollout.
- [ ] Sau period lock/finalize, cấm direct mutation; sai dữ liệu dùng correction/adjustment hoặc rerun có run mới.
- **Verify:** tính lại kỳ cũ sau khi publish rule mới vẫn ra kết quả cũ; concurrent lock/publish không overwrite; rollback transaction không để snapshot một phần.

### 6. Hoàn thiện vertical slice `production KPI → KPI_BONUS`

- [ ] Seed canonical definitions: `production.output`, `production.target_output`, `production.quality_rate`, `kpi.output_weight`, `kpi.quality_weight`, `kpi.target_bonus`, `KPI_BONUS`.
- [x] Implement pure KPI rule: `output_score = min(actual/target, cap)`, weighted score, bonus cap và rounding; xử lý target bằng 0/missing input rõ ràng.
- [x] Import dữ liệu production qua JSON batch contract; CSV adapter để phase sau, nhưng giữ cùng contract.
- [ ] Payroll result hiển thị component amount và explain trace: input, version, intermediate values, rounding, cap và final amount.
- **Verify:** fixture production gồm employee metric, missing row, duplicate row, team-scope rejection/explicit unsupported state; end-to-end run tạo đúng `KPI_BONUS` và snapshot.

### 7. API/UI cho HR configuration và preview

- [ ] Thêm API list/detail/create draft/validate/publish/retire rule; API preview với dữ liệu mẫu, không ghi payroll thật.
- [ ] Thêm UI registry/rule form tối thiểu: chọn input, operator/function, threshold/cap, effective date, preview, validation errors, version history.
- [ ] Không làm drag-and-drop ở phase đầu; UI không nhận raw code.
- [ ] States bắt buộc: loading, empty, validation error, server error/retry, conflict 409, published read-only, unauthorized/forbidden; i18n en/vi/zh và a11y.
- **Verify:** web tests cho preview/publish/version conflict; route × role matrix ít nhất `hr_manager`, `admin`, `employee`, `team_lead`, `shift_manager`; browser network/console không lộ salary/PII.

### 8. Mở rộng sau vertical slice và rollout control

- [ ] Chỉ sau khi KPI slice ổn định mới thêm `sales.revenue`, `attendance.absence_days`, `SALES_COMMISSION`, team allocation hoặc API source adapter.
- [ ] Mỗi extension phải thêm registry record + contract test; không sửa core evaluator nếu rule nằm trong DSL hiện có.
- [ ] Đặt feature flag cho user-facing formula/KPI flow; audit flag changes; chuẩn bị kill switch.
- [ ] Cập nhật data inventory, API docs, operator runbook, retention và metric/alert cho import failures, calculation failures, duration và rejected rows.
- **Verify:** compatibility suite với old salary/incentive APIs; load test bounded batch; failure/recovery test; rollout/rollback rehearsal.

### 9. Final verification — luôn chạy sau cùng

- [ ] `./leedevkit test api --unit-only` cho vòng lặp nhanh, sau đó bắt buộc `./leedevkit test api`.
- [ ] `./leedevkit test web --unit-only --pattern 'payroll|formula|kpi|salary'`, sau đó `./leedevkit test web`.
- [ ] `./leedevkit test all` full gate; đọc từng phase/log, không dựa vào dòng green cuối.
- [ ] `git diff --check`; migration forward/down; API authorization/contract checks; security scan phù hợp.
- [ ] Browser acceptance matrix theo role, masked test accounts từ `docs/dev-accounts.md`, không ghi credential/token/raw PII vào artifact.

## Done when

- [ ] Production can submit normalized KPI metrics without submitting payroll amounts.
- [ ] HR can create/validate/publish a versioned KPI rule and preview its calculation.
- [ ] Payroll calculates `KPI_BONUS` using approved inputs and effective rule version.
- [ ] Old payroll results remain unchanged after new policy/rule publication.
- [ ] Calculation result is explainable and auditable without exposing restricted data.
- [ ] Existing salary component, incentive policy, calculate and period-lock behavior remains compatible.
- [ ] API, DB integration, web and role-based acceptance tests pass.

## Decisions requiring approval before implementation

1. **MVP input source:** CSV staging first (recommended) or production API first?
2. **KPI scope:** employee-only in phase 1 (recommended), or employee + team allocation immediately?
3. **Post-lock correction:** adjustment in next period (recommended), or controlled rerun producing a new calculation run?
4. **Rule ownership:** HR manager drafts/publishes, or separate reviewer/admin approval required?
5. **Existing incentive model:** extend current incentive policy/rule tables, or introduce new versioned rule tables beside them during migration?

## Review focus

- Existing dirty worktree contains salary-management changes; implementation must not overwrite or mix unrelated files.
- Current `payroll_periods` supports only `open/locked`; richer lifecycle must be additive and migration-safe.
- `ADR-003`/`ADR-004` are proposed and partly inconsistent with current Go runtime; update source of truth before coding.
- Salary and payroll results are T2; traces, errors, logs, test fixtures and browser artifacts require masking.
- Rule evaluator is a financial calculation boundary; independent backend/security review is required before merge.
