package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrEmploymentInactive = errors.New("attendance: employment inactive")
var ErrNoActiveEmployment = errors.New("attendance: no active employment")
var ErrAttendanceAlreadyWorking = errors.New("attendance: already working")
var ErrAttendanceNotWorking = errors.New("attendance: not working")
var ErrAttendanceInvalidTransition = errors.New("attendance: invalid transition")
var ErrAttendanceBreakNotRequired = errors.New("attendance: break not required")
var ErrAttendanceBreakNotOpen = errors.New("attendance: break not open")
var ErrAttendanceIdempotencyConflict = errors.New("attendance: idempotency conflict")
var ErrInvalidAttendanceAdjustment = errors.New("attendance: invalid adjustment")
var ErrAttendancePolicyMissing = errors.New("attendance: work policy is missing")

const (
	AttendanceAdjustmentBreakWorked          = "break_worked"
	AttendanceAdjustmentMissingPunch         = "missing_punch"
	AttendanceAdjustmentAttendanceCorrection = "attendance_correction"
)

const (
	AttendanceActionWorkIn     = "work_in"
	AttendanceActionWorkOut    = "work_out"
	AttendanceActionBreakStart = "break_start"
	AttendanceActionBreakEnd   = "break_end"
)

type AttendanceActionRequest struct {
	Action string  `json:"action"`
	Note   *string `json:"note,omitempty"`
}

type AttendanceBreakRecord struct {
	ID          uuid.UUID `json:"id"`
	TimeEntryID uuid.UUID `json:"time_entry_id"`
	StartAt     string    `json:"start_at"`
	EndAt       *string   `json:"end_at,omitempty"`
	Source      string    `json:"source"`
	CloseReason *string   `json:"close_reason,omitempty"`
}

type AttendanceActionResponse struct {
	State              string                 `json:"state"`
	NextAllowedActions []string               `json:"next_allowed_actions"`
	Entry              *TimeEntryRecord       `json:"entry,omitempty"`
	Break              *AttendanceBreakRecord `json:"break,omitempty"`
	ExceptionCode      *string                `json:"exception_code,omitempty"`
	Deduplicated       bool                   `json:"deduplicated"`
}

// PunchInRequest models the self-service punch-in payload.
type PunchInRequest struct {
	EmploymentID    uuid.UUID  `json:"employment_id"`
	SegmentID       *uuid.UUID `json:"segment_id,omitempty"`
	EntryType       string     `json:"entry_type,omitempty"`
	IsBreakOvertime *bool      `json:"is_break_overtime,omitempty"`
	Note            *string    `json:"note,omitempty"`
}

// PunchOutRequest models the admin force-punch-out payload.
type PunchOutRequest struct {
	Note *string `json:"note,omitempty"`
}

// TimeHistoryQuery holds start/end range for time history queries.
type TimeHistoryQuery struct {
	Start time.Time
	End   time.Time
}

// AdjustmentRequest models a typed employee manual-adjustment submission.
type AdjustmentRequest struct {
	EmploymentID       uuid.UUID   `json:"employment_id"`
	Type               string      `json:"type,omitempty"`
	TimeEntryID        *uuid.UUID  `json:"time_entry_id,omitempty"`
	SegmentID          *uuid.UUID  `json:"segment_id,omitempty"`
	RequestedPunchTime string      `json:"requested_punch_time,omitempty"`
	Direction          string      `json:"direction,omitempty"`
	Start              string      `json:"start,omitempty"`
	End                string      `json:"end,omitempty"`
	CorrectedPunchIn   string      `json:"corrected_punch_in,omitempty"`
	CorrectedPunchOut  string      `json:"corrected_punch_out,omitempty"`
	Reason             *string     `json:"reason,omitempty"`
	AttachmentFileIDs  []uuid.UUID `json:"attachment_file_ids,omitempty"`
}

