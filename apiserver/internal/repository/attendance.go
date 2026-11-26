package repository

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/matcher"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/buckets"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// AttendanceRepository persists punch events and paired time entries.
type AttendanceRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

// PunchIn writes the raw attendance log and opens a time entry.
func (r AttendanceRepository) PunchIn(ctx context.Context, employmentID uuid.UUID, segmentID *uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (api.TimeEntryRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, tx pgx.Tx) error {
		if _, err := tx.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))`, employmentID.String()); err != nil {
			return err
		}
		var active bool
		if err := tx.QueryRow(ctx, `SELECT status = 'active' FROM employment_segments WHERE id = $1 AND deleted_at IS NULL`, employmentID).Scan(&active); err != nil {
			return err
		}
		if !active {
			return api.ErrEmploymentInactive
		}
		resolvedSegmentID, err := resolvePunchSegment(ctx, tx, employmentID, segmentID, punchTime)
		if err != nil {
			return err
		}
		rawData, err := json.Marshal(struct {
			Punch  int     `json:"punch"`
			Status int     `json:"status"`
			Note   *string `json:"note"`
			Source string  `json:"source"`
		}{Punch: 0, Status: 1, Note: note, Source: "direct_punch"})
		if err != nil {
			return err
		}
		rawLogID := uuid.New()
		cmd, err := tx.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', $4, 0, 1, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, rawLogID, employmentID, punchTime, rawData)
		if err != nil {
			return err
		}
		if cmd.RowsAffected() == 0 {
			return scanLatestTimeEntry(ctx, tx, employmentID, &record)
		}
		row := tx.QueryRow(ctx, `INSERT INTO time_entries (id, employment_id, segment_id, template_id, shift_version_id, breaks_snapshot, punch_in_time, entry_type, note, is_break_overtime, created_at, updated_at) VALUES ($1, $2, $7, (SELECT template_id FROM work_segments WHERE id = $7), (SELECT selected_shift_version_id FROM work_segments WHERE id = $7), (SELECT breaks_snapshot FROM work_segments WHERE id = $7), $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, uuid.New(), employmentID, punchTime, entryType, note, isBreakOvertime, resolvedSegmentID)
		if err := scanTimeEntry(row, &record); err != nil {
			return err
		}
		entryID := record.ID
		return enqueueClosedPeriodLatePunchReconciliation(ctx, tx, employmentID, rawLogID, &entryID, punchTime)
	})
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	return record, nil
}

// PunchOut records a raw checkout event and returns the latest paired entry.
func (r AttendanceRepository) PunchOut(ctx context.Context, employeeID uuid.UUID, note *string, punchTime time.Time) (api.TimeEntryRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		employmentID, err := resolveActiveEmployment(ctx, db, employeeID)
		if err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))`, employmentID.String()); err != nil {
			return err
		}
		rawData, err := json.Marshal(struct {
			Punch  int     `json:"punch"`
			Status int     `json:"status"`
			Note   *string `json:"note"`
			Source string  `json:"source"`
		}{Punch: 1, Status: 1, Note: note, Source: "direct_punch"})
		if err != nil {
			return err
		}
		rawLogID := uuid.New()
		cmd, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', $4, 1, 1, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, rawLogID, employmentID, punchTime, rawData)
		if err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE time_entries SET punch_out_time = $1, duration_minutes = GREATEST(0, FLOOR(EXTRACT(EPOCH FROM ($1 - punch_in_time)) / 60)::integer), note = COALESCE($2, note), updated_at = CURRENT_TIMESTAMP WHERE id = (SELECT id FROM time_entries WHERE employment_id = $3 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1)`, punchTime, note, employmentID); err != nil {
			return err
		}
		if err := scanLatestTimeEntry(ctx, db, employmentID, &record); err != nil {
			return err
		}
		if cmd.RowsAffected() > 0 {
			entryID := record.ID
			return enqueueClosedPeriodLatePunchReconciliation(ctx, db, employmentID, rawLogID, &entryID, punchTime)
		}
		return nil
	})
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	return record, nil
}

