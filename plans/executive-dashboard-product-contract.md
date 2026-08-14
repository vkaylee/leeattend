# LeeAttend Executive HR Dashboard — Product Contract

**Status:** Design/architecture contract — audit and roadmap only  
**Date:** 2026-08-12  
**Scope:** Executive HR Dashboard for HR Director, HRBP, HR Operations, Department Manager, and Recruitment Manager  
**Implementation constraint:** This phase does not modify application source code, migrations, OpenAPI, generated types, or permissions.

## 1. Contract intent

LeeAttend already has a useful HR Operations dashboard: workspace statistics, attendance trend, department attendance, unexcused absences, overtime trend, peak times, leave types, lateness by weekday, recruitment sources, turnover analytics, pending lifecycle tasks, and dashboard layout preferences. The existing experience is operational and widget-oriented. It is not yet an executive decision surface because metrics use different implicit periods, several rates lack explicit denominators, sensitive data is not separately gated, and several requested measures have no backend source.

This contract defines the semantic, security, API, accessibility, and delivery baseline for upgrading the dashboard from charts to **insight → risk → action**.

### 1.1 Non-negotiable principles

1. **One reporting context:** every executive response is evaluated for one workspace, one workspace timezone, one explicit period, one comparison basis, and one filter scope.
2. **Metric before visualization:** a KPI is not releasable without a named definition, numerator, denominator, unit, null policy, data-quality status, and freshness timestamp.
3. **No fabricated data:** unavailable backend data is returned as `data_gap`, not zero and not synthetic data.
4. **Workspace isolation:** every API is workspace-scoped and authorizes before service execution.
5. **Insight is actionable:** each risk must include severity, evidence, affected dimension (aggregated by default), and a permitted next action.
6. **Privacy by default:** executive overview is aggregate-first; employee-level attendance/absence, payroll, and salary require explicit permission and auditability.
7. **Contract first:** API/OpenAPI and metric definitions are finalized before frontend implementation.

## 2. Current-state audit

### 2.1 Existing dashboard surface

| Area | Current implementation | Evidence | Executive implication |
|---|---|---|---|
| Page shell | `DashboardPage` renders pending tasks, HR/system tabs, chart catalog, layout customization, and resilient error handling. | `webdashboard/src/pages/dashboard-page.tsx:30-185` | Good shell to extend, but HR and system concerns should be separated for executive audiences. |
| API router | Dashboard routes include workspace stats, attendance trend, department attendance, unexcused absences, overtime trend, peak times, leave types, lateness, recruitment sources, turnover, and preferences. | `apiserver/src/routes/dashboard.rs:7-51` | Reuse operational endpoints where semantics remain valid; do not overload them with executive semantics. |
| Query layer | Independent React Query hooks with workspace guard and Zod payload validation. | `webdashboard/src/hooks/use-dashboard.ts:81-205`; `dashboard-schemas.ts:6-102` | Strong typed boundary, but no shared executive period/filter query context. |
| Visual layer | Widget renderer, chart catalog/customizer, chart-specific loading/error/empty states, and tests. | `webdashboard/src/hooks/use-dashboard-charts.tsx:6-85`; dashboard component directory | Keep useful charts; add semantic KPI, table, accessible narrative, and action affordances. |
| Permissions | Dashboard analytics handlers use `DashboardRead`; preferences use `DashboardPreferences`. | `apiserver/src/handlers/workspaces/crud.rs:35,66,103,155,177,201,225,249`; `dashboard_preferences.rs:228-285` | Baseline exists, but sensitive sub-metrics and scoped manager access need finer gates. |
| Timezone | Backend commonly resolves workspace timezone; some frontend date-only filters/formatters derive dates in browser timezone. | `apiserver/src/handlers/workspaces/crud.rs:106-125`; `webdashboard/src/components/features/dashboard/unexcused-absences-chart.tsx:24-25,110-126`; `turnover-analytics-chart.tsx:33-55`; `use-datetime-utils.ts:82-89` | Executive period boundaries can disagree across users; workspace timezone must be authoritative end-to-end. |

### 2.2 Existing metric inventory and limitations

