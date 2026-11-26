package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"sort"
	"strings"
	"time"
)

type leaveUsageRequest struct {
	EmploymentID uuid.UUID
	StartDate    string
	EndDate      string
	DurationType string
	StartTime    *string
	EndTime      *string
}

type leaveScheduleBreak struct {
	StartAt         *time.Time
	EndAt           *time.Time
	DurationMinutes int
}

type leaveScheduleSegment struct {
	StartAt time.Time
	EndAt   time.Time
	Breaks  []leaveScheduleBreak
}

type leaveScheduleDay struct {
	State    string
	Timezone string
	Segments []leaveScheduleSegment
}

type leaveHoliday struct {
	HolidayType string
	IsPaid      bool
}

type leaveTimeWindow struct {
	Start time.Time
	End   time.Time
}

func mergeLeaveWindows(windows []leaveTimeWindow) []leaveTimeWindow {
	valid := make([]leaveTimeWindow, 0, len(windows))
	for _, w := range windows {
		if w.End.After(w.Start) {
			valid = append(valid, w)
		}
	}
	if len(valid) <= 1 {
		return valid
	}
	sort.Slice(valid, func(i, j int) bool {
		if valid[i].Start.Equal(valid[j].Start) {
			return valid[i].End.Before(valid[j].End)
		}
		return valid[i].Start.Before(valid[j].Start)
	})
	merged := []leaveTimeWindow{valid[0]}
	for _, w := range valid[1:] {
		last := &merged[len(merged)-1]
		if !w.Start.After(last.End) {
			if w.End.After(last.End) {
				last.End = w.End
			}
		} else {
			merged = append(merged, w)
		}
	}
	return merged
}

func clampLeaveWindows(base leaveTimeWindow, cuts []leaveTimeWindow) []leaveTimeWindow {
	clamped := make([]leaveTimeWindow, 0, len(cuts))
	for _, c := range cuts {
		s, e := c.Start, c.End
		if s.Before(base.Start) {
			s = base.Start
		}
		if e.After(base.End) {
			e = base.End
		}
		if e.After(s) {
			clamped = append(clamped, leaveTimeWindow{Start: s, End: e})
		}
	}
	return clamped
}

func subtractLeaveWindows(base leaveTimeWindow, cuts []leaveTimeWindow) []leaveTimeWindow {
	if !base.End.After(base.Start) {
		return nil
	}
	mergedCuts := mergeLeaveWindows(clampLeaveWindows(base, cuts))
	if len(mergedCuts) == 0 {
		return []leaveTimeWindow{base}
	}
	out := make([]leaveTimeWindow, 0, len(mergedCuts)+1)
	cursor := base.Start
	for _, c := range mergedCuts {
		if c.Start.After(cursor) {
			out = append(out, leaveTimeWindow{Start: cursor, End: c.Start})
		}
		if c.End.After(cursor) {
			cursor = c.End
		}
	}
	if base.End.After(cursor) {
		out = append(out, leaveTimeWindow{Start: cursor, End: base.End})
	}
	return out
}

func overlapLeaveWindowSeconds(target leaveTimeWindow, windows []leaveTimeWindow) int64 {
	if !target.End.After(target.Start) {
		return 0
	}
	var secs int64
	for _, w := range windows {
		s := target.Start
		if w.Start.After(s) {
			s = w.Start
		}
		e := target.End
		if w.End.Before(e) {
			e = w.End
		}
		if e.After(s) {
			secs += int64(e.Sub(s).Seconds())
		}
	}
	return secs
}

func parseLeaveClock(raw string) (hour, minute, second int, ok bool) {
	trimmed := strings.TrimSpace(raw)
	for _, layout := range []string{"15:04:05", "15:04"} {
		if parsed, err := time.Parse(layout, trimmed); err == nil {
			return parsed.Hour(), parsed.Minute(), parsed.Second(), true
		}
	}
	return 0, 0, 0, false
}

type rawLeaveBreakSnapshot struct {
	StartAt         *time.Time `json:"start_at"`
	EndAt           *time.Time `json:"end_at"`
	Start           string     `json:"start"`
	End             string     `json:"end"`
	DurationMinutes int        `json:"duration_minutes"`
}

