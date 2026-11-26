package repository

import (
	"context"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"time"
)

func (r DashboardAnalyticsRepository) GetExecutiveOverview(ctx context.Context, q api.NormalizedDashboardMetricsQuery, allowOvertimeCost bool) (api.DashboardMetricsResponse, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.DashboardMetricsResponse{}, err
	}

	startStr := q.StartDate.Format("2006-01-02")
	endStr := q.EndDate.Format("2006-01-02")

	var compStartStr, compEndStr string
	if q.Comparison == "previous_period" {
		durDays := int(q.EndDate.Sub(q.StartDate).Hours()/24) + 1
		compEnd := q.StartDate.AddDate(0, 0, -1)
		compStart := compEnd.AddDate(0, 0, -(durDays - 1))
		compStartStr = compStart.Format("2006-01-02")
		compEndStr = compEnd.Format("2006-01-02")
	} else if q.Comparison == "same_period_last_year" {
		compStart := q.StartDate.AddDate(-1, 0, 0)
		compEnd := q.EndDate.AddDate(-1, 0, 0)
		compStartStr = compStart.Format("2006-01-02")
		compEndStr = compEnd.Format("2006-01-02")
	} else {
		compStartStr = startStr
		compEndStr = endStr
	}

	var deptFilter, locFilter any
	if len(q.DepartmentIDs) > 0 {
		deptFilter = q.DepartmentIDs
	}
	if len(q.LocationIDs) > 0 {
		locFilter = q.LocationIDs
	}

	sql := `WITH periods AS (
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

	type snap struct {
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

	scanSnap := func(period string) (snap, error) {
		var s snap
		err := db.QueryRow(ctx, sql, startStr, endStr, compStartStr, compEndStr, deptFilter, locFilter, period, allowOvertimeCost).Scan(
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

	cur, err := scanSnap("current")
	if err != nil {
		return api.DashboardMetricsResponse{}, err
	}
	cmp, _ := scanSnap("comparison")

	now := time.Now().UTC()
	freshness := api.DashboardMetricFreshness{AsOf: now, LagSeconds: new(uint64(0))}

	metrics := make([]api.DashboardMetric, 0)

	// 1. Headcount
	hcVal := decimal.NewFromInt(cur.headcount)
	metrics = append(metrics, api.DashboardMetric{
		Key:               "headcount",
		Value:             &hcVal,
		Unit:              "count",
		Status:            "available",
		Health:            "neutral",
		DefinitionVersion: "headcount.v1",
		Direction:         "neutral",
		Freshness:         freshness,
		QualityFlags:      []string{},
		Comparison:        buildComparison(q.Comparison, compStartStr, compEndStr, decimal.NewFromInt(cmp.headcount), hcVal),
	})

	// 2. Attendance rate
	if cur.eligibleDays > 0 {
		rate := decimal.NewFromInt(cur.presentDays).Div(decimal.NewFromInt(cur.eligibleDays)).Mul(decimal.NewFromInt(100))
		target := decimal.NewFromInt(95)
		health := "ok"
		if rate.LessThan(target) {
			health = "warning"
		}
		num := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.presentDays), Unit: "employee_days"}
		den := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.eligibleDays), Unit: "employee_days"}
		metrics = append(metrics, api.DashboardMetric{
			Key:               "attendance_rate",
			Value:             &rate,
			Unit:              "percent",
			Numerator:         &num,
			Denominator:       &den,
			Status:            "available",
			Health:            health,
			DefinitionVersion: "attendance-rate.v1",
			Direction:         "higher_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{},
			Target:            &api.DashboardMetricTarget{Value: target, Operator: "gte"},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "attendance_rate",
			Value:             nil,
			Unit:              "percent",
			Status:            "not_applicable",
			Health:            "not_applicable",
			NullReason:        new("zero_denominator"),
			DefinitionVersion: "attendance-rate.v1",
			Direction:         "higher_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{"zero_denominator"},
		})
	}

	// 3. Absence rate
	if cur.eligibleDays > 0 {
		rate := decimal.NewFromInt(cur.absentDays).Div(decimal.NewFromInt(cur.eligibleDays)).Mul(decimal.NewFromInt(100))
		target := decimal.NewFromInt(5)
		health := "ok"
		if rate.GreaterThan(target) {
			health = "warning"
		}
		num := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.absentDays), Unit: "employee_days"}
		den := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.eligibleDays), Unit: "employee_days"}
		metrics = append(metrics, api.DashboardMetric{
			Key:               "absence_rate",
			Value:             &rate,
			Unit:              "percent",
			Numerator:         &num,
			Denominator:       &den,
			Status:            "available",
			Health:            health,
			DefinitionVersion: "absence-rate.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{},
			Target:            &api.DashboardMetricTarget{Value: target, Operator: "lte"},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "absence_rate",
			Value:             nil,
			Unit:              "percent",
			Status:            "not_applicable",
			Health:            "not_applicable",
			NullReason:        new("zero_denominator"),
			DefinitionVersion: "absence-rate.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{"zero_denominator"},
		})
	}

	// 4. Late rate
	metrics = append(metrics, api.DashboardMetric{
		Key:               "late_rate",
		Value:             nil,
		Unit:              "percent",
		Status:            "data_gap",
		Health:            "data_gap",
		NullReason:        new("scheduled_attendance_source_unavailable"),
		DefinitionVersion: "late-rate.v1",
		Direction:         "lower_is_better",
		Freshness:         freshness,
		QualityFlags:      []string{"scheduled_attendance_source_unavailable"},
	})

	// 5. Turnover rate
	meanHc := (cur.headcountStart + cur.headcountEnd) / 2
	if meanHc > 0 {
		rate := decimal.NewFromInt(cur.terminations).Div(decimal.NewFromInt(meanHc)).Mul(decimal.NewFromInt(100))
		target := decimal.NewFromInt(15)
		health := "ok"
		if rate.GreaterThan(target) {
			health = "warning"
		}
		num := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.terminations), Unit: "employees"}
		den := api.DashboardMetricQuantity{Value: decimal.NewFromInt(meanHc), Unit: "employees"}
		metrics = append(metrics, api.DashboardMetric{
			Key:               "turnover_rate",
			Value:             &rate,
			Unit:              "percent",
			Numerator:         &num,
			Denominator:       &den,
			Status:            "available",
			Health:            health,
			DefinitionVersion: "turnover-rate.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{},
			Target:            &api.DashboardMetricTarget{Value: target, Operator: "lte"},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "turnover_rate",
			Value:             nil,
			Unit:              "percent",
			Status:            "not_applicable",
			Health:            "not_applicable",
			NullReason:        new("zero_denominator"),
			DefinitionVersion: "turnover-rate.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{"zero_denominator"},
		})
	}

	// 6. Overtime hours (system, approved, unapproved)
	sysHVal := decimal.NewFromInt(cur.systemOTMin / 60)
	metrics = append(metrics, api.DashboardMetric{
		Key:               "system_ot_hours",
		Value:             &sysHVal,
		Unit:              "hours",
		Status:            "available",
		Health:            "neutral",
		DefinitionVersion: "system-ot-hours.v1",
		Direction:         "lower_is_better",
		Freshness:         freshness,
		QualityFlags:      []string{},
	})

	appHVal := decimal.NewFromInt(cur.approvedOTMin / 60)
	metrics = append(metrics, api.DashboardMetric{
		Key:               "approved_ot_hours",
		Value:             &appHVal,
		Unit:              "hours",
		Status:            "available",
		Health:            "neutral",
		DefinitionVersion: "approved-ot-hours.v1",
		Direction:         "lower_is_better",
		Freshness:         freshness,
		QualityFlags:      []string{},
	})

	unappHVal := decimal.NewFromInt(cur.unapprovedOTMin / 60)
	metrics = append(metrics, api.DashboardMetric{
		Key:               "unapproved_ot_hours",
		Value:             &unappHVal,
		Unit:              "hours",
		Status:            "available",
		Health:            "neutral",
		DefinitionVersion: "unapproved-ot-hours.v1",
		Direction:         "lower_is_better",
		Freshness:         freshness,
		QualityFlags:      []string{},
	})

	// 7. OT approval rate
	if cur.systemOTMin > 0 {
		rate := decimal.NewFromInt(cur.approvedOTMin).Div(decimal.NewFromInt(cur.systemOTMin)).Mul(decimal.NewFromInt(100))
		num := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.approvedOTMin), Unit: "minutes"}
		den := api.DashboardMetricQuantity{Value: decimal.NewFromInt(cur.systemOTMin), Unit: "minutes"}
		metrics = append(metrics, api.DashboardMetric{
			Key:               "ot_approval_rate",
			Value:             &rate,
			Unit:              "percent",
			Numerator:         &num,
			Denominator:       &den,
			Status:            "available",
			Health:            "neutral",
			DefinitionVersion: "ot-approval-rate.v1",
			Direction:         "higher_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "ot_approval_rate",
			Value:             nil,
			Unit:              "percent",
			Status:            "not_applicable",
			Health:            "not_applicable",
			NullReason:        new("zero_denominator"),
			DefinitionVersion: "ot-approval-rate.v1",
			Direction:         "higher_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{"zero_denominator"},
		})
	}

	// 8. Overtime cost & labor cost (conditional)
	if allowOvertimeCost && cur.costSourceAvailable && cur.overtimeCost != nil {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "overtime_cost",
			Value:             cur.overtimeCost,
			Unit:              "currency",
			Status:            "available",
			Health:            "neutral",
			DefinitionVersion: "overtime-cost.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "overtime_cost",
			Value:             nil,
			Unit:              "currency",
			Status:            "data_gap",
			Health:            "data_gap",
			NullReason:        new("no_paid_payslips"),
			DefinitionVersion: "overtime-cost.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{"no_paid_payslips"},
		})
	}

	if allowOvertimeCost && cur.costSourceAvailable && cur.laborCost != nil {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "labor_cost",
			Value:             cur.laborCost,
			Unit:              "currency",
			Status:            "available",
			Health:            "neutral",
			DefinitionVersion: "labor-cost.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "labor_cost",
			Value:             nil,
			Unit:              "currency",
			Status:            "data_gap",
			Health:            "data_gap",
			NullReason:        new("no_paid_payslips"),
			DefinitionVersion: "labor-cost.v1",
			Direction:         "lower_is_better",
			Freshness:         freshness,
			QualityFlags:      []string{"no_paid_payslips"},
		})
	}

	if allowOvertimeCost && cur.costSourceAvailable && cur.payrollActual != nil {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "payroll_actual",
			Value:             cur.payrollActual,
			Unit:              "currency",
			Status:            "available",
			Health:            "neutral",
			DefinitionVersion: "payroll-actual.v1",
			Direction:         "neutral",
			Freshness:         freshness,
			QualityFlags:      []string{},
		})
	} else {
		metrics = append(metrics, api.DashboardMetric{
			Key:               "payroll_actual",
			Value:             nil,
			Unit:              "currency",
			Status:            "data_gap",
			Health:            "data_gap",
			NullReason:        new("no_paid_payslips"),
			DefinitionVersion: "payroll-actual.v1",
			Direction:         "neutral",
			Freshness:         freshness,
			QualityFlags:      []string{"no_paid_payslips"},
		})
	}

	// 9. Payroll forecast & budget variance
	metrics = append(metrics, api.DashboardMetric{
		Key:               "payroll_forecast",
		Value:             nil,
		Unit:              "currency",
		Status:            "data_gap",
		Health:            "data_gap",
		NullReason:        new("forecast_source_unavailable"),
		DefinitionVersion: "payroll-forecast.v1",
		Direction:         "neutral",
		Freshness:         freshness,
		QualityFlags:      []string{"forecast_source_unavailable"},
	})

	metrics = append(metrics, api.DashboardMetric{
		Key:               "budget_variance",
		Value:             nil,
		Unit:              "percent",
		Status:            "data_gap",
		Health:            "data_gap",
		NullReason:        new("budget_not_configured"),
		DefinitionVersion: "budget-variance.v1",
		Direction:         "lower_is_better",
		Freshness:         freshness,
		QualityFlags:      []string{"budget_not_configured"},
	})

	// 10. Pending HR actions
	actVal := decimal.NewFromInt(cur.pendingHRActions)
	metrics = append(metrics, api.DashboardMetric{
		Key:               "pending_hr_actions",
		Value:             &actVal,
		Unit:              "count",
		Status:            "available",
		Health:            "neutral",
		DefinitionVersion: "pending-hr-actions.v1",
		Direction:         "lower_is_better",
		Freshness:         freshness,
		QualityFlags:      []string{},
	})

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
	} else if allowOvertimeCost && !cur.costSourceAvailable {
		payWarnings = append(payWarnings, "no_paid_payslips_in_period")
	}

	qualitySources := []api.DashboardQualitySource{
		{Key: "employments", Status: empStatus, Freshness: freshness, Warnings: []string{}},
		{Key: "attendance_daily_summaries", Status: attStatus, Freshness: freshness, Warnings: []string{}},
		{Key: "scheduled_attendance", Status: "data_gap", Freshness: freshness, Warnings: []string{"scheduled_attendance_source_unavailable"}},
		{Key: "payroll", Status: payStatus, Freshness: freshness, Warnings: payWarnings},
	}

	scopeKind := "organization"
	if len(q.DepartmentIDs) > 0 {
		scopeKind = "department"
	}

	deptIDs := q.DepartmentIDs
	if deptIDs == nil {
		deptIDs = []uuid.UUID{}
	}
	locIDs := q.LocationIDs
	if locIDs == nil {
		locIDs = []uuid.UUID{}
	}

	return api.DashboardMetricsResponse{
		ContractVersion: "executive-dashboard.v2",
		Organization: api.DashboardOrganizationContext{
			Timezone: q.Timezone,
			Currency: cur.currencyCode,
		},
		Context: api.DashboardMetricContext{
			Period: api.DashboardReportingPeriod{
				Key:       q.Period,
				StartDate: startStr,
				EndDate:   endStr,
				Timezone:  q.Timezone,
				AsOf:      now,
			},
			Comparison: q.Comparison,
			Filters:    map[string]string{"top_n": fmt.Sprintf("%d", q.TopN), "risk_threshold": q.RiskThreshold},
			Scope: api.DashboardMetricScope{
				Kind:              scopeKind,
				DepartmentIDs:     deptIDs,
				LocationIDs:       locIDs,
				OmittedDimensions: []string{"business_unit", "manager", "employment_type", "employee_status", "shift"},
			},
		},
		Metrics:     metrics,
		Insights:    []api.DashboardInsight{},
		Actions:     api.DashboardActionsSummary{Total: uint64(cur.pendingHRActions), Overdue: 0, ByType: []api.DashboardActionTypeCount{}},
		DataQuality: api.DashboardDataQuality{Status: qualityStatus, Sources: qualitySources},
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
