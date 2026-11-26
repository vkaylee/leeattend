package repository

import (
	"context"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/pairing"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/buckets"
)

type reprocessOfficialSegment struct {
	employmentID uuid.UUID
	date         string
	segmentID    uuid.UUID
	templateID   uuid.UUID
	versionID    *uuid.UUID
	startAt      time.Time
	endAt        time.Time
	breaks       []byte
}

func ensureReprocessPeriodsOpen(ctx context.Context, db DBTX, startDate, endDate string, employeeID *uuid.UUID) error {
	empIDs, err := loadReprocessTargetEmployments(ctx, db, startDate, endDate, employeeID)
	if err != nil || len(empIDs) == 0 {
		return err
	}
	// Lock target employments in deterministic order (matching ESSAttendanceAction/PunchIn/ResolveUnscheduled/CloseAttendancePeriod)
	// to serialize concurrent reprocess/resolve/closure operations without deadlocks.
	if _, err := db.Exec(ctx, `SELECT pg_advisory_xact_lock(hashtextextended(id::text, 0)) FROM unnest($1::uuid[]) AS id ORDER BY id`, empIDs); err != nil {
		return err
	}
	if _, err := db.Exec(ctx, `SELECT e.id FROM employees e JOIN employment_segments em ON em.employee_id = e.id WHERE em.id = ANY($1) ORDER BY e.id FOR UPDATE`, empIDs); err != nil {
		return err
	}
	if _, err := db.Exec(ctx, `SELECT id FROM employment_segments WHERE id = ANY($1) ORDER BY id FOR UPDATE`, empIDs); err != nil {
		return err
	}
	schema, err := lifecycleSchemaState(ctx, db)
	if err != nil || !schema.any() {
		return err
	}
	closed, err := hasClosedPeriod(ctx, db, schema, empIDs, startDate, endDate)
	if err != nil {
		return err
	}
	if closed {
		return api.ErrAttendanceLockedPeriod
	}
	return nil
}

