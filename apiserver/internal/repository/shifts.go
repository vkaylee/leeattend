package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/compliance"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type ShiftRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r ShiftRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "shift repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("shift repository: database does not support transactions")
	}
	return beginner, nil
}

func (r ShiftRepository) CreateShift(ctx context.Context, actorID uuid.UUID, req api.CreateShiftRequest) (api.ShiftRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.ShiftRecord{}, err
	}
	templateID := uuid.New()
	versionID := uuid.New()
	now := time.Now().UTC()
	var record api.ShiftRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if _, err := db.Exec(ctx, `INSERT INTO shift_templates (id, name, current_version_id, created_at) VALUES ($1, $2, NULL, $3)`, templateID, req.Name, now); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `INSERT INTO shift_versions (id, template_id, version_number, name, report_symbol, location_id, employer_id, effective_from, effective_to, change_type, change_reason, superseded_by, created_by, created_at) VALUES ($1, $2, 1, $3, $4, $5, $6, CURRENT_DATE, NULL, 'create', NULL, NULL, $7, $8)`, versionID, templateID, req.Name, req.ReportSymbol, req.LocationID, req.EmployerID, actorID, now); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE shift_templates SET current_version_id = $1 WHERE id = $2`, versionID, templateID); err != nil {
			return err
		}
		for index, interval := range req.Intervals {
			if _, err := db.Exec(ctx, `INSERT INTO shift_default_intervals (id, version_id, sort_order, interval_name, start_time, end_time, break_duration_minutes, break_start, break_end, break_punch_mode, start_day_offset, end_day_offset, start_utc_offset_minutes, end_utc_offset_minutes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::time, $6::time, $7, $8::time, $9::time, $10, $11, $12, $13, $14, $15, $15)`, uuid.New(), versionID, index, interval.IntervalName, interval.StartTime, interval.EndTime, interval.BreakDurationMinutes, nullableClock(interval.BreakStart), nullableClock(interval.BreakEnd), interval.BreakPunchModeValue(), interval.StartDayOffset, interval.EndDayOffset, interval.StartUTCOffsetMinutes, interval.EndUTCOffsetMinutes, now); err != nil {
				return err
			}
		}
		if _, err := db.Exec(ctx, `INSERT INTO shift_audit_log (id, template_id, version_id, action, change_type, changed_by, change_reason, old_values, new_values, created_at) VALUES ($1, $2, $3, 'version_created', 'create', $4, NULL, NULL, NULL, $5)`, uuid.New(), templateID, versionID, actorID, now); err != nil {
			return err
		}
		if err := EnqueueRosterCacheInvalidation(ctx, db); err != nil {
			return err
		}
		return scanShift(ctx, db, templateID, &record)
	})
	if err != nil {
		return api.ShiftRecord{}, err
	}
	return record, nil
}

func (r ShiftRepository) ListShifts(ctx context.Context, locationID, employerID *uuid.UUID, page, limit int) (api.PaginatedShifts, error) {
	db, err := requireDB(r.DB, "shift repository")
	if err != nil {
		return api.PaginatedShifts{}, err
	}
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM shift_templates t JOIN shift_versions v ON v.id = t.current_version_id WHERE v.effective_to IS NULL AND ($1::uuid IS NULL OR v.location_id IS NULL OR v.location_id = $1) AND ($2::uuid IS NULL OR v.employer_id IS NULL OR v.employer_id = $2)`, locationID, employerID).Scan(&total); err != nil {
		return api.PaginatedShifts{}, err
	}
	rows, err := db.Query(ctx, `SELECT t.id, v.name, v.report_symbol, v.location_id, v.employer_id, t.created_at, t.created_at, v.id FROM shift_templates t JOIN shift_versions v ON v.id = t.current_version_id WHERE v.effective_to IS NULL AND ($1::uuid IS NULL OR v.location_id IS NULL OR v.location_id = $1) AND ($2::uuid IS NULL OR v.employer_id IS NULL OR v.employer_id = $2) ORDER BY v.name ASC LIMIT $3 OFFSET $4`, locationID, employerID, limit, (page-1)*limit)
	if err != nil {
		return api.PaginatedShifts{}, err
	}
	defer rows.Close()
	data := make([]api.ShiftRecord, 0)
	for rows.Next() {
		var record api.ShiftRecord
		var versionID uuid.UUID
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&record.ID, &record.Name, &record.ReportSymbol, &record.LocationID, &record.EmployerID, &createdAt, &updatedAt, &versionID); err != nil {
			return api.PaginatedShifts{}, err
		}
		record.IsActive = true
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		if err := loadIntervals(ctx, db, versionID, &record.Intervals); err != nil {
			return api.PaginatedShifts{}, err
		}
		data = append(data, record)
	}
	return api.PaginatedShifts{Data: data, Total: total, Page: page, Limit: limit}, rows.Err()
}

