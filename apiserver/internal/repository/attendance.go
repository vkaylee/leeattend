package repository

import (
	"context"
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

// AttendanceRepository persists punch events and paired time entries.
type AttendanceRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

// TxBeginner starts a transaction on a workspace-scoped tenant connection.
type TxBeginner interface {
	Begin(context.Context) (pgx.Tx, error)
}

// PunchIn writes the raw attendance log and opens a time entry.
func (r AttendanceRepository) PunchIn(ctx context.Context, workspaceID, employmentID uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (api.TimeEntryRecord, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, tx pgx.Tx) error {
		var active bool
		if err := tx.QueryRow(ctx, `SELECT status = 'active' FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, employmentID, workspaceID).Scan(&active); err != nil {
			return err
		}
		if !active {
			return api.ErrEmploymentInactive
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
		cmd, err := tx.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at) VALUES ($1, $2, $3, $4, 'device', 'ESS_WEB', $5, 0, 1, CURRENT_TIMESTAMP) ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), workspaceID, employmentID, punchTime, rawData)
		if err != nil {
			return err
		}
		if cmd.RowsAffected() == 0 {
			return scanLatestTimeEntry(ctx, tx, workspaceID, employmentID, &record)
		}
		row := tx.QueryRow(ctx, `INSERT INTO time_entries (id, workspace_id, employment_id, punch_in_time, entry_type, note, is_break_overtime, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, workspace_id, employment_id, template_id, segment_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, uuid.New(), workspaceID, employmentID, punchTime, entryType, note, isBreakOvertime)
		return scanTimeEntry(row, &record)
	})
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	return record, nil
}

// PunchOut records a raw checkout event and returns the latest paired entry.
func (r AttendanceRepository) PunchOut(ctx context.Context, workspaceID, employeeID uuid.UUID, note *string, punchTime time.Time) (api.TimeEntryRecord, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		employmentID, err := resolveActiveEmployment(ctx, db, workspaceID, employeeID)
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
		if _, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at) VALUES ($1, $2, $3, $4, 'device', 'ESS_WEB', $5, 1, 1, CURRENT_TIMESTAMP) ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), workspaceID, employmentID, punchTime, rawData); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE time_entries SET punch_out_time = $1, duration_minutes = GREATEST(0, FLOOR(EXTRACT(EPOCH FROM ($1 - punch_in_time)) / 60)::integer), note = COALESCE($2, note), updated_at = CURRENT_TIMESTAMP WHERE id = (SELECT id FROM time_entries WHERE workspace_id = $3 AND employment_id = $4 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1)`, punchTime, note, workspaceID, employmentID); err != nil {
			return err
		}
		return scanLatestTimeEntry(ctx, db, workspaceID, employmentID, &record)
	})
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	return record, nil
}

// ESSPunch toggles punch in / punch out for an employee based on whether an active session exists.
func (r AttendanceRepository) ESSPunch(ctx context.Context, workspaceID, employeeID uuid.UUID, entryType string, note *string, punchTime time.Time) (api.TimeEntryRecord, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		employmentID, err := resolveActiveEmployment(ctx, db, workspaceID, employeeID)
		if err != nil {
			return err
		}
		var openEntryID *uuid.UUID
		err = db.QueryRow(ctx, `SELECT id FROM time_entries WHERE workspace_id = $1 AND employment_id = $2 AND punch_out_time IS NULL ORDER BY punch_in_time DESC LIMIT 1`, workspaceID, employmentID).Scan(&openEntryID)
		if err != nil && !errors.Is(err, pgx.ErrNoRows) {
			return err
		}
		if openEntryID != nil {
			rawData, err := json.Marshal(struct {
				Punch  int     `json:"punch"`
				Status int     `json:"status"`
				Note   *string `json:"note"`
				Source string  `json:"source"`
			}{Punch: 1, Status: 1, Note: note, Source: "direct_punch"})
			if err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at) VALUES ($1, $2, $3, $4, 'device', 'ESS_WEB', $5, 1, 1, CURRENT_TIMESTAMP) ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), workspaceID, employmentID, punchTime, rawData); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `UPDATE time_entries SET punch_out_time = $1, duration_minutes = GREATEST(0, FLOOR(EXTRACT(EPOCH FROM ($1 - punch_in_time)) / 60)::integer), note = COALESCE($2, note), updated_at = CURRENT_TIMESTAMP WHERE id = $3`, punchTime, note, *openEntryID); err != nil {
				return err
			}
			return scanLatestTimeEntry(ctx, db, workspaceID, employmentID, &record)
		}

		if entryType == "" {
			entryType = "work"
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
		cmd, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at) VALUES ($1, $2, $3, $4, 'device', 'ESS_WEB', $5, 0, 1, CURRENT_TIMESTAMP) ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), workspaceID, employmentID, punchTime, rawData)
		if err != nil {
			return err
		}
		if cmd.RowsAffected() == 0 {
			return scanLatestTimeEntry(ctx, db, workspaceID, employmentID, &record)
		}
		row := db.QueryRow(ctx, `INSERT INTO time_entries (id, workspace_id, employment_id, punch_in_time, entry_type, note, is_break_overtime, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, workspace_id, employment_id, template_id, segment_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, uuid.New(), workspaceID, employmentID, punchTime, entryType, note)
		return scanTimeEntry(row, &record)
	})
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	return record, nil
}

