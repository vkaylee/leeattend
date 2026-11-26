package repository

import (
	"context"
	"fmt"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func queryTurnoverCountMap[T any](ctx context.Context, db DBTX, sql string, startStr, endStr string, build func(label string, count int64) T) []T {
	rows, _ := db.Query(ctx, sql, startStr, endStr)
	out := make([]T, 0)
	if rows == nil {
		return out
	}
	defer rows.Close()
	for rows.Next() {
		var label string
		var count int64
		if err := rows.Scan(&label, &count); err == nil {
			out = append(out, build(label, count))
		}
	}
	return out
}

func queryTurnoverBySource(ctx context.Context, db DBTX, startStr, endStr string) []api.TurnoverBySource {
	rows, _ := db.Query(ctx, `SELECT COALESCE(rs.source_type::text, 'direct'), COALESCE(rs.name, 'Direct Hire'), COUNT(*)::bigint
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY rs.source_type, rs.name ORDER BY count DESC`, startStr, endStr)
	out := make([]api.TurnoverBySource, 0)
	if rows == nil {
		return out
	}
	defer rows.Close()
	for rows.Next() {
		var s api.TurnoverBySource
		if err := rows.Scan(&s.SourceType, &s.SourceName, &s.Count); err == nil {
			out = append(out, s)
		}
	}
	return out
}

func queryTurnoverClassification(ctx context.Context, db DBTX, startStr, endStr string) ([]api.TurnoverClassification, int64, int64, int64) {
	classification := queryTurnoverCountMap(ctx, db, `SELECT CASE
			WHEN emp.termination_reason IN ('voluntary_resignation', 'retirement', 'mutual_agreement') THEN 'voluntary'
			WHEN emp.termination_reason IN ('involuntary_termination', 'abandonment', 'death') THEN 'involuntary'
			ELSE 'unknown'
		END AS classification,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY classification`, startStr, endStr, func(label string, count int64) api.TurnoverClassification {
		return api.TurnoverClassification{Classification: label, Count: count}
	})
	var voluntary, involuntary, unknown int64
	for _, c := range classification {
		switch c.Classification {
		case "voluntary":
			voluntary = c.Count
		case "involuntary":
			involuntary = c.Count
		case "unknown":
			unknown = c.Count
		}
	}
	return classification, voluntary, involuntary, unknown
}

func queryTurnoverMonthlyTrend(ctx context.Context, db DBTX, startStr, endStr string, periodDays int) []api.TurnoverTrendPoint {
	monthlyTrend := make([]api.TurnoverTrendPoint, 0)
	if periodDays < 60 {
		return monthlyTrend
	}
	rows, _ := db.Query(ctx, `WITH months AS (
    SELECT generate_series(
        date_trunc('month', $1::date)::date,
        date_trunc('month', $2::date)::date,
        '1 month'::interval
    )::date AS m
),
leavers AS (
    SELECT
        date_trunc('month', emp.end_date)::date AS m,
        COUNT(DISTINCT emp.employee_id)::bigint AS leavers
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
      AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
    GROUP BY 1
),
headcounts AS (
    SELECT
        m.m AS m,
        COUNT(DISTINCT emp.employee_id)::bigint AS headcount
    FROM months m
    LEFT JOIN employments emp
      ON emp.deleted_at IS NULL
     AND emp.start_date <= m.m AND (emp.end_date IS NULL OR emp.end_date >= m.m)
    GROUP BY m.m
)
SELECT
    to_char(hc.m, 'YYYY-MM') AS month,
    COALESCE(hc.headcount, 0)::bigint AS headcount,
    COALESCE(lv.leavers, 0)::bigint AS leavers
FROM headcounts hc
LEFT JOIN leavers lv ON lv.m = hc.m
ORDER BY hc.m ASC`, startStr, endStr)
	if rows == nil {
		return monthlyTrend
	}
	defer rows.Close()
	for rows.Next() {
		var month string
		var hc, lv int64
		if err := rows.Scan(&month, &hc, &lv); err == nil {
			rate := api.NewTurnoverRate(lv, hc)
			monthlyTrend = append(monthlyTrend, api.TurnoverTrendPoint{Month: month, Headcount: hc, Leavers: lv, TurnoverRate: &rate})
		}
	}
	return monthlyTrend
}

func queryTurnoverCohortRetention(ctx context.Context, db DBTX, startStr, endStr string) []api.CohortRetentionRow {
	rows, _ := db.Query(ctx, `WITH cohort AS (
    SELECT
        date_trunc('month', emp.start_date)::date AS cohort_month,
        COUNT(DISTINCT emp.employee_id)::bigint AS cohort_size
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.deleted_at IS NULL
      AND emp.start_date >= date_trunc('month', $1::date)::date
      AND emp.start_date <= $2::date
    GROUP BY 1
    HAVING COUNT(DISTINCT emp.employee_id) >= $3
),
offsets AS (
    SELECT generate_series(0, GREATEST(0, $4::int - 1))::int AS months_since_hire
)
SELECT
    to_char(c.cohort_month, 'YYYY-MM') AS cohort_month,
    o.months_since_hire::bigint,
    c.cohort_size,
    COALESCE(surv.active_count, 0)::bigint AS active_count
FROM cohort c
CROSS JOIN offsets o
LEFT JOIN LATERAL (
    SELECT COUNT(DISTINCT emp.employee_id)::bigint AS active_count
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.deleted_at IS NULL
      AND date_trunc('month', emp.start_date)::date = c.cohort_month
      AND emp.start_date <= date_trunc('month', (c.cohort_month + make_interval(months => o.months_since_hire)))::date + interval '1 month' - interval '1 day'
      AND (emp.end_date IS NULL OR emp.end_date >= date_trunc('month', (c.cohort_month + make_interval(months => o.months_since_hire)))::date + interval '1 month' - interval '1 day')
) surv ON true
ORDER BY c.cohort_month ASC, o.months_since_hire ASC`, startStr, endStr, 3, 6)
	out := make([]api.CohortRetentionRow, 0)
	if rows == nil {
		return out
	}
	defer rows.Close()
	for rows.Next() {
		var c api.CohortRetentionRow
		if err := rows.Scan(&c.CohortMonth, &c.MonthsSinceHire, &c.CohortSize, &c.ActiveCount); err == nil {
			if c.CohortSize > 0 {
				ret := decimal.NewFromInt(c.ActiveCount).Div(decimal.NewFromInt(c.CohortSize)).Mul(decimal.NewFromInt(100))
				c.RetentionRate = &ret
			}
			out = append(out, c)
		}
	}
	return out
}

const (
	turnoverTotalTerminatedSQL = `SELECT COUNT(DISTINCT emp.employee_id)::bigint FROM employments emp
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date`

	turnoverTenureSQL = `SELECT bucket, COUNT(*)::bigint AS count FROM (
		SELECT CASE
			WHEN emp.end_date - emp.start_date < 90 THEN '< 3 months'
			WHEN emp.end_date - emp.start_date < 180 THEN '3-6 months'
			WHEN emp.end_date - emp.start_date < 365 THEN '6-12 months'
			WHEN emp.end_date - emp.start_date < 730 THEN '1-2 years'
			WHEN emp.end_date - emp.start_date < 1825 THEN '2-5 years'
			ELSE '> 5 years'
		END AS bucket
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
	) sub
	GROUP BY bucket
	ORDER BY CASE bucket
		WHEN '< 3 months' THEN 1
		WHEN '3-6 months' THEN 2
		WHEN '6-12 months' THEN 3
		WHEN '1-2 years' THEN 4
		WHEN '2-5 years' THEN 5
		WHEN '> 5 years' THEN 6
		ELSE 7
	END`

	turnoverByDeptSQL = `SELECT COALESCE(d.name, 'Unassigned'), COUNT(*)::bigint
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN departments d ON d.id = emp.department_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY d.name ORDER BY count DESC`

	turnoverReasonsSQL = `SELECT COALESCE(emp.termination_reason, 'unknown')::text AS reason,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY emp.termination_reason ORDER BY count DESC`

	turnoverByEmpTypeSQL = `SELECT COALESCE(emp.employment_type, 'unknown')::text AS employment_type,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY emp.employment_type ORDER BY count DESC`

	turnoverByLocSQL = `SELECT COALESCE(l.name, 'Unassigned')::text AS location_name,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN work_locations l ON l.id = emp.primary_location_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY l.name ORDER BY count DESC`

	turnoverByMgrSQL = `SELECT COALESCE(mgr_e.full_name, 'Unassigned')::text AS manager_name,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN employments mgr_emp ON mgr_emp.id = emp.manager_employment_id
		LEFT JOIN employees mgr_e ON mgr_e.id = mgr_emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY mgr_e.full_name ORDER BY count DESC`

	turnoverHeadcountSQL = `WITH months AS (
    SELECT generate_series(
        date_trunc('month', $1::date)::date,
        date_trunc('month', $2::date)::date,
        '1 month'::interval
    )::date AS m
)
SELECT
    COALESCE(AVG(hc), 0)::bigint AS average_headcount,
    COALESCE((SELECT COUNT(DISTINCT emp.employee_id)
              FROM employments emp
              WHERE emp.deleted_at IS NULL
                AND emp.start_date <= $1::date
                AND (emp.end_date IS NULL OR emp.end_date >= $1::date)), 0)::bigint AS headcount_start
FROM months
LEFT JOIN LATERAL (
    SELECT COUNT(DISTINCT emp.employee_id) AS hc
    FROM employments emp
    WHERE emp.deleted_at IS NULL
      AND emp.start_date <= m
      AND (emp.end_date IS NULL OR emp.end_date >= m)
) hc ON true`

	turnoverEarlyAttritionSQL = `SELECT COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		  AND (emp.end_date - emp.start_date) < 90`
)

func (r DashboardAnalyticsRepository) GetTurnoverAnalytics(ctx context.Context, q api.TurnoverAnalyticsQuery) (api.TurnoverAnalytics, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.TurnoverAnalytics{}, err
	}

	startStr, endStr := q.StartDate.Format("2006-01-02"), q.EndDate.Format("2006-01-02")
	var totalTerminated, avgHeadcount, headcountStart, earlyAttritionCount int64
	_ = db.QueryRow(ctx, turnoverTotalTerminatedSQL, startStr, endStr).Scan(&totalTerminated)
	_ = db.QueryRow(ctx, turnoverHeadcountSQL, startStr, endStr).Scan(&avgHeadcount, &headcountStart)
	_ = db.QueryRow(ctx, turnoverEarlyAttritionSQL, startStr, endStr).Scan(&earlyAttritionCount)

	tenureDist := queryTurnoverCountMap(ctx, db, turnoverTenureSQL, startStr, endStr, func(l string, c int64) api.TurnoverTenureBucket { return api.TurnoverTenureBucket{Bucket: l, Count: c} })
	byDept := queryTurnoverCountMap(ctx, db, turnoverByDeptSQL, startStr, endStr, func(l string, c int64) api.TurnoverByDepartment {
		return api.TurnoverByDepartment{DepartmentName: l, Count: c}
	})
	reasons := queryTurnoverCountMap(ctx, db, turnoverReasonsSQL, startStr, endStr, func(l string, c int64) api.TurnoverReasonRow { return api.TurnoverReasonRow{Reason: l, Count: c} })
	byEmpType := queryTurnoverCountMap(ctx, db, turnoverByEmpTypeSQL, startStr, endStr, func(l string, c int64) api.TurnoverByEmploymentType {
		return api.TurnoverByEmploymentType{EmploymentType: l, Count: c}
	})
	byLoc := queryTurnoverCountMap(ctx, db, turnoverByLocSQL, startStr, endStr, func(l string, c int64) api.TurnoverByLocation {
		return api.TurnoverByLocation{LocationName: l, Count: c}
	})
	byMgr := queryTurnoverCountMap(ctx, db, turnoverByMgrSQL, startStr, endStr, func(l string, c int64) api.TurnoverByManager { return api.TurnoverByManager{ManagerName: l, Count: c} })
	classification, voluntaryCount, involuntaryCount, unknownClassCount := queryTurnoverClassification(ctx, db, startStr, endStr)

	periodDays := int(q.EndDate.Sub(q.StartDate).Hours()/24) + 1
	var retentionRate *decimal.Decimal
	if headcountStart > 0 {
		ret := decimal.NewFromInt(max(headcountStart-totalTerminated, 0)).Div(decimal.NewFromInt(headcountStart)).Mul(decimal.NewFromInt(100))
		retentionRate = &ret
	}

	return api.TurnoverAnalytics{
		TenureDistribution:         tenureDist,
		BySource:                   queryTurnoverBySource(ctx, db, startStr, endStr),
		ByDepartment:               byDept,
		TotalTerminated:            totalTerminated,
		PeriodStart:                startStr,
		PeriodEnd:                  endStr,
		TurnoverRate:               api.NewTurnoverRate(totalTerminated, avgHeadcount),
		AverageHeadcount:           avgHeadcount,
		VoluntaryCount:             voluntaryCount,
		InvoluntaryCount:           involuntaryCount,
		UnknownClassificationCount: unknownClassCount,
		RetentionRate:              retentionRate,
		EarlyAttrition:             api.NewTurnoverRate(earlyAttritionCount, avgHeadcount),
		Target:                     api.TurnoverTarget{Value: q.Target, Operator: "lte", Source: "default"},
		MonthlyTrend:               queryTurnoverMonthlyTrend(ctx, db, startStr, endStr, periodDays),
		CohortRetention:            queryTurnoverCohortRetention(ctx, db, startStr, endStr),
		TerminationReasons:         reasons,
		Classification:             classification,
		ByEmploymentType:           byEmpType,
		ByLocation:                 byLoc,
		ByManager:                  byMgr,
		Warnings:                   []api.TurnoverWarning{},
	}, nil
}

