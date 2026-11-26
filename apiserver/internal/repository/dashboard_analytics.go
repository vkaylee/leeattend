package repository

import (
	"context"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/timezone"
	"math"
	"time"
)

type DashboardAnalyticsRepository struct {
	DB DBTX
}

func (r DashboardAnalyticsRepository) GetOrganizationTimezone(ctx context.Context) (string, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return "UTC", err
	}
	var tz string
	err = db.QueryRow(ctx, `SELECT COALESCE(timezone::text, 'UTC') FROM organization_settings LIMIT 1`).Scan(&tz)
	if err != nil || tz == "" {
		return "UTC", nil
	}
	return tz, nil
}

func (r DashboardAnalyticsRepository) GetOvertimeTrend(ctx context.Context, today time.Time) ([]api.OvertimeTrendPoint, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	todayStr := today.Format("2006-01-02")
	rows, err := db.Query(ctx, `SELECT ads.date::text,
		COALESCE(SUM(ads.system_detected_ot_minutes), 0)::bigint AS system_ot_minutes,
		COALESCE(SUM(ads.approved_ot_minutes), 0)::bigint AS approved_ot_minutes
	FROM attendance_daily_summaries ads
	WHERE ads.date >= ($1::date - INTERVAL '29 days')
	  AND ads.date <= $1::date
	GROUP BY ads.date
	ORDER BY ads.date ASC`, todayStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	points := make([]api.OvertimeTrendPoint, 0)
	for rows.Next() {
		var date string
		var sysMin, appMin int64
		if err := rows.Scan(&date, &sysMin, &appMin); err != nil {
			return nil, err
		}
		points = append(points, api.OvertimeTrendPoint{
			Date:            date,
			SystemOTHours:   float64(sysMin) / 60.0,
			ApprovedOTHours: float64(appMin) / 60.0,
		})
	}
	return points, rows.Err()
}

func (r DashboardAnalyticsRepository) GetPeakTimes(ctx context.Context, today time.Time, tzName string) ([]api.PeakTimeSlot, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	loc := today.Location()
	utcStart := timezone.StartOfDayUTC(loc, today)
	utcEndExclusive := timezone.StartOfNextDayUTC(loc, today)

	query := `WITH today_entries AS (
    SELECT te.punch_in_time, te.punch_out_time
    FROM time_entries te
    JOIN employment_segments emp ON te.employment_id = emp.id AND emp.deleted_at IS NULL
    WHERE te.punch_in_time >= $1
      AND te.punch_in_time < $2
      AND emp.status = 'active'
      AND $4::date BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
),
check_ins AS (
    SELECT
        (EXTRACT(HOUR FROM (punch_in_time AT TIME ZONE $3))::int * 2
         + FLOOR(EXTRACT(MINUTE FROM (punch_in_time AT TIME ZONE $3)) / 30)::int) AS slot_index,
        COUNT(*) AS check_in
    FROM today_entries
    GROUP BY slot_index
),
check_outs AS (
    SELECT
        (EXTRACT(HOUR FROM (punch_out_time AT TIME ZONE $3))::int * 2
         + FLOOR(EXTRACT(MINUTE FROM (punch_out_time AT TIME ZONE $3)) / 30)::int) AS slot_index,
        COUNT(*) AS check_out
    FROM today_entries
    WHERE punch_out_time IS NOT NULL
    GROUP BY slot_index
),
slots AS (
    SELECT generate_series(0, 47) AS slot_index
),
slot_labels AS (
    SELECT slot_index,
           TO_CHAR((TIME '00:00' + (slot_index * INTERVAL '30 minutes')), 'HH24:MI')
           || '-' ||
           TO_CHAR((TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')), 'HH24:MI')
           AS time_slot
    FROM slots
)
SELECT sl.time_slot::text,
       COALESCE(ci.check_in, 0)::bigint AS check_in,
       COALESCE(co.check_out, 0)::bigint AS check_out
FROM slot_labels sl
LEFT JOIN check_ins ci ON sl.slot_index = ci.slot_index
LEFT JOIN check_outs co ON sl.slot_index = co.slot_index
WHERE COALESCE(ci.check_in, 0) > 0 OR COALESCE(co.check_out, 0) > 0
ORDER BY sl.slot_index`

	rows, err := db.Query(ctx, query, utcStart, utcEndExclusive, tzName, today.Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	slots := make([]api.PeakTimeSlot, 0)
	for rows.Next() {
		var s api.PeakTimeSlot
		if err := rows.Scan(&s.TimeSlot, &s.CheckIn, &s.CheckOut); err != nil {
			return nil, err
		}
		slots = append(slots, s)
	}
	return slots, rows.Err()
}

func (r DashboardAnalyticsRepository) GetLeaveTypesBreakdown(ctx context.Context, today time.Time) ([]api.LeaveTypeBreakdown, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	monthStart := time.Date(today.Year(), today.Month(), 1, 0, 0, 0, 0, today.Location())
	query := `SELECT COALESCE(lt.name, 'Other')::text AS leave_type,
       COUNT(*)::bigint AS count
FROM leave_requests lr
LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
JOIN employment_segments emp ON lr.employment_id = emp.id
WHERE lr.status IN ('approved', 'pre_approved')
  AND lr.start_date <= $2::date
  AND lr.end_date >= $1::date
  AND emp.deleted_at IS NULL
  AND emp.status = 'active'
  AND $3::date BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
GROUP BY COALESCE(lt.name, 'Other')
ORDER BY count DESC`

	rows, err := db.Query(ctx, query, monthStart.Format("2006-01-02"), today.Format("2006-01-02"), today.Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	breakdown := make([]api.LeaveTypeBreakdown, 0)
	for rows.Next() {
		var item api.LeaveTypeBreakdown
		if err := rows.Scan(&item.LeaveType, &item.Count); err != nil {
			return nil, err
		}
		breakdown = append(breakdown, item)
	}
	return breakdown, rows.Err()
}

func (r DashboardAnalyticsRepository) GetOvertimeBreakdown(ctx context.Context, q api.OvertimeBreakdownQuery, allowOvertimeCost bool) (api.OvertimeBreakdownResponse, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.OvertimeBreakdownResponse{}, err
	}

	startStr := q.StartDate.Format("2006-01-02")
	endStr := q.EndDate.Format("2006-01-02")

	var deptFilter any
	if len(q.DeptIDs) > 0 {
		deptFilter = q.DeptIDs
	}

	query := `WITH depts AS (
    SELECT d.id AS department_id, d.name AS department_name
    FROM departments d
    WHERE d.deleted_at IS NULL
),
hours AS (
    SELECT emp.department_id,
           COALESCE(SUM(ads.system_detected_ot_minutes), 0)::bigint AS system_ot_minutes,
           COALESCE(SUM(ads.approved_ot_minutes), 0)::bigint         AS approved_ot_minutes
    FROM attendance_daily_summaries ads
    JOIN employment_segments emp ON emp.id = ads.employment_id
      AND emp.deleted_at IS NULL
      AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
    WHERE ads.date BETWEEN $1::date AND $2::date
    GROUP BY emp.department_id
),
cost AS (
    SELECT emp.department_id,
           CASE WHEN $4::boolean THEN COALESCE(SUM(ps.overtime_pay_amount), 0) END AS overtime_cost,
           CASE WHEN $4::boolean THEN COUNT(ps.id) END                              AS payslip_count
    FROM payslips ps
    JOIN employment_segments emp ON emp.id = ps.employment_id
      AND emp.deleted_at IS NULL
      AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
    WHERE ps.status IN ('paid', 'finalized')
      AND ps.period_start <= $2::date
      AND ps.period_end >= $1::date
    GROUP BY emp.department_id
)
SELECT d.department_id,
       d.department_name,
       COALESCE(h.system_ot_minutes, 0)  AS system_ot_minutes,
       COALESCE(h.approved_ot_minutes, 0) AS approved_ot_minutes,
       c.overtime_cost,
       CASE WHEN $4::boolean AND c.payslip_count IS NOT NULL AND c.payslip_count > 0
            THEN TRUE ELSE FALSE END AS cost_available
FROM depts d
LEFT JOIN hours h ON h.department_id = d.department_id
LEFT JOIN cost c  ON c.department_id = d.department_id
WHERE h.department_id IS NOT NULL
   OR (c.department_id IS NOT NULL AND $4::boolean)
ORDER BY COALESCE(h.system_ot_minutes, 0) DESC`

	rows, err := db.Query(ctx, query, startStr, endStr, deptFilter, allowOvertimeCost)
	if err != nil {
		return api.OvertimeBreakdownResponse{}, err
	}
	defer rows.Close()

	breakdownRows := make([]api.OvertimeBreakdownRow, 0)
	for rows.Next() {
		var deptID uuid.UUID
		var deptName string
		var sysMin, appMin int64
		var cost *decimal.Decimal
		var costAvail bool

		if err := rows.Scan(&deptID, &deptName, &sysMin, &appMin, &cost, &costAvail); err != nil {
			return api.OvertimeBreakdownResponse{}, err
		}

		sysH := float64(sysMin) / 60.0
		appH := float64(appMin) / 60.0
		unappH := math.Max(0, float64(sysMin-appMin)/60.0)

		if !allowOvertimeCost {
			cost = nil
			costAvail = false
		}

		breakdownRows = append(breakdownRows, api.OvertimeBreakdownRow{
			DepartmentID:      deptID,
			DepartmentName:    deptName,
			SystemOTHours:     sysH,
			ApprovedOTHours:   appH,
			UnapprovedOTHours: unappH,
			OvertimeCost:      cost,
			CostAvailable:     costAvail,
		})
	}
	if err := rows.Err(); err != nil {
		return api.OvertimeBreakdownResponse{}, err
	}

	var currency string
	if err := db.QueryRow(ctx, `SELECT COALESCE((
		SELECT currency_code
		FROM employers
		WHERE deleted_at IS NULL AND currency_code IS NOT NULL
		ORDER BY created_at
		LIMIT 1
	), 'USD')`).Scan(&currency); err != nil {
		return api.OvertimeBreakdownResponse{}, err
	}

	tzName := "UTC"
	if q.Timezone != nil && *q.Timezone != "" {
		tzName = *q.Timezone
	}

	return api.OvertimeBreakdownResponse{
		ContractVersion: "overtime-breakdown.v2",
		Organization: api.DashboardOrganizationContext{
			Timezone: tzName,
			Currency: currency,
		},
		Rows:        breakdownRows,
		GeneratedAt: time.Now().UTC().Format(time.RFC3339),
	}, nil
}

func (r DashboardAnalyticsRepository) GetDashboardStats(ctx context.Context, tzName string, deptIDs []uuid.UUID) (api.DashboardStats, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return api.DashboardStats{}, err
	}
	loc := timezone.ParseTimezone(tzName)
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)
	todayStr := today.Format("2006-01-02")
	utcStart := timezone.StartOfDayUTC(loc, today)
	utcEndExclusive := timezone.StartOfNextDayUTC(loc, today)

	var deptFilter any
	if len(deptIDs) > 0 {
		deptFilter = deptIDs
	}

	var totalEmp int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT employee_id)::bigint FROM employment_segments
		WHERE status = 'active' AND deleted_at IS NULL
		  AND start_date <= $1::date AND (end_date IS NULL OR end_date >= $1::date)
		  AND ($2::uuid[] IS NULL OR department_id = ANY($2))`, todayStr, deptFilter).Scan(&totalEmp)

	var presentToday int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT emp.employee_id)::bigint FROM time_entries te
		JOIN employment_segments emp ON emp.id = te.employment_id
		JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
		WHERE te.punch_in_time >= $1 AND te.punch_in_time < $2
		  AND emp.status = 'active' AND emp.deleted_at IS NULL
		  AND emp.start_date <= $3::date AND (emp.end_date IS NULL OR emp.end_date >= $3::date)
		  AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))`, utcStart, utcEndExclusive, todayStr, deptFilter).Scan(&presentToday)

	var pendingLeave int64
	_ = db.QueryRow(ctx, `SELECT COUNT(*)::bigint FROM leave_requests lr
		JOIN employment_segments emp ON emp.id = lr.employment_id
		JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
		WHERE lr.status = 'pending' AND emp.status = 'active'
		  AND emp.deleted_at IS NULL AND emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
		  AND ($2::uuid[] IS NULL OR emp.department_id = ANY($2))`, todayStr, deptFilter).Scan(&pendingLeave)

	var onLeaveToday int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT emp.employee_id)::bigint FROM leave_requests lr
		JOIN employment_segments emp ON emp.id = lr.employment_id
		JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
		WHERE lr.status IN ('approved', 'pre_approved') AND lr.start_date <= $1::date AND lr.end_date >= $1::date
		  AND emp.status = 'active' AND emp.deleted_at IS NULL
		  AND emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
		  AND ($2::uuid[] IS NULL OR emp.department_id = ANY($2))`, todayStr, deptFilter).Scan(&onLeaveToday)

	var payroll decimal.Decimal
	_ = db.QueryRow(ctx, `WITH active_employments AS (
    SELECT emp.id AS employment_id,
           ROW_NUMBER() OVER (
               PARTITION BY emp.employee_id
               ORDER BY emp.start_date DESC, emp.id DESC
           ) AS employment_rank
    FROM employment_segments emp
    JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
    WHERE emp.status = 'active' AND emp.deleted_at IS NULL
      AND emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
),
latest_components AS (
    SELECT esc.employment_id, esc.amount,
           ROW_NUMBER() OVER (
               PARTITION BY esc.employment_id, esc.component_definition_id
               ORDER BY esc.effective_date DESC
           ) AS component_rank
    FROM employee_salary_components esc
    JOIN active_employments ae ON ae.employment_id = esc.employment_id AND ae.employment_rank = 1
    WHERE esc.effective_date <= $1::date
)
SELECT COALESCE(SUM(amount), 0)::numeric FROM latest_components WHERE component_rank = 1`, todayStr).Scan(&payroll)

	var avgWorkMinutes float64
	_ = db.QueryRow(ctx, `SELECT COALESCE(AVG(te.duration_minutes), 0)::float8 FROM time_entries te
		JOIN employment_segments emp ON emp.id = te.employment_id
		JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
		WHERE te.punch_in_time >= $1 AND te.punch_in_time < $2
		  AND te.punch_out_time IS NOT NULL AND te.duration_minutes > 0
		  AND emp.status = 'active' AND emp.deleted_at IS NULL
		  AND emp.start_date <= $3::date AND (emp.end_date IS NULL OR emp.end_date >= $3::date)
		  AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))`, utcStart, utcEndExclusive, todayStr, deptFilter).Scan(&avgWorkMinutes)

	var lateToday, earlyDepToday int64
	_ = db.QueryRow(ctx, `SELECT
		COALESCE(SUM(CASE WHEN ads.late_minutes > 0 THEN 1 ELSE 0 END), 0)::bigint,
		COALESCE(SUM(CASE WHEN ads.early_departure_minutes > 0 THEN 1 ELSE 0 END), 0)::bigint
	FROM attendance_daily_summaries ads
	JOIN employment_segments emp ON emp.id = ads.employment_id AND emp.deleted_at IS NULL
	  AND ($2::uuid[] IS NULL OR emp.department_id = ANY($2))
	WHERE ads.date = $1::date`, todayStr, deptFilter).Scan(&lateToday, &earlyDepToday)

	absentToday := max64(0, totalEmp-presentToday-onLeaveToday)
	avgHours := math.Round((avgWorkMinutes/60.0)*10.0) / 10.0

	return api.DashboardStats{
		TotalEmployees:          totalEmp,
		PresentToday:            presentToday,
		PendingLeaveRequests:    pendingLeave,
		EstimatedMonthlyPayroll: payroll,
		LateToday:               lateToday,
		AbsentToday:             absentToday,
		OnLeaveToday:            onLeaveToday,
		EarlyDepartureToday:     earlyDepToday,
		AvgWorkHoursToday:       avgHours,
		AttendanceRate:          api.NewRateQuantity(presentToday, totalEmp, "employees"),
		AbsenceRate:             api.NewRateQuantity(absentToday, totalEmp, "employees"),
		LateRate:                api.NewRateQuantity(lateToday, totalEmp, "employees"),
	}, nil
}

func (r DashboardAnalyticsRepository) GetAttendanceTrend(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]api.AttendanceTrendPoint, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	loc := timezone.ParseTimezone(tzName)
	utcStart := timezone.StartOfDayUTC(loc, startDate)
	utcEndExclusive := timezone.StartOfNextDayUTC(loc, endDate)

	var deptFilter any
	if len(deptIDs) > 0 {
		deptFilter = deptIDs
	}

	query := `WITH days AS (
    SELECT d::date AS date
    FROM generate_series($1::date, $2::date, '1 day'::interval) AS d
),
daily_active AS (
    SELECT d.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS active_employees
    FROM days d
    JOIN employment_segments emp
      ON emp.status = 'active'
     AND emp.deleted_at IS NULL
     AND emp.start_date <= d.date
     AND (emp.end_date IS NULL OR emp.end_date >= d.date)
     AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.deleted_at IS NULL
    GROUP BY d.date
),
daily_present AS (
    SELECT d.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS present
    FROM days d
    JOIN time_entries te
      ON te.punch_in_time >= $4
     AND te.punch_in_time < $5
     AND (te.punch_in_time AT TIME ZONE $6)::date = d.date
    JOIN employment_segments emp
      ON emp.id = te.employment_id
     AND emp.status = 'active'
     AND emp.deleted_at IS NULL
     AND emp.start_date <= d.date
     AND (emp.end_date IS NULL OR emp.end_date >= d.date)
     AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.deleted_at IS NULL
    GROUP BY d.date
),
daily_leave AS (
    SELECT d.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS on_leave
    FROM days d
    JOIN leave_requests lr
      ON lr.status IN ('approved', 'pre_approved')
     AND lr.start_date <= d.date
     AND lr.end_date >= d.date
    JOIN employment_segments emp
      ON emp.id = lr.employment_id
     AND emp.status = 'active'
     AND emp.deleted_at IS NULL
     AND emp.start_date <= d.date
     AND (emp.end_date IS NULL OR emp.end_date >= d.date)
     AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.deleted_at IS NULL
    GROUP BY d.date
),
daily_late AS (
    SELECT ads.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS late
    FROM attendance_daily_summaries ads
    JOIN employment_segments emp ON emp.id = ads.employment_id
      AND emp.deleted_at IS NULL
      AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
    WHERE ads.date BETWEEN $1::date AND $2::date
      AND ads.late_minutes > 0
    GROUP BY ads.date
)
SELECT d.date::text,
       COALESCE(dp.present, 0)::bigint AS present,
       COALESCE(dl.on_leave, 0)::bigint AS on_leave,
       COALESCE(da.active_employees, 0)::bigint AS active_employees,
       COALESCE(dla.late, 0)::bigint AS late
