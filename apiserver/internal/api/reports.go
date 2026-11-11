package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strconv"
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
	RequesterID      uuid.UUID
	Status           string
	TotalBatches     int
	CompletedBatches int
	FailedBatches    int
	Error            *string
	CreatedAt        time.Time
	CompletedAt      *time.Time
}
type AttendanceSummaryRow struct {
	EmploymentID         uuid.UUID  `json:"employment_id"`
	EmployeeID           uuid.UUID  `json:"employee_id"`
	EmployeeName         string     `json:"employee_name"`
	EmployeeIDNumber     string     `json:"employee_id_number"`
	TotalStdUnits        int        `json:"total_std_units"`
	TotalOTUnits         int        `json:"total_ot_units"`
	TotalLeaveDays       int64      `json:"total_leave_days"`
	UnitType             string     `json:"unit_type"`
	UnitCoefficientHours float64    `json:"unit_coefficient_hours"`
	ComputedStdHours     float64    `json:"computed_std_hours"`
	ComputedOTHours      float64    `json:"computed_ot_hours"`
	EmployerID           *uuid.UUID `json:"employer_id"`
	EmployerName         *string    `json:"employer_name"`
}

type DailyAttendanceRow struct {
	Date              string    `json:"date"`
	EmploymentID      uuid.UUID `json:"employment_id"`
	EmployeeID        uuid.UUID `json:"employee_id"`
	EmployeeName      string    `json:"employee_name"`
	EmployeeIDNumber  string    `json:"employee_id_number"`
	ShiftSymbol       *string   `json:"shift_symbol,omitempty"`
	ShiftName         *string   `json:"shift_name,omitempty"`
	ScheduledStart    *string   `json:"scheduled_start,omitempty"`
	ScheduledEnd      *string   `json:"scheduled_end,omitempty"`
	ActualPunchIn     *string   `json:"actual_punch_in,omitempty"`
	ActualPunchOut    *string   `json:"actual_punch_out,omitempty"`
	ActualWorkMinutes int       `json:"actual_work_minutes"`
	ComputedStdHours  float64   `json:"computed_std_hours"`
	ComputedOTHours   float64   `json:"computed_ot_hours"`
	StdWorkUnits      float64   `json:"std_work_units"`
	OTWorkUnits       float64   `json:"ot_work_units"`
	OTClaimStatus     string    `json:"ot_claim_status"`
	Status            string    `json:"status"`
	Timezone          *string   `json:"timezone,omitempty"`
}

type NoShowReportRow struct {
	DailyAttendanceRow
	RecordingIssue string  `json:"recording_issue"`
	ExpectedEndAt  *string `json:"expected_end_at,omitempty"`
}

type AttendanceReportFilter struct {
	StartDate      string
	EndDate        string
	ReportType     string
	Page           int
	Limit          int
	Sort           *string
	EmployeeIDs    []uuid.UUID
	DepartmentID   *uuid.UUID
	EmployeeStatus *string
	EmployerID     *uuid.UUID
	EmploymentID   *uuid.UUID
	ShiftType      *string
	SegmentCentric bool
}

type NoShowReportFilter struct {
	Date         string
	Page         int
	Limit        int
	EmployeeIDs  []uuid.UUID
	DepartmentID *uuid.UUID
	ShiftType    *string
}

type PaginatedReportResponse struct {
	Data      json.RawMessage `json:"data"`
	Total     int64           `json:"total"`
	Page      int             `json:"page"`
	Limit     int             `json:"limit"`
	Truncated *bool           `json:"truncated,omitempty"`
}

type ReportRepository interface {
	CreateExportJob(context.Context, ReportJob, ExportReportRequest) error
	GetAttendanceReport(context.Context, AttendanceReportFilter) (PaginatedReportResponse, error)
	GetNoShowReport(context.Context, NoShowReportFilter) (PaginatedReportResponse, error)
	ResolveBehalfEmployeeIDs(context.Context, uuid.UUID) ([]uuid.UUID, error)
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
	Authorizer Authorizer
}

