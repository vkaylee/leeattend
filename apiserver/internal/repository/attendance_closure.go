package repository

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

const closureDayPayloadCTE = `
WITH days AS (
	SELECT generate_series($2::date, $3::date, interval '1 day')::date AS business_date
), current_days AS (
	SELECT em.id AS employment_id, d.business_date, r.id AS revision_id,
		r.revision_number, r.state, r.source, r.timezone, r.source_snapshot
	FROM employments em
	CROSS JOIN days d
	JOIN schedule_day_revisions r
	  ON r.employment_id = em.id
	 AND r.business_date = d.business_date
	 AND r.employer_id = $1
	 AND r.state IN ('published', 'explicit_off')
	WHERE em.employer_id = $1
	  AND em.deleted_at IS NULL
	  AND em.start_date <= d.business_date
	  AND (em.end_date IS NULL OR em.end_date >= d.business_date)
), day_payloads AS (
	SELECT cd.*,
		COALESCE((
			SELECT jsonb_agg(to_jsonb(ws) ORDER BY ws.sort_order, ws.id)
			FROM work_segments ws
			WHERE ws.schedule_revision_id = cd.revision_id
		), '[]'::jsonb) AS segments,
		COALESCE((
			SELECT to_jsonb(s)
			FROM attendance_daily_summaries s
			WHERE s.employment_id = cd.employment_id AND s.date = cd.business_date
		LIMIT 1
		), '{}'::jsonb) AS attendance,
		ARRAY(
			SELECT te.id
			FROM time_entries te
			WHERE te.employment_id = cd.employment_id
			  AND te.punch_in_time::date = cd.business_date
			ORDER BY te.id
		) AS time_entry_ids,
		ARRAY(
			SELECT ral.id
			FROM raw_attendance_logs ral
			WHERE ral.employment_id = cd.employment_id
			  AND ral.punch_time::date = cd.business_date
			ORDER BY ral.id
		) AS raw_attendance_log_ids
	FROM current_days cd
), day_documents AS (
	SELECT employment_id, business_date, revision_id, state, source_snapshot,
		jsonb_build_object(
			'revision_id', revision_id,
			'revision_number', revision_number,
			'state', state,
			'source', source,
			'timezone', timezone,
			'source_snapshot', source_snapshot,
			'segments', segments
		) AS schedule_snapshot,
		attendance AS attendance_snapshot,
		jsonb_build_object(
			'actual_work_minutes', attendance->'actual_work_minutes',
			'standard_minutes', attendance->'standard_minutes',
			'system_detected_ot_minutes', attendance->'system_detected_ot_minutes',
			'claimed_ot_minutes', attendance->'claimed_ot_minutes',
			'approved_ot_minutes', attendance->'approved_ot_minutes',
			'early_ot_minutes', attendance->'early_ot_minutes',
			'late_ot_minutes', attendance->'late_ot_minutes',
			'break_ot_minutes', attendance->'break_ot_minutes',
			'ot_status', attendance->'ot_status'
		) AS calculation_snapshot,
		time_entry_ids, raw_attendance_log_ids
	FROM day_payloads
)
`

var (
	errAttendanceClosureInvalidPeriod = errors.New("attendance closure: invalid period")
)

func (r AttendanceRepository) GetAttendancePeriodStatus(ctx context.Context, employerID uuid.UUID, year, month int) (api.AttendancePeriodStatusResponse, error) {
	if employerID == uuid.Nil {
		return api.AttendancePeriodStatusResponse{}, api.ErrAttendanceClosureValidation
	}
	if _, _, err := closurePeriodDates(year, month); err != nil {
		return api.AttendancePeriodStatusResponse{}, api.ErrAttendanceClosureValidation
	}
	db, err := r.db(ctx)
	if err != nil {
		return api.AttendancePeriodStatusResponse{}, err
	}
	var response api.AttendancePeriodStatusResponse
	var periodID, closureID *uuid.UUID
	var payrollFound bool
	if err := db.QueryRow(ctx, `
		SELECT p.id, p.status, p.version, p.current_closure_id,
			EXISTS (SELECT 1 FROM payroll_periods pp WHERE pp.employer_id = p.employer_id AND pp.period_year = p.period_year AND pp.period_month = p.period_month),
			COALESCE((SELECT pp.status FROM payroll_periods pp WHERE pp.employer_id = p.employer_id AND pp.period_year = p.period_year AND pp.period_month = p.period_month LIMIT 1), '')
		FROM attendance_periods p
		WHERE p.employer_id = $1 AND p.period_year = $2 AND p.period_month = $3`, employerID, year, month).Scan(
		&periodID, &response.Status, &response.Version, &closureID, &payrollFound, &response.PayrollStatus,
	); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			response.Status = "open"
			response.Version = 1
		} else {
			return api.AttendancePeriodStatusResponse{}, err
		}
	}
	response.PeriodID = periodID
	response.CurrentClosureID = closureID
	response.PayrollPeriodFound = payrollFound
	if closureID != nil {
		if err := db.QueryRow(ctx, `SELECT closure_number, snapshot_hash FROM attendance_closures WHERE id = $1`, *closureID).Scan(&response.ClosureNumber, &response.SnapshotHash); err != nil {
			return api.AttendancePeriodStatusResponse{}, err
		}
	}
	return response, nil
}

