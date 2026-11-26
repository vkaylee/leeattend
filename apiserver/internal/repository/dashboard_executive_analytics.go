package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

const executiveOverviewSQL = `WITH periods AS (
    SELECT $1::date AS start_date, $2::date AS end_date, 'current' AS period
    UNION ALL
    SELECT $3::date, $4::date, 'comparison'
),
active_days AS (
    SELECT p.period, COUNT(*)::bigint AS eligible_employee_days
    FROM periods p
    JOIN employments emp
      ON emp.deleted_at IS NULL
     AND emp.status = 'active'
     AND emp.start_date <= p.end_date
     AND (emp.end_date IS NULL OR emp.end_date >= p.start_date)
     AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))
     AND ($6::uuid[] IS NULL OR emp.primary_location_id = ANY($6))
    CROSS JOIN LATERAL generate_series(
        GREATEST(emp.start_date, p.start_date),
        LEAST(COALESCE(emp.end_date, p.end_date), p.end_date),
        interval '1 day'
    ) day_value
    GROUP BY p.period
),
headcounts AS (
    SELECT p.period,
           COUNT(DISTINCT emp.employee_id) FILTER (WHERE emp.start_date <= p.end_date AND (emp.end_date IS NULL OR emp.end_date >= p.end_date))::bigint AS headcount_end,
           COUNT(DISTINCT emp.employee_id) FILTER (WHERE emp.start_date <= p.start_date AND (emp.end_date IS NULL OR emp.end_date >= p.start_date))::bigint AS headcount_start
    FROM periods p
    LEFT JOIN employments emp
      ON emp.deleted_at IS NULL
     AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))
     AND ($6::uuid[] IS NULL OR emp.primary_location_id = ANY($6))
    GROUP BY p.period
),
attendance_days AS (
    SELECT p.period,
           COUNT(DISTINCT (ads.employment_id, ads.date)) FILTER (WHERE ads.actual_work_minutes > 0)::bigint AS present_employee_days,
           COUNT(DISTINCT (ads.employment_id, ads.date)) FILTER (WHERE ads.actual_work_minutes = 0 AND NOT ads.is_holiday)::bigint AS absent_employee_days,
           COUNT(DISTINCT (lr.employment_id, day_value::date))::bigint AS leave_employee_days
    FROM periods p
    LEFT JOIN attendance_daily_summaries ads
      ON ads.date BETWEEN p.start_date AND p.end_date
    LEFT JOIN employments emp ON emp.id = ads.employment_id
      AND emp.deleted_at IS NULL
      AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))
      AND ($6::uuid[] IS NULL OR emp.primary_location_id = ANY($6))
    LEFT JOIN leave_requests lr
      ON lr.status IN ('approved', 'pre_approved')
     AND lr.start_date <= p.end_date AND lr.end_date >= p.start_date
     AND lr.employment_id = emp.id
    LEFT JOIN LATERAL generate_series(
      GREATEST(lr.start_date, p.start_date), LEAST(lr.end_date, p.end_date), interval '1 day'
    ) day_value ON lr.id IS NOT NULL
    GROUP BY p.period
),
overtime AS (
    SELECT p.period,
           COALESCE(SUM(ads.approved_ot_minutes), 0)::bigint AS approved_overtime_minutes,
           COALESCE(SUM(ads.system_detected_ot_minutes), 0)::bigint AS system_detected_ot_minutes
    FROM periods p
    LEFT JOIN attendance_daily_summaries ads
      ON ads.date BETWEEN p.start_date AND p.end_date
    LEFT JOIN employments emp ON emp.id = ads.employment_id
      AND emp.deleted_at IS NULL
      AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))
      AND ($6::uuid[] IS NULL OR emp.primary_location_id = ANY($6))
    GROUP BY p.period
),
terminations AS (
    SELECT p.period, COUNT(DISTINCT t.id)::bigint AS termination_count
    FROM periods p
    LEFT JOIN employee_tenures t ON t.deleted_at IS NULL
      AND t.end_date BETWEEN p.start_date AND p.end_date
    LEFT JOIN employees e ON e.id = t.employee_id AND e.deleted_at IS NULL
    WHERE t.id IS NULL OR e.id IS NOT NULL
    GROUP BY p.period
),
actions AS (
    SELECT p.period,
      COALESCE((SELECT COUNT(*) FROM leave_requests lr JOIN employments emp ON emp.id = lr.employment_id
       WHERE lr.status = 'pending' AND emp.deleted_at IS NULL
         AND lr.created_at::date BETWEEN p.start_date AND p.end_date
         AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))), 0)
      + COALESCE((SELECT COUNT(*) FROM overtime_requests ot
         WHERE ot.status = 'pending_approval'
           AND ot.planned_date BETWEEN p.start_date AND p.end_date
           AND ($5::uuid[] IS NULL OR ot.department_id = ANY($5))), 0) AS pending_hr_actions
    FROM periods p
),
currency AS (
    SELECT COALESCE((SELECT currency_code FROM employers WHERE deleted_at IS NULL AND currency_code IS NOT NULL ORDER BY created_at LIMIT 1), 'USD') AS currency_code
),
payroll_cost AS (
    SELECT p.period,
           CASE WHEN $8::boolean THEN COALESCE(SUM(ps.overtime_pay_amount), 0) END AS overtime_cost,
           CASE WHEN $8::boolean THEN COALESCE(SUM(ps.base_pay_amount + ps.overtime_pay_amount + COALESCE(ps.holiday_work_premium_amount, 0)), 0) END AS labor_cost,
           CASE WHEN $8::boolean THEN COALESCE(SUM(ps.net_pay_amount), 0) END AS payroll_actual,
           CASE WHEN $8::boolean THEN COUNT(ps.id) END AS payslip_count
    FROM periods p
    LEFT JOIN payslips ps
     ON ps.status IN ('paid', 'finalized')
     AND ps.period_start <= p.end_date AND ps.period_end >= p.start_date
    LEFT JOIN employments emp
      ON emp.id = ps.employment_id
     AND emp.deleted_at IS NULL
     AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))
     AND ($6::uuid[] IS NULL OR emp.primary_location_id = ANY($6))
    WHERE ps.id IS NOT NULL OR $8::boolean IS FALSE
    GROUP BY p.period
)
SELECT h.headcount_end AS headcount,
       COALESCE(a.eligible_employee_days, 0)::bigint AS eligible_employee_days,
       COALESCE(ad.present_employee_days, 0)::bigint AS present_employee_days,
       COALESCE(ad.absent_employee_days, 0)::bigint AS absent_employee_days,
       COALESCE(ad.leave_employee_days, 0)::bigint AS leave_employee_days,
       COALESCE(t.termination_count, 0)::bigint AS termination_count,
       COALESCE(h.headcount_start, 0)::bigint AS headcount_start,
       COALESCE(h.headcount_end, 0)::bigint AS headcount_end,
       COALESCE(o.approved_overtime_minutes, 0)::bigint AS approved_overtime_minutes,
       COALESCE(o.system_detected_ot_minutes, 0)::bigint AS system_detected_ot_minutes,
       GREATEST(COALESCE(o.system_detected_ot_minutes, 0) - COALESCE(o.approved_overtime_minutes, 0), 0)::bigint AS unapproved_ot_minutes,
       COALESCE(ac.pending_hr_actions, 0)::bigint AS pending_hr_actions,
       c.currency_code,
       oc.overtime_cost,
       oc.labor_cost,
       oc.payroll_actual,
       CASE WHEN $8::boolean AND oc.payslip_count IS NOT NULL AND oc.payslip_count > 0 THEN TRUE ELSE FALSE END AS cost_source_available
FROM headcounts h
LEFT JOIN active_days a USING (period)
LEFT JOIN attendance_days ad USING (period)
LEFT JOIN overtime o USING (period)
LEFT JOIN terminations t USING (period)
LEFT JOIN actions ac USING (period)
LEFT JOIN payroll_cost oc USING (period)
CROSS JOIN currency c
WHERE h.period = $7`