FROM days d
LEFT JOIN daily_present dp ON dp.date = d.date
LEFT JOIN daily_leave dl ON dl.date = d.date
LEFT JOIN daily_active da ON da.date = d.date
LEFT JOIN daily_late dla ON dla.date = d.date
ORDER BY d.date ASC`

	rows, err := db.Query(ctx, query, startDate.Format("2006-01-02"), endDate.Format("2006-01-02"), utcStart, utcEndExclusive, tzName, deptFilter)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	points := make([]api.AttendanceTrendPoint, 0)
	for rows.Next() {
		var date string
		var present, onLeave, active, late int64
		if err := rows.Scan(&date, &present, &onLeave, &active, &late); err != nil {
			return nil, err
		}
		absent := max64(0, active-present-onLeave)
		points = append(points, api.AttendanceTrendPoint{
			Date:           date,
			Present:        present,
			Late:           late,
			Absent:         absent,
			OnLeave:        onLeave,
			Eligible:       active,
			AttendanceRate: api.NewRateQuantity(present, active, "employee_days"),
			AbsenceRate:    api.NewRateQuantity(absent, active, "employee_days"),
			LateRate:       api.NewRateQuantity(late, active, "employee_days"),
		})
	}
	return points, rows.Err()
}

func (r DashboardAnalyticsRepository) GetDepartmentAttendance(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID, topN *int) ([]api.DepartmentAttendancePoint, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	loc := timezone.ParseTimezone(tzName)
	utcStart := timezone.StartOfDayUTC(loc, startDate)
	utcEndExclusive := timezone.StartOfNextDayUTC(loc, endDate)

	var deptFilter any
	if len(deptIDs) > 0 {
		deptFilter = deptIDs
	}

	query := `WITH depts AS (
    SELECT d.id AS department_id,
           d.name::text AS department_name,
           COUNT(DISTINCT emp.employee_id)::bigint AS total_employees
    FROM departments d
    JOIN employment_segments emp ON emp.department_id = d.id AND emp.deleted_at IS NULL
      AND emp.status = 'active' AND emp.start_date <= $2::date AND (emp.end_date IS NULL OR emp.end_date >= $2::date)
      AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
    JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
    WHERE d.deleted_at IS NULL
    GROUP BY d.id, d.name
),
present AS (
    SELECT emp.department_id,
           COUNT(DISTINCT emp.employee_id)::bigint AS present_count
    FROM time_entries te
    JOIN employment_segments emp ON emp.id = te.employment_id AND emp.deleted_at IS NULL
      AND emp.status = 'active' AND emp.department_id IS NOT NULL
      AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
    WHERE te.punch_in_time >= $1 AND te.punch_in_time < $3
    GROUP BY emp.department_id
),
leaves AS (
    SELECT emp.department_id,
           COUNT(DISTINCT emp.employee_id)::bigint AS leave_count
    FROM leave_requests lr
    JOIN employment_segments emp ON emp.id = lr.employment_id AND emp.deleted_at IS NULL
      AND emp.status = 'active' AND emp.department_id IS NOT NULL
      AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
    WHERE lr.status IN ('approved', 'pre_approved')
      AND lr.start_date <= $2::date AND lr.end_date >= $5::date
    GROUP BY emp.department_id
),
lates AS (
    SELECT emp.department_id,
           COUNT(DISTINCT emp.employee_id)::bigint AS late_count
    FROM attendance_daily_summaries ads
    JOIN employment_segments emp ON emp.id = ads.employment_id AND emp.deleted_at IS NULL
      AND emp.status = 'active' AND emp.department_id IS NOT NULL
      AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
    WHERE ads.date BETWEEN $5::date AND $2::date AND ads.late_minutes > 0
    GROUP BY emp.department_id
)
SELECT d.department_id,
       d.department_name,
       d.total_employees,
       COALESCE(p.present_count, 0)::bigint AS present,
       COALESCE(l.leave_count, 0)::bigint   AS on_leave,
       COALESCE(la.late_count, 0)::bigint   AS late