type recruitmentFilterArgs struct {
	startStr, endStr  string
	deptFilter        any
	locFilter         any
	jobRolesFilter    any
	sourceTypesFilter any
	sourceIDFilter    any
}

func buildRecruitmentFilterArgs(q api.RecruitmentEffectivenessQuery) recruitmentFilterArgs {
	args := recruitmentFilterArgs{
		startStr: q.StartDate.Format("2006-01-02"),
		endStr:   q.EndDate.Format("2006-01-02"),
	}
	if len(q.DepartmentIDs) > 0 {
		args.deptFilter = q.DepartmentIDs
	}
	if len(q.LocationIDs) > 0 {
		args.locFilter = q.LocationIDs
	}
	if len(q.SourceIDs) > 0 {
		args.sourceIDFilter = q.SourceIDs
	}
	if len(q.JobRoles) > 0 {
		args.jobRolesFilter = q.JobRoles
	}
	if len(q.SourceTypes) > 0 {
		args.sourceTypesFilter = q.SourceTypes
	}
	return args
}

type recruitmentSourceAgg struct {
	sourceID      *uuid.UUID
	sourceType    string
	sourceName    string
	hires         int64
	totalCost     *decimal.Decimal
	costAvailable bool
	retained90d   int64
	turnover      int64
}

