package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrReportValidation     = errors.New("report: validation failed")
	ErrReportNotFound       = errors.New("report: job not found")
	ErrReportCannotCancel   = errors.New("report: cannot cancel completed or failed job")
	ErrReportResultNotFound = errors.New("report: result file not found")
)

type ExportReportRequest struct {
	ReportType  string      `json:"report_type"`
	StartDate   string      `json:"start_date"`
	EndDate     string      `json:"end_date"`
	EmployeeIDs []uuid.UUID `json:"employee_ids,omitempty"`
	Format      string      `json:"format,omitempty"`
}
type ExportJobResponse struct {
	JobID            uuid.UUID `json:"job_id"`
	Status           string    `json:"status"`
	TotalBatches     int       `json:"total_batches"`
	EstimatedSeconds int       `json:"estimated_seconds"`
	StatusURL        string    `json:"status_url"`
}
type JobProgress struct {
	TotalBatches     int     `json:"total_batches"`
	CompletedBatches int     `json:"completed_batches"`
	FailedBatches    int     `json:"failed_batches"`
	Percentage       float64 `json:"percentage"`
}
type JobStatusResponse struct {
	JobID       uuid.UUID   `json:"job_id"`
	Status      string      `json:"status"`
	Progress    JobProgress `json:"progress"`
	Error       *string     `json:"error"`
	CreatedAt   string      `json:"created_at"`
	CompletedAt *string     `json:"completed_at"`
}
type ReportJob struct {
	JobID            uuid.UUID
	WorkspaceID      uuid.UUID
	RequesterID      uuid.UUID
	Status           string
	TotalBatches     int
	CompletedBatches int
	FailedBatches    int
	Error            *string
	CreatedAt        time.Time
	CompletedAt      *time.Time
}
type ReportRepository interface {
	CreateExportJob(context.Context, ReportJob, ExportReportRequest) error
}
type ReportQueue interface {
	EnqueueReport(context.Context, ReportJob, ExportReportRequest) error
	GetReport(context.Context, uuid.UUID) (ReportJob, error)
	CancelReport(context.Context, uuid.UUID) error
	GetResultData(context.Context, uuid.UUID) ([]byte, string, error)
}
type ReportHandler struct {
	Repository ReportRepository
	Queue      ReportQueue
	Authorizer WorkspaceAuthorizer
}

func (h ReportHandler) CreateExport(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, wsID, "reports:export") {
		h.forbidden(w, r)
		return
	}
	var request ExportReportRequest
	if json.NewDecoder(r.Body).Decode(&request) != nil {
		h.validation(w, r)
		return
	}
	if err := validateExportRequest(&request); err != nil {
		h.validation(w, r)
		return
	}
	job := ReportJob{JobID: uuid.New(), WorkspaceID: wsID, RequesterID: userID, Status: "pending", TotalBatches: 1, CreatedAt: time.Now().UTC()}
	if len(request.EmployeeIDs) > 50 {
		job.TotalBatches = (len(request.EmployeeIDs) + 49) / 50
	}
	if h.Repository == nil || h.Queue == nil || h.Repository.CreateExportJob(r.Context(), job, request) != nil || h.Queue.EnqueueReport(r.Context(), job, request) != nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report export unavailable")))
		return
	}
	apperror.WriteJSON(w, http.StatusAccepted, ExportJobResponse{JobID: job.JobID, Status: job.Status, TotalBatches: job.TotalBatches, EstimatedSeconds: job.TotalBatches, StatusURL: "/api/v1/workspaces/" + wsID.String() + "/reports/jobs/" + job.JobID.String()})
}

