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

func (r AttendanceRepository) attendanceDB(ctx context.Context) (DBTX, error) {
	return r.db(ctx)
}

// CreateAttendanceImportJob inserts a processing import job.
func (r AttendanceRepository) CreateAttendanceImportJob(ctx context.Context, userID uuid.UUID, filePath, target string) (api.AttendanceImportJob, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return api.AttendanceImportJob{}, err
	}
	var job api.AttendanceImportJob
	err = db.QueryRow(ctx, `INSERT INTO attendance_import_jobs (id, user_id, status, total_rows, processed_rows, file_path, target_type, created_at, updated_at)
		VALUES ($1, $2, 'processing', 0, 0, $3, $4, NOW(), NOW())
		RETURNING id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at`,
		uuid.New(), userID, filePath, target).Scan(&job.ID, &job.UserID, &job.Status,
		&job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.TargetType, &job.CreatedAt, &job.UpdatedAt)
	return job, err
}

// ProcessAttendanceImport persists parsed rows for a job. Unknown employees are staged,
// parse failures are recorded in attendance_log_errors, and duplicate punches are skipped.
func (r AttendanceRepository) ProcessAttendanceImport(ctx context.Context, userID, jobID uuid.UUID, filePath, target string, rows []api.AttendanceImportRow) error {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return err
	}
	processed := 0
	staged := 0
	failed := 0
	var errorLines []string
	for _, row := range rows {
		employmentID, lookupErr := resolveImportedEmployment(ctx, db, row.EmployeeCode, row.PunchTime)
		if errors.Is(lookupErr, pgx.ErrNoRows) {
			if err := stageUnknownPunch(ctx, db, row); err != nil {
				return err
			}
			staged++
			processed++
			continue
		}
		if lookupErr != nil {
			failed++
			errorLines = append(errorLines, row.EmployeeCode)
			message := "no active employment"
			if errors.Is(lookupErr, errAmbiguousImportedEmployment) {
				message = "employee number is ambiguous for punch date"
			}
			if err := recordLogError(ctx, db, row.EmployeeCode, message); err != nil {
				return err
			}
			continue
		}
		if target == "log" {
			if err := insertImportedRawLog(ctx, db, employmentID, row); err != nil {
				return err
			}
		} else if err := insertApprovedAdjustment(ctx, db, employmentID, userID, row); err != nil {
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
	if _, err := db.Exec(ctx, `UPDATE attendance_import_jobs SET status = $2, total_rows = $3, processed_rows = $4, error_log = $5, updated_at = NOW() WHERE id = $1`,
		jobID, status, len(rows), processed, errorLog); err != nil {
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

var errAmbiguousImportedEmployment = errors.New("attendance import employee number matches multiple employments")

func resolveImportedEmployment(ctx context.Context, db DBTX, code string, punchTime time.Time) (uuid.UUID, error) {
	var (
		candidateCount int
		employmentID   *uuid.UUID
	)
	err := db.QueryRow(ctx, `WITH candidates AS (
		SELECT em.id
		FROM employments em
		JOIN employment_relationships er
		  ON er.id = em.relationship_id
		 AND er.employee_id = em.employee_id
		 AND er.employer_id = em.employer_id
		JOIN employee_tenures t
		  ON t.id = er.tenure_id
		 AND t.employee_id = er.employee_id
		JOIN employees e ON e.id = t.employee_id
		WHERE UPPER(t.employee_number) = UPPER($1)
		  AND e.deleted_at IS NULL
		  AND t.deleted_at IS NULL
		  AND t.start_date <= $2::date
		  AND (t.end_date IS NULL OR t.end_date >= $2::date)
		  AND er.deleted_at IS NULL
		  AND er.start_date <= $2::date
		  AND (er.end_date IS NULL OR er.end_date >= $2::date)
		  AND em.deleted_at IS NULL
		  AND em.start_date <= $2::date
		  AND (em.end_date IS NULL OR em.end_date >= $2::date)
	)
	SELECT count(*)::int, (array_agg(id ORDER BY id))[1] FROM candidates`, code, punchTime).Scan(&candidateCount, &employmentID)
	if err != nil {
		return uuid.Nil, err
	}
	if candidateCount > 1 {
		return uuid.Nil, errAmbiguousImportedEmployment
	}
	if candidateCount == 0 || employmentID == nil {
		return uuid.Nil, pgx.ErrNoRows
	}
	return *employmentID, nil
}

func stageUnknownPunch(ctx context.Context, db DBTX, row api.AttendanceImportRow) error {
	raw := map[string]any{"employee_code": row.EmployeeCode, "line": row.LineNumber}
	payload, err := json.Marshal(raw)
	if err != nil {
		return err
	}
	fingerprint := row.EmployeeCode + "|" + row.PunchTime.UTC().Format(time.RFC3339)
	_, err = db.Exec(ctx, `INSERT INTO attendance_staging_logs (raw_user_id, punch_time, raw_data, fingerprint, status, created_at)
		VALUES ($1, $2, $3, $4, 'pending', NOW())`, row.EmployeeCode, row.PunchTime, payload, fingerprint)
	return err
}

func recordLogError(ctx context.Context, db DBTX, code, message string) error {
	raw, err := json.Marshal(map[string]string{"employee_code": code})
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO attendance_log_errors (raw_content, error_message, source, created_at)
		VALUES ($1, $2, 'import', NOW())`, raw, message)
	return err
}

func insertImportedRawLog(ctx context.Context, db DBTX, employmentID uuid.UUID, row api.AttendanceImportRow) error {
	// Raw attendance ingestion remains allowed after closure; later reconciliation must not rewrite evidence.
	raw, err := json.Marshal(map[string]any{"employee_code": row.EmployeeCode, "line": row.LineNumber})
	if err != nil {
		return err
	}
	rawLogID := uuid.New()
	cmd, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at)
		VALUES ($1, $2, $3, 'import', $4, 0, 0, NOW())
		ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`,
		rawLogID, employmentID, row.PunchTime, raw)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() > 0 {
		return enqueueClosedPeriodLatePunchReconciliation(ctx, db, employmentID, rawLogID, nil, row.PunchTime)
	}
	return nil
}

func insertApprovedAdjustment(ctx context.Context, db DBTX, employmentID, userID uuid.UUID, row api.AttendanceImportRow) error {
	if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{row.PunchTime.UTC().Format("2006-01-02")}); err != nil {
		return finishLifecycleMutationDB(ctx, db, err)
	}
	_, err := db.Exec(ctx, `INSERT INTO attendance_adjustment_requests (id, requested_punch_time, reason, status, requested_by_user_id, approved_by_user_id, employment_id, created_at, updated_at)
		VALUES ($1, $2, 'Import from Excel/CSV', 'approved', $3, $3, $4, NOW(), NOW())`,
		uuid.New(), row.PunchTime, userID, employmentID)
	if err != nil {
		return err
	}
	return insertImportedRawLog(ctx, db, employmentID, row)
}

func scanAttendanceJob(row pgx.Row) (*api.AttendanceImportJob, error) {
	var j api.AttendanceImportJob
	if err := row.Scan(&j.ID, &j.UserID, &j.Status, &j.TotalRows, &j.ProcessedRows, &j.FilePath, &j.ErrorLog, &j.TargetType, &j.CreatedAt, &j.UpdatedAt); err != nil {
		return nil, err
	}
	return &j, nil
}

// GetLatestAttendanceImportJob returns the newest import job, optionally filtered by target type.
func (r AttendanceRepository) GetLatestAttendanceImportJob(ctx context.Context, target *string) (*api.AttendanceImportJob, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return nil, err
	}
	query := `SELECT id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at
		FROM attendance_import_jobs WHERE ($1::text IS NULL OR target_type = $1)`
	query += ` ORDER BY created_at DESC LIMIT 1`
	job, err := scanAttendanceJob(db.QueryRow(ctx, query, target))
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	return job, err
}