// ESSAttendanceAction applies one explicit self-service attendance action.
func (r AttendanceRepository) ESSAttendanceAction(ctx context.Context, employeeID uuid.UUID, action string, note *string, idempotencyKey string, actionTime time.Time) (api.AttendanceActionResponse, error) {
	if action != api.AttendanceActionWorkIn && action != api.AttendanceActionWorkOut && action != api.AttendanceActionBreakStart && action != api.AttendanceActionBreakEnd {
		return api.AttendanceActionResponse{}, api.ErrAttendanceInvalidTransition
	}
	if strings.TrimSpace(idempotencyKey) == "" {
		return api.AttendanceActionResponse{}, api.ErrAttendanceIdempotencyConflict
	}
	tx, err := r.begin(ctx)
	if err != nil {
		return api.AttendanceActionResponse{}, err
	}
	var response api.AttendanceActionResponse
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		employmentID, err := resolveActiveEmployment(ctx, db, employeeID)
		if err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))`, employmentID.String()); err != nil {
			return err
		}
		requestHash, err := hashAttendanceAction(action, note)
		if err != nil {
			return err
		}
		var storedHash string
		var storedResponse []byte
		idempotencyErr := db.QueryRow(ctx, `SELECT request_hash, response_snapshot FROM attendance_action_idempotency WHERE employment_id = $1 AND idempotency_key = $2 FOR UPDATE`, employmentID, idempotencyKey).Scan(&storedHash, &storedResponse)
		if idempotencyErr == nil {
			if storedHash != requestHash {
				return api.ErrAttendanceIdempotencyConflict
			}
			if err := json.Unmarshal(storedResponse, &response); err != nil {
				return err
			}
			response.Deduplicated = true
			return nil
		}
		if !errors.Is(idempotencyErr, pgx.ErrNoRows) {
			return idempotencyErr
		}

		var entry api.TimeEntryRecord
		var entryID uuid.UUID
		openErr := db.QueryRow(ctx, `SELECT id FROM time_entries WHERE employment_id = $1 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1 FOR UPDATE`, employmentID).Scan(&entryID)
		hasOpenEntry := openErr == nil
		if openErr != nil && !errors.Is(openErr, pgx.ErrNoRows) {
			return openErr
		}
		if hasOpenEntry {
			var lastEventAt time.Time
			if err := db.QueryRow(ctx, `SELECT GREATEST(punch_in_time, COALESCE((SELECT max(occurred_at) FROM attendance_punch_events WHERE time_entry_id = $1), punch_in_time)) FROM time_entries WHERE id = $1`, entryID).Scan(&lastEventAt); err != nil {
				return err
			}
			if actionTime.Before(lastEventAt) {
				return api.ErrAttendanceInvalidTransition
			}
		}
		switch action {
		case api.AttendanceActionWorkIn:
			if hasOpenEntry {
				return api.ErrAttendanceAlreadyWorking
			}
			if err := insertAttendanceRawLog(ctx, db, employmentID, actionTime, 0, action, note); err != nil {
				return err
			}
			segmentID, err := resolvePunchSegment(ctx, db, employmentID, nil, actionTime)
			if err != nil {
				return err
			}
			if err := scanTimeEntry(db.QueryRow(ctx, `INSERT INTO time_entries (id, employment_id, segment_id, template_id, shift_version_id, breaks_snapshot, punch_in_time, entry_type, note, is_break_overtime, created_at, updated_at) VALUES ($1, $2, $3, (SELECT template_id FROM work_segments WHERE id = $3), (SELECT selected_shift_version_id FROM work_segments WHERE id = $3), (SELECT breaks_snapshot FROM work_segments WHERE id = $3), $4, 'work', $5, false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, uuid.New(), employmentID, segmentID, actionTime, note), &entry); err != nil {
				return err
			}
			response = api.AttendanceActionResponse{State: "working", NextAllowedActions: []string{api.AttendanceActionBreakStart, api.AttendanceActionWorkOut}, Entry: &entry}
		case api.AttendanceActionWorkOut:
			if !hasOpenEntry {
				return api.ErrAttendanceNotWorking
			}
			var requiredBreak, completedBreak bool
			if err := db.QueryRow(ctx, `SELECT COALESCE(jsonb_path_exists(breaks_snapshot, '$[*] ? (@.break_punch_mode == "required")'), false), EXISTS (SELECT 1 FROM time_entry_breaks WHERE time_entry_id = $1 AND break_end_at IS NOT NULL) FROM time_entries WHERE id = $1`, entryID).Scan(&requiredBreak, &completedBreak); err != nil {
				return err
			}
			if requiredBreak && !completedBreak {
				return api.ErrAttendanceInvalidTransition
			}
			var impliedBreak *api.AttendanceBreakRecord
			var missingBreakEnd string
			var openBreakID uuid.UUID
			breakErr := db.QueryRow(ctx, `SELECT id FROM time_entry_breaks WHERE time_entry_id = $1 AND break_end_at IS NULL FOR UPDATE`, entryID).Scan(&openBreakID)
			if breakErr == nil {
				if requiredBreak {
					return api.ErrAttendanceInvalidTransition
				}
				if _, err := db.Exec(ctx, `UPDATE time_entry_breaks SET break_end_at = $1, source = 'system_implied', close_reason = 'work_out', updated_at = CURRENT_TIMESTAMP WHERE id = $2`, actionTime, openBreakID); err != nil {
					return err
				}
				var breakRecord api.AttendanceBreakRecord
				if err := db.QueryRow(ctx, `SELECT id, time_entry_id, break_start_at, break_end_at, source, close_reason FROM time_entry_breaks WHERE id = $1`, openBreakID).Scan(&breakRecord.ID, &breakRecord.TimeEntryID, &breakRecord.StartAt, &breakRecord.EndAt, &breakRecord.Source, &breakRecord.CloseReason); err != nil {
					return err
				}
				impliedBreak = &breakRecord
				missingBreakEnd = "missing_break_end"
			} else if !errors.Is(breakErr, pgx.ErrNoRows) {
				return breakErr
			}
			if err := insertAttendanceRawLog(ctx, db, employmentID, actionTime, 1, action, note); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `UPDATE time_entries SET punch_out_time = $1, duration_minutes = GREATEST(0, FLOOR(EXTRACT(EPOCH FROM ($1 - punch_in_time)) / 60)::integer), note = COALESCE($2, note), updated_at = CURRENT_TIMESTAMP WHERE id = $3`, actionTime, note, entryID); err != nil {
				return err
			}
			if err := scanTimeEntry(db.QueryRow(ctx, `SELECT id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE id = $1`, entryID), &entry); err != nil {
				return err
			}
			response = api.AttendanceActionResponse{State: "completed", NextAllowedActions: []string{api.AttendanceActionWorkIn}, Entry: &entry, Break: impliedBreak}
			if missingBreakEnd != "" {
				response.State = "exception"
				response.ExceptionCode = &missingBreakEnd
			}
		case api.AttendanceActionBreakStart, api.AttendanceActionBreakEnd:
			if !hasOpenEntry {
				return api.ErrAttendanceNotWorking
			}
			if action == api.AttendanceActionBreakStart {
				var required bool
				if err := db.QueryRow(ctx, `SELECT COALESCE(jsonb_path_exists(ws.breaks_snapshot, '$[*] ? (@.break_punch_mode == "required")'), false) FROM time_entries te LEFT JOIN work_segments ws ON ws.id = te.segment_id WHERE te.id = $1`, entryID).Scan(&required); err != nil {
					return err
				}
				if !required {
					return api.ErrAttendanceBreakNotRequired
				}
				var breakID uuid.UUID
				if err := db.QueryRow(ctx, `SELECT id FROM time_entry_breaks WHERE time_entry_id = $1 AND break_end_at IS NULL FOR UPDATE`, entryID).Scan(&breakID); err == nil {
					return api.ErrAttendanceInvalidTransition
				} else if !errors.Is(err, pgx.ErrNoRows) {
					return err
				}
				if err := insertAttendanceRawLog(ctx, db, employmentID, actionTime, 0, action, note); err != nil {
					return err
				}
				var breakRecord api.AttendanceBreakRecord
				if err := db.QueryRow(ctx, `INSERT INTO time_entry_breaks (id, time_entry_id, shift_interval_id, break_start_at, source) SELECT $1, $2, ws.interval_id, $3, 'punch' FROM time_entries te LEFT JOIN work_segments ws ON ws.id = te.segment_id WHERE te.id = $2 RETURNING id, time_entry_id, break_start_at, break_end_at, source, close_reason`, uuid.New(), entryID, actionTime).Scan(&breakRecord.ID, &breakRecord.TimeEntryID, &breakRecord.StartAt, &breakRecord.EndAt, &breakRecord.Source, &breakRecord.CloseReason); err != nil {
					return err
				}
				response = api.AttendanceActionResponse{State: "on_break", NextAllowedActions: []string{api.AttendanceActionBreakEnd, api.AttendanceActionWorkOut}, Break: &breakRecord}
			} else {
				if err := insertAttendanceRawLog(ctx, db, employmentID, actionTime, 1, action, note); err != nil {
					return err
				}
				var breakRecord api.AttendanceBreakRecord
				if err := db.QueryRow(ctx, `UPDATE time_entry_breaks SET break_end_at = $1, close_reason = 'punch', updated_at = CURRENT_TIMESTAMP WHERE time_entry_id = $2 AND break_end_at IS NULL RETURNING id, time_entry_id, break_start_at, break_end_at, source, close_reason`, actionTime, entryID).Scan(&breakRecord.ID, &breakRecord.TimeEntryID, &breakRecord.StartAt, &breakRecord.EndAt, &breakRecord.Source, &breakRecord.CloseReason); err != nil {
					if errors.Is(err, pgx.ErrNoRows) {
						return api.ErrAttendanceBreakNotOpen
					}
					return err
				}
				response = api.AttendanceActionResponse{State: "working", NextAllowedActions: []string{api.AttendanceActionBreakStart, api.AttendanceActionWorkOut}, Break: &breakRecord}
			}
		}
		response.Deduplicated = false
		eventEntryID := entryID
		if response.Entry != nil {
			eventEntryID = response.Entry.ID
		}
		if eventEntryID == uuid.Nil {
			return errors.New("attendance: accepted action has no time entry")
		}
		if _, err := db.Exec(ctx, `INSERT INTO attendance_punch_events (id, employment_id, time_entry_id, event_type, occurred_at, idempotency_key, source) VALUES ($1, $2, $3, $4, $5, $6, 'ess')`, uuid.New(), employmentID, eventEntryID, action, actionTime, idempotencyKey); err != nil {
			return err
		}
		encoded, err := json.Marshal(response)
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `INSERT INTO attendance_action_idempotency (employment_id, idempotency_key, action, request_hash, response_snapshot) VALUES ($1, $2, $3, $4, $5)`, employmentID, idempotencyKey, action, requestHash, encoded)
		return err
	})
	if err != nil {
		return api.AttendanceActionResponse{}, err
	}
	return response, nil
}

