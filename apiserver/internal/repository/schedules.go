package repository

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type ScheduleRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r ScheduleRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("schedule repository: database does not support transactions")
	}
	return beginner, nil
}

func (r ScheduleRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("schedule repository: database is not configured")
	}
	return r.DB, nil
}

type employeeRow struct {
	employmentID      uuid.UUID
	fullName          string
	employeeIDNumber  string
	tags              []string
	primaryLocationID *uuid.UUID
	employeeID        uuid.UUID
	startDate         time.Time
	endDate           *time.Time
}

type shiftTemplateInfo struct {
	name     string
	versions []shiftVersionInfo
}

type shiftVersionInfo struct {
	id            uuid.UUID
	reportSymbol  *string
	effectiveFrom time.Time
	effectiveTo   *time.Time
	intervals     []shiftIntervalInfo
}

type shiftIntervalInfo struct {
	sortOrder             int
	startTime             *string
	endTime               *string
	startDayOffset        int
	endDayOffset          int
	startUTCOffsetMinutes *int
	endUTCOffsetMinutes   *int
	breakDurationMinutes  *int
	breakStart            *string
	breakEnd              *string
	breakPunchMode        *string
}

func primaryShiftInterval(version shiftVersionInfo) *shiftIntervalInfo {
	if len(version.intervals) == 0 {
		return nil
	}
	return &version.intervals[0]
}

func selectEffectiveShiftVersion(shifts map[uuid.UUID]shiftTemplateInfo, templateID uuid.UUID, date time.Time) (shiftVersionInfo, bool, bool) {
	meta, ok := shifts[templateID]
	if !ok {
		return shiftVersionInfo{}, false, false
	}
	var selected shiftVersionInfo
	matches := 0
	for _, version := range meta.versions {
		if date.Before(version.effectiveFrom) || (version.effectiveTo != nil && date.After(*version.effectiveTo)) {
			continue
		}
		selected = version
		matches++
	}
	return selected, matches == 1, matches > 1
}

func calendarDayDifference(date, anchor time.Time) int {
	dateDay := time.Date(date.Year(), date.Month(), date.Day(), 0, 0, 0, 0, time.UTC)
	anchorDay := time.Date(anchor.Year(), anchor.Month(), anchor.Day(), 0, 0, 0, 0, time.UTC)
	return int(dateDay.Sub(anchorDay) / (24 * time.Hour))
}

func normalizedCycleDay(day, cycleLength int) int {
	day %= cycleLength
	if day < 0 {
		day += cycleLength
	}
	return day
}

type patternEntry struct {
	patternID       uuid.UUID
	startDate       time.Time
	endDate         *time.Time
	anchorDate      *time.Time
	patternType     string
	cycleLengthDays *int
	scheduleDay     int
	templateID      *uuid.UUID
}

type fixedShiftEntry struct {
	templateID uuid.UUID
	startDate  time.Time
	endDate    *time.Time
}

type contractEntry struct {
	employerID uuid.UUID
	startDate  time.Time
	endDate    *time.Time
}

type assignmentEntry struct {
	hostLocationID uuid.UUID
	startDate      time.Time
	endDate        *time.Time
}

type deptHistoryEntry struct {
	departmentID  uuid.UUID
	effectiveDate time.Time
}

type holidayEntry struct {
	name   string
	isPaid bool
}

type assignmentRuleWithDetails struct {
	id        uuid.UUID
	name      string
	priority  int
	startDate *time.Time
	endDate   *time.Time
	patternID uuid.UUID
	cond      api.AssignmentRuleConditions
	details   map[int][]uuid.UUID
}

type resolvedScheduleDay struct {
	date                  string
	shiftIDs              []*uuid.UUID
	pinnedShiftVersionIDs []*uuid.UUID
	breakWorkedOverrides  []api.ScheduleBreakWorkedOverride
	source                string
	traces                []api.ScheduleTrace
	isHoliday             bool
	holidayPaid           bool
	holidayName           *string
	timezone              string
}

type scheduleDayKey struct {
	employmentID uuid.UUID
	date         string
}

type resolvedEmployeeSchedule struct {
	employmentID     uuid.UUID
	employeeID       uuid.UUID
	employeeIDNumber string
	fullName         string
	days             []resolvedScheduleDay
}

type scheduleResolution struct {
	employees []resolvedEmployeeSchedule
	shifts    map[uuid.UUID]shiftTemplateInfo
}

