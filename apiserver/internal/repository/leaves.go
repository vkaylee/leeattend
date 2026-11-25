package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type LeaveRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

var (
	_ api.LeaveRepository       = LeaveRepository{}
	_ api.LeaveImportRepository = LeaveRepository{}
)

func (r LeaveRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("leave repository: database is not configured")
	}
	return r.DB, nil
}

func (r LeaveRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("leave repository: database does not support transactions")
	}
	return beginner, nil
}

func (r LeaveRepository) CreateLeaveType(ctx context.Context, req api.CreateLeaveTypeRequest) (api.LeaveTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeaveTypeRecord{}, err
	}
	id := uuid.New()
	now := time.Now().UTC()
	isPaid := true
	if req.IsPaid != nil {
		isPaid = *req.IsPaid
	}
	policyConfigJSON := req.PolicyConfig
	if len(policyConfigJSON) == 0 {
		policyConfigJSON = json.RawMessage(`{}`)
	}
	var record api.LeaveTypeRecord
	var rawConfig []byte
	var createdAt time.Time
	err = db.QueryRow(ctx, `INSERT INTO leave_types (id, code, name, is_paid, policy_config, created_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, code, name, is_paid, policy_config, created_at`, id, req.Code, req.Name, isPaid, policyConfigJSON, now).Scan(&record.ID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
	if err != nil {
		if pgErr, ok := err.(*pgconn.PgError); ok && (pgErr.Code == "23505" || pgErr.Code == "23P01") {
			return api.LeaveTypeRecord{}, api.ErrLeaveConflict
		}
		return api.LeaveTypeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.PolicyConfig = json.RawMessage(rawConfig)
	return record, nil
}

func (r LeaveRepository) ListLeaveTypes(ctx context.Context) ([]api.LeaveTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, code, name, is_paid, policy_config, created_at FROM leave_types WHERE deleted_at IS NULL ORDER BY name ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	types := make([]api.LeaveTypeRecord, 0)
	for rows.Next() {
		var item api.LeaveTypeRecord
		var rawConfig []byte
		var createdAt time.Time
		if err := rows.Scan(&item.ID, &item.Code, &item.Name, &item.IsPaid, &rawConfig, &createdAt); err != nil {
			return nil, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		item.PolicyConfig = json.RawMessage(rawConfig)
		types = append(types, item)
	}
	return types, rows.Err()
}

func (r LeaveRepository) UpdateLeaveTypeConfig(ctx context.Context, leaveTypeID uuid.UUID, policyConfig json.RawMessage) (api.LeaveTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeaveTypeRecord{}, err
	}
	var record api.LeaveTypeRecord
	var rawConfig []byte
	var createdAt time.Time
	err = db.QueryRow(ctx, `UPDATE leave_types SET policy_config = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING id, code, name, is_paid, policy_config, created_at`, policyConfig, leaveTypeID).
		Scan(&record.ID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.LeaveTypeRecord{}, api.ErrLeaveTypeNotFound
		}
		return api.LeaveTypeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.PolicyConfig = json.RawMessage(rawConfig)
	return record, nil
}

func (r LeaveRepository) DeleteLeaveType(ctx context.Context, leaveTypeID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	var inUse bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM leave_balances WHERE leave_type_id = $1)`, leaveTypeID).Scan(&inUse)
	if err != nil {
		return err
	}
	if inUse {
		return api.ErrLeaveConflict
	}
	tag, err := db.Exec(ctx, `UPDATE leave_types SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1`, leaveTypeID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrLeaveTypeNotFound
	}
	return nil
}

func (r LeaveRepository) RestoreLeaveType(ctx context.Context, leaveTypeID uuid.UUID) (api.LeaveTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeaveTypeRecord{}, err
	}
	var record api.LeaveTypeRecord
	var rawConfig []byte
	var createdAt time.Time
	err = db.QueryRow(ctx, `UPDATE leave_types SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NOT NULL RETURNING id, code, name, is_paid, policy_config, created_at`, leaveTypeID).
		Scan(&record.ID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.LeaveTypeRecord{}, api.ErrLeaveTypeNotFound
		}
		return api.LeaveTypeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.PolicyConfig = json.RawMessage(rawConfig)
	return record, nil
}

func (r LeaveRepository) SubmitLeaveRequest(ctx context.Context, requesterID uuid.UUID, req api.SubmitLeaveRequest, autoApprove bool) (api.LeaveRequestRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.LeaveRequestRecord{}, err
	}
	now := time.Now().UTC()
	var record api.LeaveRequestRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, req.EmploymentID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrLeaveNoEmployment
		}
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM leave_types WHERE id = $1 AND deleted_at IS NULL)`, req.LeaveTypeID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrLeaveTypeNotFound
		}
		status := "pending"
		var approvedBy *uuid.UUID
		if autoApprove {
			status = "pre_approved"
			approvedBy = &requesterID
		}
		duration := req.DurationType
		if duration == "" {
			duration = "full_day"
		}
		reqID := uuid.New()
		var createdAt, updatedAt time.Time
		err := db.QueryRow(ctx, `INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, reason, status, duration_type, start_time, end_time, requested_by_user_id, approved_by_user_id, created_at, updated_at, attachment_file_ids) VALUES ($1, $2, $3, $4::date, $5::date, $6, $7::leave_status_type, $8, $9, $10, $11, $12, $13, $13, $14) RETURNING id, employment_id, leave_type_id, start_date::text, end_date::text, reason, status::text, duration_type, start_time::text, end_time::text, requested_by_user_id, approved_by_user_id, created_at, updated_at, attachment_file_ids`, reqID, req.EmploymentID, req.LeaveTypeID, req.StartDate, req.EndDate, req.Reason, status, duration, req.StartTime, req.EndTime, requesterID, approvedBy, now, req.AttachmentFileIDs).Scan(
			&record.ID, &record.EmploymentID, &record.LeaveTypeID, &record.StartDate, &record.EndDate, &record.Reason, &record.Status, &record.DurationType, &record.StartTime, &record.EndTime, &record.RequestedByUserID, &record.ApprovedByUserID, &createdAt, &updatedAt, &record.AttachmentFileIDs,
		)
		if err != nil {
			if pgErr, ok := err.(*pgconn.PgError); ok && (pgErr.Code == "23P01" || pgErr.Code == "23505") {
				return api.ErrLeaveConflict
			}
			return err
		}
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.LeaveRequestRecord{}, err
	}
	return record, nil
}