func parseLocalLeaveBreak(startStr, endStr string, segStart time.Time, loc *time.Location) (leaveScheduleBreak, bool) {
	sh, sm, ss, okS := parseLeaveClock(startStr)
	eh, em, es, okE := parseLeaveClock(endStr)
	if !okS || !okE {
		return leaveScheduleBreak{}, false
	}
	base := segStart.In(loc)
	bs := time.Date(base.Year(), base.Month(), base.Day(), sh, sm, ss, 0, loc)
	if bs.Before(base) {
		bs = bs.AddDate(0, 0, 1)
	}
	be := time.Date(bs.Year(), bs.Month(), bs.Day(), eh, em, es, 0, loc)
	if !be.After(bs) {
		be = be.AddDate(0, 0, 1)
	}
	return leaveScheduleBreak{StartAt: &bs, EndAt: &be}, true
}

func parseLeaveBreakItem(item rawLeaveBreakSnapshot, segStart time.Time, loc *time.Location) (leaveScheduleBreak, bool) {
	if item.StartAt != nil && item.EndAt != nil && item.EndAt.After(*item.StartAt) {
		s, e := *item.StartAt, *item.EndAt
		return leaveScheduleBreak{StartAt: &s, EndAt: &e}, true
	}
	if item.Start != "" && item.End != "" {
		return parseLocalLeaveBreak(item.Start, item.End, segStart, loc)
	}
	if item.DurationMinutes > 0 {
		return leaveScheduleBreak{DurationMinutes: item.DurationMinutes}, true
	}
	return leaveScheduleBreak{}, false
}

func parseLeaveBreaksSnapshot(raw []byte, segStart time.Time, loc *time.Location) []leaveScheduleBreak {
	if len(raw) == 0 {
		return nil
	}
	var items []rawLeaveBreakSnapshot
	if err := json.Unmarshal(raw, &items); err != nil {
		return nil
	}
	out := make([]leaveScheduleBreak, 0, len(items))
	for _, item := range items {
		if parsed, ok := parseLeaveBreakItem(item, segStart, loc); ok {
			out = append(out, parsed)
		}
	}
	return out
}

type leaveSegmentEligible struct {
	windows []leaveTimeWindow
	netSecs int64
}

func segmentEligibleWork(seg leaveScheduleSegment) (leaveSegmentEligible, bool) {
	if !seg.EndAt.After(seg.StartAt) {
		return leaveSegmentEligible{}, false
	}
	fixedBreaks := make([]leaveTimeWindow, 0, len(seg.Breaks))
	var flexBreakSecs int64
	for _, b := range seg.Breaks {
		if b.StartAt != nil && b.EndAt != nil && b.EndAt.After(*b.StartAt) {
			fixedBreaks = append(fixedBreaks, leaveTimeWindow{Start: *b.StartAt, End: *b.EndAt})
		} else if b.DurationMinutes > 0 {
			flexBreakSecs += int64(b.DurationMinutes) * 60
		}
	}
	workWindows := subtractLeaveWindows(leaveTimeWindow{Start: seg.StartAt, End: seg.EndAt}, fixedBreaks)
	var segWorkSecs int64
	for _, w := range workWindows {
		segWorkSecs += int64(w.End.Sub(w.Start).Seconds())
	}
	segNet := segWorkSecs - min(flexBreakSecs, segWorkSecs)
	if segNet <= 0 {
		return leaveSegmentEligible{}, false
	}
	return leaveSegmentEligible{windows: workWindows, netSecs: segNet}, true
}

func buildLeaveDayEligible(segments []leaveScheduleSegment) ([]leaveSegmentEligible, int64) {
	eligibles := make([]leaveSegmentEligible, 0, len(segments))
	var dayNetSecs int64
	for _, seg := range segments {
		if item, ok := segmentEligibleWork(seg); ok {
			eligibles = append(eligibles, item)
			dayNetSecs += item.netSecs
		}
	}
	return eligibles, dayNetSecs
}

func loadLocationOrUTC(tz string) *time.Location {
	if tz != "" {
		if loaded, err := time.LoadLocation(tz); err == nil {
			return loaded
		}
	}
	return time.UTC
}

func coveredLeaveSeconds(eligibles []leaveSegmentEligible, win leaveTimeWindow) int64 {
	var covered int64
	for _, seg := range eligibles {
		covered += min(overlapLeaveWindowSeconds(win, seg.windows), seg.netSecs)
	}
	return covered
}