func (r AttendanceRepository) tenantDB(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("attendance repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func resolveActiveEmployment(ctx context.Context, db DBTX, workspaceID, employeeID uuid.UUID) (uuid.UUID, error) {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employeeID, workspaceID).Scan(&exists); err != nil {
		return uuid.Nil, err
	}
	if !exists {
		return uuid.Nil, pgx.ErrNoRows
	}
	var employmentID uuid.UUID
	var status string
	err := db.QueryRow(ctx, `SELECT id, status FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND deleted_at IS NULL ORDER BY (status = 'active') DESC, start_date DESC, id DESC LIMIT 1`, employeeID, workspaceID).Scan(&employmentID, &status)
	if err != nil {
		return uuid.Nil, api.ErrNoActiveEmployment
	}
	if status != "active" {
		return uuid.Nil, api.ErrEmploymentInactive
	}
	return employmentID, nil
}

func scanLatestTimeEntry(ctx context.Context, db DBTX, workspaceID, employmentID uuid.UUID, record *api.TimeEntryRecord) error {
	row := db.QueryRow(ctx, `SELECT id, workspace_id, employment_id, template_id, segment_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE workspace_id = $1 AND employment_id = $2 ORDER BY punch_in_time DESC LIMIT 1`, workspaceID, employmentID)
	return scanTimeEntry(row, record)
}

// ListTimeHistory returns entries for an employee's active employment.
func (r AttendanceRepository) ListTimeHistory(ctx context.Context, workspaceID, employeeID uuid.UUID, start, end time.Time) ([]api.TimeEntryRecord, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	employmentID, err := resolveActiveEmployment(ctx, db, workspaceID, employeeID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, employment_id, template_id, segment_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE workspace_id = $1 AND employment_id = $2 AND punch_in_time >= $3 AND punch_in_time <= $4 ORDER BY punch_in_time DESC`, workspaceID, employmentID, start, end)
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

// IsEmployeeOwnedByUser checks workspace-scoped employee ownership.
func (r AttendanceRepository) IsEmployeeOwnedByUser(ctx context.Context, workspaceID, employeeID, userID uuid.UUID) (bool, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return false, err
	}
	var owned bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND user_id = $3 AND deleted_at IS NULL)`, employeeID, workspaceID, userID).Scan(&owned)
	return owned, err
}

// CreateAdjustment inserts a pending manual adjustment matching schema.
func (r AttendanceRepository) CreateAdjustment(ctx context.Context, workspaceID uuid.UUID, req api.AdjustmentRequest, requestedByUserID uuid.UUID) (api.AdjustmentRecord, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	var exists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, req.EmploymentID, workspaceID).Scan(&exists)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	if !exists {
		return api.AdjustmentRecord{}, pgx.ErrNoRows
	}
	requestedTime, err := time.Parse(time.RFC3339, req.RequestedPunchTime)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	adjID := uuid.New()
	var record api.AdjustmentRecord
	var reqPunchTime, createdAt, updatedAt time.Time
	var fileIDs []uuid.UUID
	err = db.QueryRow(ctx, `INSERT INTO attendance_adjustment_requests (id, employment_id, time_entry_id, requested_punch_time, reason, status, requested_by_user_id, attachment_file_ids, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) RETURNING id, employment_id, time_entry_id, requested_punch_time, reason, status, requested_by_user_id, approved_by_user_id, attachment_file_ids, created_at, updated_at`, adjID, req.EmploymentID, req.TimeEntryID, requestedTime, req.Reason, requestedByUserID, req.AttachmentFileIDs).Scan(&record.ID, &record.EmploymentID, &record.TimeEntryID, &reqPunchTime, &record.Reason, &record.Status, &record.RequestedByUserID, &record.ApprovedByUserID, &fileIDs, &createdAt, &updatedAt)
	if err != nil {
		return api.AdjustmentRecord{}, err
	}
	record.RequestedPunchTime = reqPunchTime.UTC().Format(time.RFC3339)
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	record.AttachmentFileIDs = fileIDs
	return record, nil
}