// GetAttendanceImportJob returns one import job.
func (r AttendanceRepository) GetAttendanceImportJob(ctx context.Context, id uuid.UUID) (*api.AttendanceImportJob, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return nil, err
	}
	return scanAttendanceJob(db.QueryRow(ctx, `SELECT id, user_id, status::text, total_rows, processed_rows, file_path, error_log, target_type, created_at, updated_at
		FROM attendance_import_jobs WHERE id = $1`, id))
}

// GetAttendanceLogErrorFile reads the generated XLSX beside the job upload.
func (r AttendanceRepository) GetAttendanceLogErrorFile(ctx context.Context, id uuid.UUID) ([]byte, error) {
	job, err := r.GetAttendanceImportJob(ctx, id)
	if err != nil {
		return nil, err
	}
	if job == nil || job.FilePath == nil {
		return nil, pgx.ErrNoRows
	}
	errorPath := filepath.Join(filepath.Dir(*job.FilePath), "error_"+id.String()+".xlsx")
	return os.ReadFile(errorPath)
}

func employmentForEmployee(ctx context.Context, db DBTX, employee uuid.UUID) (uuid.UUID, error) {
	var id uuid.UUID
	err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1`, employee).Scan(&id)
	return id, err
}

func logFilter(alias string, employee *uuid.UUID, from, to *time.Time) (string, []any) {
	where := ` WHERE TRUE`
	args := []any{}
	n := 1
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

func rawFilter(employee *uuid.UUID, from, to *time.Time) (string, []any) {
	where := ` WHERE TRUE`
	args := []any{}
	n := 1
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
func (r AttendanceRepository) ListAttendanceLogs(ctx context.Context, employee *uuid.UUID, from, to *time.Time, page, limit int) ([]api.AttendanceLogRecord, int64, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return nil, 0, err
	}
	if employee != nil {
		if id, empErr := employmentForEmployee(ctx, db, *employee); empErr == nil {
			employee = &id
		}
	}
	where, args := logFilter("t", employee, from, to)
	var total int64
	if err = db.QueryRow(ctx, `SELECT COUNT(*) FROM time_entries t`+where, args...).Scan(&total); err != nil {
		return nil, 0, err
	}
	n := len(args) + 1
	query := `SELECT t.id, t.employment_id, t.punch_in_time, t.punch_out_time, t.duration_minutes, t.entry_type, t.note, t.generation_metadata, t.created_at, t.updated_at, e.full_name, COALESCE(tenure.employee_number, '')
		FROM time_entries t
		JOIN employments em ON em.id = t.employment_id
		JOIN employees e ON e.id = em.employee_id
		LEFT JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id
		LEFT JOIN employee_tenures tenure ON tenure.id = er.tenure_id AND tenure.employee_id = e.id AND tenure.deleted_at IS NULL AND tenure.start_date <= t.punch_in_time::date AND (tenure.end_date IS NULL OR tenure.end_date >= t.punch_in_time::date)` + where + ` ORDER BY t.punch_in_time DESC LIMIT $` + itoa(n) + ` OFFSET $` + itoa(n+1)
	args = append(args, limit, (page-1)*limit)
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	result := make([]api.AttendanceLogRecord, 0)
	for rows.Next() {
		var v api.AttendanceLogRecord
		if err = rows.Scan(&v.ID, &v.EmploymentID, &v.PunchInTime, &v.PunchOutTime, &v.DurationMinutes, &v.EntryType, &v.Note, &v.GenerationMetadata, &v.CreatedAt, &v.UpdatedAt, &v.EmployeeFullName, &v.EmployeeIDNumber); err != nil {
			return nil, 0, err
		}
		result = append(result, v)
	}
	return result, total, rows.Err()
}

// ListRawAttendanceLogs returns paginated raw punches joined with employee identity.
func (r AttendanceRepository) ListRawAttendanceLogs(ctx context.Context, employee *uuid.UUID, from, to *time.Time, page, limit int) ([]api.RawAttendanceLogRecord, int64, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return nil, 0, err
	}
	if employee != nil {
		if id, empErr := employmentForEmployee(ctx, db, *employee); empErr == nil {
			employee = &id
		}
	}
	where, args := rawFilter(employee, from, to)
	var total int64
	if err = db.QueryRow(ctx, `SELECT COUNT(*) FROM raw_attendance_logs l`+where, args...).Scan(&total); err != nil {
		return nil, 0, err
	}
	n := len(args) + 1
	query := `SELECT l.id, l.employment_id, l.punch_time, l.source::text, l.raw_data, l.punch, l.status, l.created_at, e.full_name, COALESCE(tenure.employee_number, '') FROM raw_attendance_logs l JOIN employments em ON em.id = l.employment_id JOIN employees e ON e.id = em.employee_id LEFT JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id LEFT JOIN employee_tenures tenure ON tenure.id = er.tenure_id AND tenure.employee_id = e.id AND tenure.deleted_at IS NULL AND tenure.start_date <= l.punch_time::date AND (tenure.end_date IS NULL OR tenure.end_date >= l.punch_time::date)` + where + ` ORDER BY l.punch_time DESC LIMIT $` + itoa(n) + ` OFFSET $` + itoa(n+1)
	args = append(args, limit, (page-1)*limit)
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	result := make([]api.RawAttendanceLogRecord, 0)
	for rows.Next() {
		var v api.RawAttendanceLogRecord
		if err = rows.Scan(&v.ID, &v.EmploymentID, &v.PunchTime, &v.Source, &v.RawData, &v.Punch, &v.Status, &v.CreatedAt, &v.EmployeeFullName, &v.EmployeeIDNumber); err != nil {
			return nil, 0, err
		}
		result = append(result, v)
	}
	return result, total, rows.Err()
}

// ListStagingLogs returns open staging logs (pending and ambiguous).
func (r AttendanceRepository) ListStagingLogs(ctx context.Context, page, limit int) ([]api.AttendanceStagingLog, int64, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return nil, 0, err
	}
	var total int64
	if err = db.QueryRow(ctx, `SELECT COUNT(*) FROM attendance_staging_logs WHERE status IN ('pending', 'ambiguous')`).Scan(&total); err != nil {
		return nil, 0, err
	}
	rows, err := db.Query(ctx, `SELECT id, raw_user_id, punch_time, raw_data, status, created_at FROM attendance_staging_logs WHERE status IN ('pending', 'ambiguous') ORDER BY created_at DESC LIMIT $1 OFFSET $2`, limit, (page-1)*limit)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()
	result := make([]api.AttendanceStagingLog, 0)
	for rows.Next() {
		var v api.AttendanceStagingLog
		if err = rows.Scan(&v.ID, &v.RawUserID, &v.PunchTime, &v.RawData, &v.Status, &v.CreatedAt); err != nil {
			return nil, 0, err
		}
		result = append(result, v)
	}
	return result, total, rows.Err()
}

// ResolveStagingLog promotes one open staging entry into raw_attendance_logs.
func (r AttendanceRepository) ResolveStagingLog(ctx context.Context, id, employee uuid.UUID, employment *uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var punchTime time.Time
		var raw json.RawMessage
		var status string
		if err := db.QueryRow(ctx, `SELECT punch_time, raw_data, status FROM attendance_staging_logs WHERE id = $1 FOR UPDATE`, id).Scan(&punchTime, &raw, &status); err != nil {
			return err
		}
		if status != "pending" && status != "ambiguous" {
			return errors.New("staging log already resolved")
		}
		employmentID := employment
		if employmentID == nil {
			var resolved uuid.UUID
			if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND deleted_at IS NULL
				AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $2::date)
				ORDER BY start_date DESC, id DESC LIMIT 1`, employee, punchTime).Scan(&resolved); err != nil {
				return err
			}
			employmentID = &resolved
		}
		var valid bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND employee_id = $2 AND deleted_at IS NULL
			AND start_date <= $3::date AND (end_date IS NULL OR end_date >= $3::date))`, *employmentID, employee, punchTime).Scan(&valid); err != nil {
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
		if _, err := db.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'import', $4, $5, $6, NOW()) ON CONFLICT (employment_id, punch_time, punch, status) DO NOTHING`, uuid.New(), *employmentID, punchTime, raw, punchCode, statusCode); err != nil {
			return err
		}
		_, err := db.Exec(ctx, `UPDATE attendance_staging_logs SET status = 'processed' WHERE id = $1`, id)
		return err
	})
	return finishLifecycleMutation(ctx, tx, err)
}

