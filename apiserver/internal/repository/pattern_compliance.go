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

// patternComplianceValidator validates schedule pattern compliance:
// expand assignment dates, map each date to a 1-indexed schedule day, group by
// shift id, then validate each group against organization compliance rules.
func patternComplianceValidator(ctx context.Context, db DBTX, employmentID uuid.UUID, req api.AssignPatternRequest) error {
	if req.PatternID == uuid.Nil {
		return api.ErrPatternValidation
	}
	var patternType string
	var cycleLength *int
	err := db.QueryRow(ctx, `SELECT pattern_type, cycle_length_days FROM working_patterns WHERE id = $1 AND deleted_at IS NULL`, req.PatternID).Scan(&patternType, &cycleLength)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.ErrPatternNotFound
	}
	if err != nil {
		return err
	}
	rows, err := db.Query(ctx, `SELECT schedule_day, shift_id FROM working_pattern_details WHERE pattern_id = $1 ORDER BY schedule_day`, req.PatternID)
	if err != nil {
		return err
	}
	defer rows.Close()
	shiftByDay := make(map[int]uuid.UUID)
	for rows.Next() {
		var day int
		var shiftID *uuid.UUID
		if err := rows.Scan(&day, &shiftID); err != nil {
			return err
		}
		if shiftID != nil {
			shiftByDay[day] = *shiftID
		}
	}
	if err := rows.Err(); err != nil {
		return err
	}
	if len(shiftByDay) == 0 {
		return nil
	}
	rules := compliance.Rules{IsEnabled: true, MinRestPeriodHours: 12, MaxWeeklyHours: 48, MaxMonthlyOTHours: 40, AllowManagerOverride: true}
	var storedRules compliance.Rules
	err = db.QueryRow(ctx, `SELECT is_enabled, min_rest_period_hours, max_weekly_hours, max_monthly_ot_hours, allow_manager_override FROM compliance_rules`).Scan(&storedRules.IsEnabled, &storedRules.MinRestPeriodHours, &storedRules.MaxWeeklyHours, &storedRules.MaxMonthlyOTHours, &storedRules.AllowManagerOverride)
	if err == nil {
		rules = storedRules
	} else if !errors.Is(err, pgx.ErrNoRows) {
		return err
	}
	if !rules.IsEnabled {
		return nil
	}
	cycleDays := 7
	if cycleLength != nil && *cycleLength > 0 {
		cycleDays = *cycleLength
	}
	start, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return api.ErrPatternValidation
	}
	end := start.AddDate(0, 0, 90)
	if req.EndDate != nil && *req.EndDate != "" {
		parsed, parseErr := time.Parse("2006-01-02", *req.EndDate)
		if parseErr != nil {
			return api.ErrPatternValidation
		}
		end = parsed
	}
	datesByShift := make(map[uuid.UUID][]string)
	count := 0
	for current := start; !current.After(end) && count < 366; current = current.AddDate(0, 0, 1) {
		daysSince := int(current.Sub(start).Hours() / 24)
		normalized := ((daysSince % cycleDays) + cycleDays) % cycleDays
		if shiftID, ok := shiftByDay[normalized+1]; ok {
			datesByShift[shiftID] = append(datesByShift[shiftID], current.Format("2006-01-02"))
		}
		count++
	}
	for shiftID, dates := range datesByShift {
		shifts, err := loadPatternComplianceShifts(ctx, db, shiftID)
		if err != nil {
			return err
		}
		rosters, err := loadPatternComplianceRosters(ctx, db, employmentID, dates)
		if err != nil {
			return err
		}
		result, validationErr := compliance.Validate(rules, compliance.Request{EmploymentID: employmentID, ShiftID: shiftID, Dates: dates, AllowWarnings: req.AllowWarnings, OverrideReason: req.OverrideReason}, rosters, shifts)
		if validationErr != nil {
			return &api.ShiftComplianceError{Hard: errors.Is(validationErr, compliance.ErrHardViolation), Violations: result.Violations}
		}
	}
	return nil
}

func loadPatternComplianceShifts(ctx context.Context, db DBTX, shiftID uuid.UUID) (map[uuid.UUID][]compliance.Interval, error) {
	rows, err := db.Query(ctx, `SELECT t.id, i.start_time::text, i.end_time::text, i.break_duration_minutes, i.is_break_auto_deduct FROM shift_templates t JOIN shift_versions v ON v.id = t.current_version_id JOIN shift_default_intervals i ON i.version_id = v.id WHERE t.id = $1 ORDER BY i.sort_order`, shiftID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	intervals := make([]compliance.Interval, 0)
	for rows.Next() {
		var templateID uuid.UUID
		var interval compliance.Interval
		if err := rows.Scan(&templateID, &interval.StartTime, &interval.EndTime, &interval.BreakDurationMinutes, &interval.IsBreakAutoDeduct); err != nil {
			return nil, err
		}
		intervals = append(intervals, interval)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return map[uuid.UUID][]compliance.Interval{shiftID: intervals}, nil
}

func loadPatternComplianceRosters(ctx context.Context, db DBTX, employmentID uuid.UUID, dates []string) ([]compliance.Roster, error) {
	if len(dates) == 0 {
		return nil, nil
	}
	rows, err := db.Query(ctx, `SELECT date::text, template_id FROM shift_rosters WHERE employment_id = $1 AND date = ANY($2::date[])`, employmentID, dates)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	rosters := make([]compliance.Roster, 0)
	for rows.Next() {
		var date string
		var shiftID *uuid.UUID
		if err := rows.Scan(&date, &shiftID); err != nil {
			return nil, err
		}
		rosters = append(rosters, compliance.Roster{EmploymentID: employmentID, ShiftID: shiftID, Date: date})
	}
	return rosters, rows.Err()
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