// AdjustmentRecord models a pending adjustment in the API response.
type AdjustmentRecord struct {
	ID                 uuid.UUID   `json:"id"`
	EmploymentID       uuid.UUID   `json:"employment_id"`
	Type               string      `json:"type"`
	TimeEntryID        *uuid.UUID  `json:"time_entry_id"`
	SegmentID          *uuid.UUID  `json:"segment_id,omitempty"`
	RequestedPunchTime string      `json:"requested_punch_time"`
	Direction          *string     `json:"direction,omitempty"`
	Start              *string     `json:"start,omitempty"`
	End                *string     `json:"end,omitempty"`
	CorrectedPunchIn   *string     `json:"corrected_punch_in,omitempty"`
	CorrectedPunchOut  *string     `json:"corrected_punch_out,omitempty"`
	Reason             *string     `json:"reason"`
	Status             string      `json:"status"`
	RequestedByUserID  *uuid.UUID  `json:"requested_by_user_id"`
	ApprovedByUserID   *uuid.UUID  `json:"approved_by_user_id"`
	CreatedAt          string      `json:"created_at"`
	UpdatedAt          string      `json:"updated_at"`
	AttachmentFileIDs  []uuid.UUID `json:"attachment_file_ids,omitempty"`
}

// PaginatedAdjustments is the adjustment listing pagination contract.
type PaginatedAdjustments struct {
	Data  []AdjustmentRecord `json:"data"`
	Total int64              `json:"total"`
	Page  int                `json:"page"`
	Limit int                `json:"limit"`
}

// ApproveAdjustmentRequest carries the terminal adjustment status.
type ApproveAdjustmentRequest struct {
	Status string `json:"status"`
}

// TimeEntryRecord models the JSON contract for an attendance time entry.
type TimeEntryRecord struct {
	ID                 uuid.UUID       `json:"id"`
	EmploymentID       uuid.UUID       `json:"employment_id"`
	ShiftID            *uuid.UUID      `json:"shift_id"`
	SegmentID          *uuid.UUID      `json:"segment_id"`
	ShiftVersionID     *uuid.UUID      `json:"shift_version_id,omitempty"`
	BreaksSnapshot     json.RawMessage `json:"breaks_snapshot,omitempty"`
	PunchInTime        string          `json:"punch_in_time"`
	PunchOutTime       *string         `json:"punch_out_time"`
	DurationMinutes    *int            `json:"duration_minutes"`
	IsBreakOvertime    bool            `json:"is_break_overtime"`
	EntryType          string          `json:"entry_type"`
	Note               *string         `json:"note"`
	GenerationMetadata json.RawMessage `json:"generation_metadata"`
	CreatedAt          string          `json:"created_at"`
	UpdatedAt          string          `json:"updated_at"`
}

// UpdateTimeEntryRequest models partial update for time entry.
type UpdateTimeEntryRequest struct {
	PunchInTime  *string    `json:"punch_in_time,omitempty"`
	PunchOutTime *string    `json:"punch_out_time,omitempty"`
	EntryType    *string    `json:"entry_type,omitempty"`
	Note         *string    `json:"note,omitempty"`
	SegmentID    *uuid.UUID `json:"segment_id,omitempty"`
}