func (h ReportHandler) CreateExport(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || !h.allowed(r, userID) {
		h.forbidden(w, r)
		return
	}
	var request ExportReportRequest
	if json.NewDecoder(r.Body).Decode(&request) != nil || validateExportRequest(&request) != nil {
		h.validation(w, r)
		return
	}
	job := ReportJob{JobID: uuid.New(), RequesterID: userID, Status: "pending", TotalBatches: 1, CreatedAt: time.Now().UTC()}
	if len(request.EmployeeIDs) > 50 {
		job.TotalBatches = (len(request.EmployeeIDs) + 49) / 50
	}
	if h.Repository == nil || h.Queue == nil || h.Repository.CreateExportJob(r.Context(), job, request) != nil || h.Queue.EnqueueReport(r.Context(), job, request) != nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report export unavailable")))
		return
	}
	apperror.WriteJSON(w, http.StatusAccepted, ExportJobResponse{JobID: job.JobID, Status: job.Status, TotalBatches: job.TotalBatches, EstimatedSeconds: job.TotalBatches, StatusURL: "/api/v1/reports/jobs/" + job.JobID.String()})
}

func (h ReportHandler) Status(w http.ResponseWriter, r *http.Request) {
	jobID, jobOK := pathUUID(r, "job_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !jobOK || !userOK || !h.allowed(r, userID) {
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
	if job.RequesterID != userID {
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
	jobID, jobOK := pathUUID(r, "job_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !jobOK || !userOK || !h.allowed(r, userID) {
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
	if job.RequesterID != userID {
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
	jobID, jobOK := pathUUID(r, "job_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !jobOK || !userOK || !h.allowed(r, userID) {
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
	if job.RequesterID != userID {
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

func (h ReportHandler) AttendanceReport(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		h.validation(w, r)
		return
	}
	hasReportAccess := h.Authorizer != nil && (h.Authorizer.HasPermission(r.Context(), userID, "reports:read") ||
		h.Authorizer.HasPermission(r.Context(), userID, "overtime:approve") ||
		h.Authorizer.HasPermission(r.Context(), userID, "attendance:summaries:read"))
	isGlobal := hasReportAccess && h.Authorizer.IsGlobalManager(r.Context(), userID)
	if !isGlobal {
		hasBehalf := h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, "overtime:claim_on_behalf")
		if !hasBehalf {
			h.forbidden(w, r)
			return
		}
	}
	startDateStr := r.URL.Query().Get("start_date")
	endDateStr := r.URL.Query().Get("end_date")
	if startDateStr == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidStartDate))
		return
	}
	start, err := time.Parse("2006-01-02", startDateStr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidStartDate))
		return
	}
	if endDateStr == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidEndDate))
		return
	}
	end, err := time.Parse("2006-01-02", endDateStr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidEndDate))
		return
	}
	if end.Before(start) || end.Sub(start) > 365*24*time.Hour {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateRange))
		return
	}
	reportType := r.URL.Query().Get("type")
	if reportType == "" {
		reportType = "summary"
	}
	page, limit := parseReportPagination(r)
	var employeeIDs []uuid.UUID
	if rawIDs := r.URL.Query().Get("employee_ids"); rawIDs != "" {
		for _, part := range strings.Split(rawIDs, ",") {
			trimmed := strings.TrimSpace(part)
			if id, err := uuid.Parse(trimmed); err == nil {
				employeeIDs = append(employeeIDs, id)
			}
		}
	}
	if !isGlobal && h.Repository != nil {
		behalfIDs, err := h.Repository.ResolveBehalfEmployeeIDs(r.Context(), userID)
		if err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
		if len(behalfIDs) == 0 {
			apperror.WriteJSON(w, http.StatusOK, PaginatedReportResponse{
				Data:  json.RawMessage(`[]`),
				Total: 0,
				Page:  page,
				Limit: limit,
			})
			return
		}
		if len(employeeIDs) > 0 {
			employeeIDs = intersectUUIDs(employeeIDs, behalfIDs)
			if len(employeeIDs) == 0 {
				apperror.WriteJSON(w, http.StatusOK, PaginatedReportResponse{
					Data:  json.RawMessage(`[]`),
					Total: 0,
					Page:  page,
					Limit: limit,
				})
				return
			}
		} else {
			employeeIDs = behalfIDs
		}
	}
	var deptID, employerID, employmentID *uuid.UUID
	if raw := r.URL.Query().Get("department_id"); raw != "" {
		if id, err := uuid.Parse(raw); err == nil {
			deptID = &id
		}
	}
	if raw := r.URL.Query().Get("employer_id"); raw != "" {
		if id, err := uuid.Parse(raw); err == nil {
			employerID = &id
		}
	}
	if raw := r.URL.Query().Get("employment_id"); raw != "" {
		if id, err := uuid.Parse(raw); err == nil {
			employmentID = &id
		}
	}
	var sortParam, statusParam, shiftParam *string
	if s := r.URL.Query().Get("sort"); s != "" {
		sortParam = &s
	}
	if s := r.URL.Query().Get("employee_status"); s != "" {
		statusParam = &s
	}
	if s := r.URL.Query().Get("shift_type"); s != "" {
		shiftParam = &s
	}
	segmentCentric := true
	if sc := r.URL.Query().Get("segment_centric"); sc == "false" {
		segmentCentric = false
	}
	filter := AttendanceReportFilter{
		StartDate:      startDateStr,
		EndDate:        endDateStr,
		ReportType:     reportType,
		Page:           page,
		Limit:          limit,
		Sort:           sortParam,
		EmployeeIDs:    employeeIDs,
		DepartmentID:   deptID,
		EmployeeStatus: statusParam,
		EmployerID:     employerID,
		EmploymentID:   employmentID,
		ShiftType:      shiftParam,
		SegmentCentric: segmentCentric,
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report repository unavailable")))
		return
	}
	report, err := h.Repository.GetAttendanceReport(r.Context(), filter)
	if err != nil {
		if errors.Is(err, ErrReportNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, report)
}

