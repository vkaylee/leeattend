package api

import (
	"context"
	"errors"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type TriggerAccrualRequest struct {
	Date string `json:"date"`
}
type AccrualJobResponse struct {
	JobID   uuid.UUID `json:"job_id"`
	Status  string    `json:"status"`
	Message string    `json:"message,omitempty"`
}
type LeaveJobStatusResponse struct {
	JobID        uuid.UUID `json:"job_id"`
	Status       string    `json:"status"`
	CompletedAt  *string   `json:"completed_at,omitempty"`
	ErrorMessage *string   `json:"error_message,omitempty"`
}
type LeaveImportRow struct {
	LineNumber    int
	EmployeeCode  string
	LeaveTypeCode string
	StartDate     string
	EndDate       string
	Reason        *string
	DurationType  string
}
type LeaveImportJob struct {
	ID            uuid.UUID `json:"id"`
	UserID        uuid.UUID `json:"user_id"`
	Status        string    `json:"status"`
	TotalRows     int       `json:"total_rows"`
	ProcessedRows int       `json:"processed_rows"`
	FilePath      *string   `json:"file_path,omitempty"`
	ErrorLog      *string   `json:"error_log,omitempty"`
	CreatedAt     time.Time `json:"created_at"`
	UpdatedAt     time.Time `json:"updated_at"`
}
type LeaveImportRepository interface {
	CreateLeaveImportJob(context.Context, uuid.UUID, string) (LeaveImportJob, error)
	ProcessLeaveImport(context.Context, uuid.UUID, LeaveImportJob, []LeaveImportRow, bool) error
	GetLatestLeaveImport(context.Context) (*LeaveImportJob, error)
	GetLeaveImport(context.Context, uuid.UUID) (*LeaveImportJob, error)
	GetLeaveImportErrorFile(context.Context, uuid.UUID) ([]byte, error)
}

func (h *LeaveHandler) TriggerAccrual(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:accrual:run") {
		writeForbidden(w, r)
		return
	}
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[TriggerAccrualRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, parseErr := time.Parse("2006-01-02", req.Date); parseErr != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	jobID, err := h.Repository.CreateAccrualJob(r.Context(), user, req.Date)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, AccrualJobResponse{
		JobID:   jobID,
		Status:  "queued",
		Message: "Accrual job enqueued for background processing",
	})
}

func (h *LeaveHandler) JobStatus(w http.ResponseWriter, r *http.Request) {

	jobID, jobOK := pathUUID(r, "job_id")
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !jobOK || !userOK {
		writeForbidden(w, r)
		return
	}
	status, err := h.Repository.GetJobStatus(r.Context(), jobID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, status)
}

func saveLeaveImportFile(storageDir, name string, content []byte) (uuid.UUID, string, error) {
	dir := storageDir
	if dir == "" {
		dir = "storage"
	}
	if err := os.MkdirAll(filepath.Join(dir, "imports"), 0o750); err != nil {
		return uuid.Nil, "", err
	}
	jobID := uuid.New()
	path := filepath.Join(dir, "imports", jobID.String()+"_"+filepath.Base(name))
	if err := os.WriteFile(path, content, 0o640); err != nil {
		return uuid.Nil, "", err
	}
	return jobID, path, nil
}

func parseLeaveImportRows(workbook []map[string]string) []LeaveImportRow {
	rows := make([]LeaveImportRow, 0, len(workbook))
	for _, row := range workbook {
		empCode := strings.TrimSpace(row["employee_code"])
		ltCode := strings.TrimSpace(row["leave_type_code"])
		startDate := strings.TrimSpace(row["start_date"])
		endDate := strings.TrimSpace(row["end_date"])
		if empCode == "" || ltCode == "" || startDate == "" || endDate == "" {
			continue
		}
		var reason *string
		if trimmed := strings.TrimSpace(row["reason"]); trimmed != "" {
			reason = &trimmed
		}
		duration := strings.TrimSpace(row["duration_type"])
		if duration == "" {
			duration = "full_day"
		}
		rows = append(rows, LeaveImportRow{
			LineNumber:    parseRowNumber(row["_row_number"]),
			EmployeeCode:  empCode,
			LeaveTypeCode: ltCode,
			StartDate:     startDate,
			EndDate:       endDate,
			Reason:        reason,
			DurationType:  duration,
		})
	}
	return rows
}

func (h *LeaveHandler) UploadExcel(w http.ResponseWriter, r *http.Request) {
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || !hasPermission(r, h.Authorizer, "leave:requests:submit") {
		writeForbidden(w, r)
		return
	}
	name, content, err := readAttendanceMultipart(r, false)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	jobID, path, err := saveLeaveImportFile(h.StorageDir, name, content)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	ir := h.importRepo()
	if ir == nil {
		apperror.WriteJSON(w, http.StatusOK, map[string]any{"job_id": jobID, "status": "enqueued"})
		return
	}
	job, err := ir.CreateLeaveImportJob(r.Context(), user, path)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	workbook, err := parseEmployeeWorkbook(content, []string{"employee_code", "leave_type_code", "start_date", "end_date"})
	if err != nil {
		writeValidation(w, r)
		return
	}
	autoApprove := hasAnyPermission(r, h.Authorizer, "leave:requests:approve", "leave:approve:l1", "leave:approve:l2")
	if err = ir.ProcessLeaveImport(r.Context(), user, job, parseLeaveImportRows(workbook), autoApprove); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]any{"job_id": job.ID, "status": "enqueued"})
}

func (h *LeaveHandler) LatestImport(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:requests:submit") {
		writeForbidden(w, r)
		return
	}
	ir := h.importRepo()
	if ir == nil {
		apperror.WriteJSON(w, http.StatusOK, nil)
		return
	}
	job, err := ir.GetLatestLeaveImport(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, job)
}

func (h *LeaveHandler) ImportErrors(w http.ResponseWriter, r *http.Request) {

	jobID, jobOK := pathUUID(r, "job_id")
	if !jobOK || !hasPermission(r, h.Authorizer, "leave:requests:submit") {
		writeForbidden(w, r)
		return
	}
	ir := h.importRepo()
	if ir == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}
	data, err := ir.GetLeaveImportErrorFile(r.Context(), jobID)
	if err != nil {
		if errors.Is(err, ErrNotFound) || errors.Is(err, os.ErrNotExist) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyErrorFileNotFound))
			return
		}
		h.writeError(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", `attachment; filename="leave_import_`+jobID.String()+`_errors.xlsx"`)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(data)
}

func (h *LeaveHandler) UploadTemplate(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:requests:submit") {
		writeForbidden(w, r)
		return
	}
	types, err := h.Repository.ListLeaveTypes(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	typeRows := [][]string{{"code", "name", "is_paid"}}
	for _, t := range types {
		paid := "false"
		if t.IsPaid {
			paid = "true"
		}
		typeRows = append(typeRows, []string{t.Code, t.Name, paid})
	}
	templateBytes, err := writeXLSX([]xlsxSheet{
		{
			Name:   "Leave Requests",
			Widths: []float64{20, 20, 15, 15, 30, 15},
			Rows: [][]string{
				{"employee_code", "leave_type_code", "start_date", "end_date", "reason", "duration_type"},
				{"EMP001", "annual", time.Now().Format("2006-01-02"), time.Now().Format("2006-01-02"), "Sample leave", "full_day"},
			},
		},
		{
			Name:   "Leave Types",
			Widths: []float64{20, 30, 15},
			Rows:   typeRows,
		},
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", `attachment; filename="leave_import_template.xlsx"`)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(templateBytes)
}
