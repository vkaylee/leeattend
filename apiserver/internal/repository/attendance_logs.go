package repository

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

func (r AttendanceRepository) attendanceDB(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	return r.tenantDB(ctx, workspaceID)
}

// CreateAttendanceImportJob inserts a processing job scoped to the workspace.
func (r AttendanceRepository) CreateAttendanceImportJob(ctx context.Context, workspaceID, userID uuid.UUID, filePath, target string) (api.AttendanceImportJob, error) {
	db, err := r.attendanceDB(ctx, workspaceID)
	if err != nil {
		return api.AttendanceImportJob{}, err
	}
	var job api.AttendanceImportJob
	err = db.QueryRow(ctx, `INSERT INTO attendance_import_jobs (id, workspace_id, user_id, status, total_rows, processed_rows, file_path, target_type, created_at, updated_at)
		VALUES ($1, $2, $3, 'processing', 0, 0, $4, $5, NOW(), NOW())
		RETURNING id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at`,
		uuid.New(), workspaceID, userID, filePath, target).Scan(&job.ID, &job.WorkspaceID, &job.UserID, &job.Status,
		&job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.TargetType, &job.CreatedAt, &job.UpdatedAt)
	return job, err
}

// ProcessAttendanceImport persists parsed rows for a job. Unknown employees are staged,
// parse failures are recorded in attendance_log_errors, and duplicate punches are skipped.
func (r AttendanceRepository) ProcessAttendanceImport(ctx context.Context, workspaceID, userID, jobID uuid.UUID, filePath, target string, rows []api.AttendanceImportRow) error {
	db, err := r.attendanceDB(ctx, workspaceID)
	if err != nil {
		return err
	}
	processed := 0
	staged := 0
	failed := 0
	var errorLines []string
	for _, row := range rows {
		employeeID, lookupErr := findEmployeeByNumber(ctx, db, workspaceID, row.EmployeeCode)
		if errors.Is(lookupErr, pgx.ErrNoRows) {
			if err := stageUnknownPunch(ctx, db, workspaceID, row); err != nil {
				return err
			}
			staged++
			processed++
			continue
		}
		if lookupErr != nil {
			return lookupErr
		}
		employmentID, empErr := employmentForEmployee(ctx, db, workspaceID, employeeID)
		if empErr != nil {
			failed++
			errorLines = append(errorLines, row.EmployeeCode)
			if err := recordLogError(ctx, db, workspaceID, row.EmployeeCode, "no active employment"); err != nil {
				return err
			}
			continue
		}
		if target == "log" {
			if err := insertImportedRawLog(ctx, db, workspaceID, employmentID, row); err != nil {
				return err
			}
		} else if err := insertApprovedAdjustment(ctx, db, workspaceID, employmentID, userID, row); err != nil {
			return err
		}
		processed++
	}
	status := "completed"
	errorLog := (*string)(nil)
	if failed > 0 {
		status = "partially_failed"
		summary := "failed employee codes: " + strings.Join(errorLines, ",")
		errorLog = &summary
	}
	if _, err := db.Exec(ctx, `UPDATE attendance_import_jobs SET status = $3, total_rows = $4, processed_rows = $5, error_log = $6, updated_at = NOW() WHERE id = $1 AND workspace_id = $2`,
		jobID, workspaceID, status, len(rows), processed, errorLog); err != nil {
		return err
	}
	if failed > 0 {
		errorPath := filepath.Join(filepath.Dir(filePath), "error_"+jobID.String()+".xlsx")
		if data, genErr := api.BuildErrorWorkbook(jobID, errorLog); genErr == nil {
			_ = os.WriteFile(errorPath, data, 0o640)
		}
	}
	_ = staged
	return nil
}