func (r LeaveRepository) ListOrganizationLeaveRequests(ctx context.Context, status *string) ([]api.LeaveRequestRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT lr.id, lr.employment_id, lr.leave_type_id, lr.start_date::text, lr.end_date::text, lr.reason, lr.status::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.requested_by_user_id, lr.approved_by_user_id, lr.created_at, lr.updated_at, lr.attachment_file_ids FROM leave_requests lr JOIN employments e ON e.id = lr.employment_id WHERE e.deleted_at IS NULL AND ($1::text IS NULL OR lr.status::text = $1) ORDER BY lr.created_at DESC`, status)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	records := make([]api.LeaveRequestRecord, 0)
	for rows.Next() {
		var rec api.LeaveRequestRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&rec.ID, &rec.EmploymentID, &rec.LeaveTypeID, &rec.StartDate, &rec.EndDate, &rec.Reason, &rec.Status, &rec.DurationType, &rec.StartTime, &rec.EndTime, &rec.RequestedByUserID, &rec.ApprovedByUserID, &createdAt, &updatedAt, &rec.AttachmentFileIDs); err != nil {
			return nil, err
		}
		rec.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		rec.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		records = append(records, rec)
	}
	return records, rows.Err()
}

func (r LeaveRepository) ApproveLeaveRequest(ctx context.Context, approverID, requestID uuid.UUID, status string) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var currentStatus string
		err := db.QueryRow(ctx, `SELECT status::text FROM leave_requests WHERE id = $1 FOR UPDATE`, requestID).Scan(&currentStatus)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNotFound
		}
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `UPDATE leave_requests SET status = $1::leave_status_type, approved_by_user_id = $2, updated_at = now() WHERE id = $3`, status, approverID, requestID)
		return err
	})
}

func (r LeaveRepository) GetEmployeeLeaveBalances(ctx context.Context, employeeID uuid.UUID, year int) ([]api.EmployeeLeaveBalanceRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT lt.id, lt.code, lt.name, lt.is_paid, COALESCE(lb.entitlement, 12), COALESCE(lb.accrued_amount::text, '0.00'), COALESCE(lb.carried_over, 0), COALESCE((
		SELECT ROUND(COALESCE(SUM(
			(SELECT COALESCE(SUM(
				CASE
					WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type <> 'half_day') THEN 0::numeric
					WHEN lr.duration_type IN ('first_half', 'second_half') THEN 0.5::numeric
					WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type = 'half_day') THEN 0.5::numeric
					ELSE 1::numeric
				END
			), 0)
			FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
			JOIN schedule_day_revisions sdr
			  ON sdr.employment_id = lr.employment_id
			 AND sdr.business_date = d::date
			 AND sdr.state = 'published')
		), 0))::bigint
		FROM leave_requests lr
		JOIN employments e ON e.id = lr.employment_id
		WHERE e.employee_id = $1 AND e.deleted_at IS NULL AND lr.leave_type_id = lt.id AND lr.status IN ('approved', 'pre_approved') AND EXTRACT(YEAR FROM lr.start_date) = $2
	), 0) AS used_count FROM leave_types lt LEFT JOIN LATERAL (SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1) active_emp ON true LEFT JOIN leave_balances lb ON lb.leave_type_id = lt.id AND lb.employment_id = active_emp.id AND lb.year = $2 WHERE lt.deleted_at IS NULL ORDER BY lt.name ASC`, employeeID, year)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	balances := make([]api.EmployeeLeaveBalanceRecord, 0)
	for rows.Next() {
		var b api.EmployeeLeaveBalanceRecord
		var accruedStr string
		var usedCount int64
		if err := rows.Scan(&b.LeaveTypeID, &b.LeaveTypeCode, &b.LeaveTypeName, &b.IsPaid, &b.Entitlement, &accruedStr, &b.CarriedOver, &usedCount); err != nil {
			return nil, err
		}
		accrued, _ := decimal.NewFromString(accruedStr)
		b.AccruedAmount = accrued
		b.Used = int(usedCount)
		total := decimal.NewFromInt(int64(b.Entitlement + b.CarriedOver)).Add(b.AccruedAmount)
		b.Remaining = total.Sub(decimal.NewFromInt(int64(b.Used)))
		balances = append(balances, b)
	}
	return balances, rows.Err()
}