func scanShift(ctx context.Context, db DBTX, templateID uuid.UUID, record *api.ShiftRecord) error {
	var versionID uuid.UUID
	var createdAt, updatedAt time.Time
	err := db.QueryRow(ctx, `SELECT t.id, v.name, v.report_symbol, v.location_id, v.employer_id, t.created_at, t.created_at, v.id FROM shift_templates t JOIN shift_versions v ON v.id = t.current_version_id WHERE t.id = $1`, templateID).Scan(&record.ID, &record.Name, &record.ReportSymbol, &record.LocationID, &record.EmployerID, &createdAt, &updatedAt, &versionID)
	if err != nil {
		return err
	}
	record.IsActive = true
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return loadIntervals(ctx, db, versionID, &record.Intervals)
}

func loadIntervals(ctx context.Context, db DBTX, versionID uuid.UUID, intervals *[]api.ShiftIntervalRecord) error {
	rows, err := db.Query(ctx, `SELECT id, version_id, sort_order, interval_name, start_time::text, end_time::text, break_duration_minutes, break_start::text, break_end::text, break_punch_mode, start_day_offset, end_day_offset, start_utc_offset_minutes, end_utc_offset_minutes FROM shift_default_intervals WHERE version_id = $1 ORDER BY sort_order ASC`, versionID)
	if err != nil {
		return err
	}
	defer rows.Close()
	*intervals = make([]api.ShiftIntervalRecord, 0)
	for rows.Next() {
		var interval api.ShiftIntervalRecord
		if err := rows.Scan(&interval.ID, &interval.VersionID, &interval.SortOrder, &interval.IntervalName, &interval.StartTime, &interval.EndTime, &interval.BreakDurationMinutes, &interval.BreakStart, &interval.BreakEnd, &interval.BreakPunchMode, &interval.StartDayOffset, &interval.EndDayOffset, &interval.StartUTCOffsetMinutes, &interval.EndUTCOffsetMinutes); err != nil {
			return err
		}
		*intervals = append(*intervals, interval)
	}
	return rows.Err()
}

func nullableClock(value *string) any {
	if value == nil || *value == "" {
		return nil
	}
	return *value
}

