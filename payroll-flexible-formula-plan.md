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
- [x] Cập nhật/tách ADR formula engine và lifecycle theo runtime Go/PostgreSQL thực tế; ghi rõ vertical slice, immutable versions, safe DSL boundary và migration/rollback. Historical Rust/template-DAG sections remain explicitly deferred.
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
- [x] Implement trạng thái `submitted/approved/locked`, accepted/rejected/warning row result và correction path. `draft` remains deferred; current row correction is blocked after batch lock.
- [x] Batch import atomic theo policy đã chọn; retry cùng idempotency key không tạo bản ghi mới.
- [x] Enforce quyền ở handler trước repository: production submit/own source, HR approve/lock, payroll calculate; default deny.
- **Verify:** API contract + integration tests cho success, malformed input, duplicate, wrong unit, unknown employment, closed/locked period, retry, partial rejection, unauthorized/forbidden và rollback.

### 4. Xây safe calculation model và rule evaluator

- [ ] Typed value/units và dependency graph deferred: phase-1 DSL is single-expression decimal evaluation with registry unit validation before persistence; no DAG/cycle surface exists yet.
- [x] Phase đầu chỉ hỗ trợ `+ - * /`, comparisons, `min`, `max`, `round`, `if`, `clamp`; evaluator enforces decimal arithmetic and resource limits. Rule-specific rounding policy remains deferred.
- [x] Tách KPI evaluator pure khỏi DB/HTTP/repository; batch input path validates before persistence.
- [x] Rule version immutable sau `published`; next draft version API tạo version mới, publish/retire không mutate published expression.
- [x] Trả calculation trace có safe metadata và audit events; không log raw salary/metric values/PII. Full trace masking review remains a release gate.
- [x] Verify evaluator arithmetic, comparisons, decimal operations, zero denominator, unknown/missing variables, invalid expression, safe functions, and resource limits. Cycle/property tests are not applicable until dependency DAG exists.

### 5. Tích hợp versioning, period lifecycle và snapshot

- [x] Resolve rule/policy theo `effective_from` và payroll period; KPI query selects latest published effective version and effective policy parameters. `effective_to` remains deferred.
- [x] Tạo `CalculationRun` với actor, period, formula version; lưu input/output snapshot, trace và payslip reference idempotently.
- [x] Gate calculate/approve/lock theo payroll period hiện có; locked period/input transitions return conflict and handlers enforce `payroll:calculate`, `payroll:approve`, and `payroll:manage`. Full calculator/approver segregation remains deferred.
- [x] Sau input batch lock/payroll period lock, direct input correction is rejected; correction history exists before lock and calculation runs are idempotent. Finalized payroll adjustment UI remains deferred.
- **Verify:** regression tests protect locked-input/effective published formula selection and bootstrap formula seed; historical calculation runs retain formula version references. Full live DB rerun rehearsal remains pending.

### 6. Hoàn thiện vertical slice `production KPI → KPI_BONUS`

- [x] Seed canonical definitions: production metrics, effective KPI policy parameters, and published `kpi.bonus` → `KPI_BONUS` formula. Fresh installs seed formula during standalone bootstrap; existing DBs use migration `20261114060000_payroll_kpi_formula_seed`.
- [x] Implement pure KPI rule: `output_score = min(actual/target, cap)`, weighted score, bonus cap và rounding; xử lý target bằng 0/missing input rõ ràng.
- [x] Import dữ liệu production qua JSON batch contract; CSV adapter để phase sau, nhưng giữ cùng contract.
- [x] Payroll result includes `KPI_BONUS` component and calculation trace endpoint/UI with input snapshot, formula version, result, and engine metadata. Intermediate KPI policy detail remains a follow-up UI enhancement.
- **Verify:** unit/repository fixtures cover employee metrics, missing formula, locked/rejected row filtering, effective formula selection, bootstrap seed, and KPI amount/snapshot inputs. Full live DB E2E remains pending.

### 7. API/UI cho HR configuration và preview

- [x] Thêm API list/version history/create rule/create draft version/preview/publish/retire; API preview không ghi payroll thật.
- [x] Thêm UI formula history, draft-version creation, preview/history states, calculation trace, validation/error/409 states, i18n en/vi/zh và a11y-focused tests. Registry builder and publish/retire controls remain follow-up.
- [ ] Không làm drag-and-drop ở phase đầu; UI không nhận raw code.
- [ ] States bắt buộc: loading, empty, validation error, server error/retry, conflict 409, published read-only, unauthorized/forbidden; i18n en/vi/zh và a11y.
- **Verify:** web tests cho preview/publish/version conflict; route × role matrix ít nhất `hr_manager`, `admin`, `employee`, `team_lead`, `shift_manager`; browser network/console không lộ salary/PII.

### 8. Mở rộng sau vertical slice và rollout control

- [ ] Chỉ sau khi KPI slice ổn định mới thêm `sales.revenue`, `attendance.absence_days`, `SALES_COMMISSION`, team allocation hoặc API source adapter.
- [ ] Mỗi extension phải thêm registry record + contract test; không sửa core evaluator nếu rule nằm trong DSL hiện có.
- [ ] Đặt feature flag cho user-facing formula/KPI flow; audit flag changes; chuẩn bị kill switch.
- [x] Cập nhật data inventory, API docs, ADRs, retention/access notes, audit events và row status documentation. Operator metrics/alerts and dedicated runbook remain rollout follow-up.
- **Verify:** compatibility suite với old salary/incentive APIs; load test bounded batch; failure/recovery test; rollout/rollback rehearsal.

### 9. Final verification — luôn chạy sau cùng

- [x] `./leedevkit test api --unit-only` cho vòng lặp nhanh, sau đó `./leedevkit test api` passed after payroll backend slices.
- [ ] Focused payroll UI tests passed. Full `./leedevkit test web` ran with 3008 passed and one pre-existing performance-budget failure; rerun required after budget issue is resolved.
- [ ] `./leedevkit test all` full gate; pending web performance-budget baseline fix and final payroll browser matrix.
- [ ] `git diff --check`; source-level migration up/down structure and API authorization checks passed. Runtime forward/down execution remains pending; security scan passed for backend dependencies.
- [ ] Browser acceptance matrix pending. Must use masked accounts from `docs/dev-accounts.md`; never write credential/token/raw PII to artifacts.

## Done when

- [x] Production can submit normalized KPI metrics without submitting payroll amounts.
- [x] HR can create/validate/publish a versioned KPI rule, create next draft versions, and preview its calculation.
- [x] Payroll calculates `KPI_BONUS` using locked inputs and effective published rule version.
- [ ] Old payroll results remain unchanged after new policy/rule publication.
- [x] Calculation result is explainable and auditable without exposing restricted data.
- [ ] Existing salary component, incentive policy, calculate and period-lock behavior remains compatible.
- [ ] API and role-based payroll acceptance pass. Web full gate remains blocked by performance-budget baseline; runtime DB migration rehearsal remains pending.

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