type executiveSnapshot struct {
	headcount           int64
	eligibleDays        int64
	presentDays         int64
	absentDays          int64
	leaveDays           int64
	terminations        int64
	headcountStart      int64
	headcountEnd        int64
	approvedOTMin       int64
	systemOTMin         int64
	unapprovedOTMin     int64
	pendingHRActions    int64
	currencyCode        string
	overtimeCost        *decimal.Decimal
	laborCost           *decimal.Decimal
	payrollActual       *decimal.Decimal
	costSourceAvailable bool
}

func resolveComparisonDateStrings(q api.NormalizedDashboardMetricsQuery, startStr, endStr string) (string, string) {
	switch q.Comparison {
	case "previous_period":
		durDays := int(q.EndDate.Sub(q.StartDate).Hours()/24) + 1
		compEnd := q.StartDate.AddDate(0, 0, -1)
		compStart := compEnd.AddDate(0, 0, -(durDays - 1))
		return compStart.Format("2006-01-02"), compEnd.Format("2006-01-02")
	case "same_period_last_year":
		return q.StartDate.AddDate(-1, 0, 0).Format("2006-01-02"), q.EndDate.AddDate(-1, 0, 0).Format("2006-01-02")
	default:
		return startStr, endStr
	}
}

func scanExecutiveSnapshot(ctx context.Context, db DBTX, startStr, endStr, compStartStr, compEndStr string, deptFilter, locFilter any, period string, allowOvertimeCost bool) (executiveSnapshot, error) {
	var s executiveSnapshot
	err := db.QueryRow(ctx, executiveOverviewSQL, startStr, endStr, compStartStr, compEndStr, deptFilter, locFilter, period, allowOvertimeCost).Scan(
		&s.headcount, &s.eligibleDays, &s.presentDays, &s.absentDays, &s.leaveDays,
		&s.terminations, &s.headcountStart, &s.headcountEnd, &s.approvedOTMin,
		&s.systemOTMin, &s.unapprovedOTMin, &s.pendingHRActions, &s.currencyCode,
		&s.overtimeCost, &s.laborCost, &s.payrollActual, &s.costSourceAvailable,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		s.currencyCode = "USD"
		return s, nil
	}
	return s, err
}

