package repository

import (
	"context"
	"errors"
	"sort"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

var (
	ErrLifecycleMutationBlocked   = errors.New("schedule lifecycle blocks legacy mutation")
	ErrLifecycleSchemaUnavailable = errors.New("schedule lifecycle schema is unavailable for managed target")
)

type lifecycleMutationBlockedError struct {
	employmentIDs []uuid.UUID
	dates         []string
}

func (e *lifecycleMutationBlockedError) Error() string { return ErrLifecycleMutationBlocked.Error() }
func (e *lifecycleMutationBlockedError) Unwrap() error { return ErrLifecycleMutationBlocked }

func blockedMutation(employmentIDs []uuid.UUID, dates []string) error {
	return &lifecycleMutationBlockedError{employmentIDs: append([]uuid.UUID(nil), employmentIDs...), dates: append([]string(nil), dates...)}
}

func writeBlockedMutationEvidence(ctx context.Context, beginner TxBeginner, blocked error) error {
	var event *lifecycleMutationBlockedError
	if !errors.As(blocked, &event) || beginner == nil {
		return nil
	}
	return service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		return recordLegacyMutationEvidence(ctx, tx, event.employmentIDs, event.dates, "guard_blocked")
	})
}

func finishLifecycleMutation(ctx context.Context, beginner TxBeginner, err error) error {
	if err == nil {
		return nil
	}
	if evidenceErr := writeBlockedMutationEvidence(ctx, beginner, err); evidenceErr != nil {
		return errors.Join(err, evidenceErr)
	}
	return err
}

func finishLifecycleMutationDB(ctx context.Context, db DBTX, err error) error {
	beginner, ok := db.(TxBeginner)
	if !ok {
		return err
	}
	return finishLifecycleMutation(ctx, beginner, err)
}

type lifecycleSchema struct {
	scheduleRevisions bool
	attendancePeriods bool
	payrollPeriods    bool
	legacyEvidence    bool
}

func recordLegacyMutationEvidence(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, dates []string, operation string) error {
	if len(employmentIDs) == 0 || len(dates) == 0 {
		return nil
	}
	_, err := db.Exec(ctx, `INSERT INTO schedule_legacy_mutation_evidence (employment_id, business_date, operation, details) SELECT e.employment_id, d.day::date, $3, jsonb_build_object('employment_ids', $1::text[], 'dates', $2::text[]) FROM unnest($1::uuid[]) AS e(employment_id) CROSS JOIN unnest($2::text[]) AS d(day)`, employmentIDs, dates, operation)
	return err
}

func normalizedMutationDates(dates []string) (string, string) {
	if len(dates) == 0 {
		return "", ""
	}
	ordered := append([]string(nil), dates...)
	sort.Strings(ordered)
	return ordered[0], ordered[len(ordered)-1]
}

func guardLifecycleDateMutation(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, dates []string) error {
	if len(employmentIDs) == 0 || len(dates) == 0 {
		return nil
	}
	schema, err := lifecycleSchemaState(ctx, db)
	if err != nil || !schema.any() {
		return err
	}
	managed, err := hasOfficialSchedule(ctx, db, schema.scheduleRevisions, employmentIDs, dates)
	if err != nil {
		return err
	}
	startDate, endDate := normalizedMutationDates(dates)
	if managed {
		if !schema.complete() || !schema.legacyEvidence {
			return ErrLifecycleSchemaUnavailable
		}
		return blockedMutation(employmentIDs, []string{startDate, endDate})
	}
	periodManaged, err := hasClosedPeriod(ctx, db, schema, employmentIDs, startDate, endDate)
	if err != nil {
		return err
	}
	if periodManaged && !schema.complete() {
		return ErrLifecycleSchemaUnavailable
	}
	if periodManaged {
		return blockedMutation(employmentIDs, []string{startDate, endDate})
	}
	activated, err := hasActivatedLifecycleScope(ctx, db, employmentIDs, startDate, endDate)
	if err != nil {
		return err
	}
	if activated {
		return blockedMutation(employmentIDs, []string{startDate, endDate})
	}
	return nil
}