func (h ReportHandler) NoShowReport(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		h.validation(w, r)
		return
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "reports:read") {
		h.forbidden(w, r)
		return
	}
	dateStr := r.URL.Query().Get("date")
	if dateStr == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
		return
	}
	if _, err := time.Parse("2006-01-02", dateStr); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
		return
	}
	page, limit := parseReportPagination(r)
	var employeeIDs []uuid.UUID
	if rawIDs := r.URL.Query().Get("employee_ids"); rawIDs != "" {
		for _, part := range strings.Split(rawIDs, ",") {
			trimmed := strings.TrimSpace(part)
			if trimmed == "" {
				continue
			}
			id, err := uuid.Parse(trimmed)
			if err != nil {
				apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
				return
			}
			employeeIDs = append(employeeIDs, id)
		}
	}
	var deptID *uuid.UUID
	if raw := r.URL.Query().Get("department_id"); raw != "" {
		if id, err := uuid.Parse(raw); err == nil {
			deptID = &id
		}
	}
	var shiftType *string
	if s := r.URL.Query().Get("shift_type"); s != "" {
		shiftType = &s
	}
	filter := NoShowReportFilter{
		Date:         dateStr,
		Page:         page,
		Limit:        limit,
		EmployeeIDs:  employeeIDs,
		DepartmentID: deptID,
		ShiftType:    shiftType,
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report repository unavailable")))
		return
	}
	report, err := h.Repository.GetNoShowReport(r.Context(), filter)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, report)
}

func parseReportPagination(r *http.Request) (int, int) {
	page := 1
	limit := 20
	if rawPage := r.URL.Query().Get("page"); rawPage != "" {
		if val, err := strconv.Atoi(rawPage); err == nil && val >= 1 {
			page = val
		}
	}
	if rawLimit := r.URL.Query().Get("limit"); rawLimit != "" {
		if val, err := strconv.Atoi(rawLimit); err == nil {
			if val < 1 {
				limit = 1
			} else if val > 3000 {
				limit = 3000
			} else {
				limit = val
			}
		}
	}
	return page, limit
}

func intersectUUIDs(a, b []uuid.UUID) []uuid.UUID {
	set := make(map[uuid.UUID]struct{}, len(b))
	for _, id := range b {
		set[id] = struct{}{}
	}
	var res []uuid.UUID
	for _, id := range a {
		if _, ok := set[id]; ok {
			res = append(res, id)
		}
	}
	return res
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

func (h ReportHandler) allowed(r *http.Request, userID uuid.UUID) bool {
	return h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, "reports:export")
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
