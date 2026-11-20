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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/buckets"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// AttendanceRepository persists punch events and paired time entries.
type AttendanceRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

// TxBeginner starts a database transaction.
type TxBeginner interface {
	Begin(context.Context) (pgx.Tx, error)
}

// PunchIn writes the raw attendance log and opens a time entry.
func (r AttendanceRepository) PunchIn(ctx context.Context, employmentID uuid.UUID, segmentID *uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (api.TimeEntryRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, tx pgx.Tx) error {
		var active bool
		if err := tx.QueryRow(ctx, `SELECT status = 'active' FROM employments WHERE id = $1 AND deleted_at IS NULL`, employmentID).Scan(&active); err != nil {
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
		cmd, err := tx.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', $4, 0, 1, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), employmentID, punchTime, rawData)
		if err != nil {
			return err
		}
		if cmd.RowsAffected() == 0 {
			return scanLatestTimeEntry(ctx, tx, employmentID, &record)
		}
		row := tx.QueryRow(ctx, `INSERT INTO time_entries (id, employment_id, segment_id, template_id, shift_version_id, breaks_snapshot, punch_in_time, entry_type, note, is_break_overtime, created_at, updated_at) VALUES ($1, $2, $7, (SELECT template_id FROM work_segments WHERE id = $7), (SELECT selected_shift_version_id FROM work_segments WHERE id = $7), (SELECT breaks_snapshot FROM work_segments WHERE id = $7), $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, employment_id, template_id, segment_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, uuid.New(), employmentID, punchTime, entryType, note, isBreakOvertime, resolvedSegmentID)
		return scanTimeEntry(row, &record)
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
		rawData, err := json.Marshal(struct {
			Punch  int     `json:"punch"`
			Status int     `json:"status"`
			Note   *string `json:"note"`
			Source string  `json:"source"`
		}{Punch: 1, Status: 1, Note: note, Source: "direct_punch"})
		if err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', $4, 1, 1, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), employmentID, punchTime, rawData); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE time_entries SET punch_out_time = $1, duration_minutes = GREATEST(0, FLOOR(EXTRACT(EPOCH FROM ($1 - punch_in_time)) / 60)::integer), note = COALESCE($2, note), updated_at = CURRENT_TIMESTAMP WHERE id = (SELECT id FROM time_entries WHERE employment_id = $3 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1)`, punchTime, note, employmentID); err != nil {
			return err
		}
		return scanLatestTimeEntry(ctx, db, employmentID, &record)
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
	_, err = db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'manual', $4, $5, 1, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), employmentID, punchTime, data, punch)
	return err
}

// resolvePunchSegment accepts an explicit segment only when it owns the employment
// and contains the punch timestamp. Without an explicit segment, it pins only a
// unique matching materialized segment; ambiguous or legacy rows stay unresolved.
func resolvePunchSegment(ctx context.Context, db DBTX, employmentID uuid.UUID, requestedID *uuid.UUID, punchTime time.Time) (*uuid.UUID, error) {
	const match = `scheduled_start_at IS NOT NULL
		AND scheduled_end_at IS NOT NULL
		AND scheduled_start_at <= $2
		AND scheduled_end_at >= $2`
	if requestedID != nil {
		var segmentID uuid.UUID
		err := db.QueryRow(ctx, `SELECT id FROM work_segments WHERE id = $1 AND employment_id = $3 AND `+match, *requestedID, punchTime, employmentID).Scan(&segmentID)
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, api.ErrWorkSegmentNotFound
		}
		if err != nil {
			return nil, err
		}
		return &segmentID, nil
	}

	rows, err := db.Query(ctx, `SELECT id FROM work_segments WHERE employment_id = $1 AND `+match+` ORDER BY scheduled_start_at, id LIMIT 2`, employmentID, punchTime)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var matches []uuid.UUID
	for rows.Next() {
		var segmentID uuid.UUID
		if err := rows.Scan(&segmentID); err != nil {
			return nil, err
		}
		matches = append(matches, segmentID)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(matches) != 1 {
		return nil, nil
	}
	return &matches[0], nil
}