func parseCustomLeaveWindow(d time.Time, startTime, endTime *string, tz string) (leaveTimeWindow, bool, bool) {
	if startTime == nil || endTime == nil {
		return leaveTimeWindow{}, false, false
	}
	sh, sm, ss, okS := parseLeaveClock(*startTime)
	eh, em, es, okE := parseLeaveClock(*endTime)
	if !okS || !okE {
		return leaveTimeWindow{}, false, false
	}
	loc := loadLocationOrUTC(tz)
	leaveStart := time.Date(d.Year(), d.Month(), d.Day(), sh, sm, ss, 0, loc)
	leaveEnd := time.Date(d.Year(), d.Month(), d.Day(), eh, em, es, 0, loc)
	sameLocalDay := leaveEnd.After(leaveStart)
	if !sameLocalDay {
		leaveEnd = leaveEnd.AddDate(0, 0, 1)
	}
	return leaveTimeWindow{Start: leaveStart, End: leaveEnd}, sameLocalDay, true
}

func calculateCustomHoursFraction(d time.Time, req leaveUsageRequest, tz string, eligibles []leaveSegmentEligible, dayNetSecs int64, isHalfHoliday bool) decimal.Decimal {
	win, sameLocalDay, ok := parseCustomLeaveWindow(d, req.StartTime, req.EndTime, tz)
	if !ok {
		return decimal.Zero
	}
	coveredSecs := coveredLeaveSeconds(eligibles, win)
	if coveredSecs == 0 && sameLocalDay {
		coveredSecs = coveredLeaveSeconds(eligibles, leaveTimeWindow{
			Start: win.Start.AddDate(0, 0, 1),
			End:   win.End.AddDate(0, 0, 1),
		})
	}
	if coveredSecs <= 0 {
		return decimal.Zero
	}
	fraction := decimal.NewFromInt(min(coveredSecs, dayNetSecs)).DivRound(decimal.NewFromInt(dayNetSecs), 4)
	half := decimal.NewFromFloat(0.5)
	if isHalfHoliday && fraction.GreaterThan(half) {
		return half
	}
	return fraction
}

func fullDayLeaveFraction(isHalfHoliday bool) decimal.Decimal {
	if isHalfHoliday {
		return decimal.NewFromFloat(0.5)
	}
	return decimal.NewFromInt(1)
}

func calculateLeaveDayFraction(d time.Time, req leaveUsageRequest, durationType string, day leaveScheduleDay, hol leaveHoliday, hasHol bool) decimal.Decimal {
	if day.State != "published" || (hasHol && hol.HolidayType != "half_day") {
		return decimal.Zero
	}
	eligibles, dayNetSecs := buildLeaveDayEligible(day.Segments)
	if dayNetSecs <= 0 {
		return decimal.Zero
	}
	isHalfHoliday := hasHol && hol.HolidayType == "half_day"
	switch durationType {
	case "full_day":
		return fullDayLeaveFraction(isHalfHoliday)
	case "first_half", "second_half":
		return decimal.NewFromFloat(0.5)
	case "custom_hours":
		return calculateCustomHoursFraction(d, req, day.Timezone, eligibles, dayNetSecs, isHalfHoliday)
	default:
		return decimal.Zero
	}
}

func calculateLeaveDaysForRequest(req leaveUsageRequest, scheduleByDate map[string]leaveScheduleDay, holidays map[string]leaveHoliday) decimal.Decimal {
	start, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return decimal.Zero
	}
	end, err := time.Parse("2006-01-02", req.EndDate)
	if err != nil || end.Before(start) {
		return decimal.Zero
	}
	durationType := req.DurationType
	if durationType == "" {
		durationType = "full_day"
	}
	total := decimal.Zero
	for d := start; !d.After(end); d = d.AddDate(0, 0, 1) {
		dateKey := d.Format("2006-01-02")
		day, ok := scheduleByDate[dateKey]
		if !ok {
			continue
		}
		hol, hasHol := holidays[dateKey]
		total = total.Add(calculateLeaveDayFraction(d, req, durationType, day, hol, hasHol))
	}
	return total
}