- **Headcount:** current active distinct employee count valid today. It is not yet a period-aware workforce snapshot, FTE measure, or employment-type split.
- **Attendance:** present/late/absent/on-leave counts and 30-day trend exist. Current API does not expose an executive attendance-rate numerator/denominator contract.
- **Absence:** absent and unexcused absence counts exist. Absence rate, approved/unapproved duration, and reason-quality dimensions do not exist as an executive contract.
- **Late:** late count and average late minutes by weekday exist. A period-aware late-rate denominator and policy/grace semantics are not explicit in the API.
- **Turnover:** terminated counts, tenure buckets, source, and department breakdown exist. Turnover rate, annualization, voluntary/involuntary classification, and retention/cohort retention do not exist.
- **Overtime:** 30-day system-detected and approved hours exist. Overtime cost and budget relationship do not exist in the dashboard contract.
- **Payroll:** `estimated_monthly_payroll` is salary-component-derived, not actual payroll run cost, budget, or variance. It is confidential and currently appears in a DashboardRead-oriented surface.
- **Pending actions:** current-user lifecycle tasks and pending leave count exist. There is no consolidated workspace-level actionable queue for HR approvals, attendance exceptions, overtime, payroll exceptions, signatures, or stale data.
- **Comparison:** existing widgets do not share one comparison model; defaults mix today, current month, last 30 days, and user-selected turnover periods.

## 3. Users, decisions, and visibility

All roles are workspace-scoped. A user may have more than one role; effective permissions are the union of explicit grants, subject to scope constraints and default deny.

| User group | Primary decisions | Default view | Sensitive access |
|---|---|---|---|
| **HR Director** | Workforce health, cost/risk trade-offs, retention, strategic actions. | Workspace aggregate executive overview; all non-PII KPIs, risk register, trend and comparison. | Payroll/salary and employee-level absence only with explicit `PayrollRead`/`CompensationRead` and `EmployeeAbsenceRead`-equivalent permission. |
| **HRBP** | Department/portfolio risks, manager follow-up, retention and absence interventions. | Workspace aggregate plus permitted department scope. | Same gates; employee-level only for assigned scope and explicit permission. |
| **HR Operations** | Data quality, attendance corrections, approvals, operational backlog. | Operational dashboard and executive drill-down where permitted. | Attendance details and actions according to existing attendance permissions; no salary by default. |
| **Department Manager** | Team attendance, lateness, absence risks, pending approvals. | Department-scoped aggregate only; no cross-workspace visibility. | Employee-level attendance/absence only when explicitly granted and within managed department scope; no payroll/salary by default. |
| **Recruitment Manager** | Hiring pipeline/source quality, turnover signals, workforce movement. | Recruitment and workforce movement aggregates. | No salary; employee-level absence not needed by default. |

### 3.1 Role-to-permission contract

Minimum permissions to define or confirm in the permission catalog:

- `dashboard:read` — aggregate executive and operational dashboard data.
- `dashboard:preferences` — read/write personal dashboard layout preferences.
- `dashboard:insights:read` — risk/insight/action recommendations if separated from base read.
- `dashboard:employee-absence:read` — employee-level absence/attendance detail; scope-aware.
- `payroll:read` or `compensation:read` — payroll totals, overtime cost, salary-derived values; never implied by `dashboard:read`.
- `payroll:budget:read` — budget and variance values if budget data is confidentially separated.
- `recruitment:analytics:read` — recruitment source and workforce movement analytics if required by role policy.
- Existing `attendance:summaries:read`, `attendance:logs:read`, `leave:requests:read`, `overtime:request:read`, `approval-workflows:read` remain domain permissions for drill-through actions.

Every new handler must perform the permission check before calling the service. Department managers must use scoped authorization, not only a global `DashboardRead` check. A 403 must not reveal whether a sensitive metric exists.

## 4. Executive reporting context

### 4.1 Periods

The API accepts a canonical period object. Date values are calendar dates in the **workspace timezone**, not browser-local dates.

| Period key | Boundary definition |
|---|---|
| `today` | Workspace-local calendar day containing the request's effective `as_of` instant. |
| `this_week` | Workspace week start through the current workspace-local day; week-start setting must be explicit and stable. |
| `this_month` | First through last/current workspace-local calendar day of the current month. |
| `this_quarter` | Workspace fiscal-quarter boundaries; fiscal calendar configuration must be explicit. |
| `year_to_date` | Workspace fiscal-year start through current workspace-local day. |
| `custom` | Inclusive `start_date` and `end_date`, validated as workspace-local dates; maximum range and pagination rules apply. |

The normalized response must echo:

```json
{
  "period": {
    "key": "this_month",
    "start_date": "YYYY-MM-DD",
    "end_date": "YYYY-MM-DD",
    "timezone": "Asia/Ho_Chi_Minh",
    "as_of": "RFC3339"
  }
}
```

Rules:

- Workspace timezone is loaded server-side and is the source of reporting boundaries.
- Browser timezone may be used only for display when clearly labeled; never for query boundaries or metric calculation.
- The server converts workspace-local inclusive dates to UTC instants using timezone-aware DST-safe helpers.
- Future end dates are rejected or clamped by contract; the chosen behavior must be consistent across all executive endpoints. Preferred behavior: reject invalid/future custom ranges with a structured validation error.
- A timezone change creates a semantic boundary. Historical values must retain their calculation timezone or be recomputed under a documented policy.