func hashAttendanceAction(action string, note *string) (string, error) {
	payload, err := json.Marshal(struct {
		Action string  `json:"action"`
		Note   *string `json:"note,omitempty"`
	}{action, note})
	if err != nil {
		return "", err
	}
	return fmt.Sprintf("%x", sha256.Sum256(payload)), nil
}

func insertAttendanceRawLog(ctx context.Context, db DBTX, employmentID uuid.UUID, punchTime time.Time, punch int, action string, note *string) error {
	data, err := json.Marshal(map[string]any{"action": action, "note": note, "source": "ess"})
	if err != nil {
		return err
	}
	rawLogID := uuid.New()
	cmd, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', $4, $5, 1, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, rawLogID, employmentID, punchTime, data, punch)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() > 0 {
		return enqueueClosedPeriodLatePunchReconciliation(ctx, db, employmentID, rawLogID, nil, punchTime)
	}
	return nil
}

// resolvePunchSegment accepts an explicit segment only when it owns the employment,
// belongs to a current published revision (or legacy day with no official revision),
// and covers the punch timestamp (with up to 2h pre-shift check-in window).
// Without an explicit segment, it prefers a unique in-shift match before falling back
// to a unique upcoming segment within the 2h early check-in window.
func resolvePunchSegment(ctx context.Context, db DBTX, employmentID uuid.UUID, requestedID *uuid.UUID, punchTime time.Time) (*uuid.UUID, error) {
	const eligible = `ws.segment_status <> 'cancelled'
		AND ws.scheduled_start_at IS NOT NULL
		AND ws.scheduled_end_at IS NOT NULL
		AND (
			(ws.schedule_revision_id IS NOT NULL AND sdr.state = 'published')
			OR (
				ws.schedule_revision_id IS NULL
				AND NOT EXISTS (
					SELECT 1 FROM schedule_day_revisions cur
					WHERE cur.employment_id = ws.employment_id
					  AND cur.business_date = ws.date
					  AND cur.state IN ('published', 'explicit_off')
				)
			)
		)`
	if requestedID != nil {
		var segmentID uuid.UUID
		err := db.QueryRow(ctx, `SELECT ws.id
			FROM work_segments ws
			LEFT JOIN schedule_day_revisions sdr ON sdr.id = ws.schedule_revision_id
			WHERE ws.id = $1 AND ws.employment_id = $3 AND `+eligible+`
			  AND ws.scheduled_start_at - INTERVAL '2 hours' <= $2
			  AND ws.scheduled_end_at >= $2`, *requestedID, punchTime, employmentID).Scan(&segmentID)
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, api.ErrWorkSegmentNotFound
		}
		if err != nil {
			return nil, err
		}
		return &segmentID, nil
	}

	queryUnique := func(windowClause string) (*uuid.UUID, int, error) {
		rows, err := db.Query(ctx, `SELECT ws.id
			FROM work_segments ws
			LEFT JOIN schedule_day_revisions sdr ON sdr.id = ws.schedule_revision_id
			WHERE ws.employment_id = $1 AND `+eligible+` AND `+windowClause+`
			ORDER BY ws.scheduled_start_at, ws.id LIMIT 2`, employmentID, punchTime)
		if err != nil {
			return nil, 0, err
		}
		defer rows.Close()
		var matches []uuid.UUID
		for rows.Next() {
			var segmentID uuid.UUID
			if err := rows.Scan(&segmentID); err != nil {
				return nil, 0, err
			}
			matches = append(matches, segmentID)
		}
		if err := rows.Err(); err != nil {
			return nil, 0, err
		}
		if len(matches) == 1 {
			return &matches[0], 1, nil
		}
		return nil, len(matches), nil
	}

	// Prefer exact in-shift window so an earlier split segment isn't shadowed by the next segment's early buffer.
	segID, count, err := queryUnique(`ws.scheduled_start_at <= $2 AND ws.scheduled_end_at >= $2`)
	if err != nil || count > 0 {
		return segID, err
	}
	segID, _, err = queryUnique(`ws.scheduled_start_at - INTERVAL '2 hours' <= $2 AND ws.scheduled_start_at > $2`)
	return segID, err
}