FROM depts d
LEFT JOIN present p ON p.department_id = d.department_id
LEFT JOIN leaves l  ON l.department_id = d.department_id
LEFT JOIN lates la  ON la.department_id = d.department_id
ORDER BY d.department_name ASC`

	rows, err := db.Query(ctx, query, utcStart, endDate.Format("2006-01-02"), utcEndExclusive, deptFilter, startDate.Format("2006-01-02"))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	points := make([]api.DepartmentAttendancePoint, 0)
	for rows.Next() {
		var deptID uuid.UUID
		var deptName string
		var total, present, onLeave, late int64
		if err := rows.Scan(&deptID, &deptName, &total, &present, &onLeave, &late); err != nil {
			return nil, err
		}
		onTime := max64(0, present-late)
		absent := max64(0, total-present-onLeave)
		points = append(points, api.DepartmentAttendancePoint{
			DepartmentID:   deptID,
			DepartmentName: deptName,
			OnTime:         onTime,
			Late:           late,
			Absent:         absent,
			OnLeave:        onLeave,
			TotalEmployees: total,
			AttendanceRate: api.NewRateQuantity(present, total, "employees"),
			AbsenceRate:    api.NewRateQuantity(absent, total, "employees"),
			LateRate:       api.NewRateQuantity(late, total, "employees"),
		})
	}
	return points, rows.Err()
}

func (r DashboardAnalyticsRepository) GetUnexcusedAbsences(ctx context.Context, startDate, endDate time.Time, tzName string, groupBy string, deptIDs []uuid.UUID) ([]api.UnexcusedAbsencesPoint, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	var deptFilter any
	if len(deptIDs) > 0 {
		deptFilter = deptIDs
	}

	query := `WITH active_emp AS (
    SELECT emp.id AS employment_id,
           emp.employee_id,
           emp.department_id,
           d.name::text AS department_name,
           e.full_name::text AS employee_name
    FROM employment_segments emp
    JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
    LEFT JOIN departments d ON d.id = emp.department_id
    WHERE emp.status = 'active' AND emp.deleted_at IS NULL
      AND emp.start_date <= $2::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
      AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
),
days AS (
    SELECT d::date AS date
    FROM generate_series($1::date, $2::date, '1 day'::interval) AS d
),
daily_leaves AS (
    SELECT lr.employment_id, lr.start_date, lr.end_date
    FROM leave_requests lr
    WHERE lr.status IN ('approved', 'pre_approved')
      AND lr.start_date <= $2::date AND lr.end_date >= $1::date
)
SELECT ae.department_id,
       COALESCE(ae.department_name, 'Unassigned') AS department_name,
       ae.employee_id,
       ae.employee_name,
       COUNT(DISTINCT ads.date) FILTER (WHERE ads.actual_work_minutes = 0 AND NOT ads.is_holiday AND dl.employment_id IS NULL)::bigint AS unexcused_count,
       COUNT(DISTINCT ads.date) FILTER (WHERE ads.actual_work_minutes = 0 AND NOT ads.is_holiday)::bigint AS total_absent_count,
       COUNT(DISTINCT ads.date) FILTER (WHERE dl.employment_id IS NOT NULL)::bigint AS total_leave_count