const (
	recruitmentHiresSQL = `SELECT
    rs.id AS source_id,
    COALESCE(rs.source_type::text, 'unknown') AS source_type,
    COALESCE(rs.name, 'Unknown source') AS source_name,
    COUNT(DISTINCT e.id)::bigint AS hires,
    SUM(
        DISTINCT CASE
            WHEN rs.source_type = 'agency' THEN rs.agency_fee_amount
            WHEN rs.source_type = 'referral' THEN rs.referral_bonus_amount
            ELSE NULL
        END
    )::numeric AS total_cost,
    BOOL_OR(
        (rs.source_type = 'agency' AND rs.agency_fee_amount IS NOT NULL)
        OR (rs.source_type = 'referral' AND rs.referral_bonus_amount IS NOT NULL)
    ) AS cost_available
FROM employee_tenures t
JOIN employees e ON e.id = t.employee_id
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
JOIN employment_relationships er ON er.tenure_id = t.id AND er.deleted_at IS NULL
JOIN employments emp
  ON emp.relationship_id = er.id
 AND emp.deleted_at IS NULL
 AND emp.start_date = t.start_date
WHERE e.deleted_at IS NULL
  AND t.deleted_at IS NULL
  AND t.start_date >= $1::date AND t.start_date <= $2::date
  AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
  AND ($4::uuid[] IS NULL OR emp.primary_location_id = ANY($4))
  AND ($5::text[] IS NULL OR emp.job_title = ANY($5))
  AND ($6::text[] IS NULL OR rs.source_type::text = ANY($6))
  AND ($7::uuid[] IS NULL OR e.recruitment_source_id = ANY($7))
GROUP BY rs.id, rs.source_type, rs.name
ORDER BY hires DESC, source_name ASC`

	recruitmentRetention90dSQL = `SELECT
    rs.id AS source_id,
    COALESCE(rs.source_type::text, 'unknown') AS source_type,
    COALESCE(rs.name, 'Unknown source') AS source_name,
    COUNT(DISTINCT t.id)::bigint AS retained_90d
FROM employee_tenures t
JOIN employees e ON e.id = t.employee_id
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
JOIN employment_relationships er ON er.tenure_id = t.id AND er.deleted_at IS NULL
JOIN employments emp
  ON emp.relationship_id = er.id
 AND emp.deleted_at IS NULL
 AND emp.start_date = t.start_date
WHERE e.deleted_at IS NULL
  AND t.deleted_at IS NULL
  AND t.start_date >= $1::date AND t.start_date <= $2::date
  AND (t.end_date IS NULL OR t.end_date > t.start_date + 90)
  AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
  AND ($4::uuid[] IS NULL OR emp.primary_location_id = ANY($4))
  AND ($5::text[] IS NULL OR emp.job_title = ANY($5))
  AND ($6::text[] IS NULL OR rs.source_type::text = ANY($6))
  AND ($7::uuid[] IS NULL OR e.recruitment_source_id = ANY($7))
GROUP BY rs.id, rs.source_type, rs.name`

	recruitmentTurnoverSQL = `SELECT
    rs.id AS source_id,
    COALESCE(rs.source_type::text, 'unknown') AS source_type,
    COALESCE(rs.name, 'Unknown source') AS source_name,
    COUNT(DISTINCT emp.employee_id)::bigint AS count
FROM employments emp
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
WHERE emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $1::date AND emp.end_date <= $2::date
  AND e.deleted_at IS NULL
  AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
  AND ($4::uuid[] IS NULL OR emp.primary_location_id = ANY($4))
  AND ($5::text[] IS NULL OR emp.job_title = ANY($5))
  AND ($6::text[] IS NULL OR rs.source_type::text = ANY($6))
  AND ($7::uuid[] IS NULL OR e.recruitment_source_id = ANY($7))
GROUP BY rs.id, rs.source_type, rs.name`
)

