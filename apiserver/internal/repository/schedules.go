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
	DB DBTX
}

func (r ScheduleRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("schedule repository: tenant database is not configured")
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
	name                           string
	reportSymbol                   *string
	startTime                      *string
	endTime                        *string
	breakDurationMinutes           *int
	breakStart                     *string
	breakEnd                       *string
	isBreakAutoDeduct              *bool
	earlyPunchInGracePeriodMinutes *int
	allowEarlyOT                   *bool
	breakIsOvertime                *bool
	maxEarlyOTMinutes              *int
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

func (r ScheduleRepository) GetWorkspaceSchedules(ctx context.Context, workspaceID uuid.UUID, startDate, endDate time.Time, employmentIDs []uuid.UUID, page, limit int) (api.PaginatedEmployeeSchedules, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	if page < 1 {
		page = 1
	}
	if limit < 1 {
		limit = 50
	}
	if limit > api.MaxLimit {
		limit = api.MaxLimit
	}
	offset := (page - 1) * limit

	var total int64
	var empRows []employeeRow
	if len(employmentIDs) > 0 {
		countErr := db.QueryRow(ctx, `SELECT count(*) FROM employees e
			JOIN employments emp ON emp.employee_id = e.id
			WHERE e.workspace_id = $1 AND emp.deleted_at IS NULL AND emp.id = ANY($2)`, workspaceID, employmentIDs).Scan(&total)
		if countErr != nil {
			return api.PaginatedEmployeeSchedules{}, countErr
		}
		rows, qErr := db.Query(ctx, `SELECT emp.id, e.full_name, e.employee_id_number, e.tags, emp.primary_location_id, e.id, emp.start_date::text, emp.end_date::text
			FROM employees e
			JOIN employments emp ON emp.employee_id = e.id
			WHERE e.workspace_id = $1 AND emp.deleted_at IS NULL AND emp.id = ANY($2)
			ORDER BY emp.start_date DESC, emp.id ASC
			LIMIT $3 OFFSET $4`, workspaceID, employmentIDs, limit, offset)
		if qErr != nil {
			return api.PaginatedEmployeeSchedules{}, qErr
		}
		defer rows.Close()
		empRows, err = scanEmployeeRows(rows)
		if err != nil {
			return api.PaginatedEmployeeSchedules{}, err
		}
	} else {
		startStr := startDate.Format("2006-01-02")
		endStr := endDate.Format("2006-01-02")
		countErr := db.QueryRow(ctx, `SELECT count(*) FROM employees e
			JOIN employments emp ON emp.employee_id = e.id
			WHERE e.workspace_id = $1 AND emp.deleted_at IS NULL
			AND emp.start_date <= $2::date AND (emp.end_date IS NULL OR emp.end_date >= $3::date)`, workspaceID, endStr, startStr).Scan(&total)
		if countErr != nil {
			return api.PaginatedEmployeeSchedules{}, countErr
		}
		rows, qErr := db.Query(ctx, `SELECT emp.id, e.full_name, e.employee_id_number, e.tags, emp.primary_location_id, e.id, emp.start_date::text, emp.end_date::text
			FROM employees e
			JOIN employments emp ON emp.employee_id = e.id
			WHERE e.workspace_id = $1 AND emp.deleted_at IS NULL
			AND emp.start_date <= $2::date AND (emp.end_date IS NULL OR emp.end_date >= $3::date)
			ORDER BY emp.start_date DESC, emp.id ASC
			LIMIT $4 OFFSET $5`, workspaceID, endStr, startStr, limit, offset)
		if qErr != nil {
			return api.PaginatedEmployeeSchedules{}, qErr
		}
		defer rows.Close()
		empRows, err = scanEmployeeRows(rows)
		if err != nil {
			return api.PaginatedEmployeeSchedules{}, err
		}
	}

	if len(empRows) == 0 {
		return api.PaginatedEmployeeSchedules{Data: []api.EmployeeSchedule{}, Total: total, Page: page, Limit: limit}, nil
	}

	targetEmploymentIDs := make([]uuid.UUID, len(empRows))
	targetEmployeeIDs := make([]uuid.UUID, len(empRows))
	for i, row := range empRows {
		targetEmploymentIDs[i] = row.employmentID
		targetEmployeeIDs[i] = row.employeeID
	}

	startStr := startDate.Format("2006-01-02")
	endStr := endDate.Format("2006-01-02")

	contracts, err := loadContracts(ctx, db, workspaceID, targetEmploymentIDs, startStr, endStr)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	assignments, err := loadAssignments(ctx, db, workspaceID, targetEmploymentIDs, startStr, endStr)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	holidays, err := loadHolidays(ctx, db, workspaceID, startStr, endStr)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	rosters, err := loadShiftRosters(ctx, db, targetEmploymentIDs, startStr, endStr)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	patternAssignments, err := loadPatternAssignments(ctx, db, workspaceID, targetEmploymentIDs, startStr, endStr)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	rules, err := loadAssignmentRules(ctx, db, workspaceID)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	fixedShifts, err := loadFixedShifts(ctx, db, targetEmploymentIDs, startStr, endStr)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	deptHistory, err := loadDeptHistory(ctx, db, targetEmployeeIDs)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}
	shifts, err := loadShifts(ctx, db, workspaceID)
	if err != nil {
		return api.PaginatedEmployeeSchedules{}, err
	}

	result := make([]api.EmployeeSchedule, 0, len(empRows))
	for _, emp := range empRows {
		schedStart := startDate
		if emp.startDate.After(schedStart) {
			schedStart = emp.startDate
		}
		schedEnd := endDate
		if emp.endDate != nil && emp.endDate.Before(schedEnd) {
			schedEnd = *emp.endDate
		}
		if schedStart.After(schedEnd) {
			continue
		}

		empContracts := contracts[emp.employmentID]
		empAssignments := assignments[emp.employmentID]
		empPatterns := patternAssignments[emp.employmentID]
		empFixed := fixedShifts[emp.employmentID]
		empDept := deptHistory[emp.employeeID]

		dailyList := make([]api.EffectiveSchedule, 0)
		for cur := schedStart; !cur.After(schedEnd); cur = cur.AddDate(0, 0, 1) {
			curStr := cur.Format("2006-01-02")
			dow := int(cur.Weekday())

			var activeEmployerID *uuid.UUID
			for _, c := range empContracts {
				if !cur.Before(c.startDate) && (c.endDate == nil || !cur.After(*c.endDate)) {
					id := c.employerID
					activeEmployerID = &id
					break
				}
			}

			activeLocID := emp.primaryLocationID
			for _, a := range empAssignments {
				if !cur.Before(a.startDate) && (a.endDate == nil || !cur.After(*a.endDate)) {
					id := a.hostLocationID
					activeLocID = &id
					break
				}
			}

			holiday, isHoliday := holidays[curStr]

			shiftIDs, source, traces := resolveEffectiveShift(
				emp.employmentID, emp.employeeID, emp.employeeIDNumber, emp.tags,
				cur, dow, activeLocID, activeEmployerID,
				rosters, empPatterns, rules, empFixed, empDept,
			)

			for _, sid := range shiftIDs {
				effective := api.EffectiveSchedule{
					Date:      curStr,
					ShiftID:   sid,
					Source:    source,
					Traces:    traces,
					IsHoliday: isHoliday,
				}
				if isHoliday {
					effective.IsHolidayPaid = holiday.isPaid
					name := holiday.name
					effective.HolidayName = &name
				}
				if sid != nil {
					if meta, ok := shifts[*sid]; ok {
						effective.ShiftName = &meta.name
						effective.ShiftSymbol = meta.reportSymbol
						effective.StartTime = meta.startTime
						effective.EndTime = meta.endTime
						effective.BreakDurationMinutes = meta.breakDurationMinutes
						effective.BreakStart = meta.breakStart
						effective.BreakEnd = meta.breakEnd
						effective.IsBreakAutoDeduct = meta.isBreakAutoDeduct
						effective.EarlyPunchInGracePeriodMinutes = meta.earlyPunchInGracePeriodMinutes
						effective.AllowEarlyOT = meta.allowEarlyOT
						effective.BreakIsOvertime = meta.breakIsOvertime
						effective.MaxEarlyOTMinutes = meta.maxEarlyOTMinutes
					}
				}
				dailyList = append(dailyList, effective)
			}
		}

		result = append(result, api.EmployeeSchedule{
			EmploymentID:     emp.employmentID,
			EmployeeID:       emp.employeeID,
			EmployeeIDNumber: emp.employeeIDNumber,
			FullName:         emp.fullName,
			Schedules:        dailyList,
		})
	}

	return api.PaginatedEmployeeSchedules{Data: result, Total: total, Page: page, Limit: limit}, nil
}