FROM active_emp ae
CROSS JOIN days d
LEFT JOIN attendance_daily_summaries ads ON ads.employment_id = ae.employment_id AND ads.date = d.date
LEFT JOIN daily_leaves dl ON dl.employment_id = ae.employment_id AND d.date BETWEEN dl.start_date AND dl.end_date
GROUP BY ae.department_id, ae.department_name, ae.employee_id, ae.employee_name`

	rows, err := db.Query(ctx, query, startDate.Format("2006-01-02"), endDate.Format("2006-01-02"), deptFilter)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	if groupBy == "employee" {
		points := make([]api.UnexcusedAbsencesPoint, 0)
		for rows.Next() {
			var deptID *uuid.UUID
			var deptName, empName string
			var empID uuid.UUID
			var unexcused, totalAbsent, totalLeave int64
			if err := rows.Scan(&deptID, &deptName, &empID, &empName, &unexcused, &totalAbsent, &totalLeave); err != nil {
				return nil, err
			}
			resolvedDeptID := uuid.Nil
			if deptID != nil {
				resolvedDeptID = *deptID
			}
			points = append(points, api.UnexcusedAbsencesPoint{
				DepartmentID:     resolvedDeptID,
				DepartmentName:   deptName,
				EmployeeID:       &empID,
				EmployeeName:     &empName,
				UnexcusedCount:   unexcused,
				ConsecutiveCount: 0,
				TotalAbsentCount: totalAbsent,
				TotalLeaveCount:  totalLeave,
			})
		}
		return points, rows.Err()
	}

	deptMap := make(map[uuid.UUID]*api.UnexcusedAbsencesPoint)
	for rows.Next() {
		var deptID *uuid.UUID
		var deptName, empName string
		var empID uuid.UUID
		var unexcused, totalAbsent, totalLeave int64
		if err := rows.Scan(&deptID, &deptName, &empID, &empName, &unexcused, &totalAbsent, &totalLeave); err != nil {
			return nil, err
		}
		resolvedDeptID := uuid.Nil
		if deptID != nil {
			resolvedDeptID = *deptID
		}
		existing, ok := deptMap[resolvedDeptID]
		if !ok {
			deptMap[resolvedDeptID] = &api.UnexcusedAbsencesPoint{
				DepartmentID:     resolvedDeptID,
				DepartmentName:   deptName,
				UnexcusedCount:   unexcused,
				ConsecutiveCount: 0,
				TotalAbsentCount: totalAbsent,
				TotalLeaveCount:  totalLeave,
			}
		} else {
			existing.UnexcusedCount += unexcused
			existing.TotalAbsentCount += totalAbsent
			existing.TotalLeaveCount += totalLeave
		}
	}

	points := make([]api.UnexcusedAbsencesPoint, 0, len(deptMap))
	for _, p := range deptMap {
		points = append(points, *p)
	}
	return points, rows.Err()
}

func (r DashboardAnalyticsRepository) GetLatenessByDay(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]api.LatenessByDay, error) {
	db, err := requireDB(r.DB, "dashboard analytics repository")
	if err != nil {
		return nil, err
	}
	var deptFilter any
	if len(deptIDs) > 0 {
		deptFilter = deptIDs
	}

	query := `SELECT
		EXTRACT(ISODOW FROM ads.date)::int AS dow,
		COUNT(*)::bigint AS late_count,
		COALESCE(AVG(ads.late_minutes), 0)::float8 AS avg_late_minutes
	FROM attendance_daily_summaries ads
	JOIN employment_segments emp ON emp.id = ads.employment_id AND emp.deleted_at IS NULL
	  AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
	WHERE ads.date BETWEEN $1::date AND $2::date
	  AND ads.late_minutes > 0
	GROUP BY EXTRACT(ISODOW FROM ads.date)::int
	ORDER BY dow ASC`

	rows, err := db.Query(ctx, query, startDate.Format("2006-01-02"), endDate.Format("2006-01-02"), deptFilter)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	dayNames := []string{"", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"}
	items := make([]api.LatenessByDay, 0)
	for rows.Next() {
		var dow int
		var count int64
		var avg float64
		if err := rows.Scan(&dow, &count, &avg); err != nil {
			return nil, err
		}
		if dow >= 1 && dow <= 7 && count > 0 {
			items = append(items, api.LatenessByDay{
				Day:            dayNames[dow],
				LateCount:      count,
				AvgLateMinutes: math.Round(avg*10.0) / 10.0,
			})
		}
	}
	return items, rows.Err()
}

func max64(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}