func queryRecruitmentHires(ctx context.Context, db DBTX, f recruitmentFilterArgs) (map[string]*recruitmentSourceAgg, int64, decimal.Decimal, int64, error) {
	hireRows, err := db.Query(ctx, recruitmentHiresSQL, f.startStr, f.endStr, f.deptFilter, f.locFilter, f.jobRolesFilter, f.sourceTypesFilter, f.sourceIDFilter)
	if err != nil {
		return nil, 0, decimal.Zero, 0, err
	}
	defer hireRows.Close()

	sourcesMap := make(map[string]*recruitmentSourceAgg)
	var totalHires, totalCostCount int64
	var totalCostSum decimal.Decimal
	for hireRows.Next() {
		var sID *uuid.UUID
		var sType, sName string
		var hires int64
		var cost *decimal.Decimal
		var costAvail *bool
		if err := hireRows.Scan(&sID, &sType, &sName, &hires, &cost, &costAvail); err != nil {
			return nil, 0, decimal.Zero, 0, err
		}
		totalHires += hires
		key := fmt.Sprintf("%v_%s", sID, sName)
		ca := costAvail != nil && *costAvail
		if ca && cost != nil {
			totalCostSum = totalCostSum.Add(*cost)
			totalCostCount += hires
		}
		sourcesMap[key] = &recruitmentSourceAgg{
			sourceID: sID, sourceType: sType, sourceName: sName,
			hires: hires, totalCost: cost, costAvailable: ca,
		}
	}
	return sourcesMap, totalHires, totalCostSum, totalCostCount, nil
}