### 4.2 Filters and scope

Required common filters:

- `workspace_id` from authenticated workspace context; never trusted from an arbitrary query parameter.
- Optional permitted `department_id`, `location_id`, `employment_type`, and `employment_status` filters.
- Employee-level filters are prohibited in aggregate endpoints unless the caller has the dedicated permission and scope.
- All collection/drill-down responses are paginated.
- Filter metadata must identify applied scope and omitted dimensions due to authorization.

### 4.3 Comparison modes

The request may select one comparison mode; the response always states its basis.

- `previous_period`: immediately preceding period of equal duration where possible.
- `same_period_last_year`: aligned calendar/fiscal period in prior year; leap/DST behavior documented.
- `target`: organizational target configured for the metric and effective period.
- `budget`: approved budget baseline, only for cost metrics and permission-gated.
- `benchmark`: named internal/external benchmark with source, effective date, and comparability metadata.
- `none`: no comparison.

Comparison response shape:

```json
{
  "comparison": {
    "basis": "previous_period",
    "period": { "start_date": "YYYY-MM-DD", "end_date": "YYYY-MM-DD" },
    "value": 0,
    "delta": 0,
    "delta_percent": 0,
    "status": "better|worse|on_target|not_applicable|data_gap"
  }
}
```

For rates, directionality is metric-specific: lower absence/late/turnover is generally better; higher attendance/retention is generally better. The API must return direction metadata so UI does not infer it.

## 5. Executive KPI dictionary

### 5.1 Shared formula conventions

- `eligible_employee_days` = sum of employees eligible for scheduled work on each workspace-local day in the period, excluding approved full-day leave and non-working/non-scheduled days according to the attendance policy. The policy version must be recorded.
- `scheduled_employee_days` = employee-days with an applicable schedule/working obligation in the period.
- `active_headcount_at_date` = distinct active employments/employees according to the workforce snapshot policy at the specified local date.
- A denominator of zero is **not** a zero rate. Return `value: null`, `status: "not_applicable"`, and an explanatory `null_reason`.
- Missing source tables, incomplete ingestion, or incompatible policy versions return `value: null`, `status: "data_gap"`; do not silently substitute zero.
- All KPI values include `unit`, `numerator`, `denominator`, `definition_version`, `data_freshness`, and `quality_flags`.

### 5.2 KPI table

| Metric | Formula / numerator | Denominator | Unit | Null and quality policy |
|---|---|---|---|---|
| **Headcount** | `active_headcount_at_date`; for a period card use end-of-period headcount and expose start/headcount movement separately. | Not a rate; optional denominator is not applicable. | Employees (integer); optionally FTE as a separate metric, never mixed. | Null only when workforce snapshot cannot be computed (`data_gap`). Empty valid workspace is `0`. |
| **Attendance rate** | Count of eligible employee-days with an attendance outcome classified `present` or policy-approved equivalent. | `eligible_employee_days`. | Percentage `[0,100]`. | Denominator zero → `not_applicable`; missing schedule/attendance coverage → `data_gap`, with coverage ratio. |
| **Absence rate** | Count of eligible employee-days classified absent/unworked without an attendance-present outcome, using approved policy classification. | `eligible_employee_days`. | Percentage `[0,100]`. | Denominator zero → `not_applicable`; do not equate missing punches with absence unless policy says so. |
| **Late rate** | Count of scheduled employee-days with a qualifying late arrival after grace/policy adjustment. | Scheduled employee-days with a valid first-arrival expectation and usable attendance data. | Percentage `[0,100]`. | No valid schedule/late policy → `data_gap`; denominator zero → `not_applicable`. Return grace policy version. |
| **Turnover rate** | `terminations_in_period / average_headcount_in_period`. Terminations must be classified and deduplicated by employment according to workforce policy. | Average of start and end active headcount by default; configurable monthly average for long custom ranges. | Percentage; period rate, with annualized value as a separate field. | No headcount snapshots → `data_gap`; zero average headcount → `not_applicable`. Return voluntary/involuntary split when source supports it. |
| **Retention rate** | `employees_present_at_period_start_and_still_active_at_period_end / active_headcount_at_period_start`, excluding approved cohort exclusions. | Active headcount at period start (cohort denominator). | Percentage `[0,100]`. | No employment history/termination effective dates → `data_gap`; zero cohort → `not_applicable`. Must expose cohort definition. |
| **Overtime hours** | Sum of approved overtime duration in the period; optionally expose system-detected duration separately, never blend them. | Not a rate. | Decimal hours, fixed precision; source status `approved|detected`. | Missing approval/state data → `data_gap`; valid no-OT period → `0`. |
| **Overtime cost** | Sum of approved overtime payable amount under effective payroll/overtime rules, currency-normalized at calculation time. | Not a rate. | Decimal monetary amount + ISO currency. | Requires payroll/compensation permission and complete salary/rule inputs. Missing cost inputs → `data_gap`, not zero. |
| **Payroll variance** | `actual_payroll_cost - approved_budget` for the same period and scope. Return both actual and budget. | Budget baseline. | Decimal monetary amount + ISO currency; also percentage variance `delta / budget`. | Requires payroll and budget permissions. Missing budget → `data_gap`; zero budget → amount may exist but percentage is `not_applicable`. |
| **Pending HR actions** | Count of actionable unresolved items assigned to or within the caller's permitted HR/manager scope, grouped by action type and urgency. | Not a rate. | Integer count; expose overdue count separately. | Valid no actions → `0`; unavailable source/action type → partial response with `data_gap` per category. Never use current-user lifecycle tasks as the whole workspace KPI. |