func loadLeaveHolidays(ctx context.Context, db DBTX, minDate, maxDate string) (map[string]leaveHoliday, error) {
	holidays := make(map[string]leaveHoliday)
	holRows, err := db.Query(ctx, `SELECT date::text, holiday_type, is_paid FROM holidays WHERE date BETWEEN $1::date AND $2::date`, minDate, maxDate)
	if err != nil {
		return nil, err
	}
	defer holRows.Close()
	for holRows.Next() {
		var dateStr, holidayType string
		var isPaid bool
		if err := holRows.Scan(&dateStr, &holidayType, &isPaid); err != nil {
			return nil, err
		}
		holidays[dateStr] = leaveHoliday{HolidayType: holidayType, IsPaid: isPaid}
	}
	return holidays, holRows.Err()
}

func appendLeaveScheduleRow(schedules map[uuid.UUID]map[string]leaveScheduleDay, empID uuid.UUID, dateStr, state, tz string, startAt, endAt *time.Time, breaksRaw []byte) {
	byDate, ok := schedules[empID]
	if !ok {
		byDate = make(map[string]leaveScheduleDay)
		schedules[empID] = byDate
	}
	day := byDate[dateStr]
	day.State = state
	day.Timezone = tz
	if startAt != nil && endAt != nil && endAt.After(*startAt) {
		day.Segments = append(day.Segments, leaveScheduleSegment{
			StartAt: *startAt,
			EndAt:   *endAt,
			Breaks:  parseLeaveBreaksSnapshot(breaksRaw, *startAt, loadLocationOrUTC(tz)),
		})
	}
	byDate[dateStr] = day
}

func loadLeaveScheduleContext(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, minDate, maxDate string) (map[uuid.UUID]map[string]leaveScheduleDay, map[string]leaveHoliday, error) {
	schedules := make(map[uuid.UUID]map[string]leaveScheduleDay, len(employmentIDs))
	if len(employmentIDs) == 0 || minDate == "" || maxDate == "" {
		return schedules, map[string]leaveHoliday{}, nil
	}
	rows, err := db.Query(ctx, `
		SELECT sdr.employment_id, sdr.business_date::text, sdr.state, sdr.timezone, ws.scheduled_start_at, ws.scheduled_end_at, ws.breaks_snapshot
		FROM schedule_day_revisions sdr
		LEFT JOIN work_segments ws ON ws.schedule_revision_id = sdr.id
		WHERE sdr.employment_id = ANY($1)
		  AND sdr.business_date BETWEEN $2::date AND $3::date
		  AND sdr.state IN ('published', 'explicit_off')
		ORDER BY sdr.employment_id, sdr.business_date, ws.sort_order, ws.id
	`, employmentIDs, minDate, maxDate)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var dateStr, state, tz string
		var startAt, endAt *time.Time
		var breaksRaw []byte
		if err := rows.Scan(&empID, &dateStr, &state, &tz, &startAt, &endAt, &breaksRaw); err != nil {
			return nil, nil, err
		}
		appendLeaveScheduleRow(schedules, empID, dateStr, state, tz, startAt, endAt, breaksRaw)
	}
	if err := rows.Err(); err != nil {
		return nil, nil, err
	}
	holidays, err := loadLeaveHolidays(ctx, db, minDate, maxDate)
	return schedules, holidays, err
}

const orgLeaveBalancesCountSQL = `
	SELECT COUNT(*)
	FROM employees e
	CROSS JOIN leave_types lt
	LEFT JOIN employments emp ON e.id = emp.employee_id AND emp.deleted_at IS NULL AND emp.status = 'active'
	WHERE e.deleted_at IS NULL AND emp.id IS NOT NULL AND lt.deleted_at IS NULL
	  AND ($1 = '' OR e.full_name ILIKE '%' || $1 || '%' OR EXISTS (SELECT 1 FROM employee_tenures search_t WHERE search_t.employee_id = e.id AND search_t.deleted_at IS NULL AND search_t.employee_number ILIKE '%' || $1 || '%') OR lt.name ILIKE '%' || $1 || '%' OR lt.code ILIKE '%' || $1 || '%')
`

