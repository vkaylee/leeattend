# P6-17 Dashboard Analytics Evidence

Task 6.1 Dashboard analytics (12 routes) completed in Go API, advancing contract route coverage from 345 to 357 (missing routes reduced from 58 to 46).

## Route Coverage Status

- Contract routes: **403**
- Go route registrations: **382** (+12)
- Contract routes covered: **357** (+12)
- Contract routes missing: **46** (-12)

---

## 12 Newly Implemented & Wired Routes

All 12 routes wired in `apiserver/main.go` and implemented in `internal/api/dashboard_analytics.go` and `internal/repository/dashboard_analytics.go`:

### 1. Executive & Workspace Overview (3 routes)
1. `GET /api/v1/dashboard/executive-overview` — 200 OK `DashboardMetricsResponse`. Workspace-scoped executive KPI cards and trends (`contract_version: "executive-dashboard.v1"`). Permission: `dashboard:read` base; conditional `payroll:sensitive:read` gates overtime cost, labor cost, and payroll actual metrics.
2. `GET /api/v1/dashboard/overtime-breakdown` — 200 OK `OvertimeBreakdownResponse`. Department-level overtime hours and costs (`contract_version: "overtime-breakdown.v1"`). Permission: `dashboard:read` base; conditional `payroll:sensitive:read` gates `overtime_cost` and `cost_available`.
3. `GET /api/v1/dashboard/workspace-stats` — 200 OK `WorkspaceStats`. Key headcount, present, late, absent, on-leave, pending leave requests, estimated payroll, and attendance rates. Permission: `dashboard:read`.

### 2. Attendance & Absence Analytics (5 routes)
4. `GET /api/v1/dashboard/attendance-trend` — 200 OK `[]AttendanceTrendPoint`. Daily attendance, absence, late, and on-leave points with typed `RateQuantity` percentage rates. Permission: `dashboard:read`.
5. `GET /api/v1/dashboard/department-attendance` — 200 OK `[]DepartmentAttendancePoint`. Department-scoped attendance rates, on-time, late, absent, and on-leave counts. Supports deprecated `range` preset (`today`, `week`, `month`) and explicit `start_date`/`end_date`. Permission: `dashboard:read`.
6. `GET /api/v1/dashboard/unexcused-absences` — 200 OK `[]UnexcusedAbsencesPoint`. Unexcused absences aggregated by department or employee. Permission: `dashboard:read` base; requires `dashboard:employee_absence:read` when `group_by=employee`.
7. `GET /api/v1/dashboard/overtime-trend` — 200 OK `[]OvertimeTrendPoint`. Daily system-detected and approved overtime hours over the last 30 days. Permission: `dashboard:read`.
8. `GET /api/v1/dashboard/peak-times` — 200 OK `[]PeakTimeSlot`. Check-in and check-out distribution by 30-minute time slots across the day. Permission: `dashboard:read`.

### 3. HR Operations & Retention (4 routes)
9. `GET /api/v1/dashboard/leave-types` — 200 OK `[]LeaveTypeBreakdown`. Approved leave request counts grouped by leave type for the current month. Permission: `dashboard:read`.
10. `GET /api/v1/dashboard/lateness-by-day` — 200 OK `[]LatenessByDay`. Late arrival distribution and average late minutes grouped by day of week (Mon–Sun). Permission: `dashboard:read`.
11. `GET /api/v1/dashboard/turnover-analytics` — 200 OK `TurnoverAnalytics`. Turnover rates with mandatory denominator, tenure buckets, voluntary/involuntary classification, retention rate, early attrition, and monthly trends. Permission: `dashboard:read`.
12. `GET /api/v1/dashboard/recruitment-effectiveness` — 200 OK `RecruitmentEffectivenessAnalytics`. Sourced hires, 90-day retention, cost per hire, turnover by source, conversion funnel (pre-hire stages surfaced with `count: null`), and structured data availability gaps. Permission: `dashboard:read`.

---

## Complete Dashboard Analytics Route Map (12/12)