func enrichRecruitmentSourceCount(ctx context.Context, db DBTX, sql string, f recruitmentFilterArgs, sourcesMap map[string]*recruitmentSourceAgg, assign func(*recruitmentSourceAgg, int64)) int64 {
	rows, _ := db.Query(ctx, sql, f.startStr, f.endStr, f.deptFilter, f.locFilter, f.jobRolesFilter, f.sourceTypesFilter, f.sourceIDFilter)
	if rows == nil {
		return 0
	}
	defer rows.Close()
	var total int64
	for rows.Next() {
		var sID *uuid.UUID
		var sType, sName string
		var count int64
		if err := rows.Scan(&sID, &sType, &sName, &count); err == nil {
			if sd, ok := sourcesMap[fmt.Sprintf("%v_%s", sID, sName)]; ok {
				assign(sd, count)
			}
			total += count
		}
	}
	return total
}

func buildRecruitmentSources(sourcesMap map[string]*recruitmentSourceAgg, totalHires int64) []api.RecruitmentEffectivenessSource {
	bySource := make([]api.RecruitmentEffectivenessSource, 0, len(sourcesMap))
	for _, sd := range sourcesMap {
		var hiresRatio, ret90d, cph *decimal.Decimal
		if totalHires > 0 {
			hr := decimal.NewFromInt(sd.hires).Div(decimal.NewFromInt(totalHires)).Mul(decimal.NewFromInt(100))
			hiresRatio = &hr
		}
		if sd.hires > 0 {
			r := decimal.NewFromInt(sd.retained90d).Div(decimal.NewFromInt(sd.hires)).Mul(decimal.NewFromInt(100))
			ret90d = &r
		}
		if sd.costAvailable && sd.totalCost != nil && sd.hires > 0 {
			c := sd.totalCost.Div(decimal.NewFromInt(sd.hires))
			cph = &c
		}
		bySource = append(bySource, api.RecruitmentEffectivenessSource{
			SourceID: sd.sourceID, SourceType: sd.sourceType, SourceName: sd.sourceName,
			Hires: sd.hires, HiresRatio: hiresRatio, Retained90d: sd.retained90d,
			Retention90d: ret90d, CostPerHire: cph, CostPerHireAvailable: sd.costAvailable,
			TurnoverInRange: sd.turnover,
		})
	}
	return bySource
}

