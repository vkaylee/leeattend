package api

import (
	"bytes"
	"context"
	"encoding/csv"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const attendanceUploadMaxBytes int64 = 10 * 1024 * 1024

const (
	// AttendanceLogsReadPermission gates attendance log listings.
	AttendanceLogsReadPermission = "attendance:logs:read"
	// AttendanceLogsImportPermission gates uploads and import-job endpoints.
	AttendanceLogsImportPermission = "attendance:logs:import"
	// AttendanceStagingPermission gates staging log review.
	AttendanceStagingPermission = "attendance:staging:manage"
	// OvertimePaperUploadPermission gates paper OT entry.
	OvertimePaperUploadPermission = "overtime:paper:upload"
)

// AttendanceImportJob is the workspace import job contract.
type AttendanceImportJob struct {
	ID            uuid.UUID `json:"id"`
	WorkspaceID   uuid.UUID `json:"workspace_id"`
	UserID        uuid.UUID `json:"user_id"`
	Status        string    `json:"status"`
	TotalRows     int       `json:"total_rows"`
	ProcessedRows int       `json:"processed_rows"`
	FilePath      *string   `json:"file_path"`
	ErrorLog      *string   `json:"error_log"`
	TargetType    string    `json:"target_type"`
	CreatedAt     time.Time `json:"created_at"`
	UpdatedAt     time.Time `json:"updated_at"`
}

// AttendanceLogRecord is a paired time entry enriched with employee identity.
type AttendanceLogRecord struct {
	ID                 uuid.UUID       `json:"id"`
	EmploymentID       uuid.UUID       `json:"employment_id"`
	WorkspaceID        uuid.UUID       `json:"workspace_id"`
	PunchInTime        time.Time       `json:"punch_in_time"`
	PunchOutTime       *time.Time      `json:"punch_out_time"`
	DurationMinutes    *int            `json:"duration_minutes"`
	EntryType          string          `json:"entry_type"`
	Note               *string         `json:"note"`
	GenerationMetadata json.RawMessage `json:"generation_metadata"`
	CreatedAt          time.Time       `json:"created_at"`
	UpdatedAt          time.Time       `json:"updated_at"`
	EmployeeFullName   string          `json:"employee_full_name"`
	EmployeeIDNumber   string          `json:"employee_id_number"`
}

// RawAttendanceLogRecord is an immutable punch with employee and device names.
type RawAttendanceLogRecord struct {
	ID               uuid.UUID       `json:"id"`
	WorkspaceID      uuid.UUID       `json:"workspace_id"`
	EmploymentID     uuid.UUID       `json:"employment_id"`
	PunchTime        time.Time       `json:"punch_time"`
	Source           string          `json:"source"`
	DeviceID         *string         `json:"device_id"`
	RawData          json.RawMessage `json:"raw_data"`
	Punch            int             `json:"punch"`
	Status           int             `json:"status"`
	CreatedAt        time.Time       `json:"created_at"`
	EmployeeFullName string          `json:"employee_full_name"`
	EmployeeIDNumber string          `json:"employee_id_number"`
	DeviceName       *string         `json:"device_name"`
}

// AttendanceStagingLog is an unresolved punch awaiting employee mapping.
type AttendanceStagingLog struct {
	ID          uuid.UUID       `json:"id"`
	WorkspaceID uuid.UUID       `json:"workspace_id"`
	DeviceID    *string         `json:"device_id"`
	RawUserID   string          `json:"raw_user_id"`
	PunchTime   time.Time       `json:"punch_time"`
	RawData     json.RawMessage `json:"raw_data"`
	Status      string          `json:"status"`
	CreatedAt   time.Time       `json:"created_at"`
}

// AttendancePage is the shared pagination envelope.
type AttendancePage[T any] struct {
	Data  []T   `json:"data"`
	Total int64 `json:"total"`
	Page  int   `json:"page"`
	Limit int   `json:"limit"`
}

// AttendanceImportRow is one validated workbook punch.
type AttendanceImportRow struct {
	LineNumber   int
	EmployeeCode string
	PunchTime    time.Time
}

// AttendanceDailySummary is the paper-OT upsert result.
type AttendanceDailySummary struct {
	ID                      uuid.UUID  `json:"id"`
	WorkspaceID             uuid.UUID  `json:"workspace_id"`
	EmploymentID            uuid.UUID  `json:"employment_id"`
	Date                    string     `json:"date"`
	ActualWorkMinutes       int        `json:"actual_work_minutes"`
	SystemDetectedOTMinutes int        `json:"system_detected_ot_minutes"`
	ClaimedOTMinutes        int        `json:"claimed_ot_minutes"`
	ApprovedOTMinutes       int        `json:"approved_ot_minutes"`
	OTStatus                string     `json:"ot_status"`
	OTReason                *string    `json:"ot_reason"`
	ManagerNote             *string    `json:"manager_note"`
	ClaimEnteredByUserID    *uuid.UUID `json:"claim_entered_by_user_id"`
	ClaimEnteredAt          *time.Time `json:"claim_entered_at"`
	PaperFormPhotoURL       *string    `json:"paper_form_photo_url"`
	EnteredVia              *string    `json:"entered_via"`
	IsHoliday               bool       `json:"is_holiday"`
	HolidayName             *string    `json:"holiday_name"`
	CreatedAt               time.Time  `json:"created_at"`
	UpdatedAt               time.Time  `json:"updated_at"`
}

// PaperOTRequest is the single-entry paper overtime payload.
type PaperOTRequest struct {
	Date                      string  `json:"date"`
	ClaimedMinutes            int     `json:"claimed_minutes"`
	Reason                    string  `json:"reason"`
	AllowExceedSystemDetected bool    `json:"allow_exceed_system_detected"`
	PaperFormPhotoURL         *string `json:"paper_form_photo_url"`
}

// BulkPaperOTRow is one CSV paper overtime row.
type BulkPaperOTRow struct {
	LineNumber     int
	EmployeeCode   string
	Date           string
	ClaimedMinutes int
	Reason         string
}

// BulkOTError is a per-line bulk paper OT failure.
type BulkOTError struct {
	LineNumber   int    `json:"line_number"`
	EmploymentID string `json:"employment_id"`
	ErrorMessage string `json:"error_message"`
}

// BulkPaperOTResult is the bulk upload response contract.
type BulkPaperOTResult struct {
	TotalRows    int           `json:"total_rows"`
	SuccessCount int           `json:"success_count"`
	FailedCount  int           `json:"failed_count"`
	Errors       []BulkOTError `json:"errors"`
}

// AttendanceImportRepository is the typed data contract for attendance logs.
type AttendanceImportRepository interface {
	CreateAttendanceImportJob(context.Context, uuid.UUID, uuid.UUID, string, string) (AttendanceImportJob, error)
	ProcessAttendanceImport(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, string, string, []AttendanceImportRow) error
	GetLatestAttendanceImportJob(context.Context, uuid.UUID, *string) (*AttendanceImportJob, error)
	GetAttendanceImportJob(context.Context, uuid.UUID, uuid.UUID) (*AttendanceImportJob, error)
	GetAttendanceLogErrorFile(context.Context, uuid.UUID, uuid.UUID) ([]byte, error)
	ListAttendanceLogs(context.Context, uuid.UUID, *uuid.UUID, *time.Time, *time.Time, int, int) ([]AttendanceLogRecord, int64, error)
	ListRawAttendanceLogs(context.Context, uuid.UUID, *uuid.UUID, *time.Time, *time.Time, int, int) ([]RawAttendanceLogRecord, int64, error)
	ListStagingLogs(context.Context, uuid.UUID, int, int) ([]AttendanceStagingLog, int64, error)
	ResolveStagingLog(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, *uuid.UUID) error
	DismissStagingLog(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	EnterPaperOT(context.Context, uuid.UUID, uuid.UUID, PaperOTRequest, uuid.UUID) (AttendanceDailySummary, error)
	BulkPaperOT(context.Context, uuid.UUID, []BulkPaperOTRow, uuid.UUID) BulkPaperOTResult
}

// AttendanceLogsHandler serves attendance log, import, staging, and paper OT routes.
type AttendanceLogsHandler struct {
	Repository AttendanceImportRepository
	Authorizer AttendanceAuthorizer
	StorageDir string
}

func (h AttendanceLogsHandler) allowed(r *http.Request, ws uuid.UUID, permission string) bool {
	user, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), user, ws, permission)
}