var orgLeaveBalancesDataSQL = `
	WITH leave_usage AS (
		SELECT
			emp_usage.employee_id,
			lr.leave_type_id,
			COALESCE(ROUND(SUM(
				(SELECT COALESCE(SUM(
					CASE
						WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type <> 'half_day') THEN 0::numeric
						WHEN lr.duration_type IN ('first_half', 'second_half') THEN 0.5::numeric
						WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type = 'half_day') THEN 0.5::numeric
						ELSE 1::numeric
					END
				), 0)
				FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
				JOIN schedule_day_revisions sdr
				  ON sdr.employment_id = lr.employment_id
				 AND sdr.business_date = d::date
				 AND sdr.state = 'published')
			)), 0)::INT as used_days
		FROM leave_requests lr
		JOIN employments emp_usage ON lr.employment_id = emp_usage.id
		WHERE lr.status IN ('approved', 'pre_approved')
		  AND EXTRACT(YEAR FROM lr.start_date)::INT = $1
		GROUP BY emp_usage.employee_id, lr.leave_type_id
	),
	best_policy AS (
		SELECT DISTINCT ON (lt_id, emp_id)
			d.leave_type_id as lt_id,
			e_inner.id as emp_id,
			d.name as policy_name,
			a.assignee_type as assignment_type
		FROM leave_policy_assignments a
		JOIN leave_policy_definitions d ON a.policy_id = d.id
		CROSS JOIN employees e_inner
		WHERE e_inner.deleted_at IS NULL
		  AND (
			(a.assignee_type = 'user' AND a.assignee_id = (
				SELECT t.user_id FROM employee_tenures t
				WHERE t.employee_id = e_inner.id
				  AND t.deleted_at IS NULL
				  AND t.start_date <= ` + orgTodaySQL + `
				  AND (t.end_date IS NULL OR t.end_date >= ` + orgTodaySQL + `)
				ORDER BY t.start_date DESC, t.id DESC LIMIT 1
			)) OR
			(a.assignee_type = 'employee' AND a.assignee_id = e_inner.id) OR
			(a.assignee_type = 'department' AND a.assignee_id = (
				SELECT department_id FROM employments
				WHERE employee_id = e_inner.id AND deleted_at IS NULL
				ORDER BY start_date DESC, id DESC LIMIT 1
			)) OR
			(a.assignee_type = 'organization')
		  )
		  AND a.effective_from <= NOW()
		  AND (a.effective_to IS NULL OR a.effective_to > NOW())
		  AND a.deleted_at IS NULL
		  AND d.deleted_at IS NULL
		ORDER BY lt_id, emp_id,
		  CASE a.assignee_type
			WHEN 'user' THEN 1
			WHEN 'employee' THEN 2
			WHEN 'department' THEN 3
			WHEN 'organization' THEN 4
		  END ASC
	)
	SELECT
		e.id as employee_id,
		e.full_name as employee_name,
		lt.id as leave_type_id,
		lt.code as leave_type_code,
		lt.name as leave_type_name,
		lt.is_paid as is_paid,
		COALESCE(lb.entitlement, 12) as entitlement,
		COALESCE(lb.accrued_amount::text, '0.00') as accrued_amount,
		COALESCE(lb.carried_over, 0) as carried_over,
		COALESCE(lu.used_days, 0) as used,
		bp.policy_name,
		bp.assignment_type
	FROM employees e
	CROSS JOIN leave_types lt
	LEFT JOIN employments emp ON e.id = emp.employee_id AND emp.deleted_at IS NULL AND emp.status = 'active'
	LEFT JOIN leave_balances lb ON emp.id = lb.employment_id AND lt.id = lb.leave_type_id AND lb.year = $1
	LEFT JOIN leave_usage lu ON e.id = lu.employee_id AND lt.id = lu.leave_type_id
	LEFT JOIN best_policy bp ON e.id = bp.emp_id AND lt.id = bp.lt_id
	WHERE e.deleted_at IS NULL AND emp.id IS NOT NULL AND lt.deleted_at IS NULL
	  AND ($2 = '' OR e.full_name ILIKE '%' || $2 || '%' OR EXISTS (SELECT 1 FROM employee_tenures search_t WHERE search_t.employee_id = e.id AND search_t.deleted_at IS NULL AND search_t.employee_number ILIKE '%' || $2 || '%') OR lt.name ILIKE '%' || $2 || '%' OR lt.code ILIKE '%' || $2 || '%')
	ORDER BY e.full_name ASC, lt.name ASC
	LIMIT $3 OFFSET $4
`

