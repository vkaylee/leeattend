package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/compliance"
)

const openEndedComplianceHorizonDays = 62

type employmentComplianceBounds struct {
	employeeID uuid.UUID
	startDate  time.Time
	endDate    *time.Time
}

func loadEmploymentComplianceBounds(ctx context.Context, db DBTX, employmentID uuid.UUID) (employmentComplianceBounds, error) {
	var bounds employmentComplianceBounds
	err := db.QueryRow(ctx, `SELECT employee_id, start_date, end_date FROM employments WHERE id = $1 AND deleted_at IS NULL`, employmentID).
		Scan(&bounds.employeeID, &bounds.startDate, &bounds.endDate)
	if errors.Is(err, pgx.ErrNoRows) {
		return employmentComplianceBounds{}, api.ErrEmploymentNotFound
	}
	return bounds, err
}

func resolveComplianceEvalEnd(startDate time.Time, explicitEnd *time.Time, bounds employmentComplianceBounds) (time.Time, bool) {
	if startDate.Before(bounds.startDate) {
		return time.Time{}, false
	}
	if bounds.endDate != nil && startDate.After(*bounds.endDate) {
		return time.Time{}, false
	}
	if explicitEnd != nil {
		if explicitEnd.Before(startDate) {
			return time.Time{}, false
		}
		if bounds.endDate != nil && explicitEnd.After(*bounds.endDate) {
			return time.Time{}, false
		}
		return *explicitEnd, true
	}
	evalEnd := startDate.AddDate(0, 0, openEndedComplianceHorizonDays)
	if bounds.endDate != nil && bounds.endDate.Before(evalEnd) {
		evalEnd = *bounds.endDate
	}
	return evalEnd, true
}

func loadComplianceRules(ctx context.Context, db DBTX) (compliance.Rules, error) {
	rules := compliance.Rules{IsEnabled: true, MinRestPeriodHours: 12, MaxWeeklyHours: 48, MaxMonthlyOTHours: 40, AllowManagerOverride: true}
	var stored compliance.Rules
	err := db.QueryRow(ctx, `SELECT is_enabled, min_rest_period_hours, max_weekly_hours, max_monthly_ot_hours, allow_manager_override FROM compliance_rules`).
		Scan(&stored.IsEnabled, &stored.MinRestPeriodHours, &stored.MaxWeeklyHours, &stored.MaxMonthlyOTHours, &stored.AllowManagerOverride)
	if err == nil {
		return stored, nil
	}
	if errors.Is(err, pgx.ErrNoRows) {
		return rules, nil
	}
	return compliance.Rules{}, err
}

func complianceContextWindow(evalStart, evalEnd time.Time) (time.Time, time.Time) {
	weekStart := evalStart.AddDate(0, 0, -int(evalStart.Weekday()+6)%7)
	monthStart := time.Date(evalStart.Year(), evalStart.Month(), 1, 0, 0, 0, 0, time.UTC)
	prevDay := evalStart.AddDate(0, 0, -1)
	winStart := weekStart
	if monthStart.Before(winStart) {
		winStart = monthStart
	}
	if prevDay.Before(winStart) {
		winStart = prevDay
	}

	weekEnd := evalEnd.AddDate(0, 0, 6-int(evalEnd.Weekday()+6)%7)
	monthEnd := time.Date(evalEnd.Year(), evalEnd.Month()+1, 0, 0, 0, 0, 0, time.UTC)
	nextDay := evalEnd.AddDate(0, 0, 1)
	winEnd := weekEnd
	if monthEnd.After(winEnd) {
		winEnd = monthEnd
	}
	if nextDay.After(winEnd) {
		winEnd = nextDay
	}
	return winStart, winEnd
}