func (r AttendanceRepository) GetESSAttendanceState(ctx context.Context, employeeID uuid.UUID) (api.AttendanceActionResponse, error) {
	db, err := r.db(ctx)
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

func (r AttendanceRepository) ReprocessAttendance(ctx context.Context, startDate, endDate string, employeeID *uuid.UUID) (api.ReprocessResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ReprocessResponse{}, err
	}
	where := "te.punch_in_time::date >= $1::date AND te.punch_in_time::date <= $2::date"
	args := []any{startDate, endDate}
	if employeeID != nil {
		where += " AND e.employee_id = $3"
		args = append(args, *employeeID)
	}
	rows, err := db.Query(ctx, `SELECT te.employment_id, te.punch_in_time::date::text, te.punch_in_time, te.punch_out_time, te.duration_minutes, te.segment_id, te.shift_version_id, ws.scheduled_start_at, ws.scheduled_end_at, ws.breaks_snapshot, wp.id, wc.minutes_per_credit, wc.early_ot_enabled, wc.early_ot_qualification_grace_minutes, wc.early_ot_max_minutes, wc.break_auto_deduct, wc.break_is_overtime
		FROM time_entries te
		JOIN employments e ON e.id = te.employment_id
		LEFT JOIN work_segments ws ON ws.id = te.segment_id
		LEFT JOIN labor_policy_memberships lm ON lm.employment_id = te.employment_id AND lm.effective_from <= te.punch_in_time::date AND (lm.effective_to IS NULL OR lm.effective_to > te.punch_in_time::date)
		LEFT JOIN labor_policy_group_versions wp ON wp.group_id = lm.group_id AND wp.effective_from <= te.punch_in_time::date AND (wp.effective_to IS NULL OR wp.effective_to > te.punch_in_time::date)
		LEFT JOIN work_credit_policies wc ON wc.id = wp.work_credit_policy_id
		WHERE `+where+` AND te.punch_out_time IS NOT NULL ORDER BY te.employment_id, te.punch_in_time`, args...)
	if err != nil {
		return api.ReprocessResponse{}, err
	}
	defer rows.Close()
	type daily struct {
		employmentID                           uuid.UUID
		date                                   string
		actual, standard, early, late, breakOT int
		policyID, shiftID                      *uuid.UUID
		minutesPerCredit                       int
	}
	results := make(map[string]*daily)
	for rows.Next() {
		var d daily
		var punchIn, punchOut time.Time
		var duration *int
		var segmentID, shiftID, policyID *uuid.UUID
		var startAt, endAt *time.Time
		var breaks []byte
		var earlyEnabled, breakAuto, breakOT bool
		var grace int
		var maxEarly *int
		if err := rows.Scan(&d.employmentID, &d.date, &punchIn, &punchOut, &duration, &segmentID, &shiftID, &startAt, &endAt, &breaks, &policyID, &d.minutesPerCredit, &earlyEnabled, &grace, &maxEarly, &breakAuto, &breakOT); err != nil {
			return api.ReprocessResponse{}, err
		}
		if policyID == nil {
			return api.ReprocessResponse{}, api.ErrPayrollPolicyMissing
		}
		var scheduledStart, scheduledEnd *time.Time = startAt, endAt
		var breakStart, breakEnd *time.Time
		var scheduledBreak int32
		if len(breaks) > 0 {
			var parsed []struct {
				Start    string `json:"start"`
				End      string `json:"end"`
				Duration int    `json:"duration_minutes"`
			}
			_ = json.Unmarshal(breaks, &parsed)
			if len(parsed) > 0 {
				if parsed[0].Start != "" {
					v, _ := time.Parse(time.RFC3339, parsed[0].Start)
					breakStart = &v
				}
				if parsed[0].End != "" {
					v, _ := time.Parse(time.RFC3339, parsed[0].End)
					breakEnd = &v
				}
				scheduledBreak = int32(parsed[0].Duration)
			}
		}
		key := d.employmentID.String() + ":" + d.date
		item := results[key]
		if item == nil {
			item = &d
			results[key] = item
		}
		item.actual += int(punchOut.Sub(punchIn) / time.Minute)
		input := buckets.DailyCalcInput{PunchIn: punchIn, PunchOut: punchOut, ShiftStart: scheduledStart, ShiftEnd: scheduledEnd, ScheduledBreakMinutes: scheduledBreak, BreakStart: breakStart, BreakEnd: breakEnd, WorkUnit: int32(max(1, d.minutesPerCredit)), GracePeriodMinutes: int32(grace), AllowEarlyOT: earlyEnabled, IsBreakAutoDeduct: breakAuto, IsBreakOvertime: breakOT, MaxEarlyOTMinutes: func() *int32 {
			if maxEarly == nil {
				return nil
			}
			v := int32(*maxEarly)
			return &v
		}()}
		calculated := buckets.CalculateBuckets(input)
		item.standard += int(calculated.StandardMinutes)
		item.early += int(calculated.EarlyOTMinutes)
		item.late += int(calculated.LateOTMinutes)
		item.breakOT += int(calculated.BreakOTMinutes)
		item.policyID = policyID
		item.shiftID = shiftID
	}
	if err := rows.Err(); err != nil {
		return api.ReprocessResponse{}, err
	}
	for _, item := range results {
		_, err := db.Exec(ctx, `INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes, system_detected_ot_minutes, early_ot_minutes, late_ot_minutes, break_ot_minutes, work_policy_version_id, shift_version_id, updated_at) VALUES ($1, $2, $3::date, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP) ON CONFLICT (employment_id, date) DO UPDATE SET actual_work_minutes = EXCLUDED.actual_work_minutes, standard_minutes = EXCLUDED.standard_minutes, system_detected_ot_minutes = EXCLUDED.system_detected_ot_minutes, early_ot_minutes = EXCLUDED.early_ot_minutes, late_ot_minutes = EXCLUDED.late_ot_minutes, break_ot_minutes = EXCLUDED.break_ot_minutes, work_policy_version_id = EXCLUDED.work_policy_version_id, shift_version_id = EXCLUDED.shift_version_id, updated_at = CURRENT_TIMESTAMP`, uuid.New(), item.employmentID, item.date, item.actual, item.standard, item.early+item.late+item.breakOT, item.early, item.late, item.breakOT, item.policyID, item.shiftID)
		if err != nil {
			return api.ReprocessResponse{}, err
		}
	}
	result := api.ReprocessResponse{StartDate: startDate, EndDate: endDate, Days: len(results), Recalculated: len(results)}
	if employeeID != nil {
		result.EmploymentID = employeeID
	}
	return result, nil
}