func guardLifecycleDateRange(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, startDate, endDate string) error {
	if len(employmentIDs) == 0 || startDate == "" || endDate == "" {
		return nil
	}
	schema, err := lifecycleSchemaState(ctx, db)
	if err != nil || !schema.any() {
		return err
	}
	managed, err := hasOfficialScheduleRange(ctx, db, schema.scheduleRevisions, employmentIDs, startDate, endDate)
	if err != nil {
		return err
	}
	if managed {
		if !schema.complete() || !schema.legacyEvidence {
			return ErrLifecycleSchemaUnavailable
		}
		return blockedMutation(employmentIDs, []string{startDate, endDate})
	}
	periodManaged, err := hasClosedPeriod(ctx, db, schema, employmentIDs, startDate, endDate)
	if err != nil {
		return err
	}
	if periodManaged && !schema.complete() {
		return ErrLifecycleSchemaUnavailable
	}
	if periodManaged {
		return blockedMutation(employmentIDs, []string{startDate, endDate})
	}
	activated, err := hasActivatedLifecycleScope(ctx, db, employmentIDs, startDate, endDate)
	if err != nil {
		return err
	}
	if activated {
		return blockedMutation(employmentIDs, []string{startDate, endDate})
	}
	return nil
}

func guardLifecycleGlobalMutation(ctx context.Context, db DBTX) error {
	schema, err := lifecycleSchemaState(ctx, db)
	if err != nil || !schema.any() {
		return err
	}
	managed, err := hasAnyLifecycleData(ctx, db, schema)
	if err != nil {
		return err
	}
	if !managed {
		return nil
	}
	if !schema.complete() {
		return ErrLifecycleSchemaUnavailable
	}
	return ErrLifecycleMutationBlocked
}

func guardLifecyclePatternMutation(ctx context.Context, db DBTX, patternID uuid.UUID) error {
	rows, err := db.Query(ctx, `SELECT employment_id, start_date::text, COALESCE(end_date, start_date)::text FROM employee_working_patterns WHERE pattern_id = $1`, patternID)
	if err != nil {
		return err
	}
	defer rows.Close()
	var employmentIDs []uuid.UUID
	var startDate, endDate string
	for rows.Next() {
		var employmentID uuid.UUID
		var start, end string
		if err := rows.Scan(&employmentID, &start, &end); err != nil {
			return err
		}
		employmentIDs = append(employmentIDs, employmentID)
		if startDate == "" || start < startDate {
			startDate = start
		}
		if end > endDate {
			endDate = end
		}
	}
	if err := rows.Err(); err != nil {
		return err
	}
	if len(employmentIDs) == 0 {
		return guardLifecycleGlobalMutation(ctx, db)
	}
	return guardLifecycleDateRange(ctx, db, employmentIDs, startDate, endDate)
}

func guardLifecycleSegmentMutation(ctx context.Context, db DBTX, segmentID uuid.UUID) error {
	var employmentID uuid.UUID
	var date string
	if err := db.QueryRow(ctx, `SELECT employment_id, date::text FROM work_segments WHERE id = $1`, segmentID).Scan(&employmentID, &date); err != nil {
		return err
	}
	return guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{date})
}

func guardLifecycleShiftMutation(ctx context.Context, db DBTX, templateID uuid.UUID) error {
	schema, err := lifecycleSchemaState(ctx, db)
	if err != nil || !schema.any() {
		return err
	}
	if !schema.complete() {
		var official bool
		if schema.scheduleRevisions {
			err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM work_segments ws JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id WHERE ws.template_id = $1 AND r.state IN ('published', 'explicit_off'))`, templateID).Scan(&official)
			if err != nil {
				return err
			}
		}
		if official {
			return ErrLifecycleSchemaUnavailable
		}
		return nil
	}
	var official bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM work_segments ws JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id WHERE ws.template_id = $1 AND r.state IN ('published', 'explicit_off'))`, templateID).Scan(&official); err != nil {
		return err
	}
	if official {
		return ErrLifecycleMutationBlocked
	}
	return nil
}

func lifecycleSchemaState(ctx context.Context, db DBTX) (lifecycleSchema, error) {
	var schema lifecycleSchema
	err := db.QueryRow(ctx, `SELECT
		EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'schedule_day_revisions'),
		EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'attendance_periods'),
		EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'payroll_periods'),
			EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'schedule_legacy_mutation_evidence')`).Scan(&schema.scheduleRevisions, &schema.attendancePeriods, &schema.payrollPeriods, &schema.legacyEvidence)
	return schema, err
}