func (r ShiftRepository) UpdateShift(ctx context.Context, templateID, actorID uuid.UUID, req api.UpdateShiftRequest) (api.ShiftRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.ShiftRecord{}, err
	}
	now := time.Now().UTC()
	newVersionID := uuid.New()
	var record api.ShiftRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if err := guardLifecycleShiftMutation(ctx, db, templateID); err != nil {
			return err
		}
		var currentVersionID uuid.UUID
		var versionNumber int
		var currentEffectiveFrom time.Time
		if err := db.QueryRow(ctx, `SELECT v.id, v.version_number, v.effective_from FROM shift_templates t JOIN shift_versions v ON v.id = t.current_version_id WHERE t.id = $1 FOR UPDATE`, templateID).Scan(&currentVersionID, &versionNumber, &currentEffectiveFrom); err != nil {
			return errNoRows
		}
		correction := req.IsCorrection || req.EffectiveDate == nil || *req.EffectiveDate == ""
		effectiveDate := currentEffectiveFrom.Format("2006-01-02")
		changeType := "correct"
		if !correction {
			parsed, parseErr := time.Parse("2006-01-02", *req.EffectiveDate)
			if parseErr != nil {
				return api.ErrShiftValidation
			}
			effectiveDate = *req.EffectiveDate
			changeType = "revise"
			closeDate := parsed.AddDate(0, 0, -1).Format("2006-01-02")
			if _, err := db.Exec(ctx, `UPDATE shift_versions SET effective_to = $1::date, superseded_by = $2 WHERE id = $3`, closeDate, newVersionID, currentVersionID); err != nil {
				return err
			}
		} else if _, err := db.Exec(ctx, `UPDATE shift_versions SET superseded_by = $1 WHERE id = $2`, newVersionID, currentVersionID); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `INSERT INTO shift_versions (id, template_id, version_number, name, report_symbol, location_id, employer_id, effective_from, effective_to, change_type, change_reason, superseded_by, created_by, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::date, NULL, $9, $10, NULL, $11, $12)`, newVersionID, templateID, versionNumber+1, req.Name, req.ReportSymbol, req.LocationID, req.EmployerID, effectiveDate, changeType, req.ChangeReason, actorID, now); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE shift_templates SET current_version_id = $1, name = $2 WHERE id = $3`, newVersionID, req.Name, templateID); err != nil {
			return err
		}
		for index, interval := range req.Intervals {
			if _, err := db.Exec(ctx, `INSERT INTO shift_default_intervals (id, version_id, sort_order, interval_name, start_time, end_time, break_duration_minutes, break_start, break_end, break_punch_mode, start_day_offset, end_day_offset, start_utc_offset_minutes, end_utc_offset_minutes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::time, $6::time, $7, $8::time, $9::time, $10, $11, $12, $13, $14, $15, $15)`, uuid.New(), newVersionID, index, interval.IntervalName, interval.StartTime, interval.EndTime, interval.BreakDurationMinutes, nullableClock(interval.BreakStart), nullableClock(interval.BreakEnd), interval.BreakPunchModeValue(), interval.StartDayOffset, interval.EndDayOffset, interval.StartUTCOffsetMinutes, interval.EndUTCOffsetMinutes, now); err != nil {
				return err
			}
		}
		if _, err := db.Exec(ctx, `INSERT INTO shift_audit_log (id, template_id, version_id, action, change_type, changed_by, change_reason, old_values, new_values, created_at) VALUES ($1, $2, $3, 'version_created', $4, $5, $6, NULL, NULL, $7)`, uuid.New(), templateID, newVersionID, changeType, actorID, req.ChangeReason, now); err != nil {
			return err
		}
		if err := EnqueueRosterCacheInvalidation(ctx, db); err != nil {
			return err
		}
		return scanShift(ctx, db, templateID, &record)
	})
	err = finishLifecycleMutation(ctx, tx, err)
	if err != nil {
		return api.ShiftRecord{}, err
	}
	return record, nil
}

func (r ShiftRepository) ListShiftVersions(ctx context.Context, templateID uuid.UUID) ([]api.ShiftVersionRecord, error) {
	db, err := requireDB(r.DB, "shift repository")
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_templates WHERE id = $1)`, templateID).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, errNoRows
	}
	rows, err := db.Query(ctx, `SELECT id, template_id, version_number, name, report_symbol, location_id, employer_id, effective_from, effective_to, change_type, change_reason, superseded_by, created_by, created_at FROM shift_versions WHERE template_id = $1 ORDER BY version_number DESC`, templateID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	versions := make([]api.ShiftVersionRecord, 0)
	versionIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var version api.ShiftVersionRecord
		var effectiveFrom, createdAt time.Time
		var effectiveTo *time.Time
		if err := rows.Scan(&version.ID, &version.TemplateID, &version.VersionNumber, &version.Name, &version.ReportSymbol, &version.LocationID, &version.EmployerID, &effectiveFrom, &effectiveTo, &version.ChangeType, &version.ChangeReason, &version.SupersededBy, &version.CreatedBy, &createdAt); err != nil {
			return nil, err
		}
		version.EffectiveFrom = effectiveFrom.Format("2006-01-02")
		if effectiveTo != nil {
			formatted := effectiveTo.Format("2006-01-02")
			version.EffectiveTo = &formatted
		}
		version.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		version.Intervals = make([]api.ShiftIntervalRecord, 0)
		versions = append(versions, version)
		versionIDs = append(versionIDs, version.ID)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(versionIDs) == 0 {
		return versions, nil
	}
	intervalRows, err := db.Query(ctx, `SELECT id, version_id, sort_order, interval_name, start_time::text, end_time::text, break_duration_minutes, break_start::text, break_end::text, break_punch_mode, start_day_offset, end_day_offset, start_utc_offset_minutes, end_utc_offset_minutes FROM shift_default_intervals WHERE version_id = ANY($1) ORDER BY sort_order ASC`, versionIDs)
	if err != nil {
		return nil, err
	}
	defer intervalRows.Close()
	byVersion := make(map[uuid.UUID][]api.ShiftIntervalRecord, len(versionIDs))
	for intervalRows.Next() {
		var interval api.ShiftIntervalRecord
		if err := intervalRows.Scan(&interval.ID, &interval.VersionID, &interval.SortOrder, &interval.IntervalName, &interval.StartTime, &interval.EndTime, &interval.BreakDurationMinutes, &interval.BreakStart, &interval.BreakEnd, &interval.BreakPunchMode, &interval.StartDayOffset, &interval.EndDayOffset, &interval.StartUTCOffsetMinutes, &interval.EndUTCOffsetMinutes); err != nil {
			return nil, err
		}
		byVersion[interval.VersionID] = append(byVersion[interval.VersionID], interval)
	}
	if err := intervalRows.Err(); err != nil {
		return nil, err
	}
	for index := range versions {
		if intervals, ok := byVersion[versions[index].ID]; ok {
			versions[index].Intervals = intervals
		}
	}
	return versions, nil
}