func (r LeaveRepository) ResolveEmployeeForUser(ctx context.Context, userID uuid.UUID) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT t.employee_id FROM employee_tenures t JOIN employees e ON e.id = t.employee_id WHERE t.user_id = $1 AND t.deleted_at IS NULL AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE) AND e.deleted_at IS NULL ORDER BY t.start_date DESC, t.id DESC LIMIT 1`, userID).Scan(&empID)
	return empID, err
}

type leaveUsageRequest struct {
	EmploymentID uuid.UUID
	StartDate    string
	EndDate      string
	DurationType string
	StartTime    *string
	EndTime      *string
}

type leaveScheduleBreak struct {
	StartAt         *time.Time
	EndAt           *time.Time
	DurationMinutes int
}

type leaveScheduleSegment struct {
	StartAt time.Time
	EndAt   time.Time
	Breaks  []leaveScheduleBreak
}

type leaveScheduleDay struct {
	State    string
	Timezone string
	Segments []leaveScheduleSegment
}

type leaveHoliday struct {
	HolidayType string
	IsPaid      bool
}

type leaveTimeWindow struct {
	Start time.Time
	End   time.Time
}

func mergeLeaveWindows(windows []leaveTimeWindow) []leaveTimeWindow {
	valid := make([]leaveTimeWindow, 0, len(windows))
	for _, w := range windows {
		if w.End.After(w.Start) {
			valid = append(valid, w)
		}
	}
	if len(valid) <= 1 {
		return valid
	}
	sort.Slice(valid, func(i, j int) bool {
		if valid[i].Start.Equal(valid[j].Start) {
			return valid[i].End.Before(valid[j].End)
		}
		return valid[i].Start.Before(valid[j].Start)
	})
	merged := []leaveTimeWindow{valid[0]}
	for _, w := range valid[1:] {
		last := &merged[len(merged)-1]
		if !w.Start.After(last.End) {
			if w.End.After(last.End) {
				last.End = w.End
			}
		} else {
			merged = append(merged, w)
		}
	}
	return merged
}

func clampLeaveWindows(base leaveTimeWindow, cuts []leaveTimeWindow) []leaveTimeWindow {
	clamped := make([]leaveTimeWindow, 0, len(cuts))
	for _, c := range cuts {
		s, e := c.Start, c.End
		if s.Before(base.Start) {
			s = base.Start
		}
		if e.After(base.End) {
			e = base.End
		}
		if e.After(s) {
			clamped = append(clamped, leaveTimeWindow{Start: s, End: e})
		}
	}
	return clamped
}

func subtractLeaveWindows(base leaveTimeWindow, cuts []leaveTimeWindow) []leaveTimeWindow {
	if !base.End.After(base.Start) {
		return nil
	}
	mergedCuts := mergeLeaveWindows(clampLeaveWindows(base, cuts))
	if len(mergedCuts) == 0 {
		return []leaveTimeWindow{base}
	}
	out := make([]leaveTimeWindow, 0, len(mergedCuts)+1)
	cursor := base.Start
	for _, c := range mergedCuts {
		if c.Start.After(cursor) {
			out = append(out, leaveTimeWindow{Start: cursor, End: c.Start})
		}
		if c.End.After(cursor) {
			cursor = c.End
		}
	}
	if base.End.After(cursor) {
		out = append(out, leaveTimeWindow{Start: cursor, End: base.End})
	}
	return out
}

func overlapLeaveWindowSeconds(target leaveTimeWindow, windows []leaveTimeWindow) int64 {
	if !target.End.After(target.Start) {
		return 0
	}
	var secs int64
	for _, w := range windows {
		s := target.Start
		if w.Start.After(s) {
			s = w.Start
		}
		e := target.End
		if w.End.Before(e) {
			e = w.End
		}
		if e.After(s) {
			secs += int64(e.Sub(s).Seconds())
		}
	}
	return secs
}

func parseLeaveClock(raw string) (hour, minute, second int, ok bool) {
	trimmed := strings.TrimSpace(raw)
	for _, layout := range []string{"15:04:05", "15:04"} {
		if parsed, err := time.Parse(layout, trimmed); err == nil {
			return parsed.Hour(), parsed.Minute(), parsed.Second(), true
		}
	}
	return 0, 0, 0, false
}

type rawLeaveBreakSnapshot struct {
	StartAt         *time.Time `json:"start_at"`
	EndAt           *time.Time `json:"end_at"`
	Start           string     `json:"start"`
	End             string     `json:"end"`
	DurationMinutes int        `json:"duration_minutes"`
}

func parseLocalLeaveBreak(startStr, endStr string, segStart time.Time, loc *time.Location) (leaveScheduleBreak, bool) {
	sh, sm, ss, okS := parseLeaveClock(startStr)
	eh, em, es, okE := parseLeaveClock(endStr)
	if !okS || !okE {
		return leaveScheduleBreak{}, false
	}
	base := segStart.In(loc)
	bs := time.Date(base.Year(), base.Month(), base.Day(), sh, sm, ss, 0, loc)
	if bs.Before(base) {
		bs = bs.AddDate(0, 0, 1)
	}
	be := time.Date(bs.Year(), bs.Month(), bs.Day(), eh, em, es, 0, loc)
	if !be.After(bs) {
		be = be.AddDate(0, 0, 1)
	}
	return leaveScheduleBreak{StartAt: &bs, EndAt: &be}, true
}

func parseLeaveBreakItem(item rawLeaveBreakSnapshot, segStart time.Time, loc *time.Location) (leaveScheduleBreak, bool) {
	if item.StartAt != nil && item.EndAt != nil && item.EndAt.After(*item.StartAt) {
		s, e := *item.StartAt, *item.EndAt
		return leaveScheduleBreak{StartAt: &s, EndAt: &e}, true
	}
	if item.Start != "" && item.End != "" {
		return parseLocalLeaveBreak(item.Start, item.End, segStart, loc)
	}
	if item.DurationMinutes > 0 {
		return leaveScheduleBreak{DurationMinutes: item.DurationMinutes}, true
	}
	return leaveScheduleBreak{}, false
}

func parseLeaveBreaksSnapshot(raw []byte, segStart time.Time, loc *time.Location) []leaveScheduleBreak {
	if len(raw) == 0 {
		return nil
	}
	var items []rawLeaveBreakSnapshot
	if err := json.Unmarshal(raw, &items); err != nil {
		return nil
	}
	out := make([]leaveScheduleBreak, 0, len(items))
	for _, item := range items {
		if parsed, ok := parseLeaveBreakItem(item, segStart, loc); ok {
			out = append(out, parsed)
		}
	}
	return out
}

type leaveSegmentEligible struct {
	windows []leaveTimeWindow
	netSecs int64
}

func segmentEligibleWork(seg leaveScheduleSegment) (leaveSegmentEligible, bool) {
	if !seg.EndAt.After(seg.StartAt) {
		return leaveSegmentEligible{}, false
	}
	fixedBreaks := make([]leaveTimeWindow, 0, len(seg.Breaks))
	var flexBreakSecs int64
	for _, b := range seg.Breaks {
		if b.StartAt != nil && b.EndAt != nil && b.EndAt.After(*b.StartAt) {
			fixedBreaks = append(fixedBreaks, leaveTimeWindow{Start: *b.StartAt, End: *b.EndAt})
		} else if b.DurationMinutes > 0 {
			flexBreakSecs += int64(b.DurationMinutes) * 60
		}
	}
	workWindows := subtractLeaveWindows(leaveTimeWindow{Start: seg.StartAt, End: seg.EndAt}, fixedBreaks)
	var segWorkSecs int64
	for _, w := range workWindows {
		segWorkSecs += int64(w.End.Sub(w.Start).Seconds())
	}
	segNet := segWorkSecs - min(flexBreakSecs, segWorkSecs)
	if segNet <= 0 {
		return leaveSegmentEligible{}, false
	}
	return leaveSegmentEligible{windows: workWindows, netSecs: segNet}, true
}

func buildLeaveDayEligible(segments []leaveScheduleSegment) ([]leaveSegmentEligible, int64) {
	eligibles := make([]leaveSegmentEligible, 0, len(segments))
	var dayNetSecs int64
	for _, seg := range segments {
		if item, ok := segmentEligibleWork(seg); ok {
			eligibles = append(eligibles, item)
			dayNetSecs += item.netSecs
		}
	}
	return eligibles, dayNetSecs
}

func loadLocationOrUTC(tz string) *time.Location {
	if tz != "" {
		if loaded, err := time.LoadLocation(tz); err == nil {
			return loaded
		}
	}
	return time.UTC
}

func coveredLeaveSeconds(eligibles []leaveSegmentEligible, win leaveTimeWindow) int64 {
	var covered int64
	for _, seg := range eligibles {
		covered += min(overlapLeaveWindowSeconds(win, seg.windows), seg.netSecs)
	}
	return covered
}

func parseCustomLeaveWindow(d time.Time, startTime, endTime *string, tz string) (leaveTimeWindow, bool, bool) {
	if startTime == nil || endTime == nil {
		return leaveTimeWindow{}, false, false
	}
	sh, sm, ss, okS := parseLeaveClock(*startTime)
	eh, em, es, okE := parseLeaveClock(*endTime)
	if !okS || !okE {
		return leaveTimeWindow{}, false, false
	}
	loc := loadLocationOrUTC(tz)
	leaveStart := time.Date(d.Year(), d.Month(), d.Day(), sh, sm, ss, 0, loc)
	leaveEnd := time.Date(d.Year(), d.Month(), d.Day(), eh, em, es, 0, loc)
	sameLocalDay := leaveEnd.After(leaveStart)
	if !sameLocalDay {
		leaveEnd = leaveEnd.AddDate(0, 0, 1)
	}
	return leaveTimeWindow{Start: leaveStart, End: leaveEnd}, sameLocalDay, true
}

func calculateCustomHoursFraction(d time.Time, req leaveUsageRequest, tz string, eligibles []leaveSegmentEligible, dayNetSecs int64, isHalfHoliday bool) decimal.Decimal {
	win, sameLocalDay, ok := parseCustomLeaveWindow(d, req.StartTime, req.EndTime, tz)
	if !ok {
		return decimal.Zero
	}
	coveredSecs := coveredLeaveSeconds(eligibles, win)
	if coveredSecs == 0 && sameLocalDay {
		coveredSecs = coveredLeaveSeconds(eligibles, leaveTimeWindow{
			Start: win.Start.AddDate(0, 0, 1),
			End:   win.End.AddDate(0, 0, 1),
		})
	}
	if coveredSecs <= 0 {
		return decimal.Zero
	}
	fraction := decimal.NewFromInt(min(coveredSecs, dayNetSecs)).DivRound(decimal.NewFromInt(dayNetSecs), 4)
	half := decimal.NewFromFloat(0.5)
	if isHalfHoliday && fraction.GreaterThan(half) {
		return half
	}
	return fraction
}

func fullDayLeaveFraction(isHalfHoliday bool) decimal.Decimal {
	if isHalfHoliday {
		return decimal.NewFromFloat(0.5)
	}
	return decimal.NewFromInt(1)
}

func calculateLeaveDayFraction(d time.Time, req leaveUsageRequest, durationType string, day leaveScheduleDay, hol leaveHoliday, hasHol bool) decimal.Decimal {
	if day.State != "published" || (hasHol && hol.HolidayType != "half_day") {
		return decimal.Zero
	}
	eligibles, dayNetSecs := buildLeaveDayEligible(day.Segments)
	if dayNetSecs <= 0 {
		return decimal.Zero
	}
	isHalfHoliday := hasHol && hol.HolidayType == "half_day"
	switch durationType {
	case "full_day":
		return fullDayLeaveFraction(isHalfHoliday)
	case "first_half", "second_half":
		return decimal.NewFromFloat(0.5)
	case "custom_hours":
		return calculateCustomHoursFraction(d, req, day.Timezone, eligibles, dayNetSecs, isHalfHoliday)
	default:
		return decimal.Zero
	}
}

func calculateLeaveDaysForRequest(req leaveUsageRequest, scheduleByDate map[string]leaveScheduleDay, holidays map[string]leaveHoliday) decimal.Decimal {
	start, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return decimal.Zero
	}
	end, err := time.Parse("2006-01-02", req.EndDate)
	if err != nil || end.Before(start) {
		return decimal.Zero
	}
	durationType := req.DurationType
	if durationType == "" {
		durationType = "full_day"
	}
	total := decimal.Zero
	for d := start; !d.After(end); d = d.AddDate(0, 0, 1) {
		dateKey := d.Format("2006-01-02")
		day, ok := scheduleByDate[dateKey]
		if !ok {
			continue
		}
		hol, hasHol := holidays[dateKey]
		total = total.Add(calculateLeaveDayFraction(d, req, durationType, day, hol, hasHol))
	}
	return total
}

func loadLeaveHolidays(ctx context.Context, db DBTX, minDate, maxDate string) (map[string]leaveHoliday, error) {
	holidays := make(map[string]leaveHoliday)
	holRows, err := db.Query(ctx, `SELECT date::text, holiday_type, is_paid FROM holidays WHERE date BETWEEN $1::date AND $2::date`, minDate, maxDate)
	if err != nil {
		return nil, err
	}
	defer holRows.Close()
	for holRows.Next() {
		var dateStr, holidayType string
		var isPaid bool
		if err := holRows.Scan(&dateStr, &holidayType, &isPaid); err != nil {
			return nil, err
		}
		holidays[dateStr] = leaveHoliday{HolidayType: holidayType, IsPaid: isPaid}
	}
	return holidays, holRows.Err()
}

func appendLeaveScheduleRow(schedules map[uuid.UUID]map[string]leaveScheduleDay, empID uuid.UUID, dateStr, state, tz string, startAt, endAt *time.Time, breaksRaw []byte) {
	byDate, ok := schedules[empID]
	if !ok {
		byDate = make(map[string]leaveScheduleDay)
		schedules[empID] = byDate
	}
	day := byDate[dateStr]
	day.State = state
	day.Timezone = tz
	if startAt != nil && endAt != nil && endAt.After(*startAt) {
		day.Segments = append(day.Segments, leaveScheduleSegment{
			StartAt: *startAt,
			EndAt:   *endAt,
			Breaks:  parseLeaveBreaksSnapshot(breaksRaw, *startAt, loadLocationOrUTC(tz)),
		})
	}
	byDate[dateStr] = day
}

func loadLeaveScheduleContext(ctx context.Context, db DBTX, employmentIDs []uuid.UUID, minDate, maxDate string) (map[uuid.UUID]map[string]leaveScheduleDay, map[string]leaveHoliday, error) {
	schedules := make(map[uuid.UUID]map[string]leaveScheduleDay, len(employmentIDs))
	if len(employmentIDs) == 0 || minDate == "" || maxDate == "" {
		return schedules, map[string]leaveHoliday{}, nil
	}
	rows, err := db.Query(ctx, `
		SELECT sdr.employment_id, sdr.business_date::text, sdr.state, sdr.timezone, ws.scheduled_start_at, ws.scheduled_end_at, ws.breaks_snapshot
		FROM schedule_day_revisions sdr
		LEFT JOIN work_segments ws ON ws.schedule_revision_id = sdr.id
		WHERE sdr.employment_id = ANY($1)
		  AND sdr.business_date BETWEEN $2::date AND $3::date
		  AND sdr.state IN ('published', 'explicit_off')
		ORDER BY sdr.employment_id, sdr.business_date, ws.sort_order, ws.id
	`, employmentIDs, minDate, maxDate)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var empID uuid.UUID
		var dateStr, state, tz string
		var startAt, endAt *time.Time
		var breaksRaw []byte
		if err := rows.Scan(&empID, &dateStr, &state, &tz, &startAt, &endAt, &breaksRaw); err != nil {
			return nil, nil, err
		}
		appendLeaveScheduleRow(schedules, empID, dateStr, state, tz, startAt, endAt, breaksRaw)
	}
	if err := rows.Err(); err != nil {
		return nil, nil, err
	}
	holidays, err := loadLeaveHolidays(ctx, db, minDate, maxDate)
	return schedules, holidays, err
}

func (r LeaveRepository) GetOrganizationLeaveBalances(ctx context.Context, year, page, limit int, q string) ([]api.OrganizationLeaveBalance, int64, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, 0, err
	}
	if limit <= 0 {
		limit = 20
	}
	if page <= 0 {
		page = 1
	}
	offset := (page - 1) * limit
	searchTerm := strings.TrimSpace(q)

	countSQL := `
		SELECT COUNT(*)
		FROM employees e
		CROSS JOIN leave_types lt
		LEFT JOIN employments emp ON e.id = emp.employee_id AND emp.deleted_at IS NULL AND emp.status = 'active'
		WHERE e.deleted_at IS NULL AND emp.id IS NOT NULL AND lt.deleted_at IS NULL
		  AND ($1 = '' OR e.full_name ILIKE '%' || $1 || '%' OR EXISTS (SELECT 1 FROM employee_tenures search_t WHERE search_t.employee_id = e.id AND search_t.deleted_at IS NULL AND search_t.employee_number ILIKE '%' || $1 || '%') OR lt.name ILIKE '%' || $1 || '%' OR lt.code ILIKE '%' || $1 || '%')
	`
	var total int64
	if err := db.QueryRow(ctx, countSQL, searchTerm).Scan(&total); err != nil {
		return nil, 0, err
	}

	dataSQL := `
		WITH leave_usage AS (
			SELECT
				emp_usage.employee_id,
				lr.leave_type_id,
				COALESCE(ROUND(SUM(
					(SELECT COALESCE(SUM(
						CASE
							WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type <> 'half_day') THEN 0::numeric
							WHEN lr.duration_type IN ('first_half', 'second_half') THEN 0.5::numeric
							WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type = 'half_day') THEN 0.5::numeric
							ELSE 1::numeric
						END
					), 0)
					FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
					JOIN schedule_day_revisions sdr
					  ON sdr.employment_id = lr.employment_id
					 AND sdr.business_date = d::date
					 AND sdr.state = 'published')
				)), 0)::INT as used_days
			FROM leave_requests lr
			JOIN employments emp_usage ON lr.employment_id = emp_usage.id
			WHERE lr.status IN ('approved', 'pre_approved')
			  AND EXTRACT(YEAR FROM lr.start_date)::INT = $1
			GROUP BY emp_usage.employee_id, lr.leave_type_id
		),
		best_policy AS (
			SELECT DISTINCT ON (lt_id, emp_id)
				d.leave_type_id as lt_id,
				e_inner.id as emp_id,
				d.name as policy_name,
				a.assignee_type as assignment_type
			FROM leave_policy_assignments a
			JOIN leave_policy_definitions d ON a.policy_id = d.id
			CROSS JOIN employees e_inner
			WHERE e_inner.deleted_at IS NULL
			  AND (
				(a.assignee_type = 'user' AND a.assignee_id = e_inner.user_id) OR
				(a.assignee_type = 'employee' AND a.assignee_id = e_inner.id) OR
				(a.assignee_type = 'department' AND a.assignee_id = (
					SELECT department_id FROM employments
					WHERE employee_id = e_inner.id AND deleted_at IS NULL
					ORDER BY start_date DESC, id DESC LIMIT 1
				)) OR
				(a.assignee_type = 'organization')
			  )
			  AND a.effective_from <= NOW()
			  AND (a.effective_to IS NULL OR a.effective_to > NOW())
			  AND a.deleted_at IS NULL
			  AND d.deleted_at IS NULL
			ORDER BY lt_id, emp_id,
			  CASE a.assignee_type
				WHEN 'user' THEN 1
				WHEN 'employee' THEN 2
				WHEN 'department' THEN 3
				WHEN 'organization' THEN 4
			  END ASC
		)
		SELECT
			e.id as employee_id,
			e.full_name as employee_name,
			lt.id as leave_type_id,
			lt.code as leave_type_code,
			lt.name as leave_type_name,
			lt.is_paid as is_paid,
			COALESCE(lb.entitlement, 12) as entitlement,
			COALESCE(lb.accrued_amount::text, '0.00') as accrued_amount,
			COALESCE(lb.carried_over, 0) as carried_over,
			COALESCE(lu.used_days, 0) as used,
			bp.policy_name,
			bp.assignment_type
		FROM employees e
		CROSS JOIN leave_types lt
		LEFT JOIN employments emp ON e.id = emp.employee_id AND emp.deleted_at IS NULL AND emp.status = 'active'
		LEFT JOIN leave_balances lb ON emp.id = lb.employment_id AND lt.id = lb.leave_type_id AND lb.year = $1
		LEFT JOIN leave_usage lu ON e.id = lu.employee_id AND lt.id = lu.leave_type_id
		LEFT JOIN best_policy bp ON e.id = bp.emp_id AND lt.id = bp.lt_id
		WHERE e.deleted_at IS NULL AND emp.id IS NOT NULL AND lt.deleted_at IS NULL
		  AND ($2 = '' OR e.full_name ILIKE '%' || $2 || '%' OR EXISTS (SELECT 1 FROM employee_tenures search_t WHERE search_t.employee_id = e.id AND search_t.deleted_at IS NULL AND search_t.employee_number ILIKE '%' || $2 || '%') OR lt.name ILIKE '%' || $2 || '%' OR lt.code ILIKE '%' || $2 || '%')
		ORDER BY e.full_name ASC, lt.name ASC
		LIMIT $3 OFFSET $4
	`
	rows, err := db.Query(ctx, dataSQL, year, searchTerm, limit, offset)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	balances := make([]api.OrganizationLeaveBalance, 0)
	for rows.Next() {
		var b api.OrganizationLeaveBalance
		var accruedStr string
		if err := rows.Scan(&b.EmployeeID, &b.EmployeeName, &b.LeaveTypeID, &b.LeaveTypeCode, &b.LeaveTypeName, &b.IsPaid, &b.Entitlement, &accruedStr, &b.CarriedOver, &b.Used, &b.PolicyName, &b.AssignmentType); err != nil {
			return nil, 0, err
		}
		b.AccruedAmount, _ = decimal.NewFromString(accruedStr)
		totalAvailable := decimal.NewFromInt(int64(b.Entitlement + b.CarriedOver)).Add(b.AccruedAmount)
		b.Remaining = totalAvailable.Sub(decimal.NewFromInt(int64(b.Used)))
		balances = append(balances, b)
	}
	return balances, total, rows.Err()
}

func (r LeaveRepository) GetEmployeeLeaveHistory(ctx context.Context, employeeID uuid.UUID, year int, leaveTypeID *uuid.UUID) ([]api.LeaveHistoryItem, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	logRows, err := db.Query(ctx, `
		SELECT lal.id, lal.accrual_date::text, lal.amount::text, lal.notes, lal.policy_assignment_id, lal.created_at
		FROM leave_accrual_logs lal
		JOIN employments emp ON lal.employment_id = emp.id
		WHERE emp.employee_id = $1 AND emp.deleted_at IS NULL
		  AND EXTRACT(YEAR FROM lal.accrual_date)::INT = $2
		  AND ($3::uuid IS NULL OR lal.leave_type_id = $3)
	`, employeeID, year, leaveTypeID)
	if err != nil {
		return nil, err
	}
	defer logRows.Close()

	items := make([]api.LeaveHistoryItem, 0)
	for logRows.Next() {
		var id uuid.UUID
		var dateStr, amountStr string
		var notes *string
		var assignmentID *uuid.UUID
		var createdAt time.Time
		if err := logRows.Scan(&id, &dateStr, &amountStr, &notes, &assignmentID, &createdAt); err != nil {
			return nil, err
		}
		amt, _ := decimal.NewFromString(amountStr)
		txType := "adjustment"
		if assignmentID != nil {
			txType = "accrual"
		}
		items = append(items, api.LeaveHistoryItem{
			ID:              id,
			Date:            dateStr,
			TransactionType: txType,
			Amount:          amt,
			Reason:          notes,
			ReferenceID:     nil,
			CreatedAt:       createdAt.UTC().Format(time.RFC3339),
		})
	}
	if err := logRows.Err(); err != nil {
		return nil, err
	}

	reqRows, err := db.Query(ctx, `
		SELECT lr.id, lr.employment_id, lr.start_date::text, lr.end_date::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.reason, lr.created_at
		FROM leave_requests lr
		JOIN employments emp ON lr.employment_id = emp.id
		WHERE emp.employee_id = $1 AND emp.deleted_at IS NULL
		  AND lr.status IN ('approved', 'pre_approved')
		  AND EXTRACT(YEAR FROM lr.start_date)::INT = $2
		  AND ($3::uuid IS NULL OR lr.leave_type_id = $3)
	`, employeeID, year, leaveTypeID)
	if err != nil {
		return nil, err
	}
	defer reqRows.Close()

	type fetchedLeaveReq struct {
		id        uuid.UUID
		usageReq  leaveUsageRequest
		reason    *string
		createdAt time.Time
	}
	fetched := make([]fetchedLeaveReq, 0)
	empSet := make(map[uuid.UUID]struct{})
	minDate, maxDate := "", ""
	for reqRows.Next() {
		var item fetchedLeaveReq
		if err := reqRows.Scan(
			&item.id,
			&item.usageReq.EmploymentID,
			&item.usageReq.StartDate,
			&item.usageReq.EndDate,
			&item.usageReq.DurationType,
			&item.usageReq.StartTime,
			&item.usageReq.EndTime,
			&item.reason,
			&item.createdAt,
		); err != nil {
			return nil, err
		}
		fetched = append(fetched, item)
		empSet[item.usageReq.EmploymentID] = struct{}{}
		if minDate == "" || item.usageReq.StartDate < minDate {
			minDate = item.usageReq.StartDate
		}
		if maxDate == "" || item.usageReq.EndDate > maxDate {
			maxDate = item.usageReq.EndDate
		}
	}
	if err := reqRows.Err(); err != nil {
		return nil, err
	}

	empIDs := make([]uuid.UUID, 0, len(empSet))
	for id := range empSet {
		empIDs = append(empIDs, id)
	}
	schedulesByEmp, holidaysByDate, err := loadLeaveScheduleContext(ctx, db, empIDs, minDate, maxDate)
	if err != nil {
		return nil, err
	}
	for _, item := range fetched {
		usedDays := calculateLeaveDaysForRequest(item.usageReq, schedulesByEmp[item.usageReq.EmploymentID], holidaysByDate)
		reqID := item.id
		items = append(items, api.LeaveHistoryItem{
			ID:              uuid.New(),
			Date:            item.usageReq.StartDate,
			TransactionType: "usage",
			Amount:          usedDays.Neg(),
			Reason:          item.reason,
			ReferenceID:     &reqID,
			CreatedAt:       item.createdAt.UTC().Format(time.RFC3339),
		})
	}

	sort.Slice(items, func(i, j int) bool {
		if items[i].Date != items[j].Date {
			return items[i].Date > items[j].Date
		}
		return items[i].CreatedAt > items[j].CreatedAt
	})
	return items, nil
}

func (r LeaveRepository) AdjustLeaveBalance(ctx context.Context, employeeID uuid.UUID, req api.AdjustBalanceRequest) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var employmentID uuid.UUID
		err := db.QueryRow(ctx, `
			SELECT id FROM employments
			WHERE employee_id = $1 AND deleted_at IS NULL
			ORDER BY start_date DESC LIMIT 1
		`, employeeID).Scan(&employmentID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNoEmployment
		}
		if err != nil {
			return err
		}
		parsedDate, err := time.Parse("2006-01-02", req.TransactionDate)
		if err != nil {
			return api.ErrLeaveValidation
		}
		year := parsedDate.Year()
		_, err = db.Exec(ctx, `
			INSERT INTO leave_accrual_logs (id, employment_id, accrual_date, amount, leave_type_id, notes, created_at)
			VALUES (gen_random_uuid(), $1, $2::date, $3, $4, $5, NOW())
		`, employmentID, req.TransactionDate, req.Amount, req.LeaveTypeID, req.Reason)
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `
			INSERT INTO leave_balances (id, employment_id, leave_type_id, year, entitlement, accrued_amount, carried_over, created_at, updated_at)
			VALUES (gen_random_uuid(), $1, $2, $3, 12, $4, 0, NOW(), NOW())
			ON CONFLICT (employment_id, leave_type_id, year)
			DO UPDATE SET accrued_amount = leave_balances.accrued_amount + $4, updated_at = NOW()
		`, employmentID, req.LeaveTypeID, year, req.Amount)
		return err
	})
}

func (r LeaveRepository) ListEmployeeLeaveRequests(ctx context.Context, employeeID uuid.UUID) ([]api.LeaveRequestRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT lr.id, lr.employment_id, lr.leave_type_id, lr.start_date::text, lr.end_date::text, lr.reason, lr.status::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.requested_by_user_id, lr.approved_by_user_id, lr.created_at, lr.updated_at, lr.attachment_file_ids
		FROM leave_requests lr
		JOIN employments e ON e.id = lr.employment_id
		JOIN employment_relationships er ON er.id = e.relationship_id AND er.employee_id = e.employee_id
		JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = er.employee_id
		WHERE e.employee_id = $1 AND e.deleted_at IS NULL AND t.deleted_at IS NULL
		  AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE)
		ORDER BY lr.created_at DESC
	`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	records := make([]api.LeaveRequestRecord, 0)
	for rows.Next() {
		var rec api.LeaveRequestRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&rec.ID, &rec.EmploymentID, &rec.LeaveTypeID, &rec.StartDate, &rec.EndDate, &rec.Reason, &rec.Status, &rec.DurationType, &rec.StartTime, &rec.EndTime, &rec.RequestedByUserID, &rec.ApprovedByUserID, &createdAt, &updatedAt, &rec.AttachmentFileIDs); err != nil {
			return nil, err
		}
		rec.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		rec.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		records = append(records, rec)
	}
	return records, rows.Err()
}