func queryRecruitmentHiresTrend(ctx context.Context, db DBTX, f recruitmentFilterArgs, periodDays int) []api.RecruitmentTrendPoint {
	hiresTrend := make([]api.RecruitmentTrendPoint, 0)
	if periodDays < 60 {
		return hiresTrend
	}
	rows, _ := db.Query(ctx, `WITH months AS (
    SELECT generate_series(
        date_trunc('month', $1::date)::date,
        date_trunc('month', $2::date)::date,
        '1 month'::interval
    )::date AS m
),
hires AS (
    SELECT
        date_trunc('month', t.start_date)::date AS hire_month,
        t.id
    FROM employee_tenures t
    JOIN employees e ON e.id = t.employee_id
    LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
    JOIN employment_relationships er ON er.tenure_id = t.id AND er.deleted_at IS NULL
    JOIN employments emp
      ON emp.relationship_id = er.id
     AND emp.deleted_at IS NULL
     AND emp.start_date = t.start_date
    WHERE e.deleted_at IS NULL
      AND t.deleted_at IS NULL
      AND t.start_date >= $1::date AND t.start_date <= $2::date
      AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
      AND ($4::uuid[] IS NULL OR emp.primary_location_id = ANY($4))
      AND ($5::text[] IS NULL OR emp.job_title = ANY($5))
      AND ($6::text[] IS NULL OR rs.source_type::text = ANY($6))
      AND ($7::uuid[] IS NULL OR e.recruitment_source_id = ANY($7))
)
SELECT
    to_char(m.m, 'YYYY-MM') AS month,
    COUNT(DISTINCT h.id)::bigint AS hires
FROM months m
LEFT JOIN hires h ON h.hire_month = m.m
GROUP BY m.m
ORDER BY m.m ASC`, f.startStr, f.endStr, f.deptFilter, f.locFilter, f.jobRolesFilter, f.sourceTypesFilter, f.sourceIDFilter)
	if rows == nil {
		return hiresTrend
	}
	defer rows.Close()
	for rows.Next() {
		var p api.RecruitmentTrendPoint
		if err := rows.Scan(&p.Month, &p.Hires); err == nil {
			hiresTrend = append(hiresTrend, p)
		}
	}
	return hiresTrend
}