| # | Method | Path | Status | Handler | Auth Guard |
|---|---|---|---|---|---|
| 1 | GET | `/api/v1/dashboard/executive-overview` | 200 OK | `DashboardAnalyticsHandler.ExecutiveOverview` | `dashboard:read` + conditional `payroll:sensitive:read` |
| 2 | GET | `/api/v1/dashboard/overtime-breakdown` | 200 OK | `DashboardAnalyticsHandler.OvertimeBreakdown` | `dashboard:read` + conditional `payroll:sensitive:read` |
| 3 | GET | `/api/v1/dashboard/workspace-stats` | 200 OK | `DashboardAnalyticsHandler.WorkspaceStats` | `dashboard:read` |
| 4 | GET | `/api/v1/dashboard/attendance-trend` | 200 OK | `DashboardAnalyticsHandler.AttendanceTrend` | `dashboard:read` |
| 5 | GET | `/api/v1/dashboard/department-attendance` | 200 OK | `DashboardAnalyticsHandler.DepartmentAttendance` | `dashboard:read` |
| 6 | GET | `/api/v1/dashboard/unexcused-absences` | 200 OK | `DashboardAnalyticsHandler.UnexcusedAbsences` | `dashboard:read` + `dashboard:employee_absence:read` (when employee group) |
| 7 | GET | `/api/v1/dashboard/overtime-trend` | 200 OK | `DashboardAnalyticsHandler.OvertimeTrend` | `dashboard:read` |
| 8 | GET | `/api/v1/dashboard/peak-times` | 200 OK | `DashboardAnalyticsHandler.PeakTimes` | `dashboard:read` |
| 9 | GET | `/api/v1/dashboard/leave-types` | 200 OK | `DashboardAnalyticsHandler.LeaveTypes` | `dashboard:read` |
| 10 | GET | `/api/v1/dashboard/lateness-by-day` | 200 OK | `DashboardAnalyticsHandler.LatenessByDay` | `dashboard:read` |
| 11 | GET | `/api/v1/dashboard/turnover-analytics` | 200 OK | `DashboardAnalyticsHandler.TurnoverAnalytics` | `dashboard:read` |
| 12 | GET | `/api/v1/dashboard/recruitment-effectiveness` | 200 OK | `DashboardAnalyticsHandler.RecruitmentEffectiveness` | `dashboard:read` |

---

## Key Implementation & Verification Details

1. **Typed DTOs & Decimal Precision**:
   - Currency amounts and rates use `github.com/shopspring/decimal.Decimal` (renders string `"-?\\d+(\\.\\d+)?"` in JSON).
   - Zero-denominator rate rule: `RateQuantity.Value` is `nil` (renders `null` in JSON) when `denominator <= 0`, never fabricating `"0.0"`.
   - Float64 used for hourly and minute averages (`avg_work_hours_today`, `avg_late_minutes`, `system_ot_hours`, `approved_ot_hours`, `unapproved_ot_hours`).

2. **Security & Department Scoping**:
   - `dashboard:read` checked on all 12 routes.
   - `dashboard:employee_absence:read` strictly checked when `group_by=employee` on `/unexcused-absences`.
   - `payroll:sensitive:read` conditionally gates cost fields; emits `dashboard.sensitive.read` audit log event when sensitive cost metrics are surfaced.
   - Department scoping enforced through `GetManagedDepartmentIDs`: requesting department IDs outside managed scope returns 403 Forbidden; omitting department filters under a scoped caller automatically scopes queries to managed departments.

3. **Recruitment Funnel & Data Availability Gaps**:
   - Pre-hire stages (`applicants`, `screening`, `interview`, `offer`) emit `count: null` and `data_source: null`.
   - Structured availability reports return 8 fixed data gaps with canonical i18n keys (`dashboard.hr.effective_gap_*`).

4. **Unit Tests (`internal/api/dashboard_analytics_test.go`)**:
   - `TestDashboardAnalytics_ForbiddenWithoutBasePermission`: verifies all 12 endpoints reject requests without `dashboard:read` (403 Forbidden).
   - `TestDashboardAnalytics_UnexcusedAbsencesPermissions`: tests department grouping (200), employee grouping without permission (403), employee grouping with permission (200), and invalid group_by (400).
   - `TestDashboardAnalytics_ExecutiveOverview`: tests query execution, conditional cost gating, invalid period (400), and date validation (400).
   - `TestDashboardAnalytics_OvertimeBreakdown`: tests default 30-day window and partial date validation (400).
   - `TestDashboardAnalytics_DepartmentScoping`: tests foreign department rejection (403) and managed department acceptance (200).
   - `TestDashboardAnalytics_AllRemainingEndpoints`: tests remaining 7 routes (workspace stats, attendance trend, department attendance, overtime trend, peak times, leave types, lateness by day, turnover analytics, recruitment effectiveness).
   - `TestRateQuantity_ZeroDenominator`: validates `RateQuantity` marshals `null` for value when denominator is 0.
