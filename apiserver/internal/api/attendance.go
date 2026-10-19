package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrEmploymentInactive = errors.New("attendance: employment inactive")
var ErrNoActiveEmployment = errors.New("attendance: no active employment")

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

// AdjustmentRequest models an employee manual-adjustment submission.
type AdjustmentRequest struct {
	EmploymentID       uuid.UUID   `json:"employment_id"`
	TimeEntryID        *uuid.UUID  `json:"time_entry_id,omitempty"`
	RequestedPunchTime string      `json:"requested_punch_time"`
	Reason             *string     `json:"reason,omitempty"`
	AttachmentFileIDs  []uuid.UUID `json:"attachment_file_ids,omitempty"`
}

// AdjustmentRecord models a pending adjustment in the API response.
type AdjustmentRecord struct {
	ID                 uuid.UUID   `json:"id"`
	EmploymentID       uuid.UUID   `json:"employment_id"`
	TimeEntryID        *uuid.UUID  `json:"time_entry_id"`
	RequestedPunchTime string      `json:"requested_punch_time"`
	Reason             *string     `json:"reason"`
	Status             string      `json:"status"`
	RequestedByUserID  *uuid.UUID  `json:"requested_by_user_id"`
	ApprovedByUserID   *uuid.UUID  `json:"approved_by_user_id"`
	CreatedAt          string      `json:"created_at"`
	UpdatedAt          string      `json:"updated_at"`
	AttachmentFileIDs  []uuid.UUID `json:"attachment_file_ids,omitempty"`
}

// PaginatedAdjustments is the workspace adjustment listing contract.
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
	WorkspaceID        uuid.UUID       `json:"workspace_id"`
	ShiftID            *uuid.UUID      `json:"shift_id"`
	SegmentID          *uuid.UUID      `json:"segment_id"`
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

// AttendanceRepository executes data operations for attendance and punch tracking.
type AttendanceRepository interface {
	PunchIn(ctx context.Context, workspaceID, employmentID uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (TimeEntryRecord, error)
	PunchOut(ctx context.Context, workspaceID, employeeID uuid.UUID, note *string, punchTime time.Time) (TimeEntryRecord, error)
	ListTimeHistory(ctx context.Context, workspaceID, employeeID uuid.UUID, start, end time.Time) ([]TimeEntryRecord, error)
	CreateAdjustment(ctx context.Context, workspaceID uuid.UUID, req AdjustmentRequest, requestedByUserID uuid.UUID) (AdjustmentRecord, error)
	ListAdjustments(ctx context.Context, workspaceID uuid.UUID, status *string, employeeID *uuid.UUID, page, limit int) (PaginatedAdjustments, error)
	ListEmployeeAdjustments(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]AdjustmentRecord, error)
	ApproveAdjustment(ctx context.Context, workspaceID, adjustmentID, approverID uuid.UUID, status string) error
	IsEmployeeOwnedByUser(ctx context.Context, workspaceID, employeeID, userID uuid.UUID) (bool, error)
}

// AttendanceAuthorizer checks workspace attendance permissions.
type AttendanceAuthorizer interface {
	HasPermission(ctx context.Context, userID, workspaceID uuid.UUID, permission string) bool
}

// AttendanceHandler serves attendance endpoints.
type AttendanceHandler struct {
	Repository AttendanceRepository
	Authorizer AttendanceAuthorizer
}

// PunchIn handles POST /api/v1/workspaces/{ws_id}/time/punch-in.
func (h AttendanceHandler) PunchIn(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, "ess:clock") {
		h.forbidden(w, r)
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
	now := time.Now().UTC()
	entry, err := h.Repository.PunchIn(r.Context(), workspaceID, request.EmploymentID, entryType, request.Note, request.IsBreakOvertime != nil && *request.IsBreakOvertime, now)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, entry)
}