func (r LeaveRepository) ListLeavePolicies(ctx context.Context) ([]api.LeavePolicyDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT id, name, description, leave_type_id, rule_config, created_at, updated_at
		FROM leave_policy_definitions
		WHERE deleted_at IS NULL
		ORDER BY created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	policies := make([]api.LeavePolicyDefinition, 0)
	for rows.Next() {
		var p api.LeavePolicyDefinition
		var rawConfig []byte
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&p.ID, &p.Name, &p.Description, &p.LeaveTypeID, &rawConfig, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		_ = json.Unmarshal(rawConfig, &p.RuleConfig)
		p.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		p.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		policies = append(policies, p)
	}
	return policies, rows.Err()
}

func (r LeaveRepository) CreateLeavePolicy(ctx context.Context, req api.CreateLeavePolicyRequest) (api.LeavePolicyDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeavePolicyDefinition{}, err
	}
	rawConfig, err := json.Marshal(req.RuleConfig)
	if err != nil {
		rawConfig = []byte("{}")
	}
	var def api.LeavePolicyDefinition
	var createdAt, updatedAt time.Time
	var outConfig []byte
	err = db.QueryRow(ctx, `
		INSERT INTO leave_policy_definitions (id, name, description, leave_type_id, rule_config, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
		RETURNING id, name, description, leave_type_id, rule_config, created_at, updated_at
	`, uuid.New(), req.Name, req.Description, req.LeaveTypeID, rawConfig).Scan(
		&def.ID, &def.Name, &def.Description, &def.LeaveTypeID, &outConfig, &createdAt, &updatedAt,
	)
	if err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && (pgErr.Code == "23505" || pgErr.Code == "23P01") {
			return api.LeavePolicyDefinition{}, api.ErrLeaveConflict
		}
		return api.LeavePolicyDefinition{}, err
	}
	_ = json.Unmarshal(outConfig, &def.RuleConfig)
	def.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	def.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return def, nil
}