### 5.3 Required metric response envelope

Each KPI item must follow this shape (exact generated OpenAPI type to be finalized):

```json
{
  "key": "attendance_rate",
  "value": 94.2,
  "unit": "percent",
  "numerator": { "value": 942, "unit": "eligible_employee_days" },
  "denominator": { "value": 1000, "unit": "eligible_employee_days" },
  "status": "available|not_applicable|data_gap|restricted",
  "null_reason": null,
  "definition_version": "attendance-rate.v1",
  "direction": "higher_is_better",
  "freshness": { "as_of": "RFC3339", "lag_seconds": 120 },
  "quality_flags": [],
  "comparison": null
}
```

For `restricted`, do not return numerator, denominator, or a value that permits inference. Return a localized-safe reason code only.

## 6. Insight, risk, and action layer

The executive dashboard must not stop at chart rendering. It must produce an explainable, permission-filtered set of findings.

### 6.1 Insight contract

Each insight contains:

- `id`, `type` (`trend`, `threshold`, `concentration`, `data_quality`, `workflow`), `severity` (`info|watch|high|critical`),
- `metric_key`, `title_key`, `summary_key`, and structured parameters for i18n,
- evidence: metric values, comparison, affected aggregate dimension, and freshness,
- `recommended_action` with an allowed route/action identifier,
- `requires_permission`, `scope`, `dismissible`, and audit metadata.

Do not generate free-form AI claims in this phase. Rules are deterministic, versioned, testable, and explainable.

### 6.2 Initial risk rules

- Attendance or absence rate crosses configured target tolerance.
- Late rate worsens against previous period and is concentrated in a permitted department.
- Overtime hours/cost exceeds target or budget threshold.
- Turnover rate exceeds threshold, with voluntary/involuntary split when available.
- Retention cohort falls below target.
- Pending HR actions are overdue or exceed SLA.
- Data freshness/coverage falls below the minimum release threshold.

### 6.3 Action routing

Actions must route to existing permissioned workflows (attendance review, leave approvals, overtime approval, payroll review, employee/department report) and must not mutate data from a read-only dashboard. Mutating actions require their existing domain permission, confirmation, audit log, and structured errors.

## 7. Privacy, sensitive data, and API security

### 7.1 Classification

- **T1 Restricted:** credentials, tokens, secrets — never returned.
- **T2 Confidential:** salary, payroll, compensation, leave/medical reasons, employee-level absence detail — role-gated, audit-logged for detail/export, never in aggregate overview unless explicitly authorized.
- **T3 Internal:** names, departments, job titles, attendance aggregates — workspace-scoped and RBAC-protected.
- **T4 Public:** workspace display metadata only.

### 7.2 Required API controls

1. Every executive endpoint is workspace-scoped through authenticated `MemberContext`; no client-supplied workspace ID may widen scope.
2. Every handler authorizes before service/repository access.
3. `dashboard:read` is required for aggregate overview; sensitive metrics require narrower permissions.
4. Department manager access uses `require_scoped_permission`/equivalent; never return cross-department aggregates by omission after querying unrestricted data.
5. Employee-level absence responses require dedicated permission and scope; aggregate response is the default.
6. Payroll/salary/overtime-cost/budget responses require separate permission and audit logging. A UI `PermissionGate` is not a security boundary.
7. API errors do not reveal sensitive metric existence, employee identifiers, or salary values.
8. Exports containing T2+ data are paginated, rate-limited, and audit-logged with actor, workspace, timestamp, scope, and row count.
9. Cache keys include workspace, permission/scope context, period, filters, and definition version; never serve a privileged response to a less-privileged caller.
10. Response metadata includes `workspace_id` only where the existing contract requires it; do not expose internal tenant/database identifiers unnecessarily.

### 7.3 Existing audit risks to resolve in roadmap