func closurePeriodDates(year, month int) (time.Time, time.Time, error) {
	if year < 2000 || year > 2100 || month < 1 || month > 12 {
		return time.Time{}, time.Time{}, errAttendanceClosureInvalidPeriod
	}
	start := time.Date(year, time.Month(month), 1, 0, 0, 0, 0, time.UTC)
	return start, start.AddDate(0, 1, -1), nil
}

func validateClosureRequest(year, month int, expectedVersion int64, reason string) (time.Time, time.Time, error) {
	start, end, err := closurePeriodDates(year, month)
	if err != nil || expectedVersion < 1 || len(strings.TrimSpace(reason)) == 0 || len(strings.TrimSpace(reason)) > 2000 {
		return time.Time{}, time.Time{}, errAttendanceClosureInvalidPeriod
	}
	return start, end, nil
}

// CloseAttendancePeriod closes or recloses one employer month with immutable evidence.
func (r AttendanceRepository) CloseAttendancePeriod(ctx context.Context, employerID uuid.UUID, year, month int, expectedVersion int64, reason string, actorID uuid.UUID) (api.AttendanceClosureResponse, error) {
	start, end, err := validateClosureRequest(year, month, expectedVersion, reason)
	if err != nil || employerID == uuid.Nil || actorID == uuid.Nil {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceClosureValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	var response api.AttendanceClosureResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		result, txErr := r.closeAttendancePeriodTx(ctx, tx, employerID, year, month, start, end, expectedVersion, strings.TrimSpace(reason), actorID)
		if txErr == nil {
			response = result
		}
		return txErr
	})
	return response, err
}