func (r AttendanceRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("attendance repository: database is not configured")
	}
	return r.DB, nil
}

func resolveActiveEmployment(ctx context.Context, db DBTX, employeeID uuid.UUID) (uuid.UUID, error) {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&exists); err != nil {
		return uuid.Nil, err
	}
	if !exists {
		return uuid.Nil, pgx.ErrNoRows
	}
	var employmentID uuid.UUID
	var status string
	err := db.QueryRow(ctx, `SELECT em.id, em.status FROM employments em JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = er.employee_id WHERE em.employee_id = $1 AND em.deleted_at IS NULL AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) ORDER BY (em.status = 'active') DESC, em.start_date DESC, em.id DESC LIMIT 1`, employeeID).Scan(&employmentID, &status)
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
	db, err := r.db(ctx)
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

// IsEmployeeOwnedByUser reports whether the employee record belongs to the user.
func (r AttendanceRepository) IsEmployeeOwnedByUser(ctx context.Context, employeeID, userID uuid.UUID) (bool, error) {
	db, err := r.db(ctx)
	if err != nil {
		return false, err
	}
	var owned bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL)`, employeeID, userID).Scan(&owned)
	return owned, err
}

// CreateAdjustment inserts a pending manual adjustment matching schema.
func (r AttendanceRepository) CreateAdjustment(ctx context.Context, req api.AdjustmentRequest, requestedByUserID ids.UserID) (api.AdjustmentRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	var exists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, req.EmploymentID).Scan(&exists)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	if !exists {
		return api.AdjustmentRecord{}, pgx.ErrNoRows
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
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedAdjustments{}, err
	}
	offset := (page - 1) * limit
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id WHERE ($1::text IS NULL OR a.status::text = $1) AND ($2::uuid IS NULL OR em.employee_id = $2)`, status, employeeID).Scan(&total); err != nil {
		return api.PaginatedAdjustments{}, err
	}
	rows, err := db.Query(ctx, `SELECT a.id, a.employment_id, a.time_entry_id, a.requested_punch_time, a.reason, a.status::text, a.requested_by_user_id, a.approved_by_user_id, a.attachment_file_ids, a.adjustment_type, a.details, a.created_at, a.updated_at FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id WHERE ($1::text IS NULL OR a.status::text = $1) AND ($2::uuid IS NULL OR em.employee_id = $2) ORDER BY a.created_at DESC LIMIT $3 OFFSET $4`, status, employeeID, limit, offset)
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
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT a.id, a.employment_id, a.time_entry_id, a.requested_punch_time, a.reason, a.status::text, a.requested_by_user_id, a.approved_by_user_id, a.attachment_file_ids, a.adjustment_type, a.details, a.created_at, a.updated_at FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id WHERE em.employee_id = $1 ORDER BY a.created_at DESC`, employeeID)
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
			return pgx.ErrNoRows
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
	if r.DB == nil {
		return nil, errors.New("attendance repository: database is not configured")
	}
	beginner, ok := r.DB.(TxBeginner)
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
	db, err := r.db(ctx)
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
		whereClauses = append(whereClauses, fmt.Sprintf("s.employment_id IN (SELECT id FROM employments WHERE employee_id = $%d AND deleted_at IS NULL)", idx))
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

	dataQuery := fmt.Sprintf(`SELECT s.id, s.employment_id, s.date::text, s.actual_work_minutes, s.standard_minutes, s.system_detected_ot_minutes, s.claimed_ot_minutes, s.approved_ot_minutes, s.early_ot_minutes, s.late_ot_minutes, s.break_ot_minutes, s.scheduled_break_minutes, s.actual_break_minutes, s.deducted_break_minutes, s.net_work_minutes, s.net_paid_minutes, s.break_provisional, s.break_exception_code, COALESCE(policy.minutes_per_credit, 0)::double precision, policy.version_id, s.shift_version_id, s.ot_status::text, s.ot_reason, s.manager_note, s.is_weekend, s.is_holiday, s.holiday_name, s.created_at, s.updated_at