func (r LeaveRepository) UpdateLeavePolicy(ctx context.Context, policyID uuid.UUID, req api.UpdateLeavePolicyRequest) (api.LeavePolicyDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeavePolicyDefinition{}, err
	}
	var existingRaw []byte
	var existingName string
	var existingDesc *string
	var existingLeaveType uuid.UUID
	err = db.QueryRow(ctx, `
		SELECT name, description, leave_type_id, rule_config
		FROM leave_policy_definitions
		WHERE id = $1 AND deleted_at IS NULL
	`, policyID).Scan(&existingName, &existingDesc, &existingLeaveType, &existingRaw)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LeavePolicyDefinition{}, api.ErrLeaveNotFound
	}
	if err != nil {
		return api.LeavePolicyDefinition{}, err
	}
	name := existingName
	if req.Name != nil && strings.TrimSpace(*req.Name) != "" {
		name = strings.TrimSpace(*req.Name)
	}
	desc := existingDesc
	if req.Description != nil {
		desc = req.Description
	}
	leaveTypeID := existingLeaveType
	if req.LeaveTypeID != nil && *req.LeaveTypeID != uuid.Nil {
		leaveTypeID = *req.LeaveTypeID
	}
	configBytes := existingRaw
	if req.RuleConfig != nil {
		if b, err := json.Marshal(req.RuleConfig); err == nil {
			configBytes = b
		}
	}
	var def api.LeavePolicyDefinition
	var outConfig []byte
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `
		UPDATE leave_policy_definitions
		SET name = $1, description = $2, leave_type_id = $3, rule_config = $4, updated_at = NOW()
		WHERE id = $5 AND deleted_at IS NULL
		RETURNING id, name, description, leave_type_id, rule_config, created_at, updated_at
	`, name, desc, leaveTypeID, configBytes, policyID).Scan(
		&def.ID, &def.Name, &def.Description, &def.LeaveTypeID, &outConfig, &createdAt, &updatedAt,
	)
	if err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && (pgErr.Code == "23505" || pgErr.Code == "23P01") {
			return api.LeavePolicyDefinition{}, api.ErrLeaveConflict
		}
		return api.LeavePolicyDefinition{}, err
	}
	_ = json.Unmarshal(outConfig, &def.RuleConfig)
	def.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	def.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return def, nil
}