func (r AttendanceRepository) ListAdjustments(ctx context.Context, workspaceID uuid.UUID, status *string, employeeID *uuid.UUID, page, limit int) (api.PaginatedAdjustments, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return api.PaginatedAdjustments{}, err
	}
	offset := (page - 1) * limit
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id JOIN employees e ON e.id = em.employee_id WHERE e.workspace_id = $1 AND ($2::text IS NULL OR a.status::text = $2) AND ($3::uuid IS NULL OR e.id = $3)`, workspaceID, status, employeeID).Scan(&total); err != nil {
		return api.PaginatedAdjustments{}, err
	}
	rows, err := db.Query(ctx, `SELECT a.id, a.employment_id, a.time_entry_id, a.requested_punch_time, a.reason, a.status::text, a.requested_by_user_id, a.approved_by_user_id, a.attachment_file_ids, a.created_at, a.updated_at FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id JOIN employees e ON e.id = em.employee_id WHERE e.workspace_id = $1 AND ($2::text IS NULL OR a.status::text = $2) AND ($3::uuid IS NULL OR e.id = $3) ORDER BY a.created_at DESC LIMIT $4 OFFSET $5`, workspaceID, status, employeeID, limit, offset)
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

func (r AttendanceRepository) ListEmployeeAdjustments(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]api.AdjustmentRecord, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT a.id, a.employment_id, a.time_entry_id, a.requested_punch_time, a.reason, a.status::text, a.requested_by_user_id, a.approved_by_user_id, a.attachment_file_ids, a.created_at, a.updated_at FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id JOIN employees e ON e.id = em.employee_id WHERE e.workspace_id = $1 AND e.id = $2 ORDER BY a.created_at DESC`, workspaceID, employeeID)
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

func (r AttendanceRepository) ApproveAdjustment(ctx context.Context, workspaceID, adjustmentID, approverID uuid.UUID, status string) error {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var employmentID uuid.UUID
		var requestedPunchTime time.Time
		row := db.QueryRow(ctx, `SELECT a.employment_id, a.requested_punch_time FROM attendance_adjustment_requests a JOIN employments em ON em.id = a.employment_id JOIN employees e ON e.id = em.employee_id WHERE a.id = $1 AND e.workspace_id = $2 FOR UPDATE`, adjustmentID, workspaceID)
		if err := row.Scan(&employmentID, &requestedPunchTime); err != nil {
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
		_, err = db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at) VALUES ($1, $2, $3, $4, 'manual', NULL, NULL, 0, 0, CURRENT_TIMESTAMP)`, uuid.New(), workspaceID, employmentID, requestedPunchTime)
		return err
	})
}

func scanAdjustment(row pgx.Row, record *api.AdjustmentRecord) error {
	var requestedPunchTime, createdAt, updatedAt time.Time
	if err := row.Scan(&record.ID, &record.EmploymentID, &record.TimeEntryID, &requestedPunchTime, &record.Reason, &record.Status, &record.RequestedByUserID, &record.ApprovedByUserID, &record.AttachmentFileIDs, &createdAt, &updatedAt); err != nil {
		return err
	}
	record.RequestedPunchTime = requestedPunchTime.UTC().Format(time.RFC3339)
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return nil
}

func (r AttendanceRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	if r.TenantDB == nil {
		return nil, errors.New("attendance repository: tenant database is not configured")
	}
	db, err := r.TenantDB(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("attendance repository: tenant database does not support transactions")
	}
	return beginner, nil
}

func scanTimeEntry(row pgx.Row, record *api.TimeEntryRecord) error {
	var punchIn, createdAt, updatedAt time.Time
	var punchOut *time.Time
	var metadata []byte
	if err := row.Scan(&record.ID, &record.WorkspaceID, &record.EmploymentID, &record.ShiftID, &record.SegmentID, &punchIn, &punchOut, &record.DurationMinutes, &record.IsBreakOvertime, &record.EntryType, &record.Note, &metadata, &createdAt, &updatedAt); err != nil {
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

// GetAttendanceSummaries queries paginated daily summaries from tenant DB.
func (r AttendanceRepository) GetAttendanceSummaries(ctx context.Context, workspaceID uuid.UUID, startDate, endDate string, employeeID *uuid.UUID, otStatus *string, page, limit int) (api.PaginatedSummaries, error) {
	db, err := r.tenantDB(ctx, workspaceID)
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

	whereClauses := []string{"s.workspace_id = $1", "s.date >= $2", "s.date <= $3"}
	args := []interface{}{workspaceID, startDate, endDate}
	idx := 4

	if employeeID != nil {
		whereClauses = append(whereClauses, fmt.Sprintf("s.employment_id IN (SELECT id FROM employments WHERE employee_id = $%d AND workspace_id = $1)", idx))
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

	dataQuery := fmt.Sprintf(`SELECT s.id, s.workspace_id, s.employment_id, s.date::text, s.actual_work_minutes, s.standard_minutes, s.system_detected_ot_minutes, s.claimed_ot_minutes, s.approved_ot_minutes, s.early_ot_minutes, s.late_ot_minutes, s.break_ot_minutes, s.ot_status::text, s.ot_reason, s.manager_note, s.is_weekend, s.is_holiday, s.holiday_name, s.created_at, s.updated_at FROM attendance_daily_summaries s WHERE %s ORDER BY s.date DESC LIMIT $%d OFFSET $%d`, whereSQL, idx, idx+1)
	args = append(args, limit, offset)

	rows, err := db.Query(ctx, dataQuery, args...)
	if err != nil {
		return api.PaginatedSummaries{}, err
	}
	defer rows.Close()

	var data []api.AttendanceDailySummaryRecord
	for rows.Next() {
		var rec api.AttendanceDailySummaryRecord
		if err := rows.Scan(&rec.ID, &rec.WorkspaceID, &rec.EmploymentID, &rec.Date, &rec.ActualWorkMinutes, &rec.StandardMinutes, &rec.SystemDetectedOTMinutes, &rec.ClaimedOTMinutes, &rec.ApprovedOTMinutes, &rec.EarlyOTMinutes, &rec.LateOTMinutes, &rec.BreakOTMinutes, &rec.OTStatus, &rec.OTReason, &rec.ManagerNote, &rec.IsWeekend, &rec.IsHoliday, &rec.HolidayName, &rec.CreatedAt, &rec.UpdatedAt); err != nil {
			return api.PaginatedSummaries{}, err
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

// GetEmployeeIDByUserID returns the employee ID for a given user ID in workspace.
func (r AttendanceRepository) GetEmployeeIDByUserID(ctx context.Context, workspaceID, userID uuid.UUID) (uuid.UUID, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employees WHERE user_id = $1 AND workspace_id = $2 AND deleted_at IS NULL LIMIT 1`, userID, workspaceID).Scan(&empID)
	if err != nil {
		return uuid.Nil, err
	}
	return empID, nil
}