func hasActivatedLifecycleScope(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, startDate, endDate string) (bool, error) {
	var activated bool
	err := db.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1
		FROM feature_flags ff
		JOIN employments e ON e.id = ANY($1) AND e.deleted_at IS NULL
		JOIN config_entries c
		  ON c.scope_type = 'employer'
		 AND c.scope_id = e.employer_id
		 AND c.config_key = 'schedule.lifecycle.enabled'
		 AND c.value_bool = true
		 AND daterange(c.effective_from, COALESCE(c.effective_to + 1, 'infinity'::date), '[)')
		     && daterange($2::date, $3::date + 1, '[)')
		WHERE ff.flag_key = 'schedule_lifecycle' AND ff.enabled = true
	)`, employmentIDs, startDate, endDate).Scan(&activated)
	return activated, err
}

func hasOfficialSchedule(ctx context.Context, db DBTX, available bool, employmentIDs []uuid.UUID, dates []string) (bool, error) {
	if !available {
		return false, nil
	}
	var exists bool
	err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date = ANY($2::date[]) AND state IN ('published', 'explicit_off'))`, employmentIDs, dates).Scan(&exists)
	return exists, err
}

func hasOfficialScheduleRange(ctx context.Context, db DBTX, available bool, employmentIDs []uuid.UUID, startDate, endDate string) (bool, error) {
	if !available {
		return false, nil
	}
	var exists bool
	err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM schedule_day_revisions WHERE employment_id = ANY($1) AND business_date BETWEEN $2::date AND $3::date AND state IN ('published', 'explicit_off'))`, employmentIDs, startDate, endDate).Scan(&exists)
	return exists, err
}

func hasClosedPeriod(ctx context.Context, db DBTX, schema lifecycleSchema, employmentIDs []uuid.UUID, startDate, endDate string) (bool, error) {
	if !schema.attendancePeriods && !schema.payrollPeriods {
		return false, nil
	}
	var attendanceClosed, payrollLocked bool
	if schema.attendancePeriods {
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM attendance_periods p JOIN employments e ON e.employer_id = p.employer_id WHERE e.id = ANY($1) AND p.status = 'closed' AND make_date(p.period_year, p.period_month, 1) <= $3::date AND (make_date(p.period_year, p.period_month, 1) + interval '1 month - 1 day') >= $2::date)`, employmentIDs, startDate, endDate).Scan(&attendanceClosed); err != nil {
			return false, err
		}
	}
	if schema.payrollPeriods {
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_periods p JOIN employments e ON e.employer_id = p.employer_id WHERE e.id = ANY($1) AND p.status = 'locked' AND make_date(p.period_year, p.period_month, 1) <= $3::date AND (make_date(p.period_year, p.period_month, 1) + interval '1 month - 1 day') >= $2::date)`, employmentIDs, startDate, endDate).Scan(&payrollLocked); err != nil {
			return false, err
		}
	}
	return attendanceClosed || payrollLocked, nil
}

func hasAnyLifecycleData(ctx context.Context, db DBTX, schema lifecycleSchema) (bool, error) {
	if schema.scheduleRevisions {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM schedule_day_revisions WHERE state IN ('published', 'explicit_off'))`).Scan(&exists); err != nil {
			return false, err
		}
		if exists {
			return true, nil
		}
	}
	if schema.attendancePeriods {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM attendance_periods WHERE status = 'closed')`).Scan(&exists); err != nil {
			return false, err
		}
		if exists {
			return true, nil
		}
	}
	if schema.payrollPeriods {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_periods WHERE status = 'locked')`).Scan(&exists); err != nil {
			return false, err
		}
		return exists, nil
	}
	return false, nil
}

func (s lifecycleSchema) any() bool {
	return s.scheduleRevisions || s.attendancePeriods || s.payrollPeriods || s.legacyEvidence
}

func (s lifecycleSchema) complete() bool {
	return s.scheduleRevisions && s.attendancePeriods && s.payrollPeriods && s.legacyEvidence
}