// AttendanceDailySummaryRecord models the JSON response for daily summaries.
type AttendanceDailySummaryRecord struct {
	ID                      uuid.UUID  `json:"id"`
	EmploymentID            uuid.UUID  `json:"employment_id"`
	Date                    string     `json:"date"`
	ActualWorkMinutes       int        `json:"actual_work_minutes"`
	StandardMinutes         int        `json:"standard_minutes"`
	SystemDetectedOTMinutes int        `json:"system_detected_ot_minutes"`
	ClaimedOTMinutes        int        `json:"claimed_ot_minutes"`
	ApprovedOTMinutes       int        `json:"approved_ot_minutes"`
	EarlyOTMinutes          int        `json:"early_ot_minutes"`
	LateOTMinutes           int        `json:"late_ot_minutes"`
	BreakOTMinutes          int        `json:"break_ot_minutes"`
	ScheduledBreakMinutes   int        `json:"scheduled_break_minutes"`
	ActualBreakMinutes      int        `json:"actual_break_minutes"`
	DeductedBreakMinutes    int        `json:"deducted_break_minutes"`
	NetWorkMinutes          int        `json:"net_work_minutes"`
	NetPaidMinutes          int        `json:"net_paid_minutes"`
	BreakProvisional        bool       `json:"break_provisional"`
	BreakExceptionCode      *string    `json:"break_exception_code,omitempty"`
	StandardCredits         float64    `json:"standard_credits"`
	WorkPolicyVersionID     *uuid.UUID `json:"work_policy_version_id,omitempty"`
	ShiftVersionID          *uuid.UUID `json:"shift_version_id,omitempty"`
	OTStatus                string     `json:"ot_status"`
	OTReason                *string    `json:"ot_reason,omitempty"`
	ManagerNote             *string    `json:"manager_note,omitempty"`
	IsWeekend               bool       `json:"is_weekend"`
	IsHoliday               bool       `json:"is_holiday"`
	HolidayName             *string    `json:"holiday_name,omitempty"`
	CreatedAt               time.Time  `json:"created_at"`
	UpdatedAt               time.Time  `json:"updated_at"`
}

// PaginatedSummaries wraps paginated attendance daily summaries.
type PaginatedSummaries struct {
	Data  []AttendanceDailySummaryRecord `json:"data"`
	Total int64                          `json:"total"`
	Page  int                            `json:"page"`
	Limit int                            `json:"limit"`
}

// AttendanceRepository executes data operations for attendance and punch tracking.
type ReprocessResponse struct {
	EmploymentID *uuid.UUID `json:"employment_id,omitempty"`
	StartDate    string     `json:"start_date"`
	EndDate      string     `json:"end_date"`
	Days         int        `json:"days"`
	Recalculated int        `json:"recalculated"`
}

type AttendanceReprocessRepository interface {
	ReprocessAttendance(ctx context.Context, startDate, endDate string, employeeID *uuid.UUID) (ReprocessResponse, error)
}

type AttendanceRepository interface {
	PunchIn(ctx context.Context, employmentID uuid.UUID, segmentID *uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (TimeEntryRecord, error)
	PunchOut(ctx context.Context, employeeID uuid.UUID, note *string, punchTime time.Time) (TimeEntryRecord, error)
	ListTimeHistory(ctx context.Context, employeeID uuid.UUID, start, end time.Time) ([]TimeEntryRecord, error)
	CreateAdjustment(ctx context.Context, req AdjustmentRequest, requestedByUserID ids.UserID) (AdjustmentRecord, error)
	ListAdjustments(ctx context.Context, status *string, employeeID *uuid.UUID, page, limit int) (PaginatedAdjustments, error)
	ListEmployeeAdjustments(ctx context.Context, employeeID uuid.UUID) ([]AdjustmentRecord, error)
	ApproveAdjustment(ctx context.Context, adjustmentID, approverID uuid.UUID, status string) error
	IsEmployeeOwnedByUser(ctx context.Context, employeeID, userID uuid.UUID) (bool, error)
	ESSAttendanceAction(ctx context.Context, employeeID uuid.UUID, action string, note *string, idempotencyKey string, actionTime time.Time) (AttendanceActionResponse, error)
	GetESSAttendanceState(ctx context.Context, employeeID uuid.UUID) (AttendanceActionResponse, error)
	GetAttendanceSummaries(ctx context.Context, startDate, endDate string, employeeID *uuid.UUID, otStatus *string, page, limit int) (PaginatedSummaries, error)
	GetEmployeeIDByUserID(ctx context.Context, userID uuid.UUID) (uuid.UUID, error)
	UpdateTimeEntry(ctx context.Context, entryID uuid.UUID, req UpdateTimeEntryRequest) (TimeEntryRecord, error)
	DeleteTimeEntry(ctx context.Context, entryID uuid.UUID) error
}

// AttendanceHandler serves attendance endpoints.
type AttendanceHandler struct {
	Repository AttendanceRepository
	Authorizer Authorizer
}

// PunchIn handles POST /api/v1/time/punch-in.
func (h *AttendanceHandler) PunchIn(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "ess:clock") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[PunchInRequest](r, 1<<20)
	if err != nil || request.EmploymentID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	entryType := request.EntryType
	if entryType == "" {
		entryType = "work"
	}
	entry, err := h.Repository.PunchIn(r.Context(), request.EmploymentID, request.SegmentID, entryType, request.Note, request.IsBreakOvertime != nil && *request.IsBreakOvertime, time.Now().UTC())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, entry)
}