func (r ScheduleRepository) GetOfficialSchedules(ctx context.Context, startDate, endDate time.Time, employmentIDs []uuid.UUID, page, limit int) (api.PaginatedEmployeeSchedules, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	page, limit = normalizedPage(page), normalizedLimit(limit)
	start, end := startDate.Format("2006-01-02"), endDate.Format("2006-01-02")
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employees e JOIN employments emp ON emp.employee_id = e.id WHERE emp.deleted_at IS NULL AND emp.start_date <= $2::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date) AND (COALESCE(cardinality($3::uuid[]), 0) = 0 OR emp.id = ANY($3::uuid[]))`, start, end, employmentIDs).Scan(&total); err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	rows, err := db.Query(ctx, `WITH eligible_employments AS (
		SELECT emp.id AS employment_id, e.id AS employee_id, e.full_name,
			COALESCE(t.employee_number, '') AS employee_id_number,
			emp.start_date, emp.end_date
		FROM employments emp
		JOIN employees e ON e.id = emp.employee_id
		LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id
		LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL
			AND t.start_date <= emp.start_date AND (t.end_date IS NULL OR t.end_date >= emp.start_date)
		WHERE emp.deleted_at IS NULL AND emp.start_date <= $2::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
			AND (COALESCE(cardinality($3::uuid[]), 0) = 0 OR emp.id = ANY($3::uuid[]))
		ORDER BY emp.start_date DESC, emp.id
		LIMIT $4 OFFSET $5
	), eligible_days AS (
		SELECT ee.*, days.day::date AS business_date
		FROM eligible_employments ee
		CROSS JOIN LATERAL generate_series(GREATEST(ee.start_date, $1::date), LEAST(COALESCE(ee.end_date, $2::date), $2::date), interval '1 day') AS days(day)
	)
	SELECT ed.employment_id, ed.employee_id, ed.full_name, ed.employee_id_number, ed.business_date::text,
		r.id, r.state, r.timezone, r.published_at, r.source,
		(array_agg(ws.template_id ORDER BY ws.sort_order, ws.id) FILTER (WHERE ws.id IS NOT NULL))[1],
		(array_agg(st.name ORDER BY ws.sort_order, ws.id) FILTER (WHERE ws.id IS NOT NULL))[1],
		(array_agg(sv.report_symbol ORDER BY ws.sort_order, ws.id) FILTER (WHERE ws.id IS NOT NULL))[1],
		(array_agg(ws.scheduled_start::text ORDER BY ws.sort_order, ws.id) FILTER (WHERE ws.id IS NOT NULL))[1],
		(array_agg(ws.scheduled_end::text ORDER BY ws.sort_order, ws.id) FILTER (WHERE ws.id IS NOT NULL))[1],
		COALESCE(jsonb_agg(jsonb_build_object(
			'shift_id', sv.template_id,
			'shift_version_id', ws.selected_shift_version_id,
			'sort_order', ws.sort_order,
			'start_time', ws.scheduled_start::text,
			'end_time', ws.scheduled_end::text,
			'scheduled_start_at', ws.scheduled_start_at,
			'scheduled_end_at', ws.scheduled_end_at,
			'breaks', ws.breaks_snapshot,
			'shift_name', st.name,
			'shift_symbol', sv.report_symbol
		) ORDER BY ws.sort_order, ws.id) FILTER (WHERE ws.id IS NOT NULL), '[]'::jsonb)
	FROM eligible_days ed
	LEFT JOIN schedule_day_revisions r ON r.employment_id = ed.employment_id AND r.business_date = ed.business_date AND r.state IN ('published', 'explicit_off')
	LEFT JOIN work_segments ws ON ws.schedule_revision_id = r.id
	LEFT JOIN shift_versions sv ON sv.id = ws.selected_shift_version_id
	LEFT JOIN shift_templates st ON st.id = ws.template_id
	GROUP BY ed.employment_id, ed.employee_id, ed.full_name, ed.employee_id_number, ed.start_date, ed.business_date, r.id, r.state, r.timezone, r.published_at, r.source
	ORDER BY ed.start_date DESC, ed.employment_id, ed.business_date
	`, start, end, employmentIDs, limit, (page-1)*limit)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	defer rows.Close()
	result := make([]api.EmployeeSchedule, 0)
	index := make(map[uuid.UUID]int)
	for rows.Next() {
		var employmentID, employeeID uuid.UUID
		var fullName, employeeNumber, date string
		var revisionID, templateID *uuid.UUID
		var state, timezone, source, shiftName, shiftSymbol, startTime, endTime *string
		var publishedAt *time.Time
		var intervalsJSON []byte
		if err := rows.Scan(&employmentID, &employeeID, &fullName, &employeeNumber, &date, &revisionID, &state, &timezone, &publishedAt, &source, &templateID, &shiftName, &shiftSymbol, &startTime, &endTime, &intervalsJSON); err != nil {
			return api.PaginatedEmployeeSchedules{}, err
		}
		status := "missing"
		if state != nil {
			status = "work"
			if *state == "explicit_off" {
				status = "explicit_off"
			}
		}
		intervals := make([]json.RawMessage, 0)
		if len(intervalsJSON) > 0 {
			if err := json.Unmarshal(intervalsJSON, &intervals); err != nil {
				return api.PaginatedEmployeeSchedules{}, err
			}
		}
		zone := timezone
		day := api.EffectiveSchedule{Date: date, RevisionID: revisionID, ScheduleStatus: status, Provisional: false, Timezone: zone, PublishedAt: publishedAt, Intervals: intervals, ShiftID: templateID, Source: "", ShiftName: shiftName, ShiftSymbol: shiftSymbol, StartTime: startTime, EndTime: endTime, Traces: []api.ScheduleTrace{}}
		if source != nil {
			day.Source = *source
		}
		position, ok := index[employmentID]
		if !ok {
			position = len(result)
			index[employmentID] = position
			result = append(result, api.EmployeeSchedule{EmploymentID: employmentID, EmployeeID: employeeID, EmployeeIDNumber: employeeNumber, FullName: fullName, Schedules: make([]api.EffectiveSchedule, 0)})
		}
		result[position].Schedules = append(result[position].Schedules, day)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	if len(result) > 0 {
		ids := make([]uuid.UUID, 0, len(result))
		for _, employee := range result {
			ids = append(ids, employee.EmploymentID)
		}
		overrides, err := loadBreakWorkedOverrides(ctx, db, ids, start, end)
		if err != nil {
			return api.PaginatedEmployeeSchedules{}, err
		}
		for i := range result {
			for j := range result[i].Schedules {
				key := scheduleDayKey{employmentID: result[i].EmploymentID, date: result[i].Schedules[j].Date}
				result[i].Schedules[j].BreakWorkedOverrides = overrides[key]
			}
		}
	}
	return api.PaginatedEmployeeSchedules{Data: result, Total: total, Page: page, Limit: limit}, nil
}

func (r ScheduleRepository) GetOfficialSchedulesWithCoverage(ctx context.Context, startDate, endDate time.Time, employmentIDs []uuid.UUID, page, limit int, coverage string) (api.PaginatedEmployeeSchedules, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	start, end := startDate.Format("2006-01-02"), endDate.Format("2006-01-02")
	rows, err := db.Query(ctx, `
		WITH eligible AS (
			SELECT emp.id AS employment_id, emp.start_date, emp.end_date
			FROM employments emp
			WHERE emp.deleted_at IS NULL
			  AND emp.start_date <= $2::date
			  AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
			  AND (COALESCE(cardinality($3::uuid[]), 0) = 0 OR emp.id = ANY($3::uuid[]))
		), days AS (
			SELECT e.employment_id, gs.day::date AS business_date
			FROM eligible e CROSS JOIN LATERAL generate_series(GREATEST(e.start_date, $1::date), LEAST(COALESCE(e.end_date, $2::date), $2::date), interval '1 day') gs(day)
		), states AS (
			SELECT d.employment_id, d.business_date, r.state
			FROM days d
			LEFT JOIN schedule_day_revisions r
			  ON r.employment_id = d.employment_id
			 AND r.business_date = d.business_date
			 AND r.state IN ('published', 'explicit_off')
		), coverage AS (
			SELECT employment_id,
				count(*) AS expected_days,
				count(*) FILTER (WHERE state = 'published') AS published_work_days,
				count(*) FILTER (WHERE state = 'explicit_off') AS published_off_days,
				count(*) FILTER (WHERE state IS NULL) AS missing_official_days
			FROM states GROUP BY employment_id
		)
		SELECT employment_id, expected_days, published_work_days, published_off_days, missing_official_days
		FROM coverage
		WHERE ($4 = 'all')
		   OR ($4 = 'missing' AND missing_official_days > 0)
		   OR ($4 = 'complete' AND missing_official_days = 0)
		ORDER BY employment_id`, start, end, employmentIDs, coverage)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	defer rows.Close()
	ids := make([]uuid.UUID, 0)
	counts := make(map[uuid.UUID]api.ScheduleCoverage)
	for rows.Next() {
		var id uuid.UUID
		var expected, work, off, missing int64
		if err := rows.Scan(&id, &expected, &work, &off, &missing); err != nil {
			return api.PaginatedEmployeeSchedules{}, err
		}
		ids = append(ids, id)
		counts[id] = api.ScheduleCoverage{ExpectedDays: expected, PublishedWorkDays: work, PublishedOffDays: off, MissingOfficialDays: missing, Complete: missing == 0}
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	if len(ids) == 0 {
		return api.PaginatedEmployeeSchedules{Data: []api.EmployeeSchedule{}, Total: 0, Page: normalizedPage(page), Limit: normalizedLimit(limit)}, nil
	}
	result, err := r.GetOfficialSchedules(ctx, startDate, endDate, ids, page, limit)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	for i := range result.Data {
		result.Data[i].Coverage = counts[result.Data[i].EmploymentID]
	}
	result.Total = int64(len(ids))
	return result, nil
}

func (r ScheduleRepository) GetActivationAwareSchedules(ctx context.Context, startDate, endDate time.Time, employmentIDs []uuid.UUID, page, limit int) (api.PaginatedEmployeeSchedules, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	var official bool
	if err := db.QueryRow(ctx, `SELECT COALESCE(bool_and(COALESCE(c.value_bool, false)), false)
		FROM employments e
		CROSS JOIN generate_series(GREATEST(e.start_date, $2::date), LEAST(COALESCE(e.end_date, $3::date), $3::date), interval '1 day') days(day)
		LEFT JOIN LATERAL (
			SELECT c.value_bool FROM config_entries c
			WHERE c.scope_type = 'employer' AND c.scope_id = e.employer_id
			  AND c.config_key = 'schedule.lifecycle.enabled' AND c.effective_from <= days.day::date
			  AND (c.effective_to IS NULL OR c.effective_to >= days.day::date)
			ORDER BY c.effective_from DESC, c.id DESC LIMIT 1
		) c ON true
		WHERE e.deleted_at IS NULL AND e.start_date <= $3::date
		  AND (e.end_date IS NULL OR e.end_date >= $2::date)
		  AND (COALESCE(cardinality($1::uuid[]), 0) = 0 OR e.id = ANY($1::uuid[]))`, employmentIDs, startDate, endDate).Scan(&official); err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	if official {
		return r.GetOfficialSchedules(ctx, startDate, endDate, employmentIDs, page, limit)
	}
	return r.GetOrganizationSchedules(ctx, startDate, endDate, employmentIDs, page, limit)
}

func (r ScheduleRepository) GetOrganizationSchedules(ctx context.Context, startDate, endDate time.Time, employmentIDs []uuid.UUID, page, limit int) (api.PaginatedEmployeeSchedules, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	resolution, total, err := resolveScheduleData(ctx, db, startDate, endDate, employmentIDs, page, limit)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	result := make([]api.EmployeeSchedule, 0, len(resolution.employees))
	for _, employee := range resolution.employees {
		dailyList := make([]api.EffectiveSchedule, 0)
		for _, day := range employee.days {
			for _, shiftID := range day.shiftIDs {
				status := "missing"
				if day.source == "conflict" {
					status = "conflict"
				} else if shiftID == nil && (day.source == "roster" || day.source == "pattern") {
					status = "explicit_off"
				} else if shiftID != nil {
					status = "work"
				}
				zone := day.timezone
				effective := api.EffectiveSchedule{Date: day.date, Source: day.source, ScheduleStatus: status, Provisional: true, Timezone: &zone, ShiftID: shiftID, BreakWorkedOverrides: day.breakWorkedOverrides, Traces: day.traces, IsHoliday: day.isHoliday, IsHolidayPaid: day.holidayPaid, HolidayName: day.holidayName}
				if shiftID != nil {
					if meta, ok := resolution.shifts[*shiftID]; ok {
						effective.ShiftName = &meta.name
						if version, selected, _ := selectEffectiveShiftVersion(resolution.shifts, *shiftID, parseScheduleDate(day.date)); selected {
							effective.ShiftSymbol = version.reportSymbol
							if interval := primaryShiftInterval(version); interval != nil {
								effective.StartTime, effective.EndTime = interval.startTime, interval.endTime
								effective.BreakDurationMinutes, effective.BreakStart, effective.BreakEnd = interval.breakDurationMinutes, interval.breakStart, interval.breakEnd
								effective.BreakPunchMode = interval.breakPunchMode
							}
						}
					}
				}
				dailyList = append(dailyList, effective)
			}
		}
		result = append(result, api.EmployeeSchedule{EmploymentID: employee.employmentID, EmployeeID: employee.employeeID, EmployeeIDNumber: employee.employeeIDNumber, FullName: employee.fullName, Schedules: dailyList})
	}
	return api.PaginatedEmployeeSchedules{Data: result, Total: total, Page: normalizedPage(page), Limit: normalizedLimit(limit)}, nil
}

func resolveScheduleData(ctx context.Context, db DBTX, startDate, endDate time.Time, employmentIDs []uuid.UUID, page, limit int) (scheduleResolution, int64, error) {
	page, limit = normalizedPage(page), normalizedLimit(limit)
	offset := (page - 1) * limit
	startStr, endStr := startDate.Format("2006-01-02"), endDate.Format("2006-01-02")
	var total int64
	var empRows []employeeRow
	if len(employmentIDs) > 0 {
		if err := db.QueryRow(ctx, `SELECT count(*) FROM employees e JOIN employments emp ON emp.employee_id = e.id WHERE emp.deleted_at IS NULL AND emp.id = ANY($1)`, employmentIDs).Scan(&total); err != nil {
			return scheduleResolution{}, 0, err
		}
		rows, err := db.Query(ctx, `SELECT emp.id, e.full_name, COALESCE(t.employee_number, ''), e.tags, emp.primary_location_id, e.id, emp.start_date::text, emp.end_date::text FROM employees e JOIN employments emp ON emp.employee_id = e.id LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= emp.start_date AND (t.end_date IS NULL OR t.end_date >= emp.start_date) WHERE emp.deleted_at IS NULL AND emp.id = ANY($1) ORDER BY emp.start_date DESC, emp.id ASC LIMIT $2 OFFSET $3`, employmentIDs, limit, offset)
		if err != nil {
			return scheduleResolution{}, 0, err
		}
		empRows, err = scanEmployeeRows(rows)
		rows.Close()
		if err != nil {
			return scheduleResolution{}, 0, err
		}
	} else {
		if err := db.QueryRow(ctx, `SELECT count(*) FROM employees e JOIN employments emp ON emp.employee_id = e.id WHERE emp.deleted_at IS NULL AND emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $2::date)`, endStr, startStr).Scan(&total); err != nil {
			return scheduleResolution{}, 0, err
		}
		rows, err := db.Query(ctx, `SELECT emp.id, e.full_name, COALESCE(t.employee_number, ''), e.tags, emp.primary_location_id, e.id, emp.start_date::text, emp.end_date::text FROM employees e JOIN employments emp ON emp.employee_id = e.id LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= $1::date AND (t.end_date IS NULL OR t.end_date >= $2::date) WHERE emp.deleted_at IS NULL AND emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $2::date) ORDER BY emp.start_date DESC, emp.id ASC LIMIT $3 OFFSET $4`, endStr, startStr, limit, offset)
		if err != nil {
			return scheduleResolution{}, 0, err
		}
		empRows, err = scanEmployeeRows(rows)
		rows.Close()
		if err != nil {
			return scheduleResolution{}, 0, err
		}
	}
	if len(empRows) == 0 {
		return scheduleResolution{employees: []resolvedEmployeeSchedule{}, shifts: map[uuid.UUID]shiftTemplateInfo{}}, total, nil
	}
	employmentIDsForLoad := make([]uuid.UUID, len(empRows))
	employeeIDs := make([]uuid.UUID, len(empRows))
	locationIDs := make([]uuid.UUID, 0, len(empRows))
	for i, employee := range empRows {
		employmentIDsForLoad[i], employeeIDs[i] = employee.employmentID, employee.employeeID
		if employee.primaryLocationID != nil {
			locationIDs = append(locationIDs, *employee.primaryLocationID)
		}
	}
	contracts, err := loadContracts(ctx, db, employmentIDsForLoad, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	assignments, err := loadAssignments(ctx, db, employmentIDsForLoad, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	for _, values := range assignments {
		for _, value := range values {
			locationIDs = append(locationIDs, value.hostLocationID)
		}
	}
	locationTimezones, err := loadLocationTimezones(ctx, db, locationIDs)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	holidays, err := loadHolidays(ctx, db, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	rosters, err := loadShiftRosters(ctx, db, employmentIDsForLoad, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	patterns, err := loadPatternAssignments(ctx, db, employmentIDsForLoad, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	rules, err := loadAssignmentRules(ctx, db)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	fixed, err := loadFixedShifts(ctx, db, employmentIDsForLoad, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	deptHistory, err := loadDeptHistory(ctx, db, employeeIDs)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	shifts, err := loadShifts(ctx, db)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	overrides, err := loadBreakWorkedOverrides(ctx, db, employmentIDsForLoad, startStr, endStr)
	if err != nil {
		return scheduleResolution{}, 0, err
	}
	result := scheduleResolution{employees: make([]resolvedEmployeeSchedule, 0, len(empRows)), shifts: shifts}
	for _, employee := range empRows {
		schedStart, schedEnd := startDate, endDate
		if employee.startDate.After(schedStart) {
			schedStart = employee.startDate
		}
		if employee.endDate != nil && employee.endDate.Before(schedEnd) {
			schedEnd = *employee.endDate
		}
		if schedStart.After(schedEnd) {
			continue
		}
		resolved := resolvedEmployeeSchedule{employmentID: employee.employmentID, employeeID: employee.employeeID, employeeIDNumber: employee.employeeIDNumber, fullName: employee.fullName}
		for current := schedStart; !current.After(schedEnd); current = current.AddDate(0, 0, 1) {
			dateStr := current.Format("2006-01-02")
			var employerID, locationID *uuid.UUID
			for _, value := range contracts[employee.employmentID] {
				if !current.Before(value.startDate) && (value.endDate == nil || !current.After(*value.endDate)) {
					id := value.employerID
					employerID = &id
					break
				}
			}
			locationID = employee.primaryLocationID
			for _, value := range assignments[employee.employmentID] {
				if !current.Before(value.startDate) && (value.endDate == nil || !current.After(*value.endDate)) {
					id := value.hostLocationID
					locationID = &id
					break
				}
			}
			holiday, isHoliday := holidays[dateStr]
			shiftIDs, source, traces := resolveEffectiveShift(employee.employmentID, employee.employeeID, employee.employeeIDNumber, employee.tags, current, int(current.Weekday()), locationID, employerID, rosters, patterns[employee.employmentID], rules, fixed[employee.employmentID], deptHistory[employee.employeeID])
			resolved.days = append(resolved.days, resolvedScheduleDay{date: dateStr, shiftIDs: shiftIDs, pinnedShiftVersionIDs: func() []*uuid.UUID {
				if assignment, ok := rosters[employee.employmentID.String()+":"+dateStr]; ok {
					return []*uuid.UUID{assignment.shiftVersionID}
				}
				return nil
			}(), breakWorkedOverrides: overrides[scheduleDayKey{employmentID: employee.employmentID, date: dateStr}], source: source, traces: traces, isHoliday: isHoliday, holidayPaid: holiday.isPaid, holidayName: stringPtr(holiday.name), timezone: resolvedTimezone(locationID, locationTimezones)})
		}
		result.employees = append(result.employees, resolved)
	}
	return result, total, nil
}

func normalizedPage(page int) int {
	if page < 1 {
		return 1
	}
	return page
}
func normalizedLimit(limit int) int {
	if limit < 1 {
		return 50
	}
	if limit > api.MaxLimit {
		return api.MaxLimit
	}
	return limit
}
func parseScheduleDate(value string) time.Time {
	parsed, _ := time.Parse("2006-01-02", value)
	return parsed
}
func resolvedTimezone(locationID *uuid.UUID, timezones map[uuid.UUID]string) string {
	if locationID != nil && timezones[*locationID] != "" {
		return timezones[*locationID]
	}
	return "UTC"
}

func loadLocationTimezones(ctx context.Context, db DBTX, locationIDs []uuid.UUID) (map[uuid.UUID]string, error) {
	result := make(map[uuid.UUID]string)
	if len(locationIDs) == 0 {
		return result, nil
	}
	rows, err := db.Query(ctx, `SELECT id, timezone FROM work_locations WHERE id = ANY($1) AND status = 'active'`, locationIDs)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var id uuid.UUID
		var timezone string
		if err := rows.Scan(&id, &timezone); err != nil {
			return nil, err
		}
		result[id] = timezone
	}
	return result, rows.Err()
}

func resolveEffectiveShift(
	employmentID, employeeID uuid.UUID,
	employeeIDNumber string,
	tags []string,
	date time.Time,
	dow int,
	activeLocID, activeEmployerID *uuid.UUID,
	rosters map[string]rosterAssignment,
	patterns []patternEntry,
	rules []assignmentRuleWithDetails,
	fixed []fixedShiftEntry,
	deptHistory []deptHistoryEntry,
) ([]*uuid.UUID, string, []api.ScheduleTrace) {
	dateStr := date.Format("2006-01-02")
	key := employmentID.String() + ":" + dateStr

	if assignment, exists := rosters[key]; exists {
		traces := []api.ScheduleTrace{{
			RuleName: "Manual Roster",
			Priority: 100,
			Status:   "applied",
			Reason:   new("Directly assigned via roster"),
		}}
		return []*uuid.UUID{assignment.shiftID}, "roster", traces
	}

	matchedPatternShifts := make([]*uuid.UUID, 0)
	var matchedPatternID *uuid.UUID
	for _, p := range patterns {
		if date.Before(p.startDate) || (p.endDate != nil && date.After(*p.endDate)) {
			continue
		}
		if p.patternType == "rotation" {
			anchor := p.startDate
			if p.anchorDate != nil {
				anchor = *p.anchorDate
			}
			cycleLen := 7
			if p.cycleLengthDays != nil && *p.cycleLengthDays > 0 {
				cycleLen = *p.cycleLengthDays
			}
			daysSince := normalizedCycleDay(calendarDayDifference(date, anchor), cycleLen)
			if p.scheduleDay == daysSince {
				matchedPatternShifts = append(matchedPatternShifts, p.templateID)
				pid := p.patternID
				matchedPatternID = &pid
			}
		} else {
			if p.scheduleDay == dow {
				matchedPatternShifts = append(matchedPatternShifts, p.templateID)
				pid := p.patternID
				matchedPatternID = &pid
			}
		}
	}
	if len(matchedPatternShifts) > 0 {
		traces := []api.ScheduleTrace{{
			RuleID:   matchedPatternID,
			RuleName: "Individual Pattern",
			Priority: 80,
			Status:   "applied",
			Reason:   new("Assigned via individual working pattern"),
		}}
		return matchedPatternShifts, "pattern", traces
	}

	var activeDeptID *uuid.UUID
	for _, d := range deptHistory {
		if !date.Before(d.effectiveDate) {
			id := d.departmentID
			activeDeptID = &id
			break
		}
	}

	traces := make([]api.ScheduleTrace, 0)
	allMatchedRules := make([]*assignmentRuleWithDetails, 0)
	highestPriority := 0
	for i := range rules {
		rule := &rules[i]
		if !matchRule(rule, date, activeDeptID, employeeIDNumber, tags, activeLocID, activeEmployerID) {
			continue
		}
		allMatchedRules = append(allMatchedRules, rule)
		if len(allMatchedRules) == 1 || rule.priority > highestPriority {
			highestPriority = rule.priority
		}
	}
	matchedRules := make([]*assignmentRuleWithDetails, 0, len(allMatchedRules))
	for _, rule := range allMatchedRules {
		if rule.priority == highestPriority {
			matchedRules = append(matchedRules, rule)
		}
	}
	for _, rule := range allMatchedRules {
		if rule.priority >= highestPriority {
			continue
		}
		rid := rule.id
		reason := "Overridden by higher priority rule"
		traces = append(traces, api.ScheduleTrace{RuleID: &rid, RuleName: rule.name, Priority: rule.priority, Status: "overridden", Reason: &reason})
	}
	if len(matchedRules) > 1 {
		for _, rule := range matchedRules {
			rid := rule.id
			reason := "Conflicts with another matching rule at same priority"
			traces = append(traces, api.ScheduleTrace{RuleID: &rid, RuleName: rule.name, Priority: rule.priority, Status: "conflict", Reason: &reason})
		}
		return []*uuid.UUID{nil}, "conflict", traces
	}
	if len(matchedRules) == 1 {
		rule := matchedRules[0]
		rid := rule.id
		reason := "Highest priority matching rule"
		traces = append(traces, api.ScheduleTrace{RuleID: &rid, RuleName: rule.name, Priority: rule.priority, Status: "applied", Reason: &reason})
		if sids, exists := rule.details[dow]; exists {
			ptrs := make([]*uuid.UUID, len(sids))
			for i := range sids {
				id := sids[i]
				if id != uuid.Nil {
					ptrs[i] = &id
				}
			}
			return ptrs, "assignment_rule", traces
		}
	}

	for _, f := range fixed {
		if !date.Before(f.startDate) && (f.endDate == nil || !date.After(*f.endDate)) {
			tid := f.templateID
			traces = append(traces, api.ScheduleTrace{
				RuleName: "Fixed Assignment",
				Priority: 20,
				Status:   "applied",
				Reason:   new("Directly assigned shift"),
			})
			return []*uuid.UUID{&tid}, "fixed", traces
		}
	}

	return []*uuid.UUID{nil}, "none", traces
}

func matchRule(rule *assignmentRuleWithDetails, date time.Time, deptID *uuid.UUID, empNumber string, tags []string, locID, employerID *uuid.UUID) bool {
	if rule.startDate != nil && date.Before(*rule.startDate) {
		return false
	}
	if rule.endDate != nil && date.After(*rule.endDate) {
		return false
	}
	if rule.cond.DepartmentID != nil {
		if deptID == nil || *deptID != *rule.cond.DepartmentID {
			return false
		}
	}
	if len(rule.cond.EmployeeIDNumbers) > 0 {
		matched := false
		for _, num := range rule.cond.EmployeeIDNumbers {
			if num == empNumber {
				matched = true
				break
			}
		}
		if !matched {
			return false
		}
	}
	if len(rule.cond.Tags) > 0 {
		matched := false
		for _, rt := range rule.cond.Tags {
			for _, et := range tags {
				if rt == et {
					matched = true
					break
				}
			}
			if matched {
				break
			}
		}
		if !matched {
			return false
		}
	}
	if rule.cond.LocationID != nil {
		if locID == nil || *locID != *rule.cond.LocationID {
			return false
		}
	}
	if rule.cond.EmployerID != nil {
		if employerID == nil || *employerID != *rule.cond.EmployerID {
			return false
		}
	}
	return true
}

func scanEmployeeRows(rows interface {
	Next() bool
	Scan(...any) error
	Err() error
}) ([]employeeRow, error) {
	empRows := make([]employeeRow, 0)
	for rows.Next() {
		var r employeeRow
		var startStr string
		var endStr *string
		if err := rows.Scan(&r.employmentID, &r.fullName, &r.employeeIDNumber, &r.tags, &r.primaryLocationID, &r.employeeID, &startStr, &endStr); err != nil {
			return nil, err
		}
		if parsed, err := time.Parse("2006-01-02", startStr); err == nil {
			r.startDate = parsed
		}
		if endStr != nil && *endStr != "" {
			if parsed, err := time.Parse("2006-01-02", *endStr); err == nil {
				r.endDate = &parsed
			}
		}
		empRows = append(empRows, r)
	}
	return empRows, rows.Err()
}

func loadContracts(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]contractEntry, error) {
	rows, err := db.Query(ctx, `SELECT ec.employment_id, emp.employer_id, ec.start_date::text, ec.end_date::text
		FROM employee_contracts ec
		JOIN employments emp ON emp.id = ec.employment_id
		WHERE ec.employment_id = ANY($1) AND ec.status = 'active' AND ec.deleted_at IS NULL
		AND ec.start_date <= $2::date AND (ec.end_date IS NULL OR ec.end_date >= $3::date)
		ORDER BY ec.start_date DESC`, empIDs, endStr, startStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID][]contractEntry)
	for rows.Next() {
		var empID, employerID uuid.UUID
		var sStr string
		var eStr *string
		if err := rows.Scan(&empID, &employerID, &sStr, &eStr); err != nil {
			return nil, err
		}
		s, _ := time.Parse("2006-01-02", sStr)
		var e *time.Time
		if eStr != nil && *eStr != "" {
			if p, err := time.Parse("2006-01-02", *eStr); err == nil {
				e = &p
			}
		}
		res[empID] = append(res[empID], contractEntry{employerID: employerID, startDate: s, endDate: e})
	}
	return res, rows.Err()
}

func loadAssignments(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]assignmentEntry, error) {
	rows, err := db.Query(ctx, `SELECT ea.employment_id, ea.host_location_id, ea.start_date::text, ea.end_date::text
		FROM employee_assignments ea
		WHERE ea.employment_id = ANY($1) AND ea.status = 'active'
		AND ea.start_date <= $2::date AND (ea.end_date IS NULL OR ea.end_date >= $3::date)
		ORDER BY ea.start_date DESC`, empIDs, endStr, startStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID][]assignmentEntry)
	for rows.Next() {
		var empID, hostID uuid.UUID
		var sStr string
		var eStr *string
		if err := rows.Scan(&empID, &hostID, &sStr, &eStr); err != nil {
			return nil, err
		}
		s, _ := time.Parse("2006-01-02", sStr)
		var e *time.Time
		if eStr != nil && *eStr != "" {
			if p, err := time.Parse("2006-01-02", *eStr); err == nil {
				e = &p
			}
		}
		res[empID] = append(res[empID], assignmentEntry{hostLocationID: hostID, startDate: s, endDate: e})
	}
	return res, rows.Err()
}

func loadBreakWorkedOverrides(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, startStr, endStr string) (map[scheduleDayKey][]api.ScheduleBreakWorkedOverride, error) {
	rows, err := db.Query(ctx, `SELECT id, employment_id, business_date::text, work_segment_id, start_at, end_at, minutes, status FROM schedule_day_overrides WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date AND override_type = 'break_worked' AND status = 'approved' ORDER BY employment_id, business_date, start_at, id`, employmentIDs, startStr, endStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make(map[scheduleDayKey][]api.ScheduleBreakWorkedOverride)
	for rows.Next() {
		var item api.ScheduleBreakWorkedOverride
		var employmentID uuid.UUID
		var date string
		if err := rows.Scan(&item.ID, &employmentID, &date, &item.SegmentID, &item.Start, &item.End, &item.Minutes, &item.Status); err != nil {
			return nil, err
		}
		key := scheduleDayKey{employmentID: employmentID, date: date}
		result[key] = append(result[key], item)
	}
	return result, rows.Err()
}

func loadHolidays(ctx context.Context, db DBTX, startStr, endStr string) (map[string]holidayEntry, error) {
	rows, err := db.Query(ctx, `SELECT date::text, name, is_paid FROM holidays WHERE date >= $1::date AND date <= $2::date`, startStr, endStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[string]holidayEntry)
	for rows.Next() {
		var dateStr, name string
		var isPaid bool
		if err := rows.Scan(&dateStr, &name, &isPaid); err != nil {
			return nil, err
		}
		res[dateStr] = holidayEntry{name: name, isPaid: isPaid}
	}
	return res, rows.Err()
}

type rosterAssignment struct {
	shiftID        *uuid.UUID
	shiftVersionID *uuid.UUID
}

func loadShiftRosters(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[string]rosterAssignment, error) {
	rows, err := db.Query(ctx, `SELECT employment_id, template_id, shift_version_id, date::text FROM shift_rosters WHERE employment_id = ANY($1) AND date >= $2::date AND date <= $3::date`, empIDs, startStr, endStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[string]rosterAssignment)
	for rows.Next() {
		var empID uuid.UUID
		var assignment rosterAssignment
		var dateStr string
		if err := rows.Scan(&empID, &assignment.shiftID, &assignment.shiftVersionID, &dateStr); err != nil {
			return nil, err
		}
		res[empID.String()+":"+dateStr] = assignment
	}
	return res, rows.Err()
}

func loadPatternAssignments(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]patternEntry, error) {
	rows, err := db.Query(ctx, `SELECT ewp.employment_id, ewp.pattern_id, ewp.start_date::text, ewp.end_date::text, ewp.anchor_date::text,
		wp.pattern_type, wp.cycle_length_days, wpd.schedule_day, wpd.template_id
		FROM employee_working_patterns ewp
		JOIN working_patterns wp ON wp.id = ewp.pattern_id
		JOIN working_pattern_details wpd ON wpd.pattern_id = wp.id
		WHERE ewp.employment_id = ANY($1) AND wp.deleted_at IS NULL AND wp.is_active = true
		AND ewp.start_date <= $2::date AND (ewp.end_date IS NULL OR ewp.end_date >= $3::date)
		ORDER BY ewp.created_at DESC`, empIDs, endStr, startStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID][]patternEntry)
	for rows.Next() {
		var empID, patID uuid.UUID
		var sStr string
		var eStr, aStr *string
		var pType string
		var cycleLen *int
		var sday int
		var tid *uuid.UUID
		if err := rows.Scan(&empID, &patID, &sStr, &eStr, &aStr, &pType, &cycleLen, &sday, &tid); err != nil {
			return nil, err
		}
		s, _ := time.Parse("2006-01-02", sStr)
		var e, a *time.Time
		if eStr != nil && *eStr != "" {
			if p, err := time.Parse("2006-01-02", *eStr); err == nil {
				e = &p
			}
		}
		if aStr != nil && *aStr != "" {
			if p, err := time.Parse("2006-01-02", *aStr); err == nil {
				a = &p
			}
		}
		res[empID] = append(res[empID], patternEntry{
			patternID:       patID,
			startDate:       s,
			endDate:         e,
			anchorDate:      a,
			patternType:     pType,
			cycleLengthDays: cycleLen,
			scheduleDay:     sday,
			templateID:      tid,
		})
	}
	return res, rows.Err()
}

func loadAssignmentRules(ctx context.Context, db DBTX) ([]assignmentRuleWithDetails, error) {
	rows, err := db.Query(ctx, `SELECT ar.id, ar.name, ar.priority, ar.conditions, ar.pattern_id, ar.start_date::text, ar.end_date::text,
		wpd.schedule_day, wpd.template_id
		FROM assignment_rules ar
		LEFT JOIN working_pattern_details wpd ON wpd.pattern_id = ar.pattern_id
		WHERE ar.is_active = true
		ORDER BY ar.priority DESC, ar.created_at ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	rulesMap := make(map[uuid.UUID]*assignmentRuleWithDetails)
	order := make([]uuid.UUID, 0)
	for rows.Next() {
		var id uuid.UUID
		var name string
		var priority int
		var condRaw []byte
		var patID uuid.UUID
		var sStr, eStr *string
		var sday *int
		var tid *uuid.UUID
		if err := rows.Scan(&id, &name, &priority, &condRaw, &patID, &sStr, &eStr, &sday, &tid); err != nil {
			return nil, err
		}
		r, exists := rulesMap[id]
		if !exists {
			var cond api.AssignmentRuleConditions
			_ = json.Unmarshal(condRaw, &cond)
			var s, e *time.Time
			if sStr != nil && *sStr != "" {
				if p, err := time.Parse("2006-01-02", *sStr); err == nil {
					s = &p
				}
			}
			if eStr != nil && *eStr != "" {
				if p, err := time.Parse("2006-01-02", *eStr); err == nil {
					e = &p
				}
			}
			r = &assignmentRuleWithDetails{
				id:        id,
				name:      name,
				priority:  priority,
				startDate: s,
				endDate:   e,
				patternID: patID,
				cond:      cond,
				details:   make(map[int][]uuid.UUID),
			}
			rulesMap[id] = r
			order = append(order, id)
		}
		if sday != nil && tid != nil {
			r.details[*sday] = append(r.details[*sday], *tid)
		}
	}
	res := make([]assignmentRuleWithDetails, len(order))
	for i, id := range order {
		res[i] = *rulesMap[id]
	}
	return res, rows.Err()
}