- `/devices` and agent/system dashboard payloads appear broader than the dashboard's display need and require explicit device/agent permissions plus field minimization.
- Current estimated payroll is salary-derived and is exposed through the general dashboard stats contract; it must be moved behind a compensation/payroll gate or replaced by a restricted aggregate.
- Unexcused absence employee grouping can return employee IDs/names under `DashboardRead`; it must be scoped and separately gated.

## 8. Data-gap register and proposed schemas

A data gap means no reliable backend source or no contract-quality semantics were found during audit. No fake value is permitted.

| ID | Gap | Current evidence | Proposed source/schema/API |
|---|---|---|---|
| DG-01 | Period-aware executive snapshot | Existing stats are today/30-day/current-month mixed. | `GET /v1/workspaces/{workspace_id}/executive-dashboard` or workspace-context equivalent with `period`, `comparison`, filters; one response envelope containing KPI items, insights, risks, freshness, and action counts. |
| DG-02 | Attendance/absence/late rate denominators | Existing responses are mostly counts. | Add a policy-versioned `employee_day_fact`/materialized summary with scheduled, eligible, present, absent, late, leave, and data-quality flags by workspace-local date and employment. Batch aggregates, indexed by workspace/date/department. |
| DG-03 | Retention | No retention endpoint/model. | Employment history snapshot/termination effective dates plus cohort query; `retention_rate` and cohort breakdown endpoint. Define exclusions and rehire policy. |
| DG-04 | Turnover rate | Existing turnover returns terminated counts only. | Add denominator snapshots and termination reason/voluntary classification. Return period and annualized rate with average-headcount method. |
| DG-05 | Overtime cost | Existing OT trend returns approved/system hours only. | Persist approved payable OT amount and calculation currency/rule version, or compute in a batch service from immutable approved inputs. Add `overtime-cost` aggregate permission. |
| DG-06 | Payroll variance | Existing `estimated_monthly_payroll` is a salary-component sum, no budget/actual run variance. | Payroll run totals, approved budget versions by period/scope/currency, variance service, and `payroll:budget:read` gate. |
| DG-07 | Pending HR actions | Current widget is current user's lifecycle checklist; leave count is isolated. | Action read model aggregating leave approvals, OT approvals/claims, attendance adjustments/exceptions, payroll exceptions, signatures, and SLA/overdue metadata. Respect actor scope. |
| DG-08 | Targets and benchmarks | No common target/budget/benchmark contract. | `metric_targets` and `metric_benchmarks` versioned by workspace/scope/period/effective dates/source; direction and tolerance included. |
| DG-09 | Fiscal calendar | Quarter/YTD semantics are not standardized in dashboard APIs. | Workspace reporting settings: timezone, week start, fiscal year start, fiscal quarter mapping, effective history. |
| DG-10 | Data freshness/coverage | Existing APIs do not expose a unified freshness/coverage quality contract. | Ingestion watermark and quality summary per fact source; response `freshness`, `coverage_ratio`, `quality_flags`, and data-gap category. |
| DG-11 | Scoped aggregate enforcement | Dashboard handlers use global DashboardRead; scoped department semantics are absent. | Scope resolver shared by executive query service, with pre-query authorization filters and tests for cross-department denial/non-inference. |
| DG-12 | Sensitive field minimization | Device/system payloads and employee absence details exceed dashboard needs. | Dedicated dashboard DTOs with allowlisted fields; remove passwords/IPs/identifiers unless an authorized operational screen explicitly needs them. |
| DG-13 | Metric definition registry | Metric semantics are currently distributed across repository/UI code. | Versioned metric registry in backend/domain docs/OpenAPI: key, formula, source, unit, null policy, direction, classification, permission, and owner. |
| DG-14 | Historical timezone policy | Workspace timezone exists, but frontend date-only calculations can use browser timezone. | Server-normalized period DTO; frontend must send period keys/custom local dates only, and render dates from response metadata. |
| DG-15 | Payroll finalization | No code transitions payslip `status` from `draft`; `paid`/`finalized` payslips are never produced. Cost metrics therefore report `data_gap` (`no_paid_payslips`) until a payroll finalization workflow lands. This is a pipeline gap, not a dashboard bug. | Payroll run service that finalizes payslips (`status IN ('paid','finalized')`); cost metrics then emit values automatically. |
| DG-16 | OT-pay formula | `OvertimeEngine::calculate_overtime_pay_from_times` is test-only; `ot_pay` is stubbed to 0 in the attendance matcher, so `overtime_pay_amount` is 0 even when payslips are finalized. Cost emission is tied to `cost_source_available` (paid/finalized payslips present), not to nonzero amounts. | Wire OT pay calculation into the attendance matcher; persist `overtime_pay_amount` from the engine. |
| DG-17 | Payroll forecast | No forecast engine exists. `payroll_forecast` is always `data_gap` (`forecast_source_unavailable`). | Forecasting service (trend/seasonal) producing a period payroll forecast with confidence band. |
| DG-18 | FTE | No FTE data; `employment_type` is a free string. `ot_cost_per_employee` uses active headcount as denominator. | FTE model (employment_type → FTE multiplier or `fte` column); then `ot_cost_per_fte` metric. |
| DG-19 | Budget | Budget lives in `workspace_config_entries` EAV keys (`dashboard.budget.overtime_cost`, `dashboard.budget.labor_cost`), workspace scope, admin-set. Missing key → `budget_variance` `data_gap` (`budget_config_missing`). | `dashboard_metric_budgets` table for per-scope (department/location) budgets with effective dates. |