// PunchOut handles POST /api/v1/time/punch-out/{employee_id}.
// Requires attendance:force_clock_out; closes the active time entry for the target employee.
func (h *AttendanceHandler) PunchOut(w http.ResponseWriter, r *http.Request) {
	employeeID, empOK := pathUUID(r, "employee_id")
	if !empOK || !hasPermission(r, h.Authorizer, "attendance:force_clock_out") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[PunchOutRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	entry, err := h.Repository.PunchOut(r.Context(), employeeID, request.Note, time.Now().UTC())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, entry)
}

// TimeHistory handles GET /api/v1/time/history/{employee_id}.
// Admin: requires attendance:logs:read; employee: self-only.
func (h *AttendanceHandler) TimeHistory(w http.ResponseWriter, r *http.Request) {
	employeeID, empOK := pathUUID(r, "employee_id")
	if !empOK {
		writeForbidden(w, r)
		return
	}
	query, queryOK := parseTimeHistoryQuery(r)
	if !queryOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !hasPermission(r, h.Authorizer, "attendance:logs:read") {
		userID, userOK := httpinfra.UserIDFromContext(r.Context())
		if !userOK {
			writeForbidden(w, r)
			return
		}
		owned, ownerErr := h.Repository.IsEmployeeOwnedByUser(r.Context(), employeeID, userID)
		if ownerErr != nil {
			h.writeError(w, r, ownerErr)
			return
		}
		if !owned {
			writeForbidden(w, r)
			return
		}
	}
	entries, err := h.Repository.ListTimeHistory(r.Context(), employeeID, query.Start, query.End)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, entries)
}

func parseTimeHistoryQuery(r *http.Request) (TimeHistoryQuery, bool) {
	startRaw := r.URL.Query().Get("start")
	endRaw := r.URL.Query().Get("end")
	if startRaw == "" || endRaw == "" {
		return TimeHistoryQuery{}, false
	}
	start, err := time.Parse(time.RFC3339, startRaw)
	if err != nil {
		return TimeHistoryQuery{}, false
	}
	end, err := time.Parse(time.RFC3339, endRaw)
	if err != nil {
		return TimeHistoryQuery{}, false
	}
	return TimeHistoryQuery{Start: start, End: end}, true
}

// CreateAdjustment handles POST /api/v1/attendance/adjustments.
// Requires attendance:adjustments:submit; creates a pending adjustment awaiting approval.
func (h *AttendanceHandler) CreateAdjustment(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "attendance:adjustments:submit") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[AdjustmentRequest](r, 1<<20)
	if err != nil || req.EmploymentID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := validateAdjustmentRequest(req); err != nil {
		h.writeError(w, r, err)
		return
	}
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	record, err := h.Repository.CreateAdjustment(r.Context(), req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, record)
}