func loadFixedShifts(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]fixedShiftEntry, error) {
	rows, err := db.Query(ctx, `SELECT employment_id, template_id, start_date::text, end_date::text
		FROM employee_shifts
		WHERE employment_id = ANY($1) AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $3::date)
		ORDER BY created_at DESC`, empIDs, endStr, startStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID][]fixedShiftEntry)
	for rows.Next() {
		var empID, tid uuid.UUID
		var sStr string
		var eStr *string
		if err := rows.Scan(&empID, &tid, &sStr, &eStr); err != nil {
			return nil, err
		}
		s, _ := time.Parse("2006-01-02", sStr)
		var e *time.Time
		if eStr != nil && *eStr != "" {
			if p, err := time.Parse("2006-01-02", *eStr); err == nil {
				e = &p
			}
		}
		res[empID] = append(res[empID], fixedShiftEntry{templateID: tid, startDate: s, endDate: e})
	}
	return res, rows.Err()
}

func loadDeptHistory(ctx context.Context, db DBTX, employeeIDs []uuid.UUID) (map[uuid.UUID][]deptHistoryEntry, error) {
	rows, err := db.Query(ctx, `SELECT employee_id, department_id, start_date::text
		FROM employments
		WHERE employee_id = ANY($1) AND deleted_at IS NULL AND department_id IS NOT NULL
		ORDER BY start_date DESC, id DESC`, employeeIDs)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID][]deptHistoryEntry)
	for rows.Next() {
		var empID, deptID uuid.UUID
		var effStr string
		if err := rows.Scan(&empID, &deptID, &effStr); err != nil {
			return nil, err
		}
		eff, err := time.Parse("2006-01-02", effStr)
		if err != nil {
			return nil, err
		}
		res[empID] = append(res[empID], deptHistoryEntry{departmentID: deptID, effectiveDate: eff})
	}
	return res, rows.Err()
}