func (r AttendanceRepository) GetESSAttendanceState(ctx context.Context, employeeID uuid.UUID) (api.AttendanceActionResponse, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return api.AttendanceActionResponse{}, err
	}
	employmentID, err := resolveActiveEmployment(ctx, db, employeeID)
	if err != nil {
		return api.AttendanceActionResponse{}, err
	}
	var entryID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM time_entries WHERE employment_id = $1 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1`, employmentID).Scan(&entryID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.AttendanceActionResponse{State: "off", NextAllowedActions: []string{api.AttendanceActionWorkIn}}, nil
	}
	if err != nil {
		return api.AttendanceActionResponse{}, err
	}
	var entry api.TimeEntryRecord
	if err := scanTimeEntry(db.QueryRow(ctx, `SELECT id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE id = $1`, entryID), &entry); err != nil {
		return api.AttendanceActionResponse{}, err
	}
	var breakRecord api.AttendanceBreakRecord
	breakErr := db.QueryRow(ctx, `SELECT id, time_entry_id, break_start_at, break_end_at, source, close_reason FROM time_entry_breaks WHERE time_entry_id = $1 ORDER BY break_start_at DESC LIMIT 1`, entryID).Scan(&breakRecord.ID, &breakRecord.TimeEntryID, &breakRecord.StartAt, &breakRecord.EndAt, &breakRecord.Source, &breakRecord.CloseReason)
	if breakErr == nil && breakRecord.EndAt == nil {
		return api.AttendanceActionResponse{State: "on_break", NextAllowedActions: []string{api.AttendanceActionBreakEnd, api.AttendanceActionWorkOut}, Entry: &entry, Break: &breakRecord}, nil
	}
	return api.AttendanceActionResponse{State: "working", NextAllowedActions: []string{api.AttendanceActionBreakStart, api.AttendanceActionWorkOut}, Entry: &entry}, nil
}

type reprocessDailyGroup struct {
	employmentID      uuid.UUID
	date              string
	actual            int
	lateMinutes       int
	scheduledBreak    int
	policyID, shiftID *uuid.UUID
	otAggMode         buckets.OtAggregationMode
	inputs            []buckets.DailyCalcInput
}

type rawReprocessRow struct {
	empID                  uuid.UUID
	dateStr                string
	punchIn, punchOut      time.Time
	duration               *int
	segmentID, shiftID     *uuid.UUID
	startAt, endAt         *time.Time
	breaks                 []byte
	intervalBreakPunchMode string
	actualBreakMins        int
	actualBreakCount       int
	hasOpenBreak           bool
	approvedBreakWorked    int
}

func loadReprocessDailyGroups(ctx context.Context, db DBTX, startDate, endDate string, employeeID *uuid.UUID) ([]*reprocessDailyGroup, error) {
	return loadReprocessDailyGroupsForScope(ctx, db, startDate, endDate, employeeID, nil)
}

func loadReprocessDailyGroupsForEmployment(ctx context.Context, db DBTX, startDate, endDate string, employmentID uuid.UUID) ([]*reprocessDailyGroup, error) {
	return loadReprocessDailyGroupsForScope(ctx, db, startDate, endDate, nil, &employmentID)
}

func loadReprocessDailyGroupsForScope(ctx context.Context, db DBTX, startDate, endDate string, employeeID, employmentID *uuid.UUID) ([]*reprocessDailyGroup, error) {
	parsedStart, err := time.Parse("2006-01-02", startDate)
	if err != nil {
		return nil, err
	}
	parsedEnd, err := time.Parse("2006-01-02", endDate)
	if err != nil {
		return nil, err
	}
	businessDateExpr := `COALESCE(ws.date, (te.punch_in_time AT TIME ZONE COALESCE(sdr.timezone, 'UTC'))::date)`
	where := businessDateExpr + ` >= $1::date AND ` + businessDateExpr + ` <= $2::date`
	args := []any{startDate, endDate}
	if employeeID != nil {
		where += " AND e.employee_id = $3"
		args = append(args, *employeeID)
	} else if employmentID != nil {
		where += " AND te.employment_id = $3"
		args = append(args, *employmentID)
	}
	rows, err := db.Query(ctx, `SELECT te.employment_id, `+businessDateExpr+`::text, te.punch_in_time, te.punch_out_time, te.duration_minutes, te.segment_id, te.shift_version_id, ws.scheduled_start_at, ws.scheduled_end_at, ws.breaks_snapshot, COALESCE(sdi.break_punch_mode, 'none'), COALESCE(tbr.actual_break_minutes, 0), COALESCE(tbr.actual_break_count, 0), COALESCE(tbr.has_open_break, false), COALESCE(bwo.approved_break_worked_minutes, 0)
		FROM time_entries te
		JOIN employment_segments e ON e.id = te.employment_id
		LEFT JOIN work_segments ws ON ws.id = te.segment_id
		LEFT JOIN shift_default_intervals sdi ON sdi.id = ws.interval_id
		LEFT JOIN schedule_day_revisions sdr ON sdr.id = ws.schedule_revision_id
		LEFT JOIN LATERAL (
			SELECT
				COALESCE(SUM(CASE WHEN tb.break_end_at IS NOT NULL THEN GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (tb.break_end_at - tb.break_start_at)) / 60)) ELSE 0 END), 0)::integer AS actual_break_minutes,
				COUNT(*)::integer AS actual_break_count,
				BOOL_OR(tb.break_end_at IS NULL) AS has_open_break
			FROM time_entry_breaks tb
			WHERE tb.time_entry_id = te.id
		) tbr ON true
		LEFT JOIN LATERAL (
			SELECT COALESCE(SUM(o.minutes), 0)::integer AS approved_break_worked_minutes
			FROM schedule_day_overrides o
			WHERE o.employment_id = te.employment_id
			  AND o.business_date = `+businessDateExpr+`
			  AND o.work_segment_id = te.segment_id
			  AND o.override_type = 'break_worked'
			  AND o.status = 'approved'
		) bwo ON true
		WHERE `+where+` AND te.punch_out_time IS NOT NULL ORDER BY te.employment_id, te.punch_in_time`, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	rawRows := make([]rawReprocessRow, 0)
	empIDSet := make(map[uuid.UUID]struct{})
	empIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var r rawReprocessRow
		if err := rows.Scan(&r.empID, &r.dateStr, &r.punchIn, &r.punchOut, &r.duration, &r.segmentID, &r.shiftID, &r.startAt, &r.endAt, &r.breaks, &r.intervalBreakPunchMode, &r.actualBreakMins, &r.actualBreakCount, &r.hasOpenBreak, &r.approvedBreakWorked); err != nil {
			return nil, err
		}
		rawRows = append(rawRows, r)
		if _, seen := empIDSet[r.empID]; !seen {
			empIDSet[r.empID] = struct{}{}
			empIDs = append(empIDs, r.empID)
		}
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	rows.Close()
	if len(rawRows) == 0 {
		return nil, nil
	}

	policiesByEmp, err := (WorkPolicyResolver{}).ResolveRangeForEmployments(ctx, db, empIDs, parsedStart, parsedEnd)
	if err != nil {
		return nil, err
	}

	byKey := make(map[string]*reprocessDailyGroup)
	var ordered []*reprocessDailyGroup
	for _, r := range rawRows {
		day, parseErr := time.Parse("2006-01-02", r.dateStr)
		if parseErr != nil {
			return nil, parseErr
		}
		policy, ok := FindResolvedPolicyForDate(policiesByEmp[r.empID], day)
		if !ok {
			return nil, api.ErrAttendancePolicyMissing
		}
		breakWindows, breakStart, breakEnd, scheduledBreak, breakPunchMode, err := parseBreakWindowsSnapshot(r.breaks)
		if err != nil {
			return nil, err
		}
		if r.intervalBreakPunchMode == string(buckets.BreakPunchModeRequired) {
			breakPunchMode = buckets.BreakPunchModeRequired
		}
		key := r.empID.String() + ":" + r.dateStr
		item := byKey[key]
		if item == nil {
			item = &reprocessDailyGroup{
				employmentID: r.empID,
				date:         r.dateStr,
				otAggMode:    buckets.OtAggregationMode(policy.OTAggregationMode),
			}
			byKey[key] = item
			ordered = append(ordered, item)
		}
		item.actual += int(r.punchOut.Sub(r.punchIn) / time.Minute)
		item.scheduledBreak += int(scheduledBreak)
		if r.startAt != nil {
			item.lateMinutes += int(matcher.ComputeLateMinutes(r.punchIn, *r.startAt, policy.LateArrivalGraceMinutes, false))
		}
		var maxEarly32 *int32
		if policy.EarlyOTMaxMinutes != nil {
			v := int32(*policy.EarlyOTMaxMinutes)
			maxEarly32 = &v
		}
		item.inputs = append(item.inputs, buckets.DailyCalcInput{
			PunchIn: r.punchIn, PunchOut: r.punchOut, ShiftStart: r.startAt, ShiftEnd: r.endAt,
			ScheduledBreakMinutes: scheduledBreak, ActualBreakMinutes: int32(r.actualBreakMins),
			ApprovedBreakWorkedMinutes: int32(r.approvedBreakWorked), ActualBreakCount: r.actualBreakCount,
			HasOpenActualBreak: r.hasOpenBreak, BreakPunchMode: breakPunchMode,
			BreakStart: breakStart, BreakEnd: breakEnd, BreakWindows: breakWindows,
			WorkUnit: int32(max(1, policy.WorkUnitMinutes)), RoundingPolicy: buckets.RoundingPolicy(policy.RoundingPolicy),
			EarlyArrivalStrategy: policy.EarlyArrivalStrategy, GracePeriodMinutes: int32(policy.EarlyOTQualificationGraceMinutes),
			AllowEarlyOT: policy.EarlyOTEnabled, IsBreakAutoDeduct: policy.BreakAutoDeduct, IsBreakOvertime: policy.BreakIsOvertime,
			MaxEarlyOTMinutes: maxEarly32, SegmentID: r.segmentID, OTAggregationMode: buckets.OtAggregationMode(policy.OTAggregationMode),
		})
		policyVerID := policy.VersionID
		item.policyID = &policyVerID
		item.shiftID = r.shiftID
	}
	return ordered, nil
}

func upsertReprocessSummary(ctx context.Context, db DBTX, item *reprocessDailyGroup) error {
	calc := buckets.CalculateDayBuckets(item.inputs, item.otAggMode)
	var excCode *string
	if calc.BreakExceptionCode != "" {
		s := string(calc.BreakExceptionCode)
		excCode = &s
	}
	sysOT := int(calc.Buckets.EarlyOTMinutes + calc.Buckets.LateOTMinutes + calc.Buckets.BreakOTMinutes)
	_, err := db.Exec(ctx, `INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes, late_minutes, system_detected_ot_minutes, early_ot_minutes, late_ot_minutes, break_ot_minutes, scheduled_break_minutes, actual_break_minutes, deducted_break_minutes, net_work_minutes, net_paid_minutes, break_provisional, break_exception_code, work_policy_version_id, shift_version_id, updated_at) VALUES ($1, $2, $3::date, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, date) DO UPDATE SET actual_work_minutes = EXCLUDED.actual_work_minutes, standard_minutes = EXCLUDED.standard_minutes, late_minutes = EXCLUDED.late_minutes, system_detected_ot_minutes = EXCLUDED.system_detected_ot_minutes, early_ot_minutes = EXCLUDED.early_ot_minutes, late_ot_minutes = EXCLUDED.late_ot_minutes, break_ot_minutes = EXCLUDED.break_ot_minutes, scheduled_break_minutes = EXCLUDED.scheduled_break_minutes, actual_break_minutes = EXCLUDED.actual_break_minutes, deducted_break_minutes = EXCLUDED.deducted_break_minutes, net_work_minutes = EXCLUDED.net_work_minutes, net_paid_minutes = EXCLUDED.net_paid_minutes, break_provisional = EXCLUDED.break_provisional, break_exception_code = EXCLUDED.break_exception_code, work_policy_version_id = EXCLUDED.work_policy_version_id, shift_version_id = EXCLUDED.shift_version_id, updated_at = CURRENT_TIMESTAMP`, uuid.New(), item.employmentID, item.date, item.actual, int(calc.Buckets.StandardMinutes), item.lateMinutes, sysOT, int(calc.Buckets.EarlyOTMinutes), int(calc.Buckets.LateOTMinutes), int(calc.Buckets.BreakOTMinutes), item.scheduledBreak, int(calc.ActualBreakMinutes), int(calc.DeductedBreakMinutes), int(calc.Buckets.StandardMinutes), int(calc.Buckets.StandardMinutes), calc.BreakProvisional, excCode, item.policyID, item.shiftID)
	return err
}

func (r AttendanceRepository) ReprocessAttendance(ctx context.Context, startDate, endDate string, employeeID *uuid.UUID) (api.ReprocessResponse, error) {
	beginner, err := r.begin(ctx)
	if err != nil {
		return api.ReprocessResponse{}, err
	}
	var response api.ReprocessResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, db pgx.Tx) error {
		if err := ensureReprocessPeriodsOpen(ctx, db, startDate, endDate, employeeID); err != nil {
			return err
		}
		if err := pairRawLogsForReprocess(ctx, db, startDate, endDate, employeeID); err != nil {
			return err
		}
		groups, err := loadReprocessDailyGroups(ctx, db, startDate, endDate, employeeID)
		if err != nil {
			return err
		}
		for _, item := range groups {
			if err := upsertReprocessSummary(ctx, db, item); err != nil {
				return err
			}
		}
		response = api.ReprocessResponse{StartDate: startDate, EndDate: endDate, Days: len(groups), Recalculated: len(groups)}
		if employeeID != nil {
			response.EmploymentID = employeeID
		}
		return nil
	})
	if err != nil {
		return api.ReprocessResponse{}, err
	}
	return response, nil
}

func resolveActiveEmployment(ctx context.Context, db DBTX, employeeID uuid.UUID) (uuid.UUID, error) {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&exists); err != nil {
		return uuid.Nil, err
	}
	if !exists {
		return uuid.Nil, errNoRows
	}
	var employmentID uuid.UUID
	var status string
	err := db.QueryRow(ctx, `SELECT em.id, em.status FROM employment_segments em JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = er.employee_id WHERE em.employee_id = $1 AND em.deleted_at IS NULL AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY (em.status = 'active') DESC, em.start_date DESC, em.id DESC LIMIT 1`, employeeID).Scan(&employmentID, &status)
	if err != nil {
		return uuid.Nil, api.ErrNoActiveEmployment
	}
	if status != "active" {
		return uuid.Nil, api.ErrEmploymentInactive
	}
	return employmentID, nil
}

func scanLatestTimeEntry(ctx context.Context, db DBTX, employmentID uuid.UUID, record *api.TimeEntryRecord) error {
	row := db.QueryRow(ctx, `SELECT id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE employment_id = $1 ORDER BY punch_in_time DESC LIMIT 1`, employmentID)
	return scanTimeEntry(row, record)
}

// ListTimeHistory returns entries for an employee's active employment.
func (r AttendanceRepository) ListTimeHistory(ctx context.Context, employeeID uuid.UUID, start, end time.Time) ([]api.TimeEntryRecord, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return nil, err
	}
	employmentID, err := resolveActiveEmployment(ctx, db, employeeID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE employment_id = $1 AND punch_in_time >= $2 AND punch_in_time <= $3 ORDER BY punch_in_time DESC`, employmentID, start, end)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	entries := make([]api.TimeEntryRecord, 0)
	for rows.Next() {
		var entry api.TimeEntryRecord
		if err := scanTimeEntry(rows, &entry); err != nil {
			return nil, err
		}
		entries = append(entries, entry)
	}
	return entries, rows.Err()
}