func validateAdjustmentRequest(req AdjustmentRequest) error {
	if req.Type == "" {
		if req.RequestedPunchTime == "" {
			return ErrInvalidAttendanceAdjustment
		}
		return nil
	}
	switch req.Type {
	case AttendanceAdjustmentBreakWorked:
		if req.SegmentID == nil || req.Start == "" || req.End == "" || req.Reason == nil || strings.TrimSpace(*req.Reason) == "" {
			return ErrInvalidAttendanceAdjustment
		}
	case AttendanceAdjustmentMissingPunch:
		if req.RequestedPunchTime == "" || (req.Direction != "in" && req.Direction != "out") || req.Reason == nil || strings.TrimSpace(*req.Reason) == "" {
			return ErrInvalidAttendanceAdjustment
		}
	case AttendanceAdjustmentAttendanceCorrection:
		if req.TimeEntryID == nil || (req.CorrectedPunchIn == "" && req.CorrectedPunchOut == "") || req.Reason == nil || strings.TrimSpace(*req.Reason) == "" {
			return ErrInvalidAttendanceAdjustment
		}
	default:
		return ErrInvalidAttendanceAdjustment
	}
	return nil
}

// ListAdjustments handles GET /api/v1/attendance/adjustments.
func (h *AttendanceHandler) ListAdjustments(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "attendance:adjustments:approve") {
		writeForbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var status *string
	if value := r.URL.Query().Get("status"); value != "" {
		if value != "pending" && value != "approved" && value != "rejected" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		status = &value
	}
	var employeeID *uuid.UUID
	if value := r.URL.Query().Get("employee_id"); value != "" {
		parsed, parseErr := uuid.Parse(value)
		if parseErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		employeeID = &parsed
	}
	result, err := h.Repository.ListAdjustments(r.Context(), status, employeeID, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// ListEmployeeAdjustments handles GET /api/v1/employees/{id}/attendance/adjustments.
func (h *AttendanceHandler) ListEmployeeAdjustments(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := pathUUID(r, "id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK {
		writeForbidden(w, r)
		return
	}
	if !hasPermission(r, h.Authorizer, "attendance:adjustments:approve") {
		owned, err := h.Repository.IsEmployeeOwnedByUser(r.Context(), employeeID, userID)
		if err != nil || !owned {
			writeForbidden(w, r)
			return
		}
	}
	result, err := h.Repository.ListEmployeeAdjustments(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// ApproveAdjustment handles PUT /api/v1/attendance/adjustments/{id}/approve.
func (h *AttendanceHandler) ApproveAdjustment(w http.ResponseWriter, r *http.Request) {
	adjustmentID, adjustmentOK := pathUUID(r, "id")
	if !adjustmentOK || !hasPermission(r, h.Authorizer, "attendance:adjustments:approve") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[ApproveAdjustmentRequest](r, 1<<20)
	if err != nil || (request.Status != "approved" && request.Status != "rejected") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.ApproveAdjustment(r.Context(), adjustmentID, userID, request.Status); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

// Summaries handles GET /api/v1/attendance/summaries.
func (h *AttendanceHandler) Summaries(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		writeForbidden(w, r)
		return
	}

	startDate := r.URL.Query().Get("start_date")
	endDate := r.URL.Query().Get("end_date")
	if startDate == "" || endDate == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	hasReadPerm := hasPermission(r, h.Authorizer, "attendance:summaries:read")
	var targetEmployeeID *uuid.UUID

	if empParam := r.URL.Query().Get("employee_id"); empParam != "" {
		parsed, err := uuid.Parse(empParam)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		targetEmployeeID = &parsed
	}

	if !hasReadPerm {
		myID, err := h.Repository.GetEmployeeIDByUserID(r.Context(), userID)
		if err != nil || myID == uuid.Nil {
			writeForbidden(w, r)
			return
		}
		if targetEmployeeID != nil && *targetEmployeeID != myID {
			writeForbidden(w, r)
			return
		}
		targetEmployeeID = &myID
	}

	var otStatus *string
	if ot := strings.TrimSpace(r.URL.Query().Get("ot_status")); ot != "" {
		otStatus = &ot
	}

	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	summaries, err := h.Repository.GetAttendanceSummaries(r.Context(), startDate, endDate, targetEmployeeID, otStatus, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, summaries)
}

// ReprocessAll handles POST /api/v1/attendance/reprocess-all.
func (h *AttendanceHandler) ReprocessAll(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "attendance:reprocess") {
		writeForbidden(w, r)
		return
	}
	startDateStr := r.URL.Query().Get("start_date")
	endDateStr := r.URL.Query().Get("end_date")
	if startDateStr == "" || endDateStr == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	start, err1 := time.Parse("2006-01-02", startDateStr)
	end, err2 := time.Parse("2006-01-02", endDateStr)
	if err1 != nil || err2 != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	days := int(end.Sub(start).Hours() / 24)
	if days < 0 || days > 90 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reprocessor, ok := h.Repository.(AttendanceReprocessRepository)
	if !ok {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	result, err := reprocessor.ReprocessAttendance(r.Context(), startDateStr, endDateStr, nil)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// ReprocessEmployee handles POST /api/v1/attendance/reprocess/{employee_id}.
func (h *AttendanceHandler) ReprocessEmployee(w http.ResponseWriter, r *http.Request) {
	employeeID, empOK := pathUUID(r, "employee_id")
	if !empOK || !hasPermission(r, h.Authorizer, "attendance:reprocess") {
		writeForbidden(w, r)
		return
	}
	startDateStr := r.URL.Query().Get("start_date")
	endDateStr := r.URL.Query().Get("end_date")
	if startDateStr == "" || endDateStr == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	start, err1 := time.Parse("2006-01-02", startDateStr)
	end, err2 := time.Parse("2006-01-02", endDateStr)
	if err1 != nil || err2 != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	days := int(end.Sub(start).Hours() / 24)
	if days < 0 || days > 90 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reprocessor, ok := h.Repository.(AttendanceReprocessRepository)
	if !ok {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	result, err := reprocessor.ReprocessAttendance(r.Context(), startDateStr, endDateStr, &employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// UpdateTimeEntry handles PUT /api/v1/time/entries/{id}.
func (h *AttendanceHandler) UpdateTimeEntry(w http.ResponseWriter, r *http.Request) {
	entryID, entryOK := pathUUID(r, "id")
	if !entryOK || !hasPermission(r, h.Authorizer, "attendance:logs:import") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateTimeEntryRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	entry, err := h.Repository.UpdateTimeEntry(r.Context(), entryID, req)
	if err != nil {
		if errors.Is(err, ErrNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, entry)
}

// DeleteTimeEntry handles DELETE /api/v1/time/entries/{id}.
func (h *AttendanceHandler) DeleteTimeEntry(w http.ResponseWriter, r *http.Request) {
	entryID, entryOK := pathUUID(r, "id")
	if !entryOK || !hasPermission(r, h.Authorizer, "attendance:logs:import") {
		writeForbidden(w, r)
		return
	}
	err := h.Repository.DeleteTimeEntry(r.Context(), entryID)
	if err != nil {
		if errors.Is(err, ErrNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *AttendanceHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if mapped := mapLifecycleGuardError(err); mapped != nil {
		apperror.WriteError(w, r, mapped)
		return
	}
	writeMappedError(w, r, err, func(err error) *apperror.AppError {
		switch {
		case errors.Is(err, ErrInvalidAttendanceAdjustment), errors.Is(err, ErrNoActiveEmployment), errors.Is(err, ErrAttendancePolicyMissing):
			return apperror.Validation(apperror.KeyValidation)
		case errors.Is(err, ErrEmploymentInactive):
			return apperror.Forbidden(apperror.KeyForbidden)
		case errors.Is(err, ErrAttendanceLockedPeriod):
			return apperror.Conflict("error.locked_period")
		case errors.Is(err, ErrAttendanceIdempotencyConflict):
			return apperror.Conflict(apperror.KeyConflict)
		case errors.Is(err, ErrNotFound), errors.Is(err, ErrWorkSegmentNotFound):
			return apperror.NotFound(apperror.KeyNotFound)
		default:
			return nil
		}
	})
}