// DismissStagingLog marks an open staging entry dismissed.
func (r AttendanceRepository) DismissStagingLog(ctx context.Context, id uuid.UUID) (bool, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return false, err
	}
	tag, err := db.Exec(ctx, `UPDATE attendance_staging_logs SET status = 'dismissed' WHERE id = $1 AND status IN ('pending', 'ambiguous')`, id)
	return tag.RowsAffected() > 0, err
}

// EnterPaperOT upserts a paper-form OT claim against the active employment.
func (r AttendanceRepository) EnterPaperOT(ctx context.Context, employee uuid.UUID, req api.PaperOTRequest, user uuid.UUID) (api.AttendanceDailySummary, error) {
	db, err := r.attendanceDB(ctx)
	if err != nil {
		return api.AttendanceDailySummary{}, err
	}
	employmentID, err := employmentForEmployee(ctx, db, employee)
	if err != nil {
		return api.AttendanceDailySummary{}, err
	}
	if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{req.Date}); err != nil {
		return api.AttendanceDailySummary{}, finishLifecycleMutationDB(ctx, db, err)
	}
	return upsertPaperOT(ctx, db, employmentID, req, user)
}

func upsertPaperOT(ctx context.Context, db DBTX, employmentID uuid.UUID, req api.PaperOTRequest, user uuid.UUID) (api.AttendanceDailySummary, error) {
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
	err = db.QueryRow(ctx, `INSERT INTO attendance_daily_summaries (id, employment_id, date, claimed_ot_minutes, ot_reason, ot_status, entered_via, claim_entered_by_user_id, paper_form_photo_url, claim_entered_at, actual_work_minutes, system_detected_ot_minutes)
		VALUES ($1, $2, $3, $4, $5, 'pending_manager', 'paper_form', $6, $7, NOW(), 0, 0)
		ON CONFLICT (employment_id, date) DO UPDATE SET
			claimed_ot_minutes = EXCLUDED.claimed_ot_minutes,
			ot_reason = EXCLUDED.ot_reason,
			ot_status = 'pending_manager',
			entered_via = 'paper_form',
			claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id,
			paper_form_photo_url = EXCLUDED.paper_form_photo_url,
			claim_entered_at = NOW(),
			updated_at = NOW()
		RETURNING id, employment_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status::text, ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at`,
		uuid.New(), employmentID, date, req.ClaimedMinutes, reason, user, req.PaperFormPhotoURL).Scan(&out.ID, &out.EmploymentID,
		&dateValue, &out.ActualWorkMinutes, &out.SystemDetectedOTMinutes, &out.ClaimedOTMinutes, &out.ApprovedOTMinutes, &out.OTStatus, &out.OTReason,
		&out.ManagerNote, &out.ClaimEnteredByUserID, &out.ClaimEnteredAt, &out.PaperFormPhotoURL, &out.EnteredVia, &out.IsHoliday, &out.HolidayName, &out.CreatedAt, &out.UpdatedAt)
	out.Date = dateValue.Format("2006-01-02")
	return out, err
}

// BulkPaperOT applies each CSV row independently and reports per-row failures.
func (r AttendanceRepository) BulkPaperOT(ctx context.Context, rows []api.BulkPaperOTRow, user uuid.UUID) api.BulkPaperOTResult {
	result := api.BulkPaperOTResult{TotalRows: len(rows), Errors: make([]api.BulkOTError, 0)}
	db, err := r.attendanceDB(ctx)
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
			date, parseErr := time.Parse("2006-01-02", row.Date)
			if parseErr != nil {
				return errors.New("invalid date format: " + row.Date + ". Expected YYYY-MM-DD")
			}
			employmentID, lookupErr := resolveImportedEmployment(ctx, db, row.EmployeeCode, date)
			if lookupErr != nil {
				if errors.Is(lookupErr, errAmbiguousImportedEmployment) {
					return errors.New("employee number is ambiguous for date")
				}
				return errors.New("employee not found")
			}
			if guardErr := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{row.Date}); guardErr != nil {
				return finishLifecycleMutationDB(ctx, db, guardErr)
			}
			_, upsertErr := upsertPaperOT(ctx, db, employmentID, api.PaperOTRequest{Date: row.Date, ClaimedMinutes: row.ClaimedMinutes, Reason: row.Reason}, user)
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