func (r LeaveRepository) DeleteLeavePolicy(ctx context.Context, policyID uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if _, err := db.Exec(ctx, `UPDATE leave_policy_assignments SET deleted_at = NOW() WHERE policy_id = $1 AND deleted_at IS NULL`, policyID); err != nil {
			return err
		}
		tag, err := db.Exec(ctx, `UPDATE leave_policy_definitions SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL`, policyID)
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 0 {
			return api.ErrLeaveNotFound
		}
		return nil
	})
}

func (r LeaveRepository) AssignLeavePolicy(ctx context.Context, req api.AssignLeavePolicyRequest) (api.LeavePolicyAssignment, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.LeavePolicyAssignment{}, err
	}
	var assignment api.LeavePolicyAssignment
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var leaveTypeID uuid.UUID
		err := db.QueryRow(ctx, `SELECT leave_type_id FROM leave_policy_definitions WHERE id = $1 AND deleted_at IS NULL`, req.PolicyID).Scan(&leaveTypeID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNotFound
		}
		if err != nil {
			return err
		}
		if _, err = db.Exec(ctx, `
			UPDATE leave_policy_assignments SET deleted_at = NOW()
			WHERE assignee_type = $1 AND assignee_id IS NOT DISTINCT FROM $2
			  AND policy_id IN (SELECT id FROM leave_policy_definitions WHERE leave_type_id = $3)
			  AND effective_from >= $4::date AND deleted_at IS NULL
		`, req.AssigneeType, req.AssigneeID, leaveTypeID, req.EffectiveFrom); err != nil {
			return err
		}
		if _, err = db.Exec(ctx, `
			UPDATE leave_policy_assignments SET effective_to = ($4::date - interval '1 day')::date
			WHERE assignee_type = $1 AND assignee_id IS NOT DISTINCT FROM $2
			  AND policy_id IN (SELECT id FROM leave_policy_definitions WHERE leave_type_id = $3)
			  AND effective_from < $4::date AND (effective_to IS NULL OR effective_to >= $4::date) AND deleted_at IS NULL
		`, req.AssigneeType, req.AssigneeID, leaveTypeID, req.EffectiveFrom); err != nil {
			return err
		}
		var createdAt time.Time
		err = db.QueryRow(ctx, `
			INSERT INTO leave_policy_assignments (id, assignee_type, assignee_id, policy_id, effective_from, effective_to, created_at)
			VALUES ($1, $2, $3, $4, $5::date, $6::date, NOW())
			RETURNING id, assignee_type, assignee_id, policy_id, effective_from::text, effective_to::text, created_at
		`, uuid.New(), req.AssigneeType, req.AssigneeID, req.PolicyID, req.EffectiveFrom, req.EffectiveTo).Scan(
			&assignment.ID, &assignment.AssigneeType, &assignment.AssigneeID, &assignment.PolicyID, &assignment.EffectiveFrom, &assignment.EffectiveTo, &createdAt,
		)
		if err != nil {
			var pgErr *pgconn.PgError
			if errors.As(err, &pgErr) && (pgErr.Code == "23505" || pgErr.Code == "23P01") {
				return api.ErrLeaveConflict
			}
			return err
		}
		assignment.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.LeavePolicyAssignment{}, err
	}
	return assignment, nil
}