func (h AttendanceLogsHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h AttendanceLogsHandler) fail(w http.ResponseWriter, r *http.Request, err error) {
	var appErr *apperror.AppError
	if errors.As(err, &appErr) {
		apperror.WriteError(w, r, appErr)
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func (h AttendanceLogsHandler) user(r *http.Request) (uuid.UUID, bool) {
	return httpinfra.UserIDFromContext(r.Context())
}

// Upload handles POST .../attendance/upload (adjustment import target).
func (h AttendanceLogsHandler) Upload(w http.ResponseWriter, r *http.Request) {
	h.upload(w, r, "adjustment")
}

// UploadLogs handles POST .../attendance/logs/upload (raw log import target).
func (h AttendanceLogsHandler) UploadLogs(w http.ResponseWriter, r *http.Request) {
	h.upload(w, r, "log")
}

func (h AttendanceLogsHandler) upload(w http.ResponseWriter, r *http.Request, target string) {
	ws, ok := pathUUID(r, "ws_id")
	user, userOK := h.user(r)
	if !ok || !userOK || !h.allowed(r, ws, AttendanceLogsImportPermission) {
		h.forbidden(w, r)
		return
	}
	name, content, err := readAttendanceMultipart(r, false)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	dir := h.StorageDir
	if dir == "" {
		dir = "storage"
	}
	if err = os.MkdirAll(filepath.Join(dir, "imports"), 0o750); err != nil {
		h.fail(w, r, err)
		return
	}
	jobID := uuid.New()
	path := filepath.Join(dir, "imports", jobID.String()+"_"+filepath.Base(name))
	if err = os.WriteFile(path, content, 0o640); err != nil {
		h.fail(w, r, err)
		return
	}
	job, err := h.Repository.CreateAttendanceImportJob(r.Context(), ws, user, path, target)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	workbook, err := parseEmployeeWorkbook(content, []string{"employee_code", "punch_time"})
	if err != nil {
		h.fail(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rows := make([]AttendanceImportRow, 0, len(workbook))
	for _, row := range workbook {
		code := strings.TrimSpace(row["employee_code"])
		punch, parseErr := parseAttendanceImportTime(strings.TrimSpace(row["punch_time"]))
		if code == "" || parseErr != nil {
			continue
		}
		rows = append(rows, AttendanceImportRow{LineNumber: parseRowNumber(row["_row_number"]), EmployeeCode: code, PunchTime: punch})
	}
	if err = h.Repository.ProcessAttendanceImport(r.Context(), ws, user, job.ID, path, target, rows); err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct {
		JobID uuid.UUID `json:"job_id"`
	}{job.ID})
}

func parseAttendanceImportTime(value string) (time.Time, error) {
	for _, layout := range []string{time.RFC3339, "2006-01-02 15:04:05", "2006-01-02 15:04", "02/01/2006 15:04:05", "02/01/2006 15:04"} {
		if parsed, err := time.Parse(layout, value); err == nil {
			return parsed.UTC(), nil
		}
	}
	return time.Time{}, errors.New("invalid attendance import timestamp")
}

// Latest handles GET .../attendance/import-jobs/latest.
func (h AttendanceLogsHandler) Latest(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, AttendanceLogsImportPermission) {
		h.forbidden(w, r)
		return
	}
	var target *string
	if raw := r.URL.Query().Get("target_type"); raw != "" {
		if raw != "log" && raw != "adjustment" && raw != "employee" && raw != "leave_import" {
			h.fail(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		target = &raw
	}
	result, err := h.Repository.GetLatestAttendanceImportJob(r.Context(), ws, target)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// Job handles GET .../attendance/import-jobs/{id}.
func (h AttendanceLogsHandler) Job(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, ws, AttendanceLogsImportPermission) {
		h.forbidden(w, r)
		return
	}
	result, err := h.Repository.GetAttendanceImportJob(r.Context(), ws, id)
	if err != nil {
		if errors.Is(err, errRowNotFound) {
			h.fail(w, r, apperror.NotFound(apperror.KeyJobNotFound))
			return
		}
		h.fail(w, r, err)
		return
	}
	if result == nil {
		h.fail(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// ErrorFile handles GET .../attendance/import-jobs/{id}/error-file.
func (h AttendanceLogsHandler) ErrorFile(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, ws, AttendanceLogsImportPermission) {
		h.forbidden(w, r)
		return
	}
	data, err := h.Repository.GetAttendanceLogErrorFile(r.Context(), ws, id)
	if err != nil {
		if errors.Is(err, errRowNotFound) {
			h.fail(w, r, apperror.NotFound(apperror.KeyJobNotFound))
			return
		}
		h.fail(w, r, apperror.NotFound(apperror.KeyErrorFileNotFound))
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", `attachment; filename="error_`+id.String()+`.xlsx"`)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(data)
}

var errRowNotFound = errors.New("no rows in result set")

// Logs handles GET .../attendance/logs.
func (h AttendanceLogsHandler) Logs(w http.ResponseWriter, r *http.Request) {
	h.listLogs(w, r, false)
}

// RawLogs handles GET .../attendance/raw-logs.
func (h AttendanceLogsHandler) RawLogs(w http.ResponseWriter, r *http.Request) {
	h.listLogs(w, r, true)
}

func (h AttendanceLogsHandler) listLogs(w http.ResponseWriter, r *http.Request, raw bool) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, AttendanceLogsReadPermission) {
		h.forbidden(w, r)
		return
	}
	employee, from, to, page, limit, err := parseAttendanceLogFilters(r)
	if err != nil {
		h.fail(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if raw {
		data, total, listErr := h.Repository.ListRawAttendanceLogs(r.Context(), ws, employee, from, to, page, limit)
		if listErr != nil {
			h.fail(w, r, listErr)
			return
		}
		apperror.WriteJSON(w, http.StatusOK, AttendancePage[RawAttendanceLogRecord]{Data: data, Total: total, Page: page, Limit: limit})
		return
	}
	data, total, listErr := h.Repository.ListAttendanceLogs(r.Context(), ws, employee, from, to, page, limit)
	if listErr != nil {
		h.fail(w, r, listErr)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, AttendancePage[AttendanceLogRecord]{Data: data, Total: total, Page: page, Limit: limit})
}

func parseAttendanceLogFilters(r *http.Request) (*uuid.UUID, *time.Time, *time.Time, int, int, error) {
	page, limit := 1, 20
	if raw := r.URL.Query().Get("page"); raw != "" {
		parsed, err := strconv.Atoi(raw)
		if err != nil || parsed < 1 {
			return nil, nil, nil, 0, 0, ErrInvalidPagination
		}
		page = parsed
	}
	if raw := r.URL.Query().Get("limit"); raw != "" {
		parsed, err := strconv.Atoi(raw)
		if err != nil || parsed < 1 {
			return nil, nil, nil, 0, 0, ErrInvalidPagination
		}
		limit = parsed
	}
	if limit > 100 {
		limit = 100
	}
	var employee *uuid.UUID
	if raw := r.URL.Query().Get("employee_id"); raw != "" {
		parsed, err := uuid.Parse(raw)
		if err != nil {
			return nil, nil, nil, 0, 0, err
		}
		employee = &parsed
	}
	parseTime := func(key string) (*time.Time, error) {
		value := r.URL.Query().Get(key)
		if value == "" {
			return nil, nil
		}
		parsed, err := time.Parse(time.RFC3339, value)
		if err != nil {
			return nil, err
		}
		return &parsed, nil
	}
	from, err := parseTime("from")
	if err != nil {
		return nil, nil, nil, 0, 0, err
	}
	to, err := parseTime("to")
	if err != nil {
		return nil, nil, nil, 0, 0, err
	}
	return employee, from, to, page, limit, nil
}

// Staging handles GET .../attendance/staging-logs.
func (h AttendanceLogsHandler) Staging(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, AttendanceStagingPermission) {
		h.forbidden(w, r)
		return
	}
	page, limit := 1, 50
	if raw := r.URL.Query().Get("page"); raw != "" {
		parsed, err := strconv.Atoi(raw)
		if err != nil || parsed < 1 {
			h.fail(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		page = parsed
	}
	if raw := r.URL.Query().Get("limit"); raw != "" {
		parsed, err := strconv.Atoi(raw)
		if err != nil || parsed < 1 {
			h.fail(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		limit = parsed
	}
	if limit > 100 {
		limit = 100
	}
	data, total, err := h.Repository.ListStagingLogs(r.Context(), ws, page, limit)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, AttendancePage[AttendanceStagingLog]{Data: data, Total: total, Page: page, Limit: limit})
}

// ResolveStagingRequest maps a staging punch to an employee/employment.
type ResolveStagingRequest struct {
	EmployeeID   uuid.UUID  `json:"employee_id"`
	EmploymentID *uuid.UUID `json:"employment_id"`
}

// Resolve handles POST .../attendance/staging-logs/{id}/resolve.
func (h AttendanceLogsHandler) Resolve(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, ws, AttendanceStagingPermission) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[ResolveStagingRequest](r, 1<<20)
	if err != nil || req.EmployeeID == uuid.Nil {
		h.fail(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err = h.Repository.ResolveStagingLog(r.Context(), ws, id, req.EmployeeID, req.EmploymentID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("Log resolved successfully"))
}

// Dismiss handles DELETE .../attendance/staging-logs/{id}.
func (h AttendanceLogsHandler) Dismiss(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, ws, AttendanceStagingPermission) {
		h.forbidden(w, r)
		return
	}
	changed, err := h.Repository.DismissStagingLog(r.Context(), ws, id)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	if !changed {
		h.fail(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

// EnterPaperOT handles POST .../attendance/enter-paper-ot/{employee_id}.
func (h AttendanceLogsHandler) EnterPaperOT(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	employeeID, empOK := pathUUID(r, "employee_id")
	user, userOK := h.user(r)
	if !ok || !empOK || !userOK || !h.allowed(r, ws, OvertimePaperUploadPermission) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[PaperOTRequest](r, 1<<20)
	if err != nil || req.ClaimedMinutes <= 0 || strings.TrimSpace(req.Reason) == "" {
		h.fail(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err = time.Parse("2006-01-02", req.Date); err != nil {
		h.fail(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.EnterPaperOT(r.Context(), ws, employeeID, req, user)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// BulkPaperOT handles POST .../attendance/bulk-upload-paper-ot.
func (h AttendanceLogsHandler) BulkPaperOT(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	user, userOK := h.user(r)
	if !ok || !userOK || !h.allowed(r, ws, OvertimePaperUploadPermission) {
		h.forbidden(w, r)
		return
	}
	_, content, err := readAttendanceMultipart(r, true)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	rows, parseErrors, err := parseBulkPaperOT(content, time.Now().UTC())
	if err != nil {
		h.fail(w, r, err)
		return
	}
	result := h.Repository.BulkPaperOT(r.Context(), ws, rows, user)
	result.TotalRows += len(parseErrors)
	result.FailedCount += len(parseErrors)
	result.Errors = append(result.Errors, parseErrors...)
	apperror.WriteJSON(w, http.StatusOK, result)
}

func readAttendanceMultipart(r *http.Request, csvOnly bool) (string, []byte, error) {
	if r.ContentLength > attendanceUploadMaxBytes {
		return "", nil, apperror.PayloadTooLarge(apperror.KeyUploadFileTooLarge)
	}
	reader, err := r.MultipartReader()
	if err != nil {
		return "", nil, apperror.Validation(apperror.KeyValidation)
	}
	for {
		part, nextErr := reader.NextPart()
		if errors.Is(nextErr, io.EOF) {
			break
		}
		if nextErr != nil {
			return "", nil, apperror.Validation(apperror.KeyValidation)
		}
		if part.FormName() != "file" {
			continue
		}
		filename := filepath.Base(part.FileName())
		if filename == "" || filename == "." {
			return "", nil, apperror.Validation(apperror.KeyValidation)
		}
		extension := strings.ToLower(filepath.Ext(filename))
		if csvOnly && extension != ".csv" {
			return "", nil, apperror.Validation("error.invalid_file_format")
		}
		if !csvOnly && extension != ".xlsx" && extension != ".xls" {
			return "", nil, apperror.Validation("error.invalid_file_format")
		}
		content, readErr := io.ReadAll(io.LimitReader(part, attendanceUploadMaxBytes+1))
		if readErr != nil {
			return "", nil, apperror.Validation(apperror.KeyValidation)
		}
		if int64(len(content)) > attendanceUploadMaxBytes {
			return "", nil, apperror.PayloadTooLarge(apperror.KeyUploadFileTooLarge)
		}
		if !csvOnly && !isXLSX(content) && !isXLS(content) {
			return "", nil, apperror.Validation("error.invalid_file_format")
		}
		return filename, content, nil
	}
	return "", nil, apperror.Validation(apperror.KeyValidation)
}

// parseBulkPaperOT validates positional CSV rows, returning valid rows and per-line failures.
func parseBulkPaperOT(data []byte, now time.Time) ([]BulkPaperOTRow, []BulkOTError, error) {
	reader := csv.NewReader(bytes.NewReader(data))
	reader.FieldsPerRecord = -1
	rows := make([]BulkPaperOTRow, 0)
	failures := make([]BulkOTError, 0)
	seen := make(map[string]struct{})
	today := now.Format("2006-01-02")
	line := 1
	for {
		record, err := reader.Read()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return nil, nil, apperror.Validation("error.csv_parsing_error")
		}
		line++
		if len(rows)+len(failures) >= 10000 {
			return nil, nil, apperror.Validation("error.max_rows_exceeded")
		}
		code := ""
		if len(record) > 0 {
			code = strings.TrimSpace(record[0])
		}
		dateValue := ""
		if len(record) > 1 {
			dateValue = strings.TrimSpace(record[1])
		}
		minutesValue := ""
		if len(record) > 2 {
			minutesValue = strings.TrimSpace(record[2])
		}
		reason := ""
		if len(record) > 3 {
			reason = strings.TrimSpace(record[3])
		}
		fail := func(message string) {
			failures = append(failures, BulkOTError{LineNumber: line, EmploymentID: code, ErrorMessage: message})
		}
		if code == "" {
			fail("Employee ID is required")
			continue
		}
		if _, parseErr := time.Parse("2006-01-02", dateValue); parseErr != nil {
			fail("Invalid date format: " + dateValue + ". Expected YYYY-MM-DD")
			continue
		}
		minutes, convErr := strconv.Atoi(minutesValue)
		if convErr != nil {
			fail("Invalid minutes value: " + minutesValue)
			continue
		}
		if minutes <= 0 {
			fail("Minutes must be greater than 0")
			continue
		}
		if reason == "" {
			fail("Reason is required")
			continue
		}
		if dateValue > today {
			fail("Date cannot be in the future")
			continue
		}
		key := code + "|" + dateValue
		if _, duplicate := seen[key]; duplicate {
			fail("Duplicate entry in this upload")
			continue
		}
		seen[key] = struct{}{}
		rows = append(rows, BulkPaperOTRow{LineNumber: line, EmployeeCode: code, Date: dateValue, ClaimedMinutes: minutes, Reason: reason})
	}
	return rows, failures, nil
}

// BuildErrorWorkbook renders a one-sheet XLSX describing an import failure summary.
func BuildErrorWorkbook(jobID uuid.UUID, message *string) ([]byte, error) {
	body := ""
	if message != nil {
		body = *message
	}
	return writeXLSX([]xlsxSheet{{
		Name:   "Errors",
		Widths: []float64{40, 80},
		Rows: [][]string{
			{"job_id", "error"},
			{jobID.String(), body},
		},
	}})
}
