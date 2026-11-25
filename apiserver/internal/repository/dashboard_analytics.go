package repository

import (
	"context"
	"errors"
	"fmt"
	"math"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/timezone"
)

type DashboardAnalyticsRepository struct {
	DB DBTX
}

func (r DashboardAnalyticsRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("dashboard analytics repository: database is not configured")
	}
	return r.DB, nil
}

func (r DashboardAnalyticsRepository) GetOrganizationTimezone(ctx context.Context) (string, error) {
	db, err := r.db(ctx)
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
	db, err := r.db(ctx)
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
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	loc := today.Location()
	utcStart := timezone.StartOfDayUTC(loc, today)
	utcEndExclusive := timezone.StartOfNextDayUTC(loc, today)

	query := `WITH today_entries AS (
    SELECT te.punch_in_time, te.punch_out_time
    FROM time_entries te
    JOIN employments emp ON te.employment_id = emp.id AND emp.deleted_at IS NULL
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
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	monthStart := time.Date(today.Year(), today.Month(), 1, 0, 0, 0, 0, today.Location())
	query := `SELECT COALESCE(lt.name, 'Other')::text AS leave_type,
       COUNT(*)::bigint AS count
FROM leave_requests lr
LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
JOIN employments emp ON lr.employment_id = emp.id
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
	db, err := r.db(ctx)
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
    JOIN employments emp ON emp.id = ads.employment_id
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
    JOIN employments emp ON emp.id = ps.employment_id
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
	db, err := r.db(ctx)
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
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT employee_id)::bigint FROM employments
		WHERE status = 'active' AND deleted_at IS NULL
		  AND start_date <= $1::date AND (end_date IS NULL OR end_date >= $1::date)
		  AND ($2::uuid[] IS NULL OR department_id = ANY($2))`, todayStr, deptFilter).Scan(&totalEmp)

	var presentToday int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT emp.employee_id)::bigint FROM time_entries te
		JOIN employments emp ON emp.id = te.employment_id
		JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
		WHERE te.punch_in_time >= $1 AND te.punch_in_time < $2
		  AND emp.status = 'active' AND emp.deleted_at IS NULL
		  AND emp.start_date <= $3::date AND (emp.end_date IS NULL OR emp.end_date >= $3::date)
		  AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))`, utcStart, utcEndExclusive, todayStr, deptFilter).Scan(&presentToday)

	var pendingLeave int64
	_ = db.QueryRow(ctx, `SELECT COUNT(*)::bigint FROM leave_requests lr
		JOIN employments emp ON emp.id = lr.employment_id
		JOIN employees e ON e.id = emp.employee_id AND e.deleted_at IS NULL
		WHERE lr.status = 'pending' AND emp.status = 'active'
		  AND emp.deleted_at IS NULL AND emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
		  AND ($2::uuid[] IS NULL OR emp.department_id = ANY($2))`, todayStr, deptFilter).Scan(&pendingLeave)

	var onLeaveToday int64
	_ = db.QueryRow(ctx, `SELECT COUNT(DISTINCT emp.employee_id)::bigint FROM leave_requests lr
		JOIN employments emp ON emp.id = lr.employment_id
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
    FROM employments emp
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
		JOIN employments emp ON emp.id = te.employment_id
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
	JOIN employments emp ON emp.id = ads.employment_id AND emp.deleted_at IS NULL
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
	db, err := r.db(ctx)
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
    JOIN employments emp
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
    JOIN employments emp
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
    JOIN employments emp
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
    JOIN employments emp ON emp.id = ads.employment_id
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
	db, err := r.db(ctx)
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
    JOIN employments emp ON emp.department_id = d.id AND emp.deleted_at IS NULL
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
    JOIN employments emp ON emp.id = te.employment_id AND emp.deleted_at IS NULL
      AND emp.status = 'active' AND emp.department_id IS NOT NULL
      AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
    WHERE te.punch_in_time >= $1 AND te.punch_in_time < $3
    GROUP BY emp.department_id
),
leaves AS (
    SELECT emp.department_id,
           COUNT(DISTINCT emp.employee_id)::bigint AS leave_count
    FROM leave_requests lr
    JOIN employments emp ON emp.id = lr.employment_id AND emp.deleted_at IS NULL
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
    JOIN employments emp ON emp.id = ads.employment_id AND emp.deleted_at IS NULL
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
	db, err := r.db(ctx)
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
    FROM employments emp
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
	db, err := r.db(ctx)
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
	JOIN employments emp ON emp.id = ads.employment_id AND emp.deleted_at IS NULL
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

func (r DashboardAnalyticsRepository) GetTurnoverAnalytics(ctx context.Context, q api.TurnoverAnalyticsQuery) (api.TurnoverAnalytics, error) {
	db, err := r.db(ctx)
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
	db, err := r.db(ctx)
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

func (r DashboardAnalyticsRepository) GetExecutiveOverview(ctx context.Context, q api.NormalizedDashboardMetricsQuery, allowOvertimeCost bool) (api.DashboardMetricsResponse, error) {
	db, err := r.db(ctx)
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

func max64(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}