func buildExecutiveRateMetric(key, defVer, direction, qtyUnit string, numerator, denominator int64, target *api.DashboardMetricTarget, freshness api.DashboardMetricFreshness) api.DashboardMetric {
	if denominator <= 0 {
		return api.DashboardMetric{
			Key: key, Unit: "percent", Status: "not_applicable", Health: "not_applicable",
			NullReason: new("zero_denominator"), DefinitionVersion: defVer, Direction: direction,
			Freshness: freshness, QualityFlags: []string{"zero_denominator"},
		}
	}
	rate := decimal.NewFromInt(numerator).Div(decimal.NewFromInt(denominator)).Mul(decimal.NewFromInt(100))
	health := "neutral"
	if target != nil {
		health = "ok"
		if (target.Operator == "gte" && rate.LessThan(target.Value)) || (target.Operator == "lte" && rate.GreaterThan(target.Value)) {
			health = "warning"
		}
	}
	num := api.DashboardMetricQuantity{Value: decimal.NewFromInt(numerator), Unit: qtyUnit}
	den := api.DashboardMetricQuantity{Value: decimal.NewFromInt(denominator), Unit: qtyUnit}
	return api.DashboardMetric{
		Key: key, Value: &rate, Unit: "percent", Numerator: &num, Denominator: &den,
		Status: "available", Health: health, DefinitionVersion: defVer, Direction: direction,
		Freshness: freshness, QualityFlags: []string{}, Target: target,
	}
}

func buildExecutiveValueMetric(key, unit, defVer, direction string, val decimal.Decimal, freshness api.DashboardMetricFreshness) api.DashboardMetric {
	return api.DashboardMetric{
		Key: key, Value: &val, Unit: unit, Status: "available", Health: "neutral",
		DefinitionVersion: defVer, Direction: direction, Freshness: freshness, QualityFlags: []string{},
	}
}

func buildExecutiveDataGapMetric(key, unit, defVer, direction, reason string, freshness api.DashboardMetricFreshness) api.DashboardMetric {
	return api.DashboardMetric{
		Key: key, Unit: unit, Status: "data_gap", Health: "data_gap",
		NullReason: new(reason), DefinitionVersion: defVer, Direction: direction,
		Freshness: freshness, QualityFlags: []string{reason},
	}
}

func buildExecutiveCostMetric(key, defVer, direction string, allow, available bool, val *decimal.Decimal, freshness api.DashboardMetricFreshness) api.DashboardMetric {
	if allow && available && val != nil {
		return api.DashboardMetric{
			Key: key, Value: val, Unit: "currency", Status: "available", Health: "neutral",
			DefinitionVersion: defVer, Direction: direction, Freshness: freshness, QualityFlags: []string{},
		}
	}
	return buildExecutiveDataGapMetric(key, "currency", defVer, direction, "no_paid_payslips", freshness)
}