func resolveEffectiveShift(
	employmentID, employeeID uuid.UUID,
	employeeIDNumber string,
	tags []string,
	date time.Time,
	dow int,
	activeLocID, activeEmployerID *uuid.UUID,
	rosters map[string]*uuid.UUID,
	patterns []patternEntry,
	rules []assignmentRuleWithDetails,
	fixed []fixedShiftEntry,
	deptHistory []deptHistoryEntry,
) ([]*uuid.UUID, string, []api.ScheduleTrace) {
	dateStr := date.Format("2006-01-02")
	key := employmentID.String() + ":" + dateStr

	if sid, exists := rosters[key]; exists {
		traces := []api.ScheduleTrace{{
			RuleName: "Manual Roster",
			Priority: 100,
			Status:   "applied",
			Reason:   new("Directly assigned via roster"),
		}}
		return []*uuid.UUID{sid}, "roster", traces
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
			daysSince := int(date.Sub(anchor).Hours()/24) % cycleLen
			if daysSince < 0 {
				daysSince += cycleLen
			}
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
	var winnerRule *assignmentRuleWithDetails
	for i := range rules {
		rule := &rules[i]
		if matchRule(rule, date, activeDeptID, employeeIDNumber, tags, activeLocID, activeEmployerID) {
			rid := rule.id
			if winnerRule == nil {
				winnerRule = rule
				traces = append(traces, api.ScheduleTrace{
					RuleID:   &rid,
					RuleName: rule.name,
					Priority: rule.priority,
					Status:   "applied",
					Reason:   new("Highest priority matching rule"),
				})
			} else {
				traces = append(traces, api.ScheduleTrace{
					RuleID:   &rid,
					RuleName: rule.name,
					Priority: rule.priority,
					Status:   "overridden",
					Reason:   new("Overridden by higher priority rule"),
				})
			}
		}
	}
	if winnerRule != nil {
		if sids, exists := winnerRule.details[dow]; exists && len(sids) > 0 {
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

func loadContracts(ctx context.Context, db DBTX, wsID uuid.UUID, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]contractEntry, error) {
	rows, err := db.Query(ctx, `SELECT ec.employment_id, emp.employer_id, ec.start_date::text, ec.end_date::text
		FROM employee_contracts ec
		JOIN employments emp ON emp.id = ec.employment_id
		WHERE ec.workspace_id = $1 AND ec.employment_id = ANY($2) AND ec.status = 'active' AND ec.deleted_at IS NULL
		AND ec.start_date <= $3::date AND (ec.end_date IS NULL OR ec.end_date >= $4::date)
		ORDER BY ec.start_date DESC`, wsID, empIDs, endStr, startStr)
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

func loadAssignments(ctx context.Context, db DBTX, wsID uuid.UUID, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]assignmentEntry, error) {
	rows, err := db.Query(ctx, `SELECT ea.employment_id, ea.host_location_id, ea.start_date::text, ea.end_date::text
		FROM employee_assignments ea
		WHERE ea.workspace_id = $1 AND ea.employment_id = ANY($2) AND ea.status = 'active'
		AND ea.start_date <= $3::date AND (ea.end_date IS NULL OR ea.end_date >= $4::date)
		ORDER BY ea.start_date DESC`, wsID, empIDs, endStr, startStr)
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

func loadHolidays(ctx context.Context, db DBTX, wsID uuid.UUID, startStr, endStr string) (map[string]holidayEntry, error) {
	rows, err := db.Query(ctx, `SELECT date::text, name, is_paid FROM holidays WHERE workspace_id = $1 AND date >= $2::date AND date <= $3::date`, wsID, startStr, endStr)
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

func loadShiftRosters(ctx context.Context, db DBTX, empIDs []uuid.UUID, startStr, endStr string) (map[string]*uuid.UUID, error) {
	rows, err := db.Query(ctx, `SELECT employment_id, template_id, date::text FROM shift_rosters WHERE employment_id = ANY($1) AND date >= $2::date AND date <= $3::date`, empIDs, startStr, endStr)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[string]*uuid.UUID)
	for rows.Next() {
		var empID uuid.UUID
		var tid *uuid.UUID
		var dateStr string
		if err := rows.Scan(&empID, &tid, &dateStr); err != nil {
			return nil, err
		}
		res[empID.String()+":"+dateStr] = tid
	}
	return res, rows.Err()
}

func loadPatternAssignments(ctx context.Context, db DBTX, wsID uuid.UUID, empIDs []uuid.UUID, startStr, endStr string) (map[uuid.UUID][]patternEntry, error) {
	rows, err := db.Query(ctx, `SELECT ewp.employment_id, ewp.pattern_id, ewp.start_date::text, ewp.end_date::text, ewp.anchor_date::text,
		wp.pattern_type, wp.cycle_length_days, wpd.schedule_day, wpd.template_id
		FROM employee_working_patterns ewp
		JOIN working_patterns wp ON wp.id = ewp.pattern_id
		JOIN working_pattern_details wpd ON wpd.pattern_id = wp.id
		WHERE ewp.workspace_id = $1 AND ewp.employment_id = ANY($2) AND wp.deleted_at IS NULL AND wp.is_active = true
		AND ewp.start_date <= $3::date AND (ewp.end_date IS NULL OR ewp.end_date >= $4::date)
		ORDER BY ewp.created_at DESC`, wsID, empIDs, endStr, startStr)
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

func loadAssignmentRules(ctx context.Context, db DBTX, wsID uuid.UUID) ([]assignmentRuleWithDetails, error) {
	rows, err := db.Query(ctx, `SELECT ar.id, ar.name, ar.priority, ar.conditions, ar.pattern_id, ar.start_date::text, ar.end_date::text,
		wpd.schedule_day, wpd.template_id
		FROM assignment_rules ar
		LEFT JOIN working_pattern_details wpd ON wpd.pattern_id = ar.pattern_id
		WHERE ar.workspace_id = $1 AND ar.is_active = true
		ORDER BY ar.priority DESC, ar.created_at ASC`, wsID)
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
	rows, err := db.Query(ctx, `SELECT employee_id, department_id, effective_date::text FROM employee_department_history WHERE employee_id = ANY($1) ORDER BY effective_date DESC`, employeeIDs)
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
		eff, _ := time.Parse("2006-01-02", effStr)
		res[empID] = append(res[empID], deptHistoryEntry{departmentID: deptID, effectiveDate: eff})
	}
	return res, rows.Err()
}

func loadShifts(ctx context.Context, db DBTX, wsID uuid.UUID) (map[uuid.UUID]shiftTemplateInfo, error) {
	rows, err := db.Query(ctx, `SELECT st.id, st.name, sv.report_symbol, sdi.start_time::text, sdi.end_time::text,
		sdi.break_duration_minutes, sdi.break_start::text, sdi.break_end::text, sdi.is_break_auto_deduct,
		sdi.early_punch_in_grace_period_minutes, sdi.allow_early_ot, sdi.break_is_overtime, sdi.max_early_ot_minutes
		FROM shift_templates st
		LEFT JOIN shift_versions sv ON sv.id = st.current_version_id
		LEFT JOIN shift_default_intervals sdi ON sdi.version_id = sv.id AND sdi.sort_order = 0
		WHERE st.workspace_id = $1`, wsID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	res := make(map[uuid.UUID]shiftTemplateInfo)
	for rows.Next() {
		var id uuid.UUID
		var info shiftTemplateInfo
		if err := rows.Scan(&id, &info.name, &info.reportSymbol, &info.startTime, &info.endTime, &info.breakDurationMinutes, &info.breakStart, &info.breakEnd, &info.isBreakAutoDeduct, &info.earlyPunchInGracePeriodMinutes, &info.allowEarlyOT, &info.breakIsOvertime, &info.maxEarlyOTMinutes); err != nil {
			return nil, err
		}
		res[id] = info
	}
	return res, rows.Err()
}