// UpdateTimeEntry updates time entry fields in tenant DB.
func (r AttendanceRepository) UpdateTimeEntry(ctx context.Context, workspaceID, entryID uuid.UUID, req api.UpdateTimeEntryRequest) (api.TimeEntryRecord, error) {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return api.TimeEntryRecord{}, err
	}
	var record api.TimeEntryRecord
	row := db.QueryRow(ctx, `SELECT id, workspace_id, employment_id, shift_id, segment_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at FROM time_entries WHERE id = $1 AND workspace_id = $2`, entryID, workspaceID)
	if err := scanTimeEntry(row, &record); err != nil {
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

	var pIn time.Time
	if t, err := time.Parse(time.RFC3339, punchInTime); err == nil {
		pIn = t
	}
	var pOut *time.Time
	var durationMinutes *int
	if punchOutTime != nil && *punchOutTime != "" {
		if t, err := time.Parse(time.RFC3339, *punchOutTime); err == nil {
			pOut = &t
			mins := int(t.Sub(pIn).Minutes())
			if mins >= 0 {
				durationMinutes = &mins
			}
		}
	}

	updateRow := db.QueryRow(ctx, `UPDATE time_entries SET punch_in_time = $1, punch_out_time = $2, duration_minutes = $3, entry_type = $4, note = $5, segment_id = $6, updated_at = CURRENT_TIMESTAMP WHERE id = $7 AND workspace_id = $8 RETURNING id, workspace_id, employment_id, shift_id, segment_id, punch_in_time, punch_out_time, duration_minutes, is_break_overtime, entry_type, note, generation_metadata, created_at, updated_at`, pIn, pOut, durationMinutes, entryType, note, segmentID, entryID, workspaceID)
	var updatedRecord api.TimeEntryRecord
	if err := scanTimeEntry(updateRow, &updatedRecord); err != nil {
		return api.TimeEntryRecord{}, err
	}
	return updatedRecord, nil
}

// DeleteTimeEntry deletes a time entry from tenant DB.
func (r AttendanceRepository) DeleteTimeEntry(ctx context.Context, workspaceID, entryID uuid.UUID) error {
	db, err := r.tenantDB(ctx, workspaceID)
	if err != nil {
		return err
	}
	cmd, err := db.Exec(ctx, `DELETE FROM time_entries WHERE id = $1 AND workspace_id = $2`, entryID, workspaceID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}