// PunchOut handles POST /api/v1/workspaces/{ws_id}/time/punch-out/{employee_id}.
// Requires attendance:force_clock_out; closes the active time entry for the target employee.
func (h AttendanceHandler) PunchOut(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	employeeID, empOK := pathUUID(r, "employee_id")
	if !ok || !empOK || !h.allowed(r, workspaceID, "attendance:force_clock_out") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[PunchOutRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	entry, err := h.Repository.PunchOut(r.Context(), workspaceID, employeeID, request.Note, time.Now().UTC())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, entry)
}

// TimeHistory handles GET /api/v1/workspaces/{ws_id}/time/history/{employee_id}.
// Admin: requires attendance:logs:read; employee: self-only.
func (h AttendanceHandler) TimeHistory(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	employeeID, empOK := pathUUID(r, "employee_id")
	if !ok || !empOK {
		h.forbidden(w, r)
		return
	}
	startRaw := r.URL.Query().Get("start")
	endRaw := r.URL.Query().Get("end")
	if startRaw == "" || endRaw == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	start, err := time.Parse(time.RFC3339, startRaw)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	end, err := time.Parse(time.RFC3339, endRaw)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	hasReadPerm := h.allowed(r, workspaceID, "attendance:logs:read")
	if !hasReadPerm {
		userID, userOK := httpinfra.UserIDFromContext(r.Context())
		if !userOK {
			h.forbidden(w, r)
			return
		}
		owned, ownerErr := h.Repository.IsEmployeeOwnedByUser(r.Context(), workspaceID, employeeID, userID)
		if ownerErr != nil {
			h.writeError(w, r, ownerErr)
			return
		}
		if !owned {
			h.forbidden(w, r)
			return
		}
	}
	entries, err := h.Repository.ListTimeHistory(r.Context(), workspaceID, employeeID, start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, entries)
}

// CreateAdjustment handles POST /api/v1/workspaces/{ws_id}/attendance/adjustments.
// Requires attendance:adjustments:submit; creates a pending adjustment awaiting approval.
func (h AttendanceHandler) CreateAdjustment(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, "attendance:adjustments:submit") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AdjustmentRequest](r, 1<<20)
	if err != nil || req.EmploymentID == uuid.Nil || req.RequestedPunchTime == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	record, err := h.Repository.CreateAdjustment(r.Context(), workspaceID, req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, record)
}

// ListAdjustments handles GET /api/v1/workspaces/{ws_id}/attendance/adjustments.
func (h AttendanceHandler) ListAdjustments(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, "attendance:adjustments:approve") {
		h.forbidden(w, r)
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
	result, err := h.Repository.ListAdjustments(r.Context(), workspaceID, status, employeeID, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// ListEmployeeAdjustments handles GET /api/v1/employees/{id}/attendance/adjustments.
func (h AttendanceHandler) ListEmployeeAdjustments(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := pathUUID(r, "id")
	workspaceID, workspaceOK := httpinfra.WorkspaceIDFromContext(r.Context())
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !workspaceOK || !userOK {
		h.forbidden(w, r)
		return
	}
	if !h.allowed(r, workspaceID, "attendance:adjustments:approve") {
		owned, err := h.Repository.IsEmployeeOwnedByUser(r.Context(), workspaceID, employeeID, userID)
		if err != nil || !owned {
			h.forbidden(w, r)
			return
		}
	}
	result, err := h.Repository.ListEmployeeAdjustments(r.Context(), workspaceID, employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

// ApproveAdjustment handles PUT /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve.
func (h AttendanceHandler) ApproveAdjustment(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	adjustmentID, adjustmentOK := pathUUID(r, "id")
	if !ok || !adjustmentOK || !h.allowed(r, workspaceID, "attendance:adjustments:approve") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[ApproveAdjustmentRequest](r, 1<<20)
	if err != nil || (request.Status != "approved" && request.Status != "rejected") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.ApproveAdjustment(r.Context(), workspaceID, adjustmentID, userID, request.Status); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h AttendanceHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}

func (h AttendanceHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h AttendanceHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrEmploymentInactive) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if errors.Is(err, ErrNoActiveEmployment) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