func (r DashboardAnalyticsRepository) GetRecruitmentEffectiveness(ctx context.Context, q api.RecruitmentEffectivenessQuery) (api.RecruitmentEffectivenessAnalytics, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.RecruitmentEffectivenessAnalytics{}, err
	}

	f := buildRecruitmentFilterArgs(q)
	sourcesMap, totalHires, totalCostSum, totalCostCount, err := queryRecruitmentHires(ctx, db, f)
	if err != nil {
		return api.RecruitmentEffectivenessAnalytics{}, err
	}
	totalRetained90d := enrichRecruitmentSourceCount(ctx, db, recruitmentRetention90dSQL, f, sourcesMap, func(sd *recruitmentSourceAgg, n int64) { sd.retained90d = n })
	totalTurnover := enrichRecruitmentSourceCount(ctx, db, recruitmentTurnoverSQL, f, sourcesMap, func(sd *recruitmentSourceAgg, n int64) { sd.turnover = n })
	periodDays := int(q.EndDate.Sub(q.StartDate).Hours()/24) + 1

	var overallRetention, avgCostPerHire *decimal.Decimal
	if totalHires > 0 {
		r := decimal.NewFromInt(totalRetained90d).Div(decimal.NewFromInt(totalHires)).Mul(decimal.NewFromInt(100))
		overallRetention = &r
	}
	if totalCostCount > 0 {
		ac := totalCostSum.Div(decimal.NewFromInt(totalCostCount))
		avgCostPerHire = &ac
	}

	return api.RecruitmentEffectivenessAnalytics{
		PeriodStart: f.startStr,
		PeriodEnd:   f.endStr,
		TotalHires:  totalHires,
		Overview: api.RecruitmentEffectivenessOverview{
			Hires:              totalHires,
			AverageCostPerHire: avgCostPerHire,
			CostAvailable:      totalCostCount > 0,
			Retained90d:        totalRetained90d,
			Retention90d:       overallRetention,
			TurnoverInRange:    totalTurnover,
		},
		Funnel:     api.DefaultRecruitmentFunnel(totalHires, totalRetained90d),
		BySource:   buildRecruitmentSources(sourcesMap, totalHires),
		HiresTrend: queryRecruitmentHiresTrend(ctx, db, f, periodDays),
		DataGaps:   api.DefaultRecruitmentDataGaps(),
		Warnings:   []api.TurnoverWarning{},
	}, nil
}