func versionToComplianceIntervals(version shiftVersionInfo) []compliance.Interval {
	out := make([]compliance.Interval, 0, len(version.intervals))
	for _, iv := range version.intervals {
		if iv.startTime == nil || iv.endTime == nil {
			continue
		}
		breakMins := 0
		if iv.breakDurationMinutes != nil {
			breakMins = *iv.breakDurationMinutes
		}
		autoDeduct := false
		if iv.isBreakAutoDeduct != nil {
			autoDeduct = *iv.isBreakAutoDeduct
		}
		out = append(out, compliance.Interval{
			StartTime:            *iv.startTime,
			EndTime:              *iv.endTime,
			BreakDurationMinutes: breakMins,
			IsBreakAutoDeduct:    autoDeduct,
			StartDayOffset:       iv.startDayOffset,
			EndDayOffset:         iv.endDayOffset,
		})
	}
	return out
}

func effectiveTemplateComplianceIntervals(shifts map[uuid.UUID]shiftTemplateInfo, templateID uuid.UUID, date time.Time) []compliance.Interval {
	version, ok, _ := selectEffectiveShiftVersion(shifts, templateID, date)
	if !ok {
		return nil
	}
	return versionToComplianceIntervals(version)
}

type compliancePersonEmployment struct {
	employmentID      uuid.UUID
	employeeID        uuid.UUID
	employeeIDNumber  string
	primaryLocationID *uuid.UUID
	startDate         time.Time
	endDate           *time.Time
	tags              []string
}

type officialComplianceKey struct {
	empID uuid.UUID
	date  string
}

func loadPersonEmploymentsForCompliance(ctx context.Context, db DBTX, employeeID uuid.UUID, startStr, endStr string) ([]compliancePersonEmployment, []uuid.UUID, error) {
	empRows, err := db.Query(ctx, `
		SELECT emp.id, emp.employee_id, e.employee_id_number, emp.primary_location_id, emp.start_date, emp.end_date, COALESCE(e.tags, '{}')
		FROM employments emp
		JOIN employees e ON e.id = emp.employee_id
		WHERE emp.employee_id = $1
		  AND emp.deleted_at IS NULL
		  AND e.deleted_at IS NULL
		  AND emp.start_date <= $3::date
		  AND (emp.end_date IS NULL OR emp.end_date >= $2::date)
		ORDER BY emp.id
	`, employeeID, startStr, endStr)
	if err != nil {
		return nil, nil, err
	}
	defer empRows.Close()
	emps := make([]compliancePersonEmployment, 0, 2)
	empIDs := make([]uuid.UUID, 0, 2)
	for empRows.Next() {
		var row compliancePersonEmployment
		if err := empRows.Scan(&row.employmentID, &row.employeeID, &row.employeeIDNumber, &row.primaryLocationID, &row.startDate, &row.endDate, &row.tags); err != nil {
			return nil, nil, err
		}
		emps = append(emps, row)
		empIDs = append(empIDs, row.employmentID)
	}
	return emps, empIDs, empRows.Err()
}

func appendOfficialComplianceSegment(officialMap map[officialComplianceKey]*compliance.Roster, key officialComplianceKey, tplID *uuid.UUID, segStart, segEnd string, startOffset, endOffset int) {
	entry, ok := officialMap[key]
	if !ok {
		entry = &compliance.Roster{EmploymentID: key.empID, ShiftID: tplID, Date: key.date}
		officialMap[key] = entry
	}
	if entry.ShiftID == nil && tplID != nil {
		entry.ShiftID = tplID
	}
	entry.Intervals = append(entry.Intervals, compliance.Interval{
		StartTime:      segStart,
		EndTime:        segEnd,
		StartDayOffset: startOffset,
		EndDayOffset:   endOffset,
	})
}

