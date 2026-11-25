package api

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrAttendanceClosureValidation    = errors.New("attendance closure: validation failed")
	ErrAttendancePayrollPeriodMissing = errors.New("attendance closure: payroll period missing")
	ErrAttendanceLockedPeriod         = errors.New("attendance closure: payroll period locked")
	ErrAttendancePeriodConflict       = errors.New("attendance closure: period conflict")
	ErrAttendancePeriodNotFound       = errors.New("attendance closure: period not found")
	ErrAttendanceMissingCoverage      = errors.New("attendance closure: missing official coverage")
	ErrAttendanceOpenPunch            = errors.New("attendance closure: open punch")
	ErrAttendanceClosureConflict      = errors.New("attendance closure: unresolved approval")
	ErrAttendanceSelfReopenForbidden  = errors.New("attendance closure: independent approver required for reopen")
	ErrAttendanceAuditFailure         = errors.New("attendance closure: audit failure")
)

type AttendanceClosureRequest struct {
	Version int64  `json:"version"`
	Reason  string `json:"reason"`
}

type AttendancePeriodStatusResponse struct {
	PeriodID           *uuid.UUID `json:"period_id,omitempty"`
	Status             string     `json:"status"`
	Version            int64      `json:"version"`
	CurrentClosureID   *uuid.UUID `json:"current_closure_id,omitempty"`
	ClosureNumber      int        `json:"closure_number,omitempty"`
	DayCount           int        `json:"day_count,omitempty"`
	SnapshotHash       string     `json:"snapshot_hash,omitempty"`
	PayrollPeriodFound bool       `json:"payroll_period_found"`
	PayrollStatus      string     `json:"payroll_status,omitempty"`
}

type AttendanceClosureResponse struct {
	PeriodID       uuid.UUID  `json:"period_id"`
	ClosureID      *uuid.UUID `json:"closure_id,omitempty"`
	PriorClosureID *uuid.UUID `json:"prior_closure_id,omitempty"`
	ClosureNumber  int        `json:"closure_number,omitempty"`
	Status         string     `json:"status"`
	Version        int64      `json:"version"`
	DayCount       int        `json:"day_count,omitempty"`
	SnapshotHash   string     `json:"snapshot_hash,omitempty"`
}

type AttendanceClosureRepository interface {
	CloseAttendancePeriod(context.Context, uuid.UUID, int, int, int64, string, ids.UserID) (AttendanceClosureResponse, error)
	ReopenAttendancePeriod(context.Context, uuid.UUID, int, int, int64, string, ids.UserID) (AttendanceClosureResponse, error)
}

type AttendanceClosureStatusRepository interface {
	GetAttendancePeriodStatus(context.Context, uuid.UUID, int, int) (AttendancePeriodStatusResponse, error)
}

func parseAttendancePeriodPath(r *http.Request) (int, int, uuid.UUID, bool) {
	year, yearErr := strconv.Atoi(r.PathValue("year"))
	month, monthErr := strconv.Atoi(r.PathValue("month"))
	employerID, employerErr := uuid.Parse(r.URL.Query().Get("employer_id"))
	return year, month, employerID, yearErr == nil && monthErr == nil && employerErr == nil
}

func (h *AttendanceHandler) closureAllowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID.UUID(), permission) {
		return false
	}
	// Existing authorizer exposes only global-manager scope for employer-wide closure.
	// Scoped department authorization cannot prove employer coverage; deny closed-loop mutation.
	return h.Authorizer.IsGlobalManager(r.Context(), userID.UUID())
}

// GetAttendancePeriodStatus returns current attendance period and payroll lock state.
func (h *AttendanceHandler) GetAttendancePeriodStatus(w http.ResponseWriter, r *http.Request) {
	if !h.closureAllowed(r, "attendance:closure:read") {
		h.forbidden(w, r)
		return
	}
	year, month, employerID, ok := parseAttendancePeriodPath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidYearMonth))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("attendance closure repository unavailable")))
		return
	}
	repository, ok := h.Repository.(AttendanceClosureStatusRepository)
	if !ok {
		apperror.WriteError(w, r, apperror.Internal(errors.New("attendance closure status repository unavailable")))
		return
	}
	status, err := repository.GetAttendancePeriodStatus(r.Context(), employerID, year, month)
	if err != nil {
		h.writeClosureError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, status)
}