func findEmployeeByNumber(ctx context.Context, db DBTX, workspaceID uuid.UUID, code string) (uuid.UUID, error) {
	var id uuid.UUID
	err := db.QueryRow(ctx, `SELECT id FROM employees WHERE workspace_id = $1 AND UPPER(employee_id_number) = UPPER($2) AND deleted_at IS NULL`, workspaceID, code).Scan(&id)
	return id, err
}

func stageUnknownPunch(ctx context.Context, db DBTX, workspaceID uuid.UUID, row api.AttendanceImportRow) error {
	raw := map[string]any{"employee_code": row.EmployeeCode, "line": row.LineNumber}
	payload, err := json.Marshal(raw)
	if err != nil {
		return err
	}
	fingerprint := row.EmployeeCode + "|" + row.PunchTime.UTC().Format(time.RFC3339)
	_, err = db.Exec(ctx, `INSERT INTO attendance_staging_logs (workspace_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at)
		VALUES ($1, $2, $3, $4, $5, 'pending', NOW())`, workspaceID, row.EmployeeCode, row.PunchTime, payload, fingerprint)
	return err
}

func recordLogError(ctx context.Context, db DBTX, workspaceID uuid.UUID, code, message string) error {
	raw, err := json.Marshal(map[string]string{"employee_code": code})
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO attendance_log_errors (workspace_id, raw_content, error_message, source, created_at)
		VALUES ($1, $2, $3, 'import', NOW())`, workspaceID, raw, message)
	return err
}

func insertImportedRawLog(ctx context.Context, db DBTX, workspaceID, employmentID uuid.UUID, row api.AttendanceImportRow) error {
	raw, err := json.Marshal(map[string]any{"employee_code": row.EmployeeCode, "line": row.LineNumber})
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, raw_data, punch, status, created_at)
		VALUES ($1, $2, $3, $4, 'import', $5, 0, 0, NOW())
		ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`,
		uuid.New(), workspaceID, employmentID, row.PunchTime, raw)
	return err
}

func insertApprovedAdjustment(ctx context.Context, db DBTX, workspaceID, employmentID, userID uuid.UUID, row api.AttendanceImportRow) error {
	_, err := db.Exec(ctx, `INSERT INTO attendance_adjustment_requests (id, requested_punch_in_time, reason, status, requested_by_user_id, approved_by_user_id, employment_id, created_at, updated_at)
		VALUES ($1, $2, 'Import from Excel/CSV', 'approved', $3, $3, $4, NOW(), NOW())`,
		uuid.New(), row.PunchTime, userID, employmentID)
	if err != nil {
		return err
	}
	return insertImportedRawLog(ctx, db, workspaceID, employmentID, row)
}

func scanAttendanceJob(row pgx.Row) (*api.AttendanceImportJob, error) {
	var j api.AttendanceImportJob
	if err := row.Scan(&j.ID, &j.WorkspaceID, &j.UserID, &j.Status, &j.TotalRows, &j.ProcessedRows, &j.FilePath, &j.ErrorLog, &j.TargetType, &j.CreatedAt, &j.UpdatedAt); err != nil {
		return nil, err
	}
	return &j, nil
}

// GetLatestAttendanceImportJob returns the newest workspace job, optionally filtered by target type.
func (r AttendanceRepository) GetLatestAttendanceImportJob(ctx context.Context, ws uuid.UUID, target *string) (*api.AttendanceImportJob, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return nil, err
	}
	query := `SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at
		FROM attendance_import_jobs WHERE workspace_id = $1`
	args := []any{ws}
	if target != nil {
		query += ` AND target_type = $2`
		args = append(args, *target)
	}
	query += ` ORDER BY created_at DESC LIMIT 1`
	job, err := scanAttendanceJob(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	return job, err
}

// GetAttendanceImportJob returns one workspace-scoped job.
func (r AttendanceRepository) GetAttendanceImportJob(ctx context.Context, ws, id uuid.UUID) (*api.AttendanceImportJob, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return nil, err
	}
	return scanAttendanceJob(db.QueryRow(ctx, `SELECT id, workspace_id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at
		FROM attendance_import_jobs WHERE id = $1 AND workspace_id = $2`, id, ws))
}