func loadOfficialComplianceRosters(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[officialComplianceKey]bool, []compliance.Roster, error) {
	revRows, err := db.Query(ctx, `
		SELECT sdr.employment_id, sdr.business_date::text, sdr.state,
		       ws.template_id, ws.scheduled_start::text, ws.scheduled_end::text,
		       COALESCE(ws.start_day_offset, 0), COALESCE(ws.end_day_offset, 0)
		FROM schedule_day_revisions sdr
		LEFT JOIN work_segments ws ON ws.schedule_revision_id = sdr.id
		WHERE sdr.employment_id = ANY($1)
		  AND sdr.business_date BETWEEN $2::date AND $3::date
		  AND sdr.state IN ('published', 'explicit_off')
		ORDER BY sdr.employment_id, sdr.business_date, ws.sort_order, ws.id
	`, empIDs, startStr, endStr)
	if err != nil {
		return nil, nil, err
	}
	defer revRows.Close()
	hasOfficial := make(map[officialComplianceKey]bool)
	officialMap := make(map[officialComplianceKey]*compliance.Roster)
	for revRows.Next() {
		var empID uuid.UUID
		var dateStr, state string
		var tplID *uuid.UUID
		var segStart, segEnd *string
		var startOffset, endOffset int
		if err := revRows.Scan(&empID, &dateStr, &state, &tplID, &segStart, &segEnd, &startOffset, &endOffset); err != nil {
			return nil, nil, err
		}
		key := officialComplianceKey{empID: empID, date: dateStr}
		hasOfficial[key] = true
		if state == "published" && segStart != nil && segEnd != nil {
			appendOfficialComplianceSegment(officialMap, key, tplID, *segStart, *segEnd, startOffset, endOffset)
		}
	}
	if err := revRows.Err(); err != nil {
		return nil, nil, err
	}
	out := make([]compliance.Roster, 0, len(officialMap))
	for _, entry := range officialMap {
		out = append(out, *entry)
	}
	return hasOfficial, out, nil
}

type provisionalComplianceInputs struct {
	contracts   map[uuid.UUID][]contractEntry
	assignments map[uuid.UUID][]assignmentEntry
	rosters     map[string]rosterAssignment
	patterns    map[uuid.UUID][]patternEntry
	rules       []assignmentRuleWithDetails
	fixed       map[uuid.UUID][]fixedShiftEntry
	deptHistory map[uuid.UUID][]deptHistoryEntry
}

func loadProvisionalComplianceInputs(ctx context.Context, db DBTX, employeeID uuid.UUID, empIDs []uuid.UUID, startStr, endStr string) (provisionalComplianceInputs, error) {
	var in provisionalComplianceInputs
	var err error
	if in.contracts, err = loadContracts(ctx, db, empIDs, startStr, endStr); err != nil {
		return in, err
	}
	if in.assignments, err = loadAssignments(ctx, db, empIDs, startStr, endStr); err != nil {
		return in, err
	}
	if in.rosters, err = loadShiftRosters(ctx, db, empIDs, startStr, endStr); err != nil {
		return in, err
	}
	if in.patterns, err = loadPatternAssignments(ctx, db, empIDs, startStr, endStr); err != nil {
		return in, err
	}
	if in.rules, err = loadAssignmentRules(ctx, db); err != nil {
		return in, err
	}
	if in.fixed, err = loadFixedShifts(ctx, db, empIDs, startStr, endStr); err != nil {
		return in, err
	}
	in.deptHistory, err = loadDeptHistory(ctx, db, []uuid.UUID{employeeID})
	return in, err
}

func activeContractEmployer(contracts []contractEntry, date time.Time) *uuid.UUID {
	for _, c := range contracts {
		if !date.Before(c.startDate) && (c.endDate == nil || !date.After(*c.endDate)) {
			id := c.employerID
			return &id
		}
	}
	return nil
}

func activeAssignmentLocation(primary *uuid.UUID, assignments []assignmentEntry, date time.Time) *uuid.UUID {
	for _, a := range assignments {
		if !date.Before(a.startDate) && (a.endDate == nil || !date.After(*a.endDate)) {
			id := a.hostLocationID
			return &id
		}
	}
	return primary
}