func loadReprocessTargetEmployments(ctx context.Context, db DBTX, startDate, endDate string, employeeID *uuid.UUID) ([]uuid.UUID, error) {
	if employeeID != nil {
		rows, err := db.Query(ctx, `SELECT id FROM employment_segments WHERE employee_id = $1 AND deleted_at IS NULL`, *employeeID)
		if err != nil {
			return nil, err
		}
		defer rows.Close()
		return scanUUIDList(rows)
	}
	rows, err := db.Query(ctx, `SELECT DISTINCT employment_id FROM (
		SELECT employment_id FROM time_entries WHERE punch_in_time >= ($1::date - interval '1 day') AND punch_in_time <= ($2::date + interval '2 days')
		UNION
		SELECT employment_id FROM raw_attendance_logs WHERE punch_time >= ($1::date - interval '1 day') AND punch_time <= ($2::date + interval '2 days')
		UNION
		SELECT employment_id FROM schedule_day_revisions WHERE business_date BETWEEN $1::date AND $2::date AND state IN ('published', 'explicit_off')
	) t`, startDate, endDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	return scanUUIDList(rows)
}

func scanUUIDList(rows interface {
	Next() bool
	Scan(...any) error
	Err() error
}) ([]uuid.UUID, error) {
	var ids []uuid.UUID
	for rows.Next() {
		var id uuid.UUID
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		ids = append(ids, id)
	}
	return ids, rows.Err()
}

func pairRawLogsForReprocess(ctx context.Context, db DBTX, startDate, endDate string, employeeID *uuid.UUID) error {
	return pairRawLogsForScope(ctx, db, startDate, endDate, employeeID, nil)
}

func pairRawLogsForEmployment(ctx context.Context, db DBTX, startDate, endDate string, employmentID uuid.UUID) error {
	return pairRawLogsForScope(ctx, db, startDate, endDate, nil, &employmentID)
}

func pairRawLogsForScope(ctx context.Context, db DBTX, startDate, endDate string, employeeID, employmentID *uuid.UUID) error {
	segmentsByEmp, err := loadOfficialSegmentsForPairing(ctx, db, startDate, endDate, employeeID, employmentID)
	if err != nil || len(segmentsByEmp) == 0 {
		return err
	}
	rawByEmployment, err := loadRawLogsForPairing(ctx, db, startDate, endDate, employeeID, employmentID)
	if err != nil || len(rawByEmployment) == 0 {
		return err
	}
	for empID, segs := range segmentsByEmp {
		logs := rawByEmployment[empID]
		if len(logs) == 0 {
			continue
		}
		if err := pairAndInsertEmploymentSessions(ctx, db, empID, segs, logs); err != nil {
			return err
		}
	}
	return nil
}

func loadOfficialSegmentsForPairing(ctx context.Context, db DBTX, startDate, endDate string, employeeID, employmentID *uuid.UUID) (map[uuid.UUID][]reprocessOfficialSegment, error) {
	where := `sdr.state = 'published' AND ws.segment_status <> 'cancelled' AND ws.date BETWEEN $1::date AND $2::date AND ws.scheduled_start_at IS NOT NULL AND ws.scheduled_end_at IS NOT NULL`
	args := []any{startDate, endDate}
	if employeeID != nil {
		where += ` AND e.employee_id = $3`
		args = append(args, *employeeID)
	} else if employmentID != nil {
		where += ` AND ws.employment_id = $3`
		args = append(args, *employmentID)
	}
	rows, err := db.Query(ctx, `SELECT ws.employment_id, ws.date::text, ws.id, ws.template_id, ws.selected_shift_version_id, ws.scheduled_start_at, ws.scheduled_end_at, ws.breaks_snapshot
		FROM work_segments ws
		JOIN schedule_day_revisions sdr ON sdr.id = ws.schedule_revision_id
		JOIN employment_segments e ON e.id = ws.employment_id
		WHERE `+where+`
		ORDER BY ws.employment_id, ws.scheduled_start_at, ws.sort_order`, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	grouped := make(map[uuid.UUID][]reprocessOfficialSegment)
	for rows.Next() {
		var seg reprocessOfficialSegment
		if err := rows.Scan(&seg.employmentID, &seg.date, &seg.segmentID, &seg.templateID, &seg.versionID, &seg.startAt, &seg.endAt, &seg.breaks); err != nil {
			return nil, err
		}
		grouped[seg.employmentID] = append(grouped[seg.employmentID], seg)
	}
	return grouped, rows.Err()
}

func loadRawLogsForPairing(ctx context.Context, db DBTX, startDate, endDate string, employeeID, employmentID *uuid.UUID) (map[uuid.UUID][]pairing.LogEvent, error) {
	where := `ral.punch_time >= ($1::date - interval '1 day') AND ral.punch_time <= ($2::date + interval '2 days')`
	args := []any{startDate, endDate}
	if employeeID != nil {
		where += ` AND e.employee_id = $3`
		args = append(args, *employeeID)
	} else if employmentID != nil {
		where += ` AND ral.employment_id = $3`
		args = append(args, *employmentID)
	}
	rows, err := db.Query(ctx, `SELECT ral.id, ral.employment_id, ral.punch_time
		FROM raw_attendance_logs ral
		JOIN employment_segments e ON e.id = ral.employment_id
		WHERE `+where+`
		ORDER BY ral.employment_id, ral.punch_time, ral.id`, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	grouped := make(map[uuid.UUID][]pairing.LogEvent)
	for rows.Next() {
		var id, empID uuid.UUID
		var punchTime time.Time
		if err := rows.Scan(&id, &empID, &punchTime); err != nil {
			return nil, err
		}
		minutes := int32(punchTime.UTC().Hour()*60 + punchTime.UTC().Minute())
		grouped[empID] = append(grouped[empID], pairing.LogEvent{ID: id, Minutes: minutes, PunchTime: punchTime})
	}
	return grouped, rows.Err()
}

func pairAndInsertEmploymentSessions(ctx context.Context, db DBTX, employmentID uuid.UUID, segs []reprocessOfficialSegment, allLogs []pairing.LogEvent) error {
	minStart, maxEnd := segs[0].startAt, segs[0].endAt
	segByID := make(map[uuid.UUID]reprocessOfficialSegment, len(segs))
	configs := make([]pairing.ShiftConfig, 0, len(segs))
	for _, s := range segs {
		if s.startAt.Before(minStart) {
			minStart = s.startAt
		}
		if s.endAt.After(maxEnd) {
			maxEnd = s.endAt
		}
		segID := s.segmentID
		startAt, endAt := s.startAt, s.endAt
		segByID[segID] = s
		configs = append(configs, pairing.ShiftConfig{
			StartMinutes: int32(startAt.UTC().Hour()*60 + startAt.UTC().Minute()),
			EndMinutes:   int32(endAt.UTC().Hour()*60 + endAt.UTC().Minute()),
			StartTime:    &startAt,
			EndTime:      &endAt,
			SegmentID:    &segID,
		})
	}
	windowLogs := filterLogsInWindow(allLogs, minStart.Add(-2*time.Hour), maxEnd.Add(2*time.Hour))
	if len(windowLogs) == 0 {
		return nil
	}
	sessions := pairing.PairMulti(windowLogs, &pairing.MultiSegmentConfig{Segments: configs})
	for _, session := range sessions {
		if session.SegmentID == nil {
			continue
		}
		seg, ok := segByID[*session.SegmentID]
		if !ok {
			continue
		}
		if err := upsertPairedTimeEntry(ctx, db, employmentID, seg, session); err != nil {
			return err
		}
	}
	return nil
}

func filterLogsInWindow(logs []pairing.LogEvent, from, to time.Time) []pairing.LogEvent {
	out := make([]pairing.LogEvent, 0, len(logs))
	for _, ev := range logs {
		if !ev.PunchTime.Before(from) && !ev.PunchTime.After(to) {
			out = append(out, ev)
		}
	}
	return out
}

func upsertPairedTimeEntry(ctx context.Context, db DBTX, employmentID uuid.UUID, seg reprocessOfficialSegment, session pairing.PairedSession) error {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM time_entries WHERE employment_id = $1 AND (segment_id = $2 OR punch_in_time = $3))`, employmentID, seg.segmentID, session.StartTime).Scan(&exists); err != nil {
		return err
	}
	if exists {
		return nil
	}
	var duration *int
	if session.EndTime != nil {
		mins := max(0, int(session.EndTime.Sub(session.StartTime)/time.Minute))
		duration = &mins
	}
	meta, err := json.Marshal(map[string]any{"source": "reprocess_pairing", "source_log_ids": session.SourceLogIDs})
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO time_entries (id, employment_id, segment_id, template_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, entry_type, note, is_break_overtime, generation_metadata, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'work', $10, false, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`, uuid.New(), employmentID, seg.segmentID, seg.templateID, seg.versionID, seg.breaks, session.StartTime, session.EndTime, duration, session.Note, meta)
	return err
}

type breakSnapshotRow struct {
	Start          string `json:"start"`
	End            string `json:"end"`
	StartAt        string `json:"start_at"`
	EndAt          string `json:"end_at"`
	Duration       int    `json:"duration_minutes"`
	BreakPunchMode string `json:"break_punch_mode"`
}

func parseBreakSnapshotRow(item breakSnapshotRow) (buckets.BreakPair, int32, bool) {
	startRaw, endRaw := firstNonEmptyString(item.StartAt, item.Start), firstNonEmptyString(item.EndAt, item.End)
	if startRaw == "" || endRaw == "" {
		return buckets.BreakPair{}, 0, false
	}
	start, startErr := time.Parse(time.RFC3339, startRaw)
	end, endErr := time.Parse(time.RFC3339, endRaw)
	if startErr != nil || endErr != nil || !end.After(start) {
		return buckets.BreakPair{}, 0, false
	}
	dur := int32(item.Duration)
	if dur <= 0 {
		dur = int32(end.Sub(start) / time.Minute)
	}
	return buckets.BreakPair{Start: start, End: end}, dur, true
}

func firstNonEmptyString(a, b string) string {
	if a != "" {
		return a
	}
	return b
}

func parseBreakWindowsSnapshot(breaks []byte) ([]buckets.BreakPair, *time.Time, *time.Time, int32, buckets.BreakPunchMode, error) {
	if len(breaks) == 0 {
		return nil, nil, nil, 0, buckets.BreakPunchModeNone, nil
	}
	var parsed []breakSnapshotRow
	if err := json.Unmarshal(breaks, &parsed); err != nil {
		return nil, nil, nil, 0, buckets.BreakPunchModeNone, err
	}
	var windows []buckets.BreakPair
	var firstStart, firstEnd *time.Time
	var scheduledBreak int32
	punchMode := buckets.BreakPunchModeNone
	for _, item := range parsed {
		if item.BreakPunchMode == string(buckets.BreakPunchModeRequired) {
			punchMode = buckets.BreakPunchModeRequired
		}
		pair, dur, ok := parseBreakSnapshotRow(item)
		if !ok {
			continue
		}
		windows = append(windows, pair)
		if firstStart == nil {
			s, e := pair.Start, pair.End
			firstStart, firstEnd = &s, &e
		}
		scheduledBreak += dur
	}
	return windows, firstStart, firstEnd, scheduledBreak, punchMode, nil
}