func (r LeaveRepository) GetOrganizationLeaveBalances(ctx context.Context, year, page, limit int, q string) ([]api.OrganizationLeaveBalance, int64, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, 0, err
	}
	if limit <= 0 {
		limit = 20
	}
	if page <= 0 {
		page = 1
	}
	offset := (page - 1) * limit
	searchTerm := strings.TrimSpace(q)

	var total int64
	if err := db.QueryRow(ctx, orgLeaveBalancesCountSQL, searchTerm).Scan(&total); err != nil {
		return nil, 0, err
	}
	rows, err := db.Query(ctx, orgLeaveBalancesDataSQL, year, searchTerm, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	balances := make([]api.OrganizationLeaveBalance, 0)
	for rows.Next() {
		var b api.OrganizationLeaveBalance
		var accruedStr string
		if err := rows.Scan(&b.EmployeeID, &b.EmployeeName, &b.LeaveTypeID, &b.LeaveTypeCode, &b.LeaveTypeName, &b.IsPaid, &b.Entitlement, &accruedStr, &b.CarriedOver, &b.Used, &b.PolicyName, &b.AssignmentType); err != nil {
			return nil, 0, err
		}
		b.AccruedAmount, _ = decimal.NewFromString(accruedStr)
		totalAvailable := decimal.NewFromInt(int64(b.Entitlement + b.CarriedOver)).Add(b.AccruedAmount)
		b.Remaining = totalAvailable.Sub(decimal.NewFromInt(int64(b.Used)))
		balances = append(balances, b)
	}
	return balances, total, rows.Err()
}

func loadEmployeeLeaveAccrualHistory(ctx context.Context, db DBTX, employeeID uuid.UUID, year int, leaveTypeID *uuid.UUID) ([]api.LeaveHistoryItem, error) {
	logRows, err := db.Query(ctx, `
		SELECT lal.id, lal.accrual_date::text, lal.amount::text, lal.notes, lal.policy_assignment_id, lal.created_at
		FROM leave_accrual_logs lal
		JOIN employments emp ON lal.employment_id = emp.id
		WHERE emp.employee_id = $1 AND emp.deleted_at IS NULL
		  AND EXTRACT(YEAR FROM lal.accrual_date)::INT = $2
		  AND ($3::uuid IS NULL OR lal.leave_type_id = $3)
	`, employeeID, year, leaveTypeID)
	if err != nil {
		return nil, err
	}
	defer logRows.Close()

	items := make([]api.LeaveHistoryItem, 0)
	for logRows.Next() {
		var id uuid.UUID
		var dateStr, amountStr string
		var notes *string
		var assignmentID *uuid.UUID
		var createdAt time.Time
		if err := logRows.Scan(&id, &dateStr, &amountStr, &notes, &assignmentID, &createdAt); err != nil {
			return nil, err
		}
		amt, _ := decimal.NewFromString(amountStr)
		txType := "adjustment"
		if assignmentID != nil {
			txType = "accrual"
		}
		items = append(items, api.LeaveHistoryItem{
			ID:              id,
			Date:            dateStr,
			TransactionType: txType,
			Amount:          amt,
			Reason:          notes,
			ReferenceID:     nil,
			CreatedAt:       createdAt.UTC().Format(time.RFC3339),
		})
	}
	return items, logRows.Err()
}

type fetchedLeaveHistoryReq struct {
	id        uuid.UUID
	usageReq  leaveUsageRequest
	reason    *string
	createdAt time.Time
}