func (r LeaveRepository) ListLeavePolicyAssignments(ctx context.Context) ([]api.LeavePolicyAssignmentDetail, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT a.id, a.assignee_type, a.assignee_id, a.policy_id, a.effective_from::text, a.effective_to::text, a.created_at, d.name, lt.name
		FROM leave_policy_assignments a
		JOIN leave_policy_definitions d ON a.policy_id = d.id
		JOIN leave_types lt ON d.leave_type_id = lt.id
		WHERE a.deleted_at IS NULL AND d.deleted_at IS NULL AND lt.deleted_at IS NULL
		ORDER BY a.created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	details := make([]api.LeavePolicyAssignmentDetail, 0)
	for rows.Next() {
		var d api.LeavePolicyAssignmentDetail
		var createdAt time.Time
		if err := rows.Scan(&d.ID, &d.AssigneeType, &d.AssigneeID, &d.PolicyID, &d.EffectiveFrom, &d.EffectiveTo, &createdAt, &d.PolicyName, &d.LeaveTypeName); err != nil {
			return nil, err
		}
		d.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		details = append(details, d)
	}
	return details, rows.Err()
}

func (r LeaveRepository) DeleteLeavePolicyAssignment(ctx context.Context, assignmentID uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var assigneeType string
		var assigneeID *uuid.UUID
		var policyID uuid.UUID
		var effectiveFrom, effectiveTo *string
		err := db.QueryRow(ctx, `SELECT assignee_type, assignee_id, policy_id, effective_from::text, effective_to::text FROM leave_policy_assignments WHERE id = $1 AND deleted_at IS NULL`, assignmentID).Scan(&assigneeType, &assigneeID, &policyID, &effectiveFrom, &effectiveTo)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNotFound
		}
		if err != nil {
			return err
		}
		var leaveTypeID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT leave_type_id FROM leave_policy_definitions WHERE id = $1`, policyID).Scan(&leaveTypeID); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE leave_policy_assignments SET deleted_at = NOW() WHERE id = $1`, assignmentID); err != nil {
			return err
		}
		if effectiveFrom != nil {
			tFrom, parseErr := time.Parse("2006-01-02", *effectiveFrom)
			if parseErr == nil {
				prevEnd := tFrom.AddDate(0, 0, -1).Format("2006-01-02")
				var prevAssignID uuid.UUID
				err = db.QueryRow(ctx, `
					SELECT a.id FROM leave_policy_assignments a
					JOIN leave_policy_definitions d ON a.policy_id = d.id
					WHERE a.assignee_type = $1 AND a.assignee_id IS NOT DISTINCT FROM $2
					  AND d.leave_type_id = $3 AND a.effective_to = $4::date AND a.deleted_at IS NULL
					ORDER BY a.created_at DESC LIMIT 1
				`, assigneeType, assigneeID, leaveTypeID, prevEnd).Scan(&prevAssignID)
				if err == nil {
					_, _ = db.Exec(ctx, `UPDATE leave_policy_assignments SET effective_to = $1::date WHERE id = $2`, effectiveTo, prevAssignID)
				}
			}
		}
		return nil
	})
}
func (r LeaveRepository) CreateAccrualJob(ctx context.Context, userID uuid.UUID, dateStr string) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	jobID := uuid.New()
	payload, _ := json.Marshal(map[string]any{"date": dateStr, "user_id": userID.String(), "force": true})
	_, err = db.Exec(ctx, `INSERT INTO outbox_jobs (id, job_type, payload, status, error_message, created_at, updated_at) VALUES ($1, 'LeaveAccrual', $2, 'pending', NULL, NOW(), NOW())`, jobID, payload)
	if err != nil {
		return uuid.Nil, err
	}
	return jobID, nil
}