func (h ReportHandler) Status(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	jobID, jobOK := pathUUID(r, "job_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !jobOK || !userOK || !h.allowed(r, wsID, userID) {
		h.forbidden(w, r)
		return
	}
	if h.Queue == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report queue unavailable")))
		return
	}
	job, err := h.Queue.GetReport(r.Context(), jobID)
	if err != nil {
		h.writeReportError(w, r, err)
		return
	}
	if job.WorkspaceID != wsID {
		h.forbidden(w, r)
		return
	}
	percentage := 0.0
	if job.TotalBatches > 0 {
		percentage = float64(job.CompletedBatches) * 100 / float64(job.TotalBatches)
	}
	response := JobStatusResponse{JobID: job.JobID, Status: job.Status, Progress: JobProgress{TotalBatches: job.TotalBatches, CompletedBatches: job.CompletedBatches, FailedBatches: job.FailedBatches, Percentage: percentage}, Error: job.Error, CreatedAt: job.CreatedAt.UTC().Format(time.RFC3339), CompletedAt: formatTime(job.CompletedAt)}
	apperror.WriteJSON(w, http.StatusOK, response)
}

func (h ReportHandler) Cancel(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	jobID, jobOK := pathUUID(r, "job_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !jobOK || !userOK || !h.allowed(r, wsID, userID) {
		h.forbidden(w, r)
		return
	}
	if h.Queue == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report queue unavailable")))
		return
	}
	job, err := h.Queue.GetReport(r.Context(), jobID)
	if err != nil {
		h.writeReportError(w, r, err)
		return
	}
	if job.WorkspaceID != wsID {
		h.forbidden(w, r)
		return
	}
	if err := h.Queue.CancelReport(r.Context(), jobID); err != nil {
		h.writeReportError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ReportHandler) Download(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	jobID, jobOK := pathUUID(r, "job_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !jobOK || !userOK || !h.allowed(r, wsID, userID) {
		h.forbidden(w, r)
		return
	}
	if h.Queue == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report queue unavailable")))
		return
	}
	job, err := h.Queue.GetReport(r.Context(), jobID)
	if err != nil {
		h.writeReportError(w, r, err)
		return
	}
	if job.WorkspaceID != wsID {
		h.forbidden(w, r)
		return
	}
	switch job.Status {
	case "pending", "processing":
		w.WriteHeader(http.StatusAccepted)
		_, _ = w.Write([]byte("Job not yet complete. Continue polling status endpoint."))
	case "failed":
		apperror.WriteError(w, r, apperror.Internal(errors.New("report job failed")))
	case "completed":
		data, filename, err := h.Queue.GetResultData(r.Context(), jobID)
		if err != nil {
			h.writeReportError(w, r, err)
			return
		}
		contentType := "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
		if strings.HasSuffix(strings.ToLower(filename), ".csv") {
			contentType = "text/csv"
		}
		w.Header().Set("Content-Type", contentType)
		w.Header().Set("Content-Disposition", fmt.Sprintf(`attachment; filename="%s"`, filename))
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write(data)
	default:
		apperror.WriteError(w, r, apperror.Internal(fmt.Errorf("unknown report job status: %s", job.Status)))
	}
}

func validateExportRequest(request *ExportReportRequest) error {
	if request.ReportType != "summary" && request.ReportType != "detail" || request.Format != "" && request.Format != "xlsx" && request.Format != "csv" {
		return ErrReportValidation
	}
	if request.Format == "" {
		request.Format = "xlsx"
	}
	start, startErr := time.Parse("2006-01-02", request.StartDate)
	end, endErr := time.Parse("2006-01-02", request.EndDate)
	if startErr != nil || endErr != nil || end.Before(start) || end.Sub(start) > 366*24*time.Hour {
		return ErrReportValidation
	}
	return nil
}
func formatTime(value *time.Time) *string {
	if value == nil {
		return nil
	}
	formatted := value.UTC().Format(time.RFC3339)
	return &formatted
}
func (h ReportHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h ReportHandler) validation(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
}

func (h ReportHandler) allowed(r *http.Request, wsID, userID uuid.UUID) bool {
	return h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, wsID, "reports:export")
}

func (h ReportHandler) writeReportError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrReportNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
	case errors.Is(err, ErrReportCannotCancel):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyCannotCancelJob))
	case errors.Is(err, ErrReportResultNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNoReportFileGenerated))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