## 9. API contract proposal

### 9.1 Executive overview

Preferred endpoint (workspace is still derived/validated from auth context; path style must follow the project's established convention):

`GET /v1/dashboard/executive-overview`

Query parameters:

- `period=today|this_week|this_month|this_quarter|year_to_date|custom`
- `start_date`, `end_date` required only for `custom`
- `comparison=none|previous_period|same_period_last_year|target|budget|benchmark`
- permitted dimension filters (`department_id`, `location_id`, `employment_type`)
- `include=metrics,insights,actions,breakdowns` (server permission-filtered)
- pagination parameters for breakdowns/actions

Response top-level:

```json
{
  "contract_version": "executive-dashboard.v1",
  "workspace": { "timezone": "Asia/Ho_Chi_Minh", "currency": "VND" },
  "context": { "period": {}, "comparison": {}, "filters": {}, "scope": {} },
  "metrics": [],
  "insights": [],
  "actions": { "total": 0, "overdue": 0, "by_type": [] },
  "data_quality": { "status": "complete|partial|data_gap", "sources": [] },
  "generated_at": "RFC3339"
}
```

### 9.2 Drill-down endpoints

Keep existing operational endpoints for backward compatibility where their current semantics are stable. Add explicit period/context parameters rather than silently changing defaults. Proposed additions:

- `GET /v1/dashboard/executive-metrics/{metric_key}` — paginated breakdown, permission-filtered.
- `GET /v1/dashboard/executive-actions` — consolidated actionable queue, paginated.
- `GET /v1/dashboard/retention` — cohort/retention details.
- `GET /v1/dashboard/payroll-variance` — restricted actual/budget/variance.
- `GET /v1/dashboard/overtime-cost` — restricted approved cost and rule/currency metadata.

All new and changed paths require OpenAPI schemas and generated frontend types. Do not introduce hand-written duplicate generated types.

## 10. Accessibility and presentation contract

The dashboard is a decision surface, not a chart gallery.

- Use semantic headings and landmarks; each KPI has an accessible label, value, unit, comparison, and status text.
- Never communicate status by color alone; pair status with text/icon and accessible description.
- All controls are keyboard-operable with visible focus; dialogs trap/restore focus and close on Escape.
- Every chart has a table/structured data view and a text summary. Legends are present for multi-series charts; direct labels are selective.
- Use a fixed semantic palette: status colors are not reused as ordinary series colors. Validate light and dark palettes before implementation.
- Keep filters in one visible row/group above the KPI and chart region; expose active period, timezone, comparison, and scope.
- Loading state: skeletons that preserve layout. Empty state: explain “no eligible data” vs “data gap” vs “restricted”. Error state: retry and non-technical localized message.
- Responsive behavior: mobile-first; charts may stack, but KPI meaning/order and action priority remain stable.
- Avoid dual axes; use separate charts/small multiples for different units.
- Provide reduced-motion behavior and ensure tooltips do not contain the only interpretation of a mark.

## 11. Risk register

| ID | Severity | Risk | Consequence | Mitigation / acceptance gate |
|---|---|---|---|---|
| R-01 | Critical | Cross-scope data leakage for department managers. | Confidential workforce data exposure. | Pre-query scope filters, dedicated scoped permissions, negative integration tests, cache isolation. |
| R-02 | Critical | Payroll/salary-derived data reachable through `DashboardRead`. | T2 exposure and legal/compliance impact. | Separate payroll/compensation permission, restricted DTO, audit logs, 403 tests. |
| R-03 | High | Browser timezone changes reporting boundary. | Different executives see different “today/month” values. | Server-normalized workspace-local period; timezone contract tests around midnight/DST. |
| R-04 | High | Counts shown as rates without denominators. | Misleading executive decisions. | KPI envelope requires numerator/denominator and null policy; contract tests. |
| R-05 | High | Missing source data represented as zero. | False confidence and hidden data outage. | `data_gap` status, freshness/coverage flags, no synthetic fallback. |
| R-06 | High | Turnover/retention definitions inconsistent with HR policy. | Incorrect workforce strategy. | Versioned definitions, cohort/termination policy sign-off, fixture-based tests. |
| R-07 | High | Payroll variance uses estimate instead of actual/budget. | Incorrect cost decisions. | Separate actual/budget sources; no variance until both exist. |
| R-08 | High | Consolidated pending-action count misses workflow domains. | Backlog and SLA risk hidden. | Action read model with source coverage and partial-data flags. |
| R-09 | Medium | Widget-level independent fetches produce incoherent periods/freshness. | Dashboard appears internally contradictory. | One executive snapshot query/context; parallel drill-down only within same context. |
| R-10 | Medium | Sensitive employee absence names/IDs overexposed in aggregate chart. | Privacy and employee-relations risk. | Aggregate default, minimum cohort threshold where policy requires, dedicated detail permission. |
| R-11 | Medium | Unbounded breakdowns cause latency/N+1. | Slow dashboard and pool pressure. | Batch SQL/materialized facts, pagination, indexes, query budgets, no repository calls in loops. |
| R-12 | Medium | API/schema drift between backend and React. | Runtime parsing failures or hidden data loss. | OpenAPI-first, generated types, Zod contract tests, no `any`/escape hatch. |
| R-13 | Medium | Insight actions mutate from a read-only surface. | Unauthorized state change. | Route actions through existing domain permissions and confirmation/audit boundaries. |
| R-14 | Low | Charts are visually polished but not interpretable in forced colors/print. | Accessibility and decision quality degradation. | Table/text alternative, texture/secondary encoding, WCAG review. |

## 12. Implementation roadmap

### Phase 0 — Contract and policy lock

1. Approve this metric dictionary with HR and Finance owners.
2. Decide workforce policy details: employee vs employment grain, FTE, rehire, leave, schedule, grace period, termination classification, fiscal calendar, currency, benchmark provenance.
3. Confirm permission names and role mappings; document scoped manager semantics.
4. Define data classification and update `docs/data-inventory.md` for new fact/target/budget sources.

**Exit gate:** no KPI proceeds with an unresolved formula, owner, permission, or null policy.

### Phase 1 — Reporting context and secure executive shell

1. Introduce canonical period/comparison/filter DTOs and server-side workspace timezone normalization.
2. Add executive overview response envelope with data quality/freshness metadata.
3. Implement permission matrix, scoped filters, restricted DTOs, cache-key isolation, rate limits, and OpenAPI schemas.
4. Frontend adds a shared executive context provider/query; existing operational widgets remain available and are labeled by period.

**Exit gate:** workspace/browser timezone, 403, scope, and schema contract tests pass.

### Phase 2 — Reuse and normalize existing operational metrics

1. Map existing workspace stats, attendance trend, department attendance, unexcused absence, lateness, and OT-hour sources into the KPI envelope.
2. Add explicit numerators/denominators and policy/quality metadata.
3. Keep existing endpoint compatibility; do not silently change old defaults.
4. Add executive KPI tiles, accessible tables/text summaries, target/comparison states, and risk rules.

**Exit gate:** attendance/absence/late/headcount/OT-hours agree with authoritative report fixtures across all periods.

### Phase 3 — Workforce movement and action intelligence

1. Build retention cohorts and normalized turnover rate.
2. Build consolidated pending HR action read model with SLA/overdue counts.
3. Add deterministic insight/risk/action routing with permission-aware drill-through.
4. Add recruitment-manager views using aggregate workforce movement/source semantics.

**Exit gate:** retention/turnover policy fixtures, action-source coverage report, and no-PII overview tests pass.

### Phase 4 — Restricted cost intelligence

1. Add approved OT cost with currency/rule provenance.
2. Add actual payroll run totals, approved budget versions, and payroll variance.
3. Gate all cost/salary data separately, audit sensitive reads/exports, and minimize responses.
4. Add Finance/HR sign-off and reconciliation tests against payroll source-of-truth.

**Exit gate:** no cost metric displays without complete source/budget/permission/freshness metadata.

### Phase 5 — Performance, accessibility, and rollout

1. Batch/materialize employee-day facts and index workspace/date/dimension access paths.
2. Run query-plan/N+1 checks, load tests, and dashboard latency budgets.
3. Complete WCAG 2.1 AA, keyboard, screen-reader, reduced-motion, dark-mode, print/forced-colors review.
4. Feature-flag by workspace/role; observe adoption, errors, data-gap rate, latency, and denied-access events.
5. Roll out gradually with rollback plan and documented operational ownership.

**Exit gate:** full security → lint → schema → unit → integration/E2E → UX/accessibility → performance pipeline passes using `./leedevkit test` wrappers.

## 13. Program acceptance criteria

The Executive HR Dashboard program is accepted only when all criteria below are true:

### Product and semantics

- [ ] All five user groups have role-appropriate default views and documented decisions/actions.
- [ ] All ten executive KPIs have approved formulas, numerator/denominator, unit, null policy, owner, direction, definition version, freshness, and data-quality semantics.
- [ ] All six periods are supported with one canonical workspace-timezone boundary model.
- [ ] All five comparison modes return explicit basis, period/value, delta, direction, and status.
- [ ] Existing useful operational charts remain available or have a documented replacement with equivalent semantics.
- [ ] Every insight/risk is deterministic, explainable, versioned, localized, permission-filtered, and linked to a permitted action.

### Data and API

- [ ] Executive overview returns one coherent context for period, comparison, filters, scope, timezone, freshness, and data quality.
- [ ] No missing backend source is represented as fake data or an unexplained zero; each gap is `data_gap` or `not_applicable`.
- [ ] All collection/breakdown endpoints are paginated and avoid N+1 queries.
- [ ] OpenAPI is canonical; generated frontend types and runtime schemas are synchronized; no new `any` or unsafe type cast is introduced.
- [ ] Workspace isolation and scoped authorization are enforced before business logic and covered by positive/negative integration tests.

### Security and privacy

- [ ] Dashboard aggregate access requires `dashboard:read` (or approved equivalent) and every endpoint is workspace-scoped.
- [ ] Payroll, salary, overtime cost, budget, and employee-level absence are separately permission-gated, minimized, and audit-logged where required.
- [ ] No PII/T1/T2 data appears in logs, generic errors, aggregate overview, unauthorized cache entries, or unapproved exports.
- [ ] Device/agent dashboard responses expose only allowlisted fields and require their domain read permissions.
- [ ] Permission failures fail closed and do not reveal sensitive metric existence.

### Timezone and correctness

- [ ] Server tests cover workspace-local midnight, browser timezone mismatch, DST transitions, custom inclusive ranges, week/month/quarter/YTD boundaries, and timezone changes.
- [ ] Frontend never derives reporting boundaries from browser timezone; it renders the server-normalized context.
- [ ] Metric values reconcile with authoritative attendance, employment, payroll, and workflow fixtures within approved tolerance.

### Accessibility and UX

- [ ] KPI cards and charts have semantic labels, visible units/status/comparison, keyboard access, focus states, and table/text alternatives.
- [ ] Loading, empty, restricted, partial/data-gap, and error states are distinct and localized.
- [ ] Status is never color-only; light/dark palettes and forced-colors/print alternatives are validated.
- [ ] Responsive layouts preserve decision priority and do not overflow or hide critical actions.
- [ ] No chart uses dual axes; multi-series charts provide legends and selective direct labels.

### Operations and delivery

- [ ] APIs include tracing/metrics, freshness instrumentation, rate limits, and auditable sensitive-read events.
- [ ] Feature flag, migration rollback, backfill/recompute, and data-retention plans are documented.
- [ ] Full lint, unit, integration, and E2E tests pass through the hermetic `./leedevkit test` workflow; no direct cargo/npm/diesel commands are used.
- [ ] Rollout has an owner, dashboards/alerts, support runbook, and rollback trigger.

## 14. Audit evidence index

Primary evidence used for this contract:

- `webdashboard/src/pages/dashboard-page.tsx:30-185`
- `webdashboard/src/hooks/use-dashboard.ts:81-205`
- `webdashboard/src/hooks/use-dashboard-types.ts:4-96`
- `webdashboard/src/hooks/dashboard-schemas.ts:6-102`
- `webdashboard/src/hooks/use-dashboard-charts.tsx:6-85`
- `webdashboard/src/components/features/dashboard/pending-tasks-widget.tsx:81-151`
- `apiserver/src/routes/dashboard.rs:7-51`
- `apiserver/src/handlers/workspaces/crud.rs:31-185,197-249`
- `apiserver/src/handlers/recruitment_sources.rs:166-227`
- `apiserver/src/handlers/dashboard_preferences.rs:228-285`
- `apiserver/src/handlers/common.rs:132-194`
- `apiserver/src/domain/permissions.rs:32-144`
- `apiserver/src/repositories/workspace_stats_repository.rs:60-95,101-451,777-927`
- `apiserver/src/repositories/sql/workspace_stats/*.sql`
- `apiserver/src/models/common.rs:14-49`
- `apiserver/src/models/agent.rs:15-85`
- `webdashboard/src/components/features/dashboard/unexcused-absences-chart.tsx:24-171`
- `webdashboard/src/components/features/dashboard/turnover-analytics-chart.tsx:33-231`
- `webdashboard/src/hooks/use-datetime-utils.ts:20-89`

This document is intentionally a product/API contract and roadmap. It does not claim that data-gap items are implemented.