// IsEmployeeOwnedByUser reports whether the employee's currently effective tenure belongs to the user.
func (r AttendanceRepository) IsEmployeeOwnedByUser(ctx context.Context, employeeID, userID uuid.UUID) (bool, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return false, err
	}
	var owned bool
	err = db.QueryRow(ctx, `
		SELECT EXISTS (
			SELECT 1
			FROM employee_tenures t
			JOIN employees e ON e.id = t.employee_id
			WHERE t.employee_id = $1
			  AND t.user_id = $2
			  AND t.deleted_at IS NULL
			  AND e.deleted_at IS NULL
			  AND t.start_date <= `+orgTodaySQL+`
			  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
		)
	`, employeeID, userID).Scan(&owned)
	return owned, err
}

// CreateAdjustment inserts a pending manual adjustment matching schema.
func (r AttendanceRepository) CreateAdjustment(ctx context.Context, req api.AdjustmentRequest, requestedByUserID ids.UserID) (api.AdjustmentRecord, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	var exists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employment_segments WHERE id = $1 AND deleted_at IS NULL)`, req.EmploymentID).Scan(&exists)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	if !exists {
		return api.AdjustmentRecord{}, errNoRows
	}
	requestedTime := time.Time{}
	requestedValue := req.RequestedPunchTime
	if requestedValue == "" && req.Type == api.AttendanceAdjustmentBreakWorked {
		requestedValue = req.Start
	}
	if requestedValue != "" {
		requestedTime, err = time.Parse(time.RFC3339, requestedValue)
		if err != nil {
			return api.AdjustmentRecord{}, api.ErrInvalidAttendanceAdjustment
		}
	}
	adjustmentType := req.Type
	if adjustmentType == "" {
		adjustmentType = api.AttendanceAdjustmentAttendanceCorrection
	}
	details, err := json.Marshal(map[string]any{
		"segment_id": req.SegmentID, "direction": req.Direction, "start": req.Start, "end": req.End,
		"corrected_punch_in": req.CorrectedPunchIn, "corrected_punch_out": req.CorrectedPunchOut,
	})
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	adjID := uuid.New()
	var record api.AdjustmentRecord
	var reqPunchTime, createdAt, updatedAt time.Time
	var fileIDs []uuid.UUID
	err = db.QueryRow(ctx, `INSERT INTO attendance_adjustment_requests (id, employment_id, time_entry_id, requested_punch_time, reason, status, requested_by_user_id, attachment_file_ids, adjustment_type, details, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, employment_id, time_entry_id, requested_punch_time, reason, status, requested_by_user_id, approved_by_user_id, attachment_file_ids, adjustment_type, details, created_at, updated_at`, adjID, req.EmploymentID, req.TimeEntryID, requestedTime, req.Reason, requestedByUserID.UUID(), req.AttachmentFileIDs, adjustmentType, details).Scan(&record.ID, &record.EmploymentID, &record.TimeEntryID, &reqPunchTime, &record.Reason, &record.Status, &record.RequestedByUserID, &record.ApprovedByUserID, &fileIDs, &record.Type, &details, &createdAt, &updatedAt)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	record.RequestedPunchTime = reqPunchTime.UTC().Format(time.RFC3339)
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	record.AttachmentFileIDs = fileIDs
	return record, nil
}

func (r AttendanceRepository) ListAdjustments(ctx context.Context, status *string, employeeID *uuid.UUID, page, limit int) (api.PaginatedAdjustments, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return api.PaginatedAdjustments{}, err
	}
	offset := (page - 1) * limit
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM attendance_adjustment_requests a JOIN employment_segments em ON em.id = a.employment_id WHERE ($1::text IS NULL OR a.status::text = $1) AND ($2::uuid IS NULL OR em.employee_id = $2)`, status, employeeID).Scan(&total); err != nil {
		return api.PaginatedAdjustments{}, err
	}
	rows, err := db.Query(ctx, `SELECT a.id, a.employment_id, a.time_entry_id, a.requested_punch_time, a.reason, a.status::text, a.requested_by_user_id, a.approved_by_user_id, a.attachment_file_ids, a.adjustment_type, a.details, a.created_at, a.updated_at FROM attendance_adjustment_requests a JOIN employment_segments em ON em.id = a.employment_id WHERE ($1::text IS NULL OR a.status::text = $1) AND ($2::uuid IS NULL OR em.employee_id = $2) ORDER BY a.created_at DESC LIMIT $3 OFFSET $4`, status, employeeID, limit, offset)
	if err != nil {
		return api.PaginatedAdjustments{}, err
	}
	defer rows.Close()
	data := make([]api.AdjustmentRecord, 0)
	for rows.Next() {
		var record api.AdjustmentRecord
		if err := scanAdjustment(rows, &record); err != nil {
			return api.PaginatedAdjustments{}, err
		}
		data = append(data, record)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedAdjustments{}, err
	}
	return api.PaginatedAdjustments{Data: data, Total: total, Page: page, Limit: limit}, nil
}