func (r AttendanceRepository) closeAttendancePeriodTx(ctx context.Context, tx pgx.Tx, employerID uuid.UUID, year, month int, start, end time.Time, expectedVersion int64, reason string, actorID uuid.UUID) (api.AttendanceClosureResponse, error) {
	var lockedEmployerID uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT id FROM employers WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, employerID).Scan(&lockedEmployerID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.AttendanceClosureResponse{}, api.ErrEmployerNotFound
		}
		return api.AttendanceClosureResponse{}, err
	}

	var payrollPeriodID uuid.UUID
	var payrollStatus string
	if err := tx.QueryRow(ctx, `SELECT id, status FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, employerID, year, month).Scan(&payrollPeriodID, &payrollStatus); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.AttendanceClosureResponse{}, api.ErrAttendancePayrollPeriodMissing
		}
		return api.AttendanceClosureResponse{}, err
	}
	if payrollStatus == "locked" {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceLockedPeriod
	}

	if _, err := tx.Exec(ctx, `INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, $3, $4, 'open', 1) ON CONFLICT (employer_id, period_year, period_month) DO NOTHING`, uuid.New(), employerID, year, month); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	var periodID, currentClosureID *uuid.UUID
	var periodStatus string
	var periodVersion int64
	if err := tx.QueryRow(ctx, `SELECT id, status, version, current_closure_id FROM attendance_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, employerID, year, month).Scan(&periodID, &periodStatus, &periodVersion, &currentClosureID); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if periodVersion != expectedVersion || periodStatus == "closed" {
		return api.AttendanceClosureResponse{}, api.ErrAttendancePeriodConflict
	}
	if err := lockClosureRows(ctx, tx, `
		SELECT em.id
		FROM employments em
		WHERE em.employer_id = $1 AND em.deleted_at IS NULL
		ORDER BY em.id
		FOR UPDATE`, employerID); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if err := lockClosureRows(ctx, tx, `
		SELECT r.id
		FROM schedule_day_revisions r
		JOIN employments em ON em.id = r.employment_id
		WHERE em.employer_id = $1
		  AND r.business_date BETWEEN $2::date AND $3::date
		  AND r.state IN ('published', 'explicit_off')
		ORDER BY r.employment_id, r.business_date, r.revision_number
		FOR UPDATE`, employerID, start, end); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if err := lockClosureRows(ctx, tx, `
		SELECT te.id
		FROM time_entries te
		JOIN employments em ON em.id = te.employment_id
		WHERE em.employer_id = $1
		  AND te.punch_in_time::date BETWEEN $2::date AND $3::date
		ORDER BY te.employment_id, te.punch_in_time, te.id
		FOR UPDATE`, employerID, start, end); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if err := lockClosureRows(ctx, tx, `
		SELECT s.id
		FROM attendance_daily_summaries s
		JOIN employments em ON em.id = s.employment_id
		WHERE em.employer_id = $1 AND s.date BETWEEN $2::date AND $3::date
		ORDER BY s.employment_id, s.date, s.id
		FOR UPDATE`, employerID, start, end); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if err := lockClosureRows(ctx, tx, `
		SELECT ral.id
		FROM raw_attendance_logs ral
		JOIN employments em ON em.id = ral.employment_id
		WHERE em.employer_id = $1
		  AND ral.punch_time::date BETWEEN $2::date AND $3::date
		ORDER BY ral.employment_id, ral.punch_time, ral.id
		FOR UPDATE`, employerID, start, end); err != nil {
		return api.AttendanceClosureResponse{}, err
	}

	var missingDays, totalDays int
	if err := tx.QueryRow(ctx, `
		WITH days AS (SELECT generate_series($2::date, $3::date, interval '1 day')::date AS business_date),
		employment_days AS (
			SELECT em.id AS employment_id, d.business_date
			FROM employments em CROSS JOIN days d
			WHERE em.employer_id = $1 AND em.deleted_at IS NULL
			  AND em.start_date <= d.business_date
			  AND (em.end_date IS NULL OR em.end_date >= d.business_date)
		)
		SELECT COUNT(*) FILTER (WHERE r.id IS NULL), COUNT(*)
		FROM employment_days ed
		LEFT JOIN schedule_day_revisions r
		  ON r.employment_id = ed.employment_id AND r.business_date = ed.business_date
		 AND r.employer_id = $1 AND r.state IN ('published', 'explicit_off')`, employerID, start, end).Scan(&missingDays, &totalDays); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if missingDays > 0 {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceMissingCoverage
	}

	var openPunch bool
	if err := tx.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM time_entries te JOIN employments em ON em.id = te.employment_id
		WHERE em.employer_id = $1 AND em.deleted_at IS NULL AND te.punch_out_time IS NULL
		  AND te.punch_in_time::date BETWEEN $2::date AND $3::date
	)`, employerID, start, end).Scan(&openPunch); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if openPunch {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceOpenPunch
	}

	var unresolvedApproval bool
	if err := tx.QueryRow(ctx, `SELECT EXISTS (
		SELECT 1 FROM attendance_adjustment_requests ar JOIN employments em ON em.id = ar.employment_id
		WHERE em.employer_id = $1 AND ar.status = 'pending'
		  AND ar.requested_punch_time::date BETWEEN $2::date AND $3::date
		UNION ALL
		SELECT 1 FROM attendance_daily_summaries ads JOIN employments em ON em.id = ads.employment_id
		WHERE em.employer_id = $1 AND ads.ot_status IN ('pending_employee', 'pending_manager')
		  AND ads.date BETWEEN $2::date AND $3::date
	)`, employerID, start, end).Scan(&unresolvedApproval); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if unresolvedApproval {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceClosureConflict
	}

	var snapshot []byte
	if err := tx.QueryRow(ctx, closureDayPayloadCTE+`SELECT COUNT(*), COALESCE(jsonb_agg(to_jsonb(day_documents) ORDER BY employment_id, business_date), '[]'::jsonb) FROM day_documents`, employerID, start, end).Scan(&totalDays, &snapshot); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	closureNumber := 1
	if err := tx.QueryRow(ctx, `SELECT COALESCE(MAX(closure_number), 0) + 1 FROM attendance_closures WHERE period_id = $1`, *periodID).Scan(&closureNumber); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	closureID := uuid.New()
	hash := sha256.Sum256(snapshot)
	snapshotHash := hex.EncodeToString(hash[:])
	if _, err := tx.Exec(ctx, `INSERT INTO attendance_closures (id, period_id, employer_id, closure_number, status, closed_by, closed_actor_type, snapshot_hash) VALUES ($1, $2, $3, $4, 'closed', $5, 'human', $6)`, closureID, *periodID, employerID, closureNumber, actorID, snapshotHash); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if _, err := tx.Exec(ctx, closureDayPayloadCTE+`INSERT INTO attendance_closure_days (
		id, closure_id, period_id, employer_id, employment_id, business_date, schedule_revision_id,
		schedule_state, schedule_snapshot, attendance_snapshot, calculation_snapshot, time_entry_ids, raw_attendance_log_ids
	) SELECT gen_random_uuid(), $4, $5, $1, employment_id, business_date, revision_id, state,
		schedule_snapshot, attendance_snapshot, calculation_snapshot, time_entry_ids, raw_attendance_log_ids
	FROM day_documents`, employerID, start, end, closureID, *periodID); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	var newVersion int64
	if err := tx.QueryRow(ctx, `UPDATE attendance_periods SET status = 'closed', version = version + 1, current_closure_id = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING version`, closureID, *periodID).Scan(&newVersion); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	details, err := json.Marshal(map[string]any{"year": year, "month": month, "version": newVersion, "closure_number": closureNumber, "day_count": totalDays, "reason": reason})
	if err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details) VALUES ($1, $2, 'attendance.closed', $3)`, actorID, *periodID, details); err != nil {
		return api.AttendanceClosureResponse{}, fmt.Errorf("%w: %v", api.ErrAttendanceAuditFailure, err)
	}
	return api.AttendanceClosureResponse{PeriodID: *periodID, ClosureID: &closureID, ClosureNumber: closureNumber, Status: "closed", Version: newVersion, DayCount: totalDays, SnapshotHash: snapshotHash}, nil
}