func buildExecutiveMetrics(q api.NormalizedDashboardMetricsQuery, compStartStr, compEndStr string, cur, cmp executiveSnapshot, allowOvertimeCost bool, freshness api.DashboardMetricFreshness) []api.DashboardMetric {
	hcVal := decimal.NewFromInt(cur.headcount)
	hcMetric := buildExecutiveValueMetric("headcount", "count", "headcount.v1", "neutral", hcVal, freshness)
	hcMetric.Comparison = buildComparison(q.Comparison, compStartStr, compEndStr, decimal.NewFromInt(cmp.headcount), hcVal)

	meanHc := (cur.headcountStart + cur.headcountEnd) / 2
	attTarget := &api.DashboardMetricTarget{Value: decimal.NewFromInt(95), Operator: "gte"}
	absTarget := &api.DashboardMetricTarget{Value: decimal.NewFromInt(5), Operator: "lte"}
	turnTarget := &api.DashboardMetricTarget{Value: decimal.NewFromInt(15), Operator: "lte"}

	return []api.DashboardMetric{
		hcMetric,
		buildExecutiveRateMetric("attendance_rate", "attendance-rate.v1", "higher_is_better", "employee_days", cur.presentDays, cur.eligibleDays, attTarget, freshness),
		buildExecutiveRateMetric("absence_rate", "absence-rate.v1", "lower_is_better", "employee_days", cur.absentDays, cur.eligibleDays, absTarget, freshness),
		buildExecutiveDataGapMetric("late_rate", "percent", "late-rate.v1", "lower_is_better", "scheduled_attendance_source_unavailable", freshness),
		buildExecutiveRateMetric("turnover_rate", "turnover-rate.v1", "lower_is_better", "employees", cur.terminations, meanHc, turnTarget, freshness),
		buildExecutiveValueMetric("system_ot_hours", "hours", "system-ot-hours.v1", "lower_is_better", decimal.NewFromInt(cur.systemOTMin/60), freshness),
		buildExecutiveValueMetric("approved_ot_hours", "hours", "approved-ot-hours.v1", "lower_is_better", decimal.NewFromInt(cur.approvedOTMin/60), freshness),
		buildExecutiveValueMetric("unapproved_ot_hours", "hours", "unapproved-ot-hours.v1", "lower_is_better", decimal.NewFromInt(cur.unapprovedOTMin/60), freshness),
		buildExecutiveRateMetric("ot_approval_rate", "ot-approval-rate.v1", "higher_is_better", "minutes", cur.approvedOTMin, cur.systemOTMin, nil, freshness),
		buildExecutiveCostMetric("overtime_cost", "overtime-cost.v1", "lower_is_better", allowOvertimeCost, cur.costSourceAvailable, cur.overtimeCost, freshness),
		buildExecutiveCostMetric("labor_cost", "labor-cost.v1", "lower_is_better", allowOvertimeCost, cur.costSourceAvailable, cur.laborCost, freshness),
		buildExecutiveCostMetric("payroll_actual", "payroll-actual.v1", "neutral", allowOvertimeCost, cur.costSourceAvailable, cur.payrollActual, freshness),
		buildExecutiveDataGapMetric("payroll_forecast", "currency", "payroll-forecast.v1", "neutral", "forecast_source_unavailable", freshness),
		buildExecutiveDataGapMetric("budget_variance", "percent", "budget-variance.v1", "lower_is_better", "budget_not_configured", freshness),
		buildExecutiveValueMetric("pending_hr_actions", "count", "pending-hr-actions.v1", "lower_is_better", decimal.NewFromInt(cur.pendingHRActions), freshness),
	}
}