func (r LeaveRepository) GetJobStatus(ctx context.Context, jobID uuid.UUID) (api.LeaveJobStatusResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeaveJobStatusResponse{}, err
	}
	var status string
	var errorMsg *string
	var updatedAt time.Time
	if err = db.QueryRow(ctx, `SELECT status, error_message, updated_at FROM outbox_jobs WHERE id = $1`, jobID).Scan(&status, &errorMsg, &updatedAt); err == nil {
		var completedAt *string
		if status == "completed" || status == "failed" {
			value := updatedAt.UTC().Format(time.RFC3339)
			completedAt = &value
		}
		return api.LeaveJobStatusResponse{JobID: jobID, Status: status, CompletedAt: completedAt, ErrorMessage: errorMsg}, nil
	}
	if err = db.QueryRow(ctx, `SELECT status::text, error_log, updated_at FROM attendance_import_jobs WHERE id = $1`, jobID).Scan(&status, &errorMsg, &updatedAt); err == nil {
		var completedAt *string
		if status == "completed" || status == "failed" || status == "partially_failed" {
			value := updatedAt.UTC().Format(time.RFC3339)
			completedAt = &value
		}
		return api.LeaveJobStatusResponse{JobID: jobID, Status: status, CompletedAt: completedAt, ErrorMessage: errorMsg}, nil
	}
	return api.LeaveJobStatusResponse{}, api.ErrLeaveNotFound
}

func (r LeaveRepository) CreateLeaveImportJob(ctx context.Context, userID uuid.UUID, filePath string) (api.LeaveImportJob, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LeaveImportJob{}, err
	}
	var job api.LeaveImportJob
	err = db.QueryRow(ctx, `INSERT INTO attendance_import_jobs (id, user_id, status, total_rows, processed_rows, file_path, target_type, created_at, updated_at) VALUES ($1, $2, 'processing', 0, 0, $3, 'leave_import', NOW(), NOW()) RETURNING id, user_id, status::text, total_rows, processed_rows, file_path, error_log, created_at, updated_at`, uuid.New(), userID, filePath).Scan(
		&job.ID, &job.UserID, &job.Status, &job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.CreatedAt, &job.UpdatedAt,
	)
	return job, err
}

func (r LeaveRepository) ProcessLeaveImport(ctx context.Context, userID uuid.UUID, job api.LeaveImportJob, rows []api.LeaveImportRow, autoApprove bool) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	processed := 0
	failed := 0
	var errorLines []string
	for _, row := range rows {
		startDate, startErr := time.Parse("2006-01-02", row.StartDate)
		if startErr != nil {
			failed++
			errorLines = append(errorLines, fmt.Sprintf("line %d: invalid start date", row.LineNumber))
			continue
		}
		employmentID, resolveErr := resolveImportedEmployment(ctx, db, row.EmployeeCode, startDate)
		if resolveErr != nil {
			failed++
			message := "employee not found"
			if errors.Is(resolveErr, errAmbiguousImportedEmployment) {
				message = "employee number is ambiguous for date"
			}
			errorLines = append(errorLines, fmt.Sprintf("line %d: %s", row.LineNumber, message))
			continue
		}
		var leaveTypeID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT id FROM leave_types WHERE UPPER(code) = UPPER($1) AND deleted_at IS NULL LIMIT 1`, row.LeaveTypeCode).Scan(&leaveTypeID); err != nil {
			failed++
			errorLines = append(errorLines, fmt.Sprintf("line %d: leave type %s not found", row.LineNumber, row.LeaveTypeCode))
			continue
		}
		status := "pending"
		var approvedBy *uuid.UUID
		if autoApprove {
			status = "pre_approved"
			approvedBy = &userID
		}
		if _, err := db.Exec(ctx, `INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, reason, status, duration_type, requested_by_user_id, approved_by_user_id, created_at, updated_at) VALUES (gen_random_uuid(), $1, $2, $3::date, $4::date, $5, $6::leave_status_type, $7, $8, $9, NOW(), NOW())`, employmentID, leaveTypeID, row.StartDate, row.EndDate, row.Reason, status, row.DurationType, userID, approvedBy); err != nil {
			failed++
			errorLines = append(errorLines, fmt.Sprintf("line %d: %v", row.LineNumber, err))
			continue
		}
		processed++
	}
	finalStatus := "completed"
	var errorLog *string
	if failed > 0 {
		finalStatus = "partially_failed"
		message := strings.Join(errorLines, "; ")
		errorLog = &message
		if job.FilePath != nil {
			errorPath := filepath.Join(filepath.Dir(*job.FilePath), "error_"+job.ID.String()+".xlsx")
			if data, genErr := api.BuildErrorWorkbook(job.ID, errorLog); genErr == nil {
				_ = os.WriteFile(errorPath, data, 0o640)
			}
		}
	}
	_, err = db.Exec(ctx, `UPDATE attendance_import_jobs SET status = $1, total_rows = $2, processed_rows = $3, error_log = $4, updated_at = NOW() WHERE id = $5`, finalStatus, len(rows), processed, errorLog, job.ID)
	return err
}

func (r LeaveRepository) GetLatestLeaveImport(ctx context.Context) (*api.LeaveImportJob, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var job api.LeaveImportJob
	err = db.QueryRow(ctx, `SELECT id, user_id, status::text, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE target_type = 'leave_import' ORDER BY created_at DESC LIMIT 1`).Scan(
		&job.ID, &job.UserID, &job.Status, &job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.CreatedAt, &job.UpdatedAt,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &job, nil
}

func (r LeaveRepository) GetLeaveImport(ctx context.Context, jobID uuid.UUID) (*api.LeaveImportJob, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var job api.LeaveImportJob
	err = db.QueryRow(ctx, `SELECT id, user_id, status::text, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE id = $1 AND target_type = 'leave_import'`, jobID).Scan(
		&job.ID, &job.UserID, &job.Status, &job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.CreatedAt, &job.UpdatedAt,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &job, nil
}

func (r LeaveRepository) GetLeaveImportErrorFile(ctx context.Context, jobID uuid.UUID) ([]byte, error) {
	job, err := r.GetLeaveImport(ctx, jobID)
	if err != nil {
		return nil, err
	}
	if job == nil || job.FilePath == nil {
		return nil, pgx.ErrNoRows
	}
	errorPath := filepath.Join(filepath.Dir(*job.FilePath), "error_"+jobID.String()+".xlsx")
	return os.ReadFile(errorPath)
}