FROM attendance_daily_summaries s
LEFT JOIN LATERAL (
	SELECT wc.minutes_per_credit, v.id AS version_id
	FROM labor_policy_memberships m
	JOIN labor_policy_group_versions v ON v.group_id = m.group_id
	JOIN work_credit_policies wc ON wc.id = v.work_credit_policy_id
	WHERE m.employment_id = s.employment_id
	  AND m.effective_from <= s.date
	  AND (m.effective_to IS NULL OR m.effective_to > s.date)
	  AND v.effective_from <= s.date
	  AND (v.effective_to IS NULL OR v.effective_to > s.date)
	ORDER BY v.effective_from DESC, v.id DESC
	LIMIT 1
) policy ON TRUE
WHERE %s ORDER BY s.date DESC LIMIT $%d OFFSET $%d`, whereSQL, idx, idx+1)
	args = append(args, limit, offset)

	rows, err := db.Query(ctx, dataQuery, args...)
	if err != nil {
		return api.PaginatedSummaries{}, err
	}
	defer rows.Close()

	var data []api.AttendanceDailySummaryRecord
	for rows.Next() {
		var rec api.AttendanceDailySummaryRecord
		var minutesPerCredit float64
		if err := rows.Scan(&rec.ID, &rec.EmploymentID, &rec.Date, &rec.ActualWorkMinutes, &rec.StandardMinutes, &rec.SystemDetectedOTMinutes, &rec.ClaimedOTMinutes, &rec.ApprovedOTMinutes, &rec.EarlyOTMinutes, &rec.LateOTMinutes, &rec.BreakOTMinutes, &rec.ScheduledBreakMinutes, &rec.ActualBreakMinutes, &rec.DeductedBreakMinutes, &rec.NetWorkMinutes, &rec.NetPaidMinutes, &rec.BreakProvisional, &rec.BreakExceptionCode, &minutesPerCredit, &rec.WorkPolicyVersionID, &rec.ShiftVersionID, &rec.OTStatus, &rec.OTReason, &rec.ManagerNote, &rec.IsWeekend, &rec.IsHoliday, &rec.HolidayName, &rec.CreatedAt, &rec.UpdatedAt); err != nil {
			return api.PaginatedSummaries{}, err
		}
		if minutesPerCredit > 0 {
			rec.StandardCredits = math.Round(float64(rec.StandardMinutes)/minutesPerCredit*100) / 100
		}
		data = append(data, rec)
	}
	if data == nil {
		data = []api.AttendanceDailySummaryRecord{}
	}
	return api.PaginatedSummaries{
		Data:  data,
		Total: total,
		Page:  page,
		Limit: limit,
	}, nil
}

// GetEmployeeIDByUserID returns the employee ID for a given user ID.
func (r AttendanceRepository) GetEmployeeIDByUserID(ctx context.Context, userID uuid.UUID) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employees WHERE user_id = $1 AND deleted_at IS NULL LIMIT 1`, userID).Scan(&empID)
	if err != nil {
		return uuid.Nil, err
	}
	return empID, nil
}

// UpdateTimeEntry updates time entry fields.
// UpdateTimeEntry changes an attendance entry and remains guarded because it can alter closed evidence.
func (r AttendanceRepository) UpdateTimeEntry(ctx context.Context, entryID uuid.UUID, req api.UpdateTimeEntryRequest) (api.TimeEntryRecord, error) {
	db, err := r.db(ctx)
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
	db, err := r.db(ctx)
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
		return pgx.ErrNoRows
	}
	return nil
}