// GetAttendanceLogErrorFile reads the generated XLSX beside the job upload.
func (r AttendanceRepository) GetAttendanceLogErrorFile(ctx context.Context, ws, id uuid.UUID) ([]byte, error) {
	job, err := r.GetAttendanceImportJob(ctx, ws, id)
	if err != nil {
		return nil, err
	}
	if job == nil || job.FilePath == nil {
		return nil, pgx.ErrNoRows
	}
	errorPath := filepath.Join(filepath.Dir(*job.FilePath), "error_"+id.String()+".xlsx")
	return os.ReadFile(errorPath)
}

func employmentForEmployee(ctx context.Context, db DBTX, ws, employee uuid.UUID) (uuid.UUID, error) {
	var id uuid.UUID
	err := db.QueryRow(ctx, `SELECT id FROM employments WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, ws, employee).Scan(&id)
	return id, err
}

func logFilter(alias string, ws uuid.UUID, employee *uuid.UUID, from, to *time.Time) (string, []any) {
	where := ` WHERE ` + alias + `.workspace_id = $1`
	args := []any{ws}
	n := 2
	if employee != nil {
		where += ` AND ` + alias + `.employment_id = $` + itoa(n)
		args = append(args, *employee)
		n++
	}
	if from != nil {
		where += ` AND ` + alias + `.punch_in_time >= $` + itoa(n)
		args = append(args, *from)
		n++
	}
	if to != nil {
		where += ` AND ` + alias + `.punch_in_time <= $` + itoa(n)
		args = append(args, *to)
	}
	return where, args
}

func rawFilter(ws uuid.UUID, employee *uuid.UUID, from, to *time.Time) (string, []any) {
	where := ` WHERE l.workspace_id = $1`
	args := []any{ws}
	n := 2
	if employee != nil {
		where += ` AND l.employment_id = $` + itoa(n)
		args = append(args, *employee)
		n++
	}
	if from != nil {
		where += ` AND l.punch_time >= $` + itoa(n)
		args = append(args, *from)
		n++
	}
	if to != nil {
		where += ` AND l.punch_time <= $` + itoa(n)
		args = append(args, *to)
	}
	return where, args
}

// ListAttendanceLogs returns paginated paired entries joined with employee identity.
func (r AttendanceRepository) ListAttendanceLogs(ctx context.Context, ws uuid.UUID, employee *uuid.UUID, from, to *time.Time, page, limit int) ([]api.AttendanceLogRecord, int64, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return nil, 0, err
	}
	if employee != nil {
		if id, empErr := employmentForEmployee(ctx, db, ws, *employee); empErr == nil {
			employee = &id
		}
	}
	where, args := logFilter("t", ws, employee, from, to)
	var total int64
	if err = db.QueryRow(ctx, `SELECT COUNT(*) FROM time_entries t`+where, args...).Scan(&total); err != nil {
		return nil, 0, err
	}
	n := len(args) + 1
	query := `SELECT t.id, t.employment_id, t.workspace_id, t.punch_in_time, t.punch_out_time, t.duration_minutes, t.entry_type, t.note, t.generation_metadata, t.created_at, t.updated_at, e.full_name, e.employee_id_number
		FROM time_entries t
		JOIN employments em ON em.id = t.employment_id
		JOIN employees e ON e.id = em.employee_id` + where + ` ORDER BY t.punch_in_time DESC LIMIT $` + itoa(n) + ` OFFSET $` + itoa(n+1)
	args = append(args, limit, (page-1)*limit)
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	result := make([]api.AttendanceLogRecord, 0)
	for rows.Next() {
		var v api.AttendanceLogRecord
		if err = rows.Scan(&v.ID, &v.EmploymentID, &v.WorkspaceID, &v.PunchInTime, &v.PunchOutTime, &v.DurationMinutes, &v.EntryType, &v.Note, &v.GenerationMetadata, &v.CreatedAt, &v.UpdatedAt, &v.EmployeeFullName, &v.EmployeeIDNumber); err != nil {
			return nil, 0, err
		}
		result = append(result, v)
	}
	return result, total, rows.Err()
}

// ListRawAttendanceLogs returns paginated raw punches joined with employee and device names.
func (r AttendanceRepository) ListRawAttendanceLogs(ctx context.Context, ws uuid.UUID, employee *uuid.UUID, from, to *time.Time, page, limit int) ([]api.RawAttendanceLogRecord, int64, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return nil, 0, err
	}
	if employee != nil {
		if id, empErr := employmentForEmployee(ctx, db, ws, *employee); empErr == nil {
			employee = &id
		}
	}
	where, args := rawFilter(ws, employee, from, to)
	var total int64
	if err = db.QueryRow(ctx, `SELECT COUNT(*) FROM raw_attendance_logs l`+where, args...).Scan(&total); err != nil {
		return nil, 0, err
	}
	n := len(args) + 1
	query := `SELECT l.id, l.workspace_id, l.employment_id, l.punch_time, l.source::text, l.device_id, l.raw_data, l.punch, l.status, l.created_at, e.full_name, e.employee_id_number, d.name
		FROM raw_attendance_logs l
		JOIN employments em ON em.id = l.employment_id
		JOIN employees e ON e.id = em.employee_id
		LEFT JOIN devices d ON d.id::text = l.device_id` + where + ` ORDER BY l.punch_time DESC LIMIT $` + itoa(n) + ` OFFSET $` + itoa(n+1)
	args = append(args, limit, (page-1)*limit)
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	result := make([]api.RawAttendanceLogRecord, 0)
	for rows.Next() {
		var v api.RawAttendanceLogRecord
		if err = rows.Scan(&v.ID, &v.WorkspaceID, &v.EmploymentID, &v.PunchTime, &v.Source, &v.DeviceID, &v.RawData, &v.Punch, &v.Status, &v.CreatedAt, &v.EmployeeFullName, &v.EmployeeIDNumber, &v.DeviceName); err != nil {
			return nil, 0, err
		}
		result = append(result, v)
	}
	return result, total, rows.Err()
}

// ListStagingLogs returns open staging logs (pending and ambiguous).
func (r AttendanceRepository) ListStagingLogs(ctx context.Context, ws uuid.UUID, page, limit int) ([]api.AttendanceStagingLog, int64, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return nil, 0, err
	}
	var total int64
	if err = db.QueryRow(ctx, `SELECT COUNT(*) FROM attendance_staging_logs WHERE workspace_id = $1 AND status IN ('pending', 'ambiguous')`, ws).Scan(&total); err != nil {
		return nil, 0, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, status, created_at
		FROM attendance_staging_logs WHERE workspace_id = $1 AND status IN ('pending', 'ambiguous')
		ORDER BY created_at DESC LIMIT $2 OFFSET $3`, ws, limit, (page-1)*limit)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	result := make([]api.AttendanceStagingLog, 0)
	for rows.Next() {
		var v api.AttendanceStagingLog
		if err = rows.Scan(&v.ID, &v.WorkspaceID, &v.DeviceID, &v.RawUserID, &v.PunchTime, &v.RawData, &v.Status, &v.CreatedAt); err != nil {
			return nil, 0, err
		}
		result = append(result, v)
	}
	return result, total, rows.Err()
}