func (r AttendanceRepository) ListEmployeeAdjustments(ctx context.Context, employeeID uuid.UUID) ([]api.AdjustmentRecord, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT a.id, a.employment_id, a.time_entry_id, a.requested_punch_time, a.reason, a.status::text, a.requested_by_user_id, a.approved_by_user_id, a.attachment_file_ids, a.adjustment_type, a.details, a.created_at, a.updated_at FROM attendance_adjustment_requests a JOIN employment_segments em ON em.id = a.employment_id WHERE em.employee_id = $1 ORDER BY a.created_at DESC`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	data := make([]api.AdjustmentRecord, 0)
	for rows.Next() {
		var record api.AdjustmentRecord
		if err := scanAdjustment(rows, &record); err != nil {
			return nil, err
		}
		data = append(data, record)
	}
	return data, rows.Err()
}

func (r AttendanceRepository) ApproveAdjustment(ctx context.Context, adjustmentID, approverID uuid.UUID, status string) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var employmentID uuid.UUID
		var requestedPunchTime time.Time
		var currentStatus, adjustmentType string
		var details []byte
		row := db.QueryRow(ctx, `SELECT employment_id, requested_punch_time, status::text, adjustment_type, details FROM attendance_adjustment_requests WHERE id = $1 FOR UPDATE`, adjustmentID)
		if err := row.Scan(&employmentID, &requestedPunchTime, &currentStatus, &adjustmentType, &details); err != nil {
			return err
		}
		if currentStatus != "pending" {
			return api.ErrAttendanceIdempotencyConflict
		}
		if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{requestedPunchTime.UTC().Format("2006-01-02")}); err != nil {
			return err
		}
		result, err := db.Exec(ctx, `UPDATE attendance_adjustment_requests SET status = $1::adjustment_status_type, approved_by_user_id = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3`, status, approverID, adjustmentID)
		if err != nil {
			return err
		}
		if result.RowsAffected() != 1 {
			return errNoRows
		}
		if status != "approved" {
			return nil
		}
		if _, err = db.Exec(ctx, `INSERT INTO audit_logs (actor_id, target_resource_id, action, details, actor_type, created_at) VALUES ($1, $2, 'attendance.adjustment.approved', jsonb_build_object('employment_id', $3, 'adjustment_type', $4, 'details', $5), 'human', CURRENT_TIMESTAMP)`, approverID, adjustmentID, employmentID, adjustmentType, details); err != nil {
			return err
		}
		if adjustmentType == api.AttendanceAdjustmentBreakWorked {
			return nil
		}
		_, err = db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', NULL, 0, 0, CURRENT_TIMESTAMP)`, uuid.New(), employmentID, requestedPunchTime)
		return err
	})
	return finishLifecycleMutation(ctx, tx, err)
}