func (r ShiftRepository) AssignShift(ctx context.Context, actorID uuid.UUID, req api.AssignShiftRequest) error {
	startDate, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return api.ErrShiftValidation
	}
	var endDate *time.Time
	if req.EndDate != nil && *req.EndDate != "" {
		parsed, parseErr := time.Parse("2006-01-02", *req.EndDate)
		if parseErr != nil || parsed.Before(startDate) {
			return api.ErrShiftValidation
		}
		endDate = &parsed
		if calendarDayDifference(parsed, startDate) >= 366 {
			return api.ErrShiftValidation
		}
	}
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	now := time.Now().UTC()
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		bounds, err := loadEmploymentComplianceBounds(ctx, db, req.EmploymentID)
		if err != nil {
			return err
		}
		evalEnd, validBounds := resolveComplianceEvalEnd(startDate, endDate, bounds)
		if !validBounds {
			return api.ErrShiftValidation
		}
		if _, err := db.Exec(ctx, `SELECT id FROM employees WHERE id = $1 FOR UPDATE`, bounds.employeeID); err != nil {
			return err
		}
		if err := guardLifecycleDateRange(ctx, db, []uuid.UUID{req.EmploymentID}, req.StartDate, shiftEndDate(req.StartDate, req.EndDate)); err != nil {
			return err
		}
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_templates WHERE id = $1)`, req.ShiftID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrShiftNotFound
		}
		rules, err := loadComplianceRules(ctx, db)
		if err != nil {
			return err
		}
		if rules.IsEnabled {
			existing, shiftsMeta, err := loadResolvedComplianceSchedule(ctx, db, bounds.employeeID, startDate, evalEnd)
			if err != nil {
				return err
			}
			dates := make([]string, 0, calendarDayDifference(evalEnd, startDate)+1)
			intervalsByDate := make(map[string][]compliance.Interval, calendarDayDifference(evalEnd, startDate)+1)
			for current := startDate; !current.After(evalEnd); current = current.AddDate(0, 0, 1) {
				dateStr := current.Format("2006-01-02")
				dates = append(dates, dateStr)
				intervalsByDate[dateStr] = effectiveTemplateComplianceIntervals(shiftsMeta, req.ShiftID, current)
			}
			result, validationErr := compliance.Validate(rules, compliance.Request{
				EmploymentID:    req.EmploymentID,
				ShiftID:         req.ShiftID,
				Dates:           dates,
				IntervalsByDate: intervalsByDate,
				AllowWarnings:   req.AllowWarnings,
				OverrideReason:  req.OverrideReason,
			}, existing, nil)
			if validationErr != nil {
				return &api.ShiftComplianceError{Hard: errors.Is(validationErr, compliance.ErrHardViolation), Violations: result.Violations}
			}
		}
		_, err = db.Exec(ctx, `INSERT INTO employee_shifts (id, employment_id, template_id, start_date, end_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6)`, uuid.New(), req.EmploymentID, req.ShiftID, startDate, endDate, now)
		if err != nil {
			if pgErr, ok := err.(*pgconn.PgError); ok && pgErr.Code == "23505" {
				return api.ErrShiftAssignmentConflict
			}
			return err
		}
		if err := EnqueueRosterCacheInvalidation(ctx, db); err != nil {
			return err
		}
		_ = actorID
		return nil
	})
	return finishLifecycleMutation(ctx, tx, err)
}

func shiftEndDate(startDate string, endDate *string) string {
	if endDate == nil || *endDate == "" {
		return startDate
	}
	return *endDate
}