// ResolveStagingLog promotes one open staging entry into raw_attendance_logs.
func (r AttendanceRepository) ResolveStagingLog(ctx context.Context, ws, id, employee uuid.UUID, employment *uuid.UUID) error {
	tx, err := r.begin(ctx, ws)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var punchTime time.Time
		var raw json.RawMessage
		var device *string
		var status string
		if err := db.QueryRow(ctx, `SELECT punch_time, raw_data, device_id, status FROM attendance_staging_logs WHERE id = $1 AND workspace_id = $2 FOR UPDATE`, id, ws).Scan(&punchTime, &raw, &device, &status); err != nil {
			return err
		}
		if status != "pending" && status != "ambiguous" {
			return errors.New("staging log already resolved")
		}
		employmentID := employment
		if employmentID == nil {
			var resolved uuid.UUID
			if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE workspace_id = $1 AND employee_id = $2 AND deleted_at IS NULL
				AND start_date <= $3::date AND (end_date IS NULL OR end_date >= $3::date)
				ORDER BY start_date DESC, id DESC LIMIT 1`, ws, employee, punchTime).Scan(&resolved); err != nil {
				return err
			}
			employmentID = &resolved
		}
		var valid bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND workspace_id = $2 AND employee_id = $3 AND deleted_at IS NULL
			AND start_date <= $4::date AND (end_date IS NULL OR end_date >= $4::date))`, *employmentID, ws, employee, punchTime).Scan(&valid); err != nil {
			return err
		}
		if !valid {
			return errors.New("employment does not cover punch date")
		}
		punchCode, statusCode := -1, -1
		if len(raw) > 0 {
			var data struct {
				Punch  int `json:"punch"`
				Status int `json:"status"`
			}
			if json.Unmarshal(raw, &data) == nil {
				punchCode, statusCode = data.Punch, data.Status
			}
		}
		if _, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, workspace_id, employment_id, punch_time, source, device_id, raw_data, punch, status, created_at)
			VALUES ($1, $2, $3, $4, 'device', $5, $6, $7, $8, NOW()) ON CONFLICT (workspace_id, employment_id, punch_time, punch, status) DO NOTHING`,
			uuid.New(), ws, *employmentID, punchTime, device, raw, punchCode, statusCode); err != nil {
			return err
		}
		_, err := db.Exec(ctx, `UPDATE attendance_staging_logs SET status = 'processed' WHERE id = $1 AND workspace_id = $2`, id, ws)
		return err
	})
}

// DismissStagingLog marks an open staging entry dismissed.
func (r AttendanceRepository) DismissStagingLog(ctx context.Context, ws, id uuid.UUID) (bool, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return false, err
	}
	tag, err := db.Exec(ctx, `UPDATE attendance_staging_logs SET status = 'dismissed' WHERE id = $1 AND workspace_id = $2 AND status IN ('pending', 'ambiguous')`, id, ws)
	return tag.RowsAffected() > 0, err
}

// EnterPaperOT upserts a paper-form OT claim against the active employment.
func (r AttendanceRepository) EnterPaperOT(ctx context.Context, ws, employee uuid.UUID, req api.PaperOTRequest, user uuid.UUID) (api.AttendanceDailySummary, error) {
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		return api.AttendanceDailySummary{}, err
	}
	employmentID, err := employmentForEmployee(ctx, db, ws, employee)
	if err != nil {
		return api.AttendanceDailySummary{}, err
	}
	return upsertPaperOT(ctx, db, ws, employmentID, req, user)
}

func upsertPaperOT(ctx context.Context, db DBTX, ws, employmentID uuid.UUID, req api.PaperOTRequest, user uuid.UUID) (api.AttendanceDailySummary, error) {
	date, err := time.Parse("2006-01-02", req.Date)
	if err != nil {
		return api.AttendanceDailySummary{}, err
	}
	var reason *string
	if req.Reason != "" {
		reason = &req.Reason
	}
	var out api.AttendanceDailySummary
	var dateValue time.Time
	err = db.QueryRow(ctx, `INSERT INTO attendance_daily_summaries (id, workspace_id, employment_id, date, claimed_ot_minutes, ot_reason, ot_status, entered_via, claim_entered_by_user_id, paper_form_photo_url, claim_entered_at, actual_work_minutes, system_detected_ot_minutes)
		VALUES ($1, $2, $3, $4, $5, $6, 'pending_manager', 'paper_form', $7, $8, NOW(), 0, 0)
		ON CONFLICT (employment_id, date) DO UPDATE SET
			claimed_ot_minutes = EXCLUDED.claimed_ot_minutes,
			ot_reason = EXCLUDED.ot_reason,
			ot_status = 'pending_manager',
			entered_via = 'paper_form',
			claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id,
			paper_form_photo_url = EXCLUDED.paper_form_photo_url,
			claim_entered_at = NOW(),
			updated_at = NOW()
		RETURNING id, workspace_id, employment_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status::text, ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at`,
		uuid.New(), ws, employmentID, date, req.ClaimedMinutes, reason, user, req.PaperFormPhotoURL).Scan(&out.ID, &out.WorkspaceID, &out.EmploymentID,
		&dateValue, &out.ActualWorkMinutes, &out.SystemDetectedOTMinutes, &out.ClaimedOTMinutes, &out.ApprovedOTMinutes, &out.OTStatus, &out.OTReason,
		&out.ManagerNote, &out.ClaimEnteredByUserID, &out.ClaimEnteredAt, &out.PaperFormPhotoURL, &out.EnteredVia, &out.IsHoliday, &out.HolidayName, &out.CreatedAt, &out.UpdatedAt)
	out.Date = dateValue.Format("2006-01-02")
	return out, err
}

// BulkPaperOT applies each CSV row independently and reports per-row failures.
func (r AttendanceRepository) BulkPaperOT(ctx context.Context, ws uuid.UUID, rows []api.BulkPaperOTRow, user uuid.UUID) api.BulkPaperOTResult {
	result := api.BulkPaperOTResult{TotalRows: len(rows), Errors: make([]api.BulkOTError, 0)}
	db, err := r.attendanceDB(ctx, ws)
	if err != nil {
		result.FailedCount = len(rows)
		for _, row := range rows {
			result.Errors = append(result.Errors, api.BulkOTError{LineNumber: row.LineNumber, EmploymentID: row.EmployeeCode, ErrorMessage: err.Error()})
		}
		return result
	}
	for _, row := range rows {
		rowErr := func() error {
			if row.EmployeeCode == "" {
				return errors.New("employee ID is required")
			}
			if row.ClaimedMinutes <= 0 {
				return errors.New("minutes must be greater than 0")
			}
			if row.Reason == "" {
				return errors.New("reason is required")
			}
			if _, parseErr := time.Parse("2006-01-02", row.Date); parseErr != nil {
				return errors.New("invalid date format: " + row.Date + ". Expected YYYY-MM-DD")
			}
			employeeID, lookupErr := findEmployeeByNumber(ctx, db, ws, row.EmployeeCode)
			if lookupErr != nil {
				return errors.New("employee not found in workspace")
			}
			employmentID, empErr := employmentForEmployee(ctx, db, ws, employeeID)
			if empErr != nil {
				return errors.New("no active employment found for employee")
			}
			_, upsertErr := upsertPaperOT(ctx, db, ws, employmentID, api.PaperOTRequest{Date: row.Date, ClaimedMinutes: row.ClaimedMinutes, Reason: row.Reason}, user)
			return upsertErr
		}()
		if rowErr != nil {
			result.FailedCount++
			result.Errors = append(result.Errors, api.BulkOTError{LineNumber: row.LineNumber, EmploymentID: row.EmployeeCode, ErrorMessage: rowErr.Error()})
			continue
		}
		result.SuccessCount++
	}
	return result
}

func itoa(v int) string {
	const digits = "0123456789"
	if v == 0 {
		return "0"
	}
	var buf [20]byte
	i := len(buf)
	for v > 0 {
		i--
		buf[i] = digits[v%10]
		v /= 10
	}
	return string(buf[i:])
}