func buildExecutiveDataQuality(metrics []api.DashboardMetric, cur executiveSnapshot, allowOvertimeCost bool, freshness api.DashboardMetricFreshness) api.DashboardDataQuality {
	qualityStatus := "complete"
	for _, m := range metrics {
		if m.Status == "data_gap" {
			qualityStatus = "partial"
			break
		}
	}
	empStatus := "data_gap"
	if cur.headcount > 0 {
		empStatus = "complete"
	}
	attStatus := "data_gap"
	if cur.eligibleDays > 0 {
		attStatus = "complete"
	}
	payStatus := "data_gap"
	payWarnings := []string{}
	if allowOvertimeCost && cur.costSourceAvailable {
		payStatus = "complete"
	} else if allowOvertimeCost {
		payWarnings = append(payWarnings, "no_paid_payslips_in_period")
	}
	return api.DashboardDataQuality{
		Status: qualityStatus,
		Sources: []api.DashboardQualitySource{
			{Key: "employments", Status: empStatus, Freshness: freshness, Warnings: []string{}},
			{Key: "attendance_daily_summaries", Status: attStatus, Freshness: freshness, Warnings: []string{}},
			{Key: "scheduled_attendance", Status: "data_gap", Freshness: freshness, Warnings: []string{"scheduled_attendance_source_unavailable"}},
			{Key: "payroll", Status: payStatus, Freshness: freshness, Warnings: payWarnings},
		},
	}
}

func (r DashboardAnalyticsRepository) GetExecutiveOverview(ctx context.Context, q api.NormalizedDashboardMetricsQuery, allowOvertimeCost bool) (api.DashboardMetricsResponse, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.DashboardMetricsResponse{}, err
	}

	startStr := q.StartDate.Format("2006-01-02")
	endStr := q.EndDate.Format("2006-01-02")
	compStartStr, compEndStr := resolveComparisonDateStrings(q, startStr, endStr)

	var deptFilter, locFilter any
	if len(q.DepartmentIDs) > 0 {
		deptFilter = q.DepartmentIDs
	}
	if len(q.LocationIDs) > 0 {
		locFilter = q.LocationIDs
	}

	cur, err := scanExecutiveSnapshot(ctx, db, startStr, endStr, compStartStr, compEndStr, deptFilter, locFilter, "current", allowOvertimeCost)
	if err != nil {
		return api.DashboardMetricsResponse{}, err
	}
	cmp, _ := scanExecutiveSnapshot(ctx, db, startStr, endStr, compStartStr, compEndStr, deptFilter, locFilter, "comparison", allowOvertimeCost)

	now := time.Now().UTC()
	freshness := api.DashboardMetricFreshness{AsOf: now, LagSeconds: new(uint64(0))}
	metrics := buildExecutiveMetrics(q, compStartStr, compEndStr, cur, cmp, allowOvertimeCost, freshness)

	scopeKind := "organization"
	if len(q.DepartmentIDs) > 0 {
		scopeKind = "department"
	}
	deptIDs, locIDs := q.DepartmentIDs, q.LocationIDs
	if deptIDs == nil {
		deptIDs = []uuid.UUID{}
	}
	if locIDs == nil {
		locIDs = []uuid.UUID{}
	}

	return api.DashboardMetricsResponse{
		ContractVersion: "executive-dashboard.v2",
		Organization:    api.DashboardOrganizationContext{Timezone: q.Timezone, Currency: cur.currencyCode},
		Context: api.DashboardMetricContext{
			Period: api.DashboardReportingPeriod{
				Key: q.Period, StartDate: startStr, EndDate: endStr, Timezone: q.Timezone, AsOf: now,
			},
			Comparison: q.Comparison,
			Filters:    map[string]string{"top_n": fmt.Sprintf("%d", q.TopN), "risk_threshold": q.RiskThreshold},
			Scope: api.DashboardMetricScope{
				Kind: scopeKind, DepartmentIDs: deptIDs, LocationIDs: locIDs,
				OmittedDimensions: []string{"business_unit", "manager", "employment_type", "employee_status", "shift"},
			},
		},
		Metrics:     metrics,
		Insights:    []api.DashboardInsight{},
		Actions:     api.DashboardActionsSummary{Total: uint64(cur.pendingHRActions), Overdue: 0, ByType: []api.DashboardActionTypeCount{}},
		DataQuality: buildExecutiveDataQuality(metrics, cur, allowOvertimeCost, freshness),
		GeneratedAt: now,
	}, nil
}

func buildComparison(basis, start, end string, previousVal, currentVal decimal.Decimal) *api.DashboardMetricComparison {
	if basis == "none" {
		return nil
	}
	delta := currentVal.Sub(previousVal)
	return &api.DashboardMetricComparison{
		Basis:       basis,
		PeriodStart: start,
		PeriodEnd:   end,
		Value:       &previousVal,
		Delta:       &delta,
		Status:      "available",
	}
}
