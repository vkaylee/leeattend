package repository

import (
	"context"
	"fmt"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func (r DashboardAnalyticsRepository) GetTurnoverAnalytics(ctx context.Context, q api.TurnoverAnalyticsQuery) (api.TurnoverAnalytics, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.TurnoverAnalytics{}, err
	}

	startStr := q.StartDate.Format("2006-01-02")
	endStr := q.EndDate.Format("2006-01-02")

	var totalTerminated int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT emp.employee_id)::bigint FROM employments emp
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date`, startStr, endStr).Scan(&totalTerminated)

	tenureRows, _ := db.Query(ctx, `SELECT bucket, COUNT(*)::bigint AS count FROM (
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
	END`, startStr, endStr)
	tenureDist := make([]api.TurnoverTenureBucket, 0)
	if tenureRows != nil {
		defer tenureRows.Close()
		for tenureRows.Next() {
			var b api.TurnoverTenureBucket
			if err := tenureRows.Scan(&b.Bucket, &b.Count); err == nil {
				tenureDist = append(tenureDist, b)
			}
		}
	}

	sourceRows, _ := db.Query(ctx, `SELECT COALESCE(rs.source_type::text, 'direct'), COALESCE(rs.name, 'Direct Hire'), COUNT(*)::bigint
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY rs.source_type, rs.name ORDER BY count DESC`, startStr, endStr)
	bySource := make([]api.TurnoverBySource, 0)
	if sourceRows != nil {
		defer sourceRows.Close()
		for sourceRows.Next() {
			var s api.TurnoverBySource
			if err := sourceRows.Scan(&s.SourceType, &s.SourceName, &s.Count); err == nil {
				bySource = append(bySource, s)
			}
		}
	}

	deptRows, _ := db.Query(ctx, `SELECT COALESCE(d.name, 'Unassigned'), COUNT(*)::bigint
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN departments d ON d.id = emp.department_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY d.name ORDER BY count DESC`, startStr, endStr)
	byDept := make([]api.TurnoverByDepartment, 0)
	if deptRows != nil {
		defer deptRows.Close()
		for deptRows.Next() {
			var d api.TurnoverByDepartment
			if err := deptRows.Scan(&d.DepartmentName, &d.Count); err == nil {
				byDept = append(byDept, d)
			}
		}
	}

	classRows, _ := db.Query(ctx, `SELECT CASE
			WHEN emp.termination_reason IN ('voluntary_resignation', 'retirement', 'mutual_agreement') THEN 'voluntary'
			WHEN emp.termination_reason IN ('involuntary_termination', 'abandonment', 'death') THEN 'involuntary'
			ELSE 'unknown'
		END AS classification,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY classification`, startStr, endStr)
	var voluntaryCount, involuntaryCount, unknownClassCount int64
	classification := make([]api.TurnoverClassification, 0)
	if classRows != nil {
		defer classRows.Close()
		for classRows.Next() {
			var c api.TurnoverClassification
			if err := classRows.Scan(&c.Classification, &c.Count); err == nil {
				classification = append(classification, c)
				switch c.Classification {
				case "voluntary":
					voluntaryCount = c.Count
				case "involuntary":
					involuntaryCount = c.Count
				case "unknown":
					unknownClassCount = c.Count
				}
			}
		}
	}

	reasonRows, _ := db.Query(ctx, `SELECT COALESCE(emp.termination_reason, 'unknown')::text AS reason,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY emp.termination_reason ORDER BY count DESC`, startStr, endStr)
	reasons := make([]api.TurnoverReasonRow, 0)
	if reasonRows != nil {
		defer reasonRows.Close()
		for reasonRows.Next() {
			var r api.TurnoverReasonRow
			if err := reasonRows.Scan(&r.Reason, &r.Count); err == nil {
				reasons = append(reasons, r)
			}
		}
	}

	empTypeRows, _ := db.Query(ctx, `SELECT COALESCE(emp.employment_type, 'unknown')::text AS employment_type,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY emp.employment_type ORDER BY count DESC`, startStr, endStr)
	byEmpType := make([]api.TurnoverByEmploymentType, 0)
	if empTypeRows != nil {
		defer empTypeRows.Close()
		for empTypeRows.Next() {
			var et api.TurnoverByEmploymentType
			if err := empTypeRows.Scan(&et.EmploymentType, &et.Count); err == nil {
				byEmpType = append(byEmpType, et)
			}
		}
	}

	locRows, _ := db.Query(ctx, `SELECT COALESCE(l.name, 'Unassigned')::text AS location_name,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN work_locations l ON l.id = emp.primary_location_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY l.name ORDER BY count DESC`, startStr, endStr)
	byLoc := make([]api.TurnoverByLocation, 0)
	if locRows != nil {
		defer locRows.Close()
		for locRows.Next() {
			var l api.TurnoverByLocation
			if err := locRows.Scan(&l.LocationName, &l.Count); err == nil {
				byLoc = append(byLoc, l)
			}
		}
	}

	mgrRows, _ := db.Query(ctx, `SELECT COALESCE(mgr_e.full_name, 'Unassigned')::text AS manager_name,
		COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		LEFT JOIN employments mgr_emp ON mgr_emp.id = emp.manager_employment_id
		LEFT JOIN employees mgr_e ON mgr_e.id = mgr_emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		GROUP BY mgr_e.full_name ORDER BY count DESC`, startStr, endStr)
	byMgr := make([]api.TurnoverByManager, 0)
	if mgrRows != nil {
		defer mgrRows.Close()
		for mgrRows.Next() {
			var m api.TurnoverByManager
			if err := mgrRows.Scan(&m.ManagerName, &m.Count); err == nil {
				byMgr = append(byMgr, m)
			}
		}
	}

	var avgHeadcount, headcountStart int64
	_ = db.QueryRow(ctx, `WITH months AS (
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
) hc ON true`, startStr, endStr).Scan(&avgHeadcount, &headcountStart)

	var earlyAttritionCount int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT emp.employee_id)::bigint AS count
		FROM employees e
		JOIN employments emp ON e.id = emp.employee_id
		WHERE emp.deleted_at IS NULL AND emp.status = 'terminated'
		  AND emp.end_date IS NOT NULL AND emp.end_date >= $1::date AND emp.end_date <= $2::date
		  AND (emp.end_date - emp.start_date) < 90`, startStr, endStr).Scan(&earlyAttritionCount)

	periodDays := int(q.EndDate.Sub(q.StartDate).Hours()/24) + 1
	monthlyTrend := make([]api.TurnoverTrendPoint, 0)
	if periodDays >= 60 {
		trendRows, _ := db.Query(ctx, `WITH months AS (
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
		if trendRows != nil {
			defer trendRows.Close()
			for trendRows.Next() {
				var month string
				var hc, lv int64
				if err := trendRows.Scan(&month, &hc, &lv); err == nil {
					rate := api.NewTurnoverRate(lv, hc)
					monthlyTrend = append(monthlyTrend, api.TurnoverTrendPoint{
						Month:        month,
						Headcount:    hc,
						Leavers:      lv,
						TurnoverRate: &rate,
					})
				}
			}
		}
	}

	cohortRows, _ := db.Query(ctx, `WITH cohort AS (
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
	cohortRetention := make([]api.CohortRetentionRow, 0)
	if cohortRows != nil {
		defer cohortRows.Close()
		for cohortRows.Next() {
			var c api.CohortRetentionRow
			if err := cohortRows.Scan(&c.CohortMonth, &c.MonthsSinceHire, &c.CohortSize, &c.ActiveCount); err == nil {
				if c.CohortSize > 0 {
					ret := decimal.NewFromInt(c.ActiveCount).Div(decimal.NewFromInt(c.CohortSize)).Mul(decimal.NewFromInt(100))
					c.RetentionRate = &ret
				}
				cohortRetention = append(cohortRetention, c)
			}
		}
	}

	var retentionRate *decimal.Decimal
	if headcountStart > 0 {
		activeAtEnd := headcountStart - totalTerminated
		if activeAtEnd < 0 {
			activeAtEnd = 0
		}
		ret := decimal.NewFromInt(activeAtEnd).Div(decimal.NewFromInt(headcountStart)).Mul(decimal.NewFromInt(100))
		retentionRate = &ret
	}

	return api.TurnoverAnalytics{
		TenureDistribution:         tenureDist,
		BySource:                   bySource,
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
		RegrettableAttrition:       nil,
		Comparison:                 nil,
		Target: api.TurnoverTarget{
			Value:    q.Target,
			Operator: "lte",
			Source:   "default",
		},
		MonthlyTrend:       monthlyTrend,
		CohortRetention:    cohortRetention,
		TerminationReasons: reasons,
		Classification:     classification,
		ByEmploymentType:   byEmpType,
		ByLocation:         byLoc,
		ByManager:          byMgr,
		Warnings:           []api.TurnoverWarning{},
	}, nil
}

func (r DashboardAnalyticsRepository) GetRecruitmentEffectiveness(ctx context.Context, q api.RecruitmentEffectivenessQuery) (api.RecruitmentEffectivenessAnalytics, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.RecruitmentEffectivenessAnalytics{}, err
	}

	startStr := q.StartDate.Format("2006-01-02")
	endStr := q.EndDate.Format("2006-01-02")

	var deptFilter, locFilter, sourceIDFilter any
	if len(q.DepartmentIDs) > 0 {
		deptFilter = q.DepartmentIDs
	}
	if len(q.LocationIDs) > 0 {
		locFilter = q.LocationIDs
	}
	if len(q.SourceIDs) > 0 {
		sourceIDFilter = q.SourceIDs
	}
	var jobRolesFilter, sourceTypesFilter any
	if len(q.JobRoles) > 0 {
		jobRolesFilter = q.JobRoles
	}
	if len(q.SourceTypes) > 0 {
		sourceTypesFilter = q.SourceTypes
	}

	hireQuery := `SELECT
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

	hireRows, err := db.Query(ctx, hireQuery, startStr, endStr, deptFilter, locFilter, jobRolesFilter, sourceTypesFilter, sourceIDFilter)
	if err != nil {
		return api.RecruitmentEffectivenessAnalytics{}, err
	}
	defer hireRows.Close()

	type sourceData struct {
		sourceID      *uuid.UUID
		sourceType    string
		sourceName    string
		hires         int64
		totalCost     *decimal.Decimal
		costAvailable bool
		retained90d   int64
		turnover      int64
	}

	sourcesMap := make(map[string]*sourceData)
	var totalHires int64
	var totalCostSum decimal.Decimal
	var totalCostCount int64

	for hireRows.Next() {
		var sID *uuid.UUID
		var sType, sName string
		var hires int64
		var cost *decimal.Decimal
		var costAvail *bool

		if err := hireRows.Scan(&sID, &sType, &sName, &hires, &cost, &costAvail); err != nil {
			return api.RecruitmentEffectivenessAnalytics{}, err
		}
		totalHires += hires
		key := fmt.Sprintf("%v_%s", sID, sName)
		ca := costAvail != nil && *costAvail
		if ca && cost != nil {
			totalCostSum = totalCostSum.Add(*cost)
			totalCostCount += hires
		}
		sourcesMap[key] = &sourceData{
			sourceID:      sID,
			sourceType:    sType,
			sourceName:    sName,
			hires:         hires,
			totalCost:     cost,
			costAvailable: ca,
		}
	}

	retRows, _ := db.Query(ctx, `SELECT
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
GROUP BY rs.id, rs.source_type, rs.name`, startStr, endStr, deptFilter, locFilter, jobRolesFilter, sourceTypesFilter, sourceIDFilter)
	var totalRetained90d int64
	if retRows != nil {
		defer retRows.Close()
		for retRows.Next() {
			var sID *uuid.UUID
			var sType, sName string
			var retained int64
			if err := retRows.Scan(&sID, &sType, &sName, &retained); err == nil {
				key := fmt.Sprintf("%v_%s", sID, sName)
				if sd, ok := sourcesMap[key]; ok {
					sd.retained90d = retained
				}
				totalRetained90d += retained
			}
		}
	}

	turnRows, _ := db.Query(ctx, `SELECT
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
GROUP BY rs.id, rs.source_type, rs.name`, startStr, endStr, deptFilter, locFilter, jobRolesFilter, sourceTypesFilter, sourceIDFilter)
	var totalTurnover int64
	if turnRows != nil {
		defer turnRows.Close()
		for turnRows.Next() {
			var sID *uuid.UUID
			var sType, sName string
			var count int64
			if err := turnRows.Scan(&sID, &sType, &sName, &count); err == nil {
				key := fmt.Sprintf("%v_%s", sID, sName)
				if sd, ok := sourcesMap[key]; ok {
					sd.turnover = count
				}
				totalTurnover += count
			}
		}
	}

	bySource := make([]api.RecruitmentEffectivenessSource, 0, len(sourcesMap))
	for _, sd := range sourcesMap {
		var hiresRatio *decimal.Decimal
		if totalHires > 0 {
			hr := decimal.NewFromInt(sd.hires).Div(decimal.NewFromInt(totalHires)).Mul(decimal.NewFromInt(100))
			hiresRatio = &hr
		}
		var ret90d *decimal.Decimal
		if sd.hires > 0 {
			r := decimal.NewFromInt(sd.retained90d).Div(decimal.NewFromInt(sd.hires)).Mul(decimal.NewFromInt(100))
			ret90d = &r
		}
		var cph *decimal.Decimal
		if sd.costAvailable && sd.totalCost != nil && sd.hires > 0 {
			c := sd.totalCost.Div(decimal.NewFromInt(sd.hires))
			cph = &c
		}
		bySource = append(bySource, api.RecruitmentEffectivenessSource{
			SourceID:             sd.sourceID,
			SourceType:           sd.sourceType,
			SourceName:           sd.sourceName,
			Hires:                sd.hires,
			HiresRatio:           hiresRatio,
			Retained90d:          sd.retained90d,
			Retention90d:         ret90d,
			CostPerHire:          cph,
			CostPerHireAvailable: sd.costAvailable,
			TurnoverInRange:      sd.turnover,
		})
	}

	periodDays := int(q.EndDate.Sub(q.StartDate).Hours()/24) + 1
	hiresTrend := make([]api.RecruitmentTrendPoint, 0)
	if periodDays >= 60 {
		trendRows, _ := db.Query(ctx, `WITH months AS (
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
ORDER BY m.m ASC`, startStr, endStr, deptFilter, locFilter, jobRolesFilter, sourceTypesFilter, sourceIDFilter)
		if trendRows != nil {
			defer trendRows.Close()
			for trendRows.Next() {
				var p api.RecruitmentTrendPoint
				if err := trendRows.Scan(&p.Month, &p.Hires); err == nil {
					hiresTrend = append(hiresTrend, p)
				}
			}
		}
	}

	var overallRetention *decimal.Decimal
	if totalHires > 0 {
		r := decimal.NewFromInt(totalRetained90d).Div(decimal.NewFromInt(totalHires)).Mul(decimal.NewFromInt(100))
		overallRetention = &r
	}

	var avgCostPerHire *decimal.Decimal
	if totalCostCount > 0 {
		ac := totalCostSum.Div(decimal.NewFromInt(totalCostCount))
		avgCostPerHire = &ac
	}

	return api.RecruitmentEffectivenessAnalytics{
		PeriodStart: startStr,
		PeriodEnd:   endStr,
		TotalHires:  totalHires,
		Overview: api.RecruitmentEffectivenessOverview{
			Hires:                totalHires,
			AverageCostPerHire:   avgCostPerHire,
			CostAvailable:        totalCostCount > 0,
			Retained90d:          totalRetained90d,
			Retention90d:         overallRetention,
			TurnoverInRange:      totalTurnover,
			ComparisonHires:      nil,
			ComparisonHiresDelta: nil,
		},
		Funnel:     api.DefaultRecruitmentFunnel(totalHires, totalRetained90d),
		BySource:   bySource,
		HiresTrend: hiresTrend,
		DataGaps:   api.DefaultRecruitmentDataGaps(),
		Warnings:   []api.TurnoverWarning{},
	}, nil
}