// ReopenAttendancePeriod reopens attendance without changing immutable closure evidence.
func (r AttendanceRepository) ReopenAttendancePeriod(ctx context.Context, employerID uuid.UUID, year, month int, expectedVersion int64, reason string, actorID uuid.UUID) (api.AttendanceClosureResponse, error) {
	_, _, err := validateClosureRequest(year, month, expectedVersion, reason)
	if err != nil || employerID == uuid.Nil || actorID == uuid.Nil {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceClosureValidation
	}
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	var response api.AttendanceClosureResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		result, txErr := r.reopenAttendancePeriodTx(ctx, tx, employerID, year, month, expectedVersion, strings.TrimSpace(reason), actorID)
		if txErr == nil {
			response = result
		}
		return txErr
	})
	return response, err
}

func lockClosureRows(ctx context.Context, tx pgx.Tx, query string, args ...any) error {
	rows, err := tx.Query(ctx, query, args...)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
	}
	return rows.Err()
}

func (r AttendanceRepository) reopenAttendancePeriodTx(ctx context.Context, tx pgx.Tx, employerID uuid.UUID, year, month int, expectedVersion int64, reason string, actorID uuid.UUID) (api.AttendanceClosureResponse, error) {
	var lockedEmployerID uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT id FROM employers WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, employerID).Scan(&lockedEmployerID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.AttendanceClosureResponse{}, api.ErrEmployerNotFound
		}
		return api.AttendanceClosureResponse{}, err
	}
	var payrollStatus string
	if err := tx.QueryRow(ctx, `SELECT status FROM payroll_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, employerID, year, month).Scan(&payrollStatus); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.AttendanceClosureResponse{}, api.ErrAttendancePayrollPeriodMissing
		}
		return api.AttendanceClosureResponse{}, err
	}
	if payrollStatus == "locked" {
		return api.AttendanceClosureResponse{}, api.ErrAttendanceLockedPeriod
	}
	var periodID, priorClosureID *uuid.UUID
	var status string
	var version int64
	if err := tx.QueryRow(ctx, `SELECT id, status, version, current_closure_id FROM attendance_periods WHERE employer_id = $1 AND period_year = $2 AND period_month = $3 FOR UPDATE`, employerID, year, month).Scan(&periodID, &status, &version, &priorClosureID); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.AttendanceClosureResponse{}, api.ErrAttendancePeriodNotFound
		}
		return api.AttendanceClosureResponse{}, err
	}
	if version != expectedVersion || status != "closed" {
		return api.AttendanceClosureResponse{}, api.ErrAttendancePeriodConflict
	}
	var newVersion int64
	if err := tx.QueryRow(ctx, `UPDATE attendance_periods SET status = 'reopened', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = $1 RETURNING version`, *periodID).Scan(&newVersion); err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	details, err := json.Marshal(map[string]any{"year": year, "month": month, "old_version": version, "new_version": newVersion, "closure_id": priorClosureID, "reason": reason})
	if err != nil {
		return api.AttendanceClosureResponse{}, err
	}
	if _, err := tx.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details) VALUES ($1, $2, 'attendance.reopened', $3)`, actorID, *periodID, details); err != nil {
		return api.AttendanceClosureResponse{}, fmt.Errorf("%w: %v", api.ErrAttendanceAuditFailure, err)
	}
	return api.AttendanceClosureResponse{PeriodID: *periodID, ClosureID: priorClosureID, Status: "reopened", Version: newVersion, PriorClosureID: priorClosureID}, nil
}
