package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
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
	ErrReportExportTooLarge = errors.New("report: export exceeds synchronous limit")
	ErrReportNotFound       = errors.New("report: not found")
)

const maxExportRequestBytes = 1 << 20

type ExportReportRequest struct {
	ReportType      string                 `json:"report_type"`
	StartDate       string                 `json:"start_date"`
	EndDate         string                 `json:"end_date"`
	EmployeeIDs     []uuid.UUID            `json:"employee_ids,omitempty"`
	Format          string                 `json:"format,omitempty"`
	TenureSnapshots []ExportTenureSnapshot `json:"tenure_snapshots,omitempty"`
}
type ExportTenureSnapshot struct {
	EmployeeID     uuid.UUID `json:"employee_id"`
	EmployeeNumber string    `json:"employee_number"`
	StartDate      string    `json:"start_date"`
	EndDate        *string   `json:"end_date,omitempty"`
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
	StandardCredits      float64    `json:"standard_credits"`
	EmployerID           *uuid.UUID `json:"employer_id"`
	EmployerName         *string    `json:"employer_name"`
}

type DailyAttendanceRow struct {
	Date                  string    `json:"date"`
	EmploymentID          uuid.UUID `json:"employment_id"`
	EmployeeID            uuid.UUID `json:"employee_id"`
	EmployeeName          string    `json:"employee_name"`
	EmployeeIDNumber      string    `json:"employee_id_number"`
	ShiftSymbol           *string   `json:"shift_symbol,omitempty"`
	ShiftName             *string   `json:"shift_name,omitempty"`
	ScheduledStart        *string   `json:"scheduled_start,omitempty"`
	ScheduledEnd          *string   `json:"scheduled_end,omitempty"`
	ActualPunchIn         *string   `json:"actual_punch_in,omitempty"`
	ActualPunchOut        *string   `json:"actual_punch_out,omitempty"`
	ActualWorkMinutes     int       `json:"actual_work_minutes"`
	ScheduledBreakMinutes int       `json:"scheduled_break_minutes"`
	ActualBreakMinutes    int       `json:"actual_break_minutes"`
	DeductedBreakMinutes  int       `json:"deducted_break_minutes"`
	NetWorkMinutes        int       `json:"net_work_minutes"`
	NetPaidMinutes        int       `json:"net_paid_minutes"`
	BreakProvisional      bool      `json:"break_provisional"`
	BreakExceptionCode    *string   `json:"break_exception_code,omitempty"`
	ComputedStdHours      float64   `json:"computed_std_hours"`
	ComputedOTHours       float64   `json:"computed_ot_hours"`
	StdWorkUnits          float64   `json:"std_work_units"`
	OTWorkUnits           float64   `json:"ot_work_units"`
	StandardCredits       float64   `json:"standard_credits"`
	OTClaimStatus         string    `json:"ot_claim_status"`
	Status                string    `json:"status"`
	Timezone              *string   `json:"timezone,omitempty"`
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
	ExportAttendanceReport(context.Context, ExportReportRequest) ([]byte, string, error)
	GetAttendanceReport(context.Context, AttendanceReportFilter) (PaginatedReportResponse, error)
	GetNoShowReport(context.Context, NoShowReportFilter) (PaginatedReportResponse, error)
	ResolveBehalfEmployeeIDs(context.Context, uuid.UUID) ([]uuid.UUID, error)
}
type ReportHandler struct {
	Repository ReportRepository
	Authorizer Authorizer
}

func (h *ReportHandler) CreateExport(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || !h.allowed(r, userID) {
		writeForbidden(w, r)
		return
	}
	if r.ContentLength > maxExportRequestBytes {
		apperror.WriteError(w, r, apperror.PayloadTooLarge(apperror.KeyFileTooLarge))
		return
	}
	var request ExportReportRequest
	decoder := json.NewDecoder(io.LimitReader(r.Body, maxExportRequestBytes+1))
	if decoder.Decode(&request) != nil || validateExportRequest(&request) != nil {
		h.validation(w, r)
		return
	}
	if decoder.Decode(&struct{}{}) != io.EOF {
		h.validation(w, r)
		return
	}
	if len(request.EmployeeIDs) > 1000 {
		apperror.WriteError(w, r, apperror.PayloadTooLarge(apperror.KeyFileTooLarge))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("report repository unavailable")))
		return
	}
	var err error
	request, err = h.scopeExportRequest(r.Context(), userID, request)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if len(request.EmployeeIDs) > 1000 {
		apperror.WriteError(w, r, apperror.PayloadTooLarge(apperror.KeyFileTooLarge))
		return
	}
	if resolver, ok := h.Repository.(interface {
		SnapshotExportRequest(context.Context, ExportReportRequest) (ExportReportRequest, error)
	}); ok {
		request, err = resolver.SnapshotExportRequest(r.Context(), request)
		if err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}
	data, filename, err := h.Repository.ExportAttendanceReport(r.Context(), request)
	if errors.Is(err, ErrReportExportTooLarge) {
		apperror.WriteError(w, r, apperror.PayloadTooLarge(apperror.KeyFileTooLarge))
		return
	}
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", fmt.Sprintf(`attachment; filename="%s"`, filename))
	w.Header().Set("X-Content-Type-Options", "nosniff")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(data)
}

func (h *ReportHandler) scopeExportRequest(ctx context.Context, userID uuid.UUID, request ExportReportRequest) (ExportReportRequest, error) {
	if h.Authorizer == nil || h.Authorizer.IsGlobalManager(ctx, userID) {
		return request, nil
	}
	behalfIDs, err := h.Repository.ResolveBehalfEmployeeIDs(ctx, userID)
	if err != nil {
		return request, err
	}
	if len(request.EmployeeIDs) == 0 {
		request.EmployeeIDs = behalfIDs
	} else {
		request.EmployeeIDs = intersectUUIDs(request.EmployeeIDs, behalfIDs)
	}
	return request, nil
}

func (h *ReportHandler) AttendanceReport(w http.ResponseWriter, r *http.Request) {
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
			writeForbidden(w, r)
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

func (h *ReportHandler) NoShowReport(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		h.validation(w, r)
		return
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, "reports:read") {
		writeForbidden(w, r)
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
	if request.ReportType != "summary" && request.ReportType != "detail" || request.Format != "" && request.Format != "xlsx" {
		return ErrReportValidation
	}
	request.Format = "xlsx"
	start, startErr := time.Parse("2006-01-02", request.StartDate)
	end, endErr := time.Parse("2006-01-02", request.EndDate)
	if startErr != nil || endErr != nil || end.Before(start) || end.Sub(start) > 366*24*time.Hour {
		return ErrReportValidation
	}
	return nil
}
func (h *ReportHandler) validation(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
}

func (h *ReportHandler) allowed(r *http.Request, userID uuid.UUID) bool {
	if h.Authorizer == nil {
		return false
	}
	return h.Authorizer.HasPermission(r.Context(), userID, "reports:export") ||
		h.Authorizer.HasPermission(r.Context(), userID, "overtime:claim_on_behalf")
}