func scanAdjustment(row pgx.Row, record *api.AdjustmentRecord) error {
	var requestedPunchTime, createdAt, updatedAt time.Time
	var details []byte
	if err := row.Scan(&record.ID, &record.EmploymentID, &record.TimeEntryID, &requestedPunchTime, &record.Reason, &record.Status, &record.RequestedByUserID, &record.ApprovedByUserID, &record.AttachmentFileIDs, &record.Type, &details, &createdAt, &updatedAt); err != nil {
		return err
	}
	if record.Type == "" {
		record.Type = api.AttendanceAdjustmentAttendanceCorrection
	}
	var typed struct {
		SegmentID         *uuid.UUID `json:"segment_id"`
		Direction         *string    `json:"direction"`
		Start             *string    `json:"start"`
		End               *string    `json:"end"`
		CorrectedPunchIn  *string    `json:"corrected_punch_in"`
		CorrectedPunchOut *string    `json:"corrected_punch_out"`
	}
	if len(details) > 0 && json.Unmarshal(details, &typed) == nil {
		record.SegmentID, record.Direction, record.Start, record.End = typed.SegmentID, typed.Direction, typed.Start, typed.End
		record.CorrectedPunchIn, record.CorrectedPunchOut = typed.CorrectedPunchIn, typed.CorrectedPunchOut
	}
	record.RequestedPunchTime = requestedPunchTime.UTC().Format(time.RFC3339)
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return nil
}

func (r AttendanceRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("attendance repository: database does not support transactions")
	}
	return beginner, nil
}

func scanTimeEntry(row pgx.Row, record *api.TimeEntryRecord) error {
	var punchIn, createdAt, updatedAt time.Time
	var punchOut *time.Time
	var metadata []byte
	if err := row.Scan(&record.ID, &record.EmploymentID, &record.ShiftID, &record.SegmentID, &record.ShiftVersionID, &record.BreaksSnapshot, &punchIn, &punchOut, &record.DurationMinutes, &record.IsBreakOvertime, &record.EntryType, &record.Note, &metadata, &createdAt, &updatedAt); err != nil {
		return err
	}
	record.PunchInTime = punchIn.UTC().Format(time.RFC3339)
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	if punchOut != nil {
		formatted := punchOut.UTC().Format(time.RFC3339)
		record.PunchOutTime = &formatted
	}
	if len(metadata) > 0 {
		record.GenerationMetadata = json.RawMessage(metadata)
	}
	return nil
}

// GetAttendanceSummaries queries paginated daily summaries.
func (r AttendanceRepository) GetAttendanceSummaries(ctx context.Context, startDate, endDate string, employeeID *uuid.UUID, otStatus *string, page, limit int) (api.PaginatedSummaries, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return api.PaginatedSummaries{}, err
	}
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 3000 {
		limit = 50
	}
	offset := (page - 1) * limit

	whereClauses := []string{"s.date >= $1", "s.date <= $2"}
	args := []interface{}{startDate, endDate}
	idx := 3

	if employeeID != nil {
		whereClauses = append(whereClauses, fmt.Sprintf("s.employment_id IN (SELECT id FROM employment_segments WHERE employee_id = $%d AND deleted_at IS NULL)", idx))
		args = append(args, *employeeID)
		idx++
	}
	if otStatus != nil && *otStatus != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("s.ot_status = $%d", idx))
		args = append(args, *otStatus)
		idx++
	}

	whereSQL := strings.Join(whereClauses, " AND ")

	countQuery := fmt.Sprintf(`SELECT COUNT(*) FROM attendance_daily_summaries s WHERE %s`, whereSQL)
	var total int64
	if err := db.QueryRow(ctx, countQuery, args...).Scan(&total); err != nil {
		return api.PaginatedSummaries{}, err
	}

	dataQuery := fmt.Sprintf(`SELECT s.id, s.employment_id, s.date::text, s.actual_work_minutes, s.standard_minutes, s.system_detected_ot_minutes, s.claimed_ot_minutes, s.approved_ot_minutes, s.early_ot_minutes, s.late_ot_minutes, s.break_ot_minutes, s.scheduled_break_minutes, s.actual_break_minutes, s.deducted_break_minutes, s.net_work_minutes, s.net_paid_minutes, s.break_provisional, s.break_exception_code, s.work_policy_version_id, s.shift_version_id, s.ot_status::text, s.ot_reason, s.manager_note, s.is_weekend, s.is_holiday, s.holiday_name, s.created_at, s.updated_at
FROM attendance_daily_summaries s
WHERE %s ORDER BY s.date DESC LIMIT $%d OFFSET $%d`, whereSQL, idx, idx+1)
	args = append(args, limit, offset)

	rows, err := db.Query(ctx, dataQuery, args...)
	if err != nil {
		return api.PaginatedSummaries{}, err
	}
	defer rows.Close()

	var data []api.AttendanceDailySummaryRecord
	empSet := make(map[uuid.UUID]struct{})
	empIDs := make([]uuid.UUID, 0)
	var minDate, maxDate time.Time
	for rows.Next() {
		var rec api.AttendanceDailySummaryRecord
		if err := rows.Scan(&rec.ID, &rec.EmploymentID, &rec.Date, &rec.ActualWorkMinutes, &rec.StandardMinutes, &rec.SystemDetectedOTMinutes, &rec.ClaimedOTMinutes, &rec.ApprovedOTMinutes, &rec.EarlyOTMinutes, &rec.LateOTMinutes, &rec.BreakOTMinutes, &rec.ScheduledBreakMinutes, &rec.ActualBreakMinutes, &rec.DeductedBreakMinutes, &rec.NetWorkMinutes, &rec.NetPaidMinutes, &rec.BreakProvisional, &rec.BreakExceptionCode, &rec.WorkPolicyVersionID, &rec.ShiftVersionID, &rec.OTStatus, &rec.OTReason, &rec.ManagerNote, &rec.IsWeekend, &rec.IsHoliday, &rec.HolidayName, &rec.CreatedAt, &rec.UpdatedAt); err != nil {
			return api.PaginatedSummaries{}, err
		}
		if _, seen := empSet[rec.EmploymentID]; !seen {
			empSet[rec.EmploymentID] = struct{}{}
			empIDs = append(empIDs, rec.EmploymentID)
		}
		if d, parseErr := time.Parse("2006-01-02", rec.Date); parseErr == nil {
			if minDate.IsZero() || d.Before(minDate) {
				minDate = d
			}
			if maxDate.IsZero() || d.After(maxDate) {
				maxDate = d
			}
		}
		data = append(data, rec)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedSummaries{}, err
	}
	rows.Close()
	if data == nil {
		data = []api.AttendanceDailySummaryRecord{}
	} else if len(empIDs) > 0 && !minDate.IsZero() {
		policiesByEmp, err := (WorkPolicyResolver{}).ResolveRangeForEmployments(ctx, db, empIDs, minDate, maxDate)
		if err != nil {
			return api.PaginatedSummaries{}, err
		}
		for i := range data {
			if d, parseErr := time.Parse("2006-01-02", data[i].Date); parseErr == nil {
				if policy, ok := FindResolvedPolicyForDate(policiesByEmp[data[i].EmploymentID], d); ok {
					if data[i].WorkPolicyVersionID == nil {
						vid := policy.VersionID
						data[i].WorkPolicyVersionID = &vid
					}
					if policy.MinutesPerCredit > 0 {
						data[i].StandardCredits = math.Round(float64(data[i].StandardMinutes)/float64(policy.MinutesPerCredit)*100) / 100
					}
				}
			}
		}
	}
	return api.PaginatedSummaries{
		Data:  data,
		Total: total,
		Page:  page,
		Limit: limit,
	}, nil
}