func loadShifts(ctx context.Context, db DBTX) (map[uuid.UUID]shiftTemplateInfo, error) {
	rows, err := db.Query(ctx, `SELECT st.id, st.name, sv.id, sv.report_symbol, sv.effective_from, sv.effective_to,
		sdi.sort_order, sdi.start_time::text, sdi.end_time::text, sdi.break_duration_minutes, sdi.break_start::text, sdi.break_end::text, sdi.break_punch_mode,
				COALESCE(sdi.start_day_offset, 0), COALESCE(sdi.end_day_offset, 0), sdi.start_utc_offset_minutes, sdi.end_utc_offset_minutes
		FROM shift_templates st
		LEFT JOIN shift_versions sv ON sv.template_id = st.id
		LEFT JOIN shift_default_intervals sdi ON sdi.version_id = sv.id
		ORDER BY st.id, sv.effective_from, sv.id, sdi.sort_order`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID]shiftTemplateInfo)
	for rows.Next() {
		var templateID uuid.UUID
		var info shiftTemplateInfo
		var versionID *uuid.UUID
		var reportSymbol *string
		var effectiveFrom, effectiveTo *time.Time
		var sortOrder *int
		var interval shiftIntervalInfo
		if err := rows.Scan(&templateID, &info.name, &versionID, &reportSymbol, &effectiveFrom, &effectiveTo,
			&sortOrder, &interval.startTime, &interval.endTime, &interval.breakDurationMinutes, &interval.breakStart, &interval.breakEnd,
			&interval.breakPunchMode, &interval.startDayOffset, &interval.endDayOffset, &interval.startUTCOffsetMinutes, &interval.endUTCOffsetMinutes); err != nil {
			return nil, err
		}
		current := res[templateID]
		current.name = info.name
		if versionID != nil && effectiveFrom != nil {
			if len(current.versions) == 0 || current.versions[len(current.versions)-1].id != *versionID {
				current.versions = append(current.versions, shiftVersionInfo{id: *versionID, reportSymbol: reportSymbol, effectiveFrom: *effectiveFrom, effectiveTo: effectiveTo})
			}
			if sortOrder != nil {
				interval.sortOrder = *sortOrder
				current.versions[len(current.versions)-1].intervals = append(current.versions[len(current.versions)-1].intervals, interval)
			}
		}
		res[templateID] = current
	}
	return res, rows.Err()
}