func appendProvisionalEmploymentRosters(result []compliance.Roster, emp compliancePersonEmployment, winStart, winEnd time.Time, hasOfficial map[officialComplianceKey]bool, inputs provisionalComplianceInputs, shiftsMeta map[uuid.UUID]shiftTemplateInfo) []compliance.Roster {
	schedStart, schedEnd := clampDateRange(winStart, winEnd, emp.startDate, emp.endDate)
	for current := schedStart; !current.After(schedEnd); current = current.AddDate(0, 0, 1) {
		dateStr := current.Format("2006-01-02")
		if hasOfficial[officialComplianceKey{empID: emp.employmentID, date: dateStr}] {
			continue
		}
		shiftIDs, _, _ := resolveEffectiveShift(
			emp.employmentID, emp.employeeID, emp.employeeIDNumber, emp.tags,
			current, int(current.Weekday()),
			activeAssignmentLocation(emp.primaryLocationID, inputs.assignments[emp.employmentID], current),
			activeContractEmployer(inputs.contracts[emp.employmentID], current),
			inputs.rosters, inputs.patterns[emp.employmentID], inputs.rules, inputs.fixed[emp.employmentID], inputs.deptHistory[emp.employeeID],
		)
		for _, shiftID := range shiftIDs {
			if shiftID == nil {
				continue
			}
			idCopy := *shiftID
			result = append(result, compliance.Roster{
				EmploymentID: emp.employmentID,
				ShiftID:      &idCopy,
				Date:         dateStr,
				Intervals:    effectiveTemplateComplianceIntervals(shiftsMeta, *shiftID, current),
			})
		}
	}
	return result
}

func loadResolvedComplianceSchedule(ctx context.Context, db DBTX, employeeID uuid.UUID, evalStart, evalEnd time.Time) ([]compliance.Roster, map[uuid.UUID]shiftTemplateInfo, error) {
	winStart, winEnd := complianceContextWindow(evalStart, evalEnd)
	startStr, endStr := winStart.Format("2006-01-02"), winEnd.Format("2006-01-02")

	emps, empIDs, err := loadPersonEmploymentsForCompliance(ctx, db, employeeID, startStr, endStr)
	if err != nil {
		return nil, nil, err
	}
	shiftsMeta, err := loadShifts(ctx, db)
	if err != nil || len(empIDs) == 0 {
		return nil, shiftsMeta, err
	}
	hasOfficial, result, err := loadOfficialComplianceRosters(ctx, db, empIDs, startStr, endStr)
	if err != nil {
		return nil, nil, err
	}
	inputs, err := loadProvisionalComplianceInputs(ctx, db, employeeID, empIDs, startStr, endStr)
	if err != nil {
		return nil, nil, err
	}
	for _, emp := range emps {
		result = appendProvisionalEmploymentRosters(result, emp, winStart, winEnd, hasOfficial, inputs, shiftsMeta)
	}
	return result, shiftsMeta, nil
}

func clampDateRange(winStart, winEnd, empStart time.Time, empEnd *time.Time) (time.Time, time.Time) {
	s, e := winStart, winEnd
	if empStart.After(s) {
		s = empStart
	}
	if empEnd != nil && empEnd.Before(e) {
		e = *empEnd
	}
	return s, e
}