func queryEmployeeApprovedLeaveRequests(ctx context.Context, db DBTX, employeeID uuid.UUID, year int, leaveTypeID *uuid.UUID) ([]fetchedLeaveHistoryReq, []uuid.UUID, string, string, error) {
	reqRows, err := db.Query(ctx, `
		SELECT lr.id, lr.employment_id, lr.start_date::text, lr.end_date::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.reason, lr.created_at
		FROM leave_requests lr
		JOIN employments emp ON lr.employment_id = emp.id
		WHERE emp.employee_id = $1 AND emp.deleted_at IS NULL
		  AND lr.status IN ('approved', 'pre_approved')
		  AND EXTRACT(YEAR FROM lr.start_date)::INT = $2
		  AND ($3::uuid IS NULL OR lr.leave_type_id = $3)
	`, employeeID, year, leaveTypeID)
	if err != nil {
		return nil, nil, "", "", err
	}
	defer reqRows.Close()

	fetched := make([]fetchedLeaveHistoryReq, 0)
	empSet := make(map[uuid.UUID]struct{})
	minDate, maxDate := "", ""
	for reqRows.Next() {
		var item fetchedLeaveHistoryReq
		if err := reqRows.Scan(
			&item.id, &item.usageReq.EmploymentID, &item.usageReq.StartDate, &item.usageReq.EndDate,
			&item.usageReq.DurationType, &item.usageReq.StartTime, &item.usageReq.EndTime, &item.reason, &item.createdAt,
		); err != nil {
			return nil, nil, "", "", err
		}
		fetched = append(fetched, item)
		empSet[item.usageReq.EmploymentID] = struct{}{}
		if minDate == "" || item.usageReq.StartDate < minDate {
			minDate = item.usageReq.StartDate
		}
		if maxDate == "" || item.usageReq.EndDate > maxDate {
			maxDate = item.usageReq.EndDate
		}
	}
	if err := reqRows.Err(); err != nil {
		return nil, nil, "", "", err
	}
	empIDs := make([]uuid.UUID, 0, len(empSet))
	for id := range empSet {
		empIDs = append(empIDs, id)
	}
	return fetched, empIDs, minDate, maxDate, nil
}

func (r LeaveRepository) GetEmployeeLeaveHistory(ctx context.Context, employeeID uuid.UUID, year int, leaveTypeID *uuid.UUID) ([]api.LeaveHistoryItem, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	items, err := loadEmployeeLeaveAccrualHistory(ctx, db, employeeID, year, leaveTypeID)
	if err != nil {
		return nil, err
	}
	fetched, empIDs, minDate, maxDate, err := queryEmployeeApprovedLeaveRequests(ctx, db, employeeID, year, leaveTypeID)
	if err != nil {
		return nil, err
	}
	schedulesByEmp, holidaysByDate, err := loadLeaveScheduleContext(ctx, db, empIDs, minDate, maxDate)
	if err != nil {
		return nil, err
	}
	for _, item := range fetched {
		usedDays := calculateLeaveDaysForRequest(item.usageReq, schedulesByEmp[item.usageReq.EmploymentID], holidaysByDate)
		reqID := item.id
		items = append(items, api.LeaveHistoryItem{
			ID:              uuid.New(),
			Date:            item.usageReq.StartDate,
			TransactionType: "usage",
			Amount:          usedDays.Neg(),
			Reason:          item.reason,
			ReferenceID:     &reqID,
			CreatedAt:       item.createdAt.UTC().Format(time.RFC3339),
		})
	}
	sort.Slice(items, func(i, j int) bool {
		if items[i].Date != items[j].Date {
			return items[i].Date > items[j].Date
		}
		return items[i].CreatedAt > items[j].CreatedAt
	})
	return items, nil
}

func (r LeaveRepository) AdjustLeaveBalance(ctx context.Context, employeeID uuid.UUID, req api.AdjustBalanceRequest) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var employmentID uuid.UUID
		err := db.QueryRow(ctx, `
			SELECT id FROM employments
			WHERE employee_id = $1 AND deleted_at IS NULL
			ORDER BY start_date DESC LIMIT 1
		`, employeeID).Scan(&employmentID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNoEmployment
		}
		if err != nil {
			return err
		}
		parsedDate, err := time.Parse("2006-01-02", req.TransactionDate)
		if err != nil {
			return api.ErrLeaveValidation
		}
		year := parsedDate.Year()
		_, err = db.Exec(ctx, `
			INSERT INTO leave_accrual_logs (id, employment_id, accrual_date, amount, leave_type_id, notes, created_at)
			VALUES (gen_random_uuid(), $1, $2::date, $3, $4, $5, NOW())
		`, employmentID, req.TransactionDate, req.Amount, req.LeaveTypeID, req.Reason)
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `
			INSERT INTO leave_balances (id, employment_id, leave_type_id, year, entitlement, accrued_amount, carried_over, created_at, updated_at)
			VALUES (gen_random_uuid(), $1, $2, $3, 12, $4, 0, NOW(), NOW())
			ON CONFLICT (employment_id, leave_type_id, year)
			DO UPDATE SET accrued_amount = leave_balances.accrued_amount + $4, updated_at = NOW()
		`, employmentID, req.LeaveTypeID, year, req.Amount)
		return err
	})
}