// CloseAttendancePeriod handles monthly attendance closure.
func (h *AttendanceHandler) CloseAttendancePeriod(w http.ResponseWriter, r *http.Request) {
	h.closeAttendancePeriod(w, r)
}

func (h *AttendanceHandler) closeAttendancePeriod(w http.ResponseWriter, r *http.Request) {
	if !h.closureAllowed(r, "attendance:closure:close") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[AttendanceClosureRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(request.Reason) == "" || request.Version < 1 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, month, employerID, ok := parseAttendancePeriodPath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidYearMonth))
		return
	}
	actorID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || h.Repository == nil {
		h.forbidden(w, r)
		return
	}
	repository, ok := h.Repository.(AttendanceClosureRepository)
	if !ok {
		apperror.WriteError(w, r, apperror.Internal(errors.New("attendance closure repository unavailable")))
		return
	}
	response, err := repository.CloseAttendancePeriod(r.Context(), employerID, year, month, request.Version, request.Reason, actorID)
	if err != nil {
		h.writeClosureError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, response)
}

// ReopenAttendancePeriod reopens monthly attendance without changing closure evidence.
func (h *AttendanceHandler) ReopenAttendancePeriod(w http.ResponseWriter, r *http.Request) {
	h.reopenAttendancePeriod(w, r)
}

func (h *AttendanceHandler) reopenAttendancePeriod(w http.ResponseWriter, r *http.Request) {
	if !h.closureAllowed(r, "attendance:closure:reopen") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[AttendanceClosureRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(request.Reason) == "" || request.Version < 1 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, month, employerID, ok := parseAttendancePeriodPath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidYearMonth))
		return
	}
	actorID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || h.Repository == nil {
		h.forbidden(w, r)
		return
	}
	repository, ok := h.Repository.(AttendanceClosureRepository)
	if !ok {
		apperror.WriteError(w, r, apperror.Internal(errors.New("attendance closure repository unavailable")))
		return
	}
	response, err := repository.ReopenAttendancePeriod(r.Context(), employerID, year, month, request.Version, request.Reason, actorID)
	if err != nil {
		h.writeClosureError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, response)
}

func (h *AttendanceHandler) writeClosureError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrAttendanceMissingCoverage):
		apperror.WriteError(w, r, apperror.Conflict("error.missing_coverage"))
	case errors.Is(err, ErrAttendanceOpenPunch):
		apperror.WriteError(w, r, apperror.Conflict("error.open_punch"))
	case errors.Is(err, ErrAttendanceSelfReopenForbidden):
		apperror.WriteError(w, r, apperror.Forbidden("error.self_reopen_forbidden"))
	case errors.Is(err, ErrAttendanceClosureConflict):
		apperror.WriteError(w, r, apperror.Conflict("error.closure_conflict"))
	case errors.Is(err, ErrAttendancePeriodConflict):
		apperror.WriteError(w, r, apperror.Conflict("error.stale_period"))
	case errors.Is(err, ErrAttendanceLockedPeriod):
		apperror.WriteError(w, r, apperror.Conflict("error.locked_period"))
	case errors.Is(err, ErrAttendancePayrollPeriodMissing):
		apperror.WriteError(w, r, apperror.Conflict("error.closure_conflict"))
	case errors.Is(err, ErrAttendanceClosureValidation):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	case errors.Is(err, ErrAttendancePeriodNotFound), errors.Is(err, pgx.ErrNoRows):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrEmployerNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrAttendanceAuditFailure):
		apperror.WriteError(w, r, apperror.Internal(err))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