// patternComplianceValidator validates schedule pattern compliance:
// expand assignment dates using calendar-date arithmetic, resolve effective shift
// versions per date, load full resolved schedules across adjacent/week/month and
// cross-employment windows, then validate the entire pattern atomically.
func patternComplianceValidator(ctx context.Context, db DBTX, employmentID uuid.UUID, req api.AssignPatternRequest) error {
	if req.PatternID == uuid.Nil {
		return api.ErrPatternValidation
	}
	bounds, err := loadEmploymentComplianceBounds(ctx, db, employmentID)
	if err != nil {
		return err
	}
	var patternType string
	var cycleLength *int
	err = db.QueryRow(ctx, `SELECT pattern_type, cycle_length_days FROM working_patterns WHERE id = $1 AND deleted_at IS NULL`, req.PatternID).Scan(&patternType, &cycleLength)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.ErrPatternNotFound
	}
	if err != nil {
		return err
	}
	rows, err := db.Query(ctx, `SELECT schedule_day, template_id FROM working_pattern_details WHERE pattern_id = $1 ORDER BY schedule_day`, req.PatternID)
	if err != nil {
		return err
	}
	defer rows.Close()
	shiftByDay := make(map[int][]uuid.UUID)
	for rows.Next() {
		var day int
		var templateID *uuid.UUID
		if err := rows.Scan(&day, &templateID); err != nil {
			return err
		}
		if templateID != nil {
			shiftByDay[day] = append(shiftByDay[day], *templateID)
		}
	}
	if err := rows.Err(); err != nil {
		return err
	}
	if len(shiftByDay) == 0 {
		return nil
	}

	start, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return api.ErrPatternValidation
	}
	var explicitEnd *time.Time
	if req.EndDate != nil && *req.EndDate != "" {
		parsed, parseErr := time.Parse("2006-01-02", *req.EndDate)
		if parseErr != nil {
			return api.ErrPatternValidation
		}
		explicitEnd = &parsed
	}
	evalEnd, validBounds := resolveComplianceEvalEnd(start, explicitEnd, bounds)
	if !validBounds {
		return api.ErrPatternValidation
	}

	anchor := start
	if req.AnchorDate != nil && *req.AnchorDate != "" {
		parsedAnchor, parseErr := time.Parse("2006-01-02", *req.AnchorDate)
		if parseErr != nil {
			return api.ErrPatternValidation
		}
		anchor = parsedAnchor
	}

	rules, err := loadComplianceRules(ctx, db)
	if err != nil {
		return err
	}
	if !rules.IsEnabled {
		return nil
	}

	cycleDays := 7
	if cycleLength != nil && *cycleLength > 0 {
		cycleDays = *cycleLength
	}
	existing, shiftsMeta, err := loadResolvedComplianceSchedule(ctx, db, bounds.employeeID, start, evalEnd)
	if err != nil {
		return err
	}

	candidates := make([]compliance.Roster, 0)
	count := 0
	for current := start; !current.After(evalEnd) && count < 366; current = current.AddDate(0, 0, 1) {
		var dayKey int
		if patternType == "rotation" {
			dayKey = normalizedCycleDay(calendarDayDifference(current, anchor), cycleDays)
		} else {
			dayKey = int(current.Weekday())
		}
		for _, tplID := range shiftByDay[dayKey] {
			idCopy := tplID
			candidates = append(candidates, compliance.Roster{
				EmploymentID: employmentID,
				ShiftID:      &idCopy,
				Date:         current.Format("2006-01-02"),
				Intervals:    effectiveTemplateComplianceIntervals(shiftsMeta, tplID, current),
			})
		}
		count++
	}
	if len(candidates) == 0 {
		return nil
	}

	result, validationErr := compliance.Validate(rules, compliance.Request{
		EmploymentID:   employmentID,
		CandidateDays:  candidates,
		AllowWarnings:  req.AllowWarnings,
		OverrideReason: req.OverrideReason,
	}, existing, nil)
	if validationErr != nil {
		return &api.ShiftComplianceError{Hard: errors.Is(validationErr, compliance.ErrHardViolation), Violations: result.Violations}
	}
	return nil
}

func (r WorkingPatternRepository) CheckPatternCompliance(ctx context.Context, employmentID uuid.UUID, req api.AssignPatternRequest) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	return patternComplianceValidator(ctx, db, employmentID, req)
}

func (r WorkingPatternRepository) CheckBulkPatternCompliance(ctx context.Context, req api.BulkAssignPatternRequest) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	for _, empID := range req.EmploymentIDs {
		assignReq := api.AssignPatternRequest{
			PatternID:      req.PatternID,
			StartDate:      req.StartDate,
			EndDate:        req.EndDate,
			AnchorDate:     req.AnchorDate,
			AllowWarnings:  req.AllowWarnings,
			OverrideReason: req.OverrideReason,
		}
		if err := patternComplianceValidator(ctx, db, empID, assignReq); err != nil {
			return err
		}
	}
	return nil
}