// GetEmployeeIDByUserID returns the employee ID for a given user ID via their active tenure.
func (r AttendanceRepository) GetEmployeeIDByUserID(ctx context.Context, userID uuid.UUID) (uuid.UUID, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `
		SELECT t.employee_id
		FROM employee_tenures t
		JOIN employees e ON e.id = t.employee_id
		WHERE t.user_id = $1
		  AND t.deleted_at IS NULL
		  AND e.deleted_at IS NULL
		  AND t.start_date <= `+orgTodaySQL+`
		  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
		ORDER BY t.start_date DESC, t.id DESC
		LIMIT 1
	`, userID).Scan(&empID)
	if err != nil {
		return uuid.Nil, err
	}
	return empID, nil
}

// UpdateTimeEntry updates time entry fields.
// UpdateTimeEntry changes an attendance entry and remains guarded because it can alter closed evidence.
func (r AttendanceRepository) UpdateTimeEntry(ctx context.Context, entryID uuid.UUID, req api.UpdateTimeEntryRequest) (api.TimeEntryRecord, error) {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	row := db.QueryRow(ctx, `SELECT id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE id = $1`, entryID)
	if err := scanTimeEntry(row, &record); err != nil {
		return api.TimeEntryRecord{}, err
	}

	originalPunchIn, err := time.Parse(time.RFC3339, record.PunchInTime)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	punchInTime := record.PunchInTime
	if req.PunchInTime != nil {
		punchInTime = *req.PunchInTime
	}
	var punchOutTime *string = record.PunchOutTime
	if req.PunchOutTime != nil {
		punchOutTime = req.PunchOutTime
	}
	entryType := record.EntryType
	if req.EntryType != nil {
		entryType = *req.EntryType
	}
	note := record.Note
	if req.Note != nil {
		note = req.Note
	}
	segmentID := record.SegmentID
	if req.SegmentID != nil {
		segmentID = req.SegmentID
	}

	pIn, err := time.Parse(time.RFC3339, punchInTime)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var pOut *time.Time
	var durationMinutes *int
	if punchOutTime != nil && *punchOutTime != "" {
		t, parseErr := time.Parse(time.RFC3339, *punchOutTime)
		if parseErr != nil {
			return api.TimeEntryRecord{}, parseErr
		}
		pOut = &t
		mins := int(t.Sub(pIn).Minutes())
		if mins >= 0 {
			durationMinutes = &mins
		}
	}
	guardDates := []string{originalPunchIn.UTC().Format("2006-01-02"), pIn.UTC().Format("2006-01-02")}
	if pOut != nil {
		guardDates = append(guardDates, pOut.UTC().Format("2006-01-02"))
	}
	if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{record.EmploymentID}, guardDates); err != nil {
		return api.TimeEntryRecord{}, finishLifecycleMutationDB(ctx, db, err)
	}

	updateRow := db.QueryRow(ctx, `UPDATE time_entries SET punch_in_time = $1, punch_out_time = $2, duration_minutes = $3, entry_type = $4, note = $5, segment_id = $6, template_id = (SELECT template_id FROM work_segments WHERE id = $6), shift_version_id = (SELECT selected_shift_version_id FROM work_segments WHERE id = $6), breaks_snapshot = (SELECT breaks_snapshot FROM work_segments WHERE id = $6), updated_at = CURRENT_TIMESTAMP WHERE id = $7 RETURNING id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, pIn, pOut, durationMinutes, entryType, note, segmentID, entryID)
	var updatedRecord api.TimeEntryRecord
	if err := scanTimeEntry(updateRow, &updatedRecord); err != nil {
		return api.TimeEntryRecord{}, err
	}
	return updatedRecord, nil
}

// DeleteTimeEntry deletes a time entry.
func (r AttendanceRepository) DeleteTimeEntry(ctx context.Context, entryID uuid.UUID) error {
	db, err := requireDB(r.DB, "attendance repository")
	if err != nil {
		return err
	}
	var employmentID uuid.UUID
	var punchInTime time.Time
	if err := db.QueryRow(ctx, `SELECT employment_id, punch_in_time FROM time_entries WHERE id = $1`, entryID).Scan(&employmentID, &punchInTime); err != nil {
		return err
	}
	if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{punchInTime.UTC().Format("2006-01-02")}); err != nil {
		return finishLifecycleMutationDB(ctx, db, err)
	}
	cmd, err := db.Exec(ctx, `DELETE FROM time_entries WHERE id = $1`, entryID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return errNoRows
	}
	return nil
}
