package api

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type ShiftSwapStatus string

const (
	ShiftSwapPendingTarget ShiftSwapStatus = "pending_target"
	ShiftSwapPendingAdmin  ShiftSwapStatus = "pending_admin"
	ShiftSwapApproved      ShiftSwapStatus = "approved"
	ShiftSwapRejected      ShiftSwapStatus = "rejected"
	ShiftSwapCancelled     ShiftSwapStatus = "cancelled"
)

type ShiftSwapRequest struct {
	ID                      uuid.UUID       `json:"id"`
	RequesterID             uuid.UUID       `json:"requester_id"`
	TargetID                uuid.UUID       `json:"target_id"`
	SwapDate                string          `json:"swap_date"`
	RequesterRevisionID     *uuid.UUID      `json:"requester_revision_id,omitempty"`
	RequesterRevisionNumber *int64          `json:"requester_revision_number,omitempty"`
	TargetRevisionID        *uuid.UUID      `json:"target_revision_id,omitempty"`
	TargetRevisionNumber    *int64          `json:"target_revision_number,omitempty"`
	RequesterShiftID        *uuid.UUID      `json:"requester_shift_id"`
	TargetShiftID           *uuid.UUID      `json:"target_shift_id"`
	Status                  ShiftSwapStatus `json:"status"`
	Reason                  *string         `json:"reason"`
	RequestedByUserID       *uuid.UUID      `json:"requested_by_user_id"`
	ApprovedByUserID        *uuid.UUID      `json:"approved_by_user_id"`
	CreatedAt               string          `json:"created_at"`
	UpdatedAt               string          `json:"updated_at"`
}

type ShiftSwapRequestWithNames struct {
	ShiftSwapRequest
	RequesterName      *string `json:"requester_name"`
	TargetName         *string `json:"target_name"`
	RequesterShiftName *string `json:"requester_shift_name"`
	TargetShiftName    *string `json:"target_shift_name"`
}

type CreateShiftSwapRequest struct {
	RequesterID uuid.UUID `json:"requester_id"`
	TargetID    uuid.UUID `json:"target_id"`
	SwapDate    string    `json:"swap_date"`
	Reason      *string   `json:"reason"`
}

type ApproveShiftSwapRequest struct {
	Status ShiftSwapStatus `json:"status"`
}

type EssCreateShiftSwapRequest struct {
	TargetID uuid.UUID `json:"target_id"`
	SwapDate string    `json:"swap_date"`
	Reason   *string   `json:"reason,omitempty"`
}

type RespondShiftSwapRequest struct {
	Accept bool `json:"accept"`
}

type SwapStats struct {
	PendingTarget     int64 `json:"pending_target"`
	PendingAdmin      int64 `json:"pending_admin"`
	Approved          int64 `json:"approved"`
	Rejected          int64 `json:"rejected"`
	Cancelled         int64 `json:"cancelled"`
	ApprovedThisMonth int64 `json:"approved_this_month"`
	RejectedThisMonth int64 `json:"rejected_this_month"`
}

type ShiftSwapRepository interface {
	CreateSwap(context.Context, uuid.UUID, CreateShiftSwapRequest) (ShiftSwapRequest, error)
	ListSwaps(context.Context, *ShiftSwapStatus) ([]ShiftSwapRequestWithNames, error)
	ApproveSwap(context.Context, uuid.UUID, uuid.UUID, ShiftSwapStatus) error
	Stats(context.Context) (SwapStats, error)
	ListEmployeeSwaps(context.Context, uuid.UUID) ([]ShiftSwapRequestWithNames, error)
	CreateESSSwap(context.Context, uuid.UUID, uuid.UUID, EssCreateShiftSwapRequest) (ShiftSwapRequest, error)
	RespondSwap(context.Context, uuid.UUID, uuid.UUID, bool) error
	CancelSwap(context.Context, uuid.UUID, uuid.UUID) error
}

type ShiftSwapHandler struct {
	Repository ShiftSwapRepository
	Authorizer Authorizer
}

func (h *ShiftSwapHandler) Create(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || !hasPermission(r, h.Authorizer, "shift_swaps:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateShiftSwapRequest](r, 1<<20)
	if err != nil || req.RequesterID == uuid.Nil || req.TargetID == uuid.Nil || req.RequesterID == req.TargetID || !validDate(req.SwapDate) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	swap, err := h.Repository.CreateSwap(r.Context(), userID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, swap)
}

func (h *ShiftSwapHandler) List(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "shift_swaps:manage") {
		writeForbidden(w, r)
		return
	}
	var status *ShiftSwapStatus
	if raw := r.URL.Query().Get("status"); raw != "" {
		candidate := ShiftSwapStatus(raw)
		if !validSwapStatus(candidate) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		status = &candidate
	}
	data, err := h.Repository.ListSwaps(r.Context(), status)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h *ShiftSwapHandler) Approve(w http.ResponseWriter, r *http.Request) {
	requestID, requestOK := pathUUID(r, "id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !requestOK || !userOK || !hasPermission(r, h.Authorizer, "shift_swaps:approve") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[ApproveShiftSwapRequest](r, 1<<20)
	if err != nil || (req.Status != ShiftSwapApproved && req.Status != ShiftSwapRejected) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.ApproveSwap(r.Context(), requestID, userID, req.Status); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h *ShiftSwapHandler) Stats(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "shift_swaps:manage") {
		writeForbidden(w, r)
		return
	}
	stats, err := h.Repository.Stats(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, stats)
}

func (h *ShiftSwapHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if mapped := mapLifecycleGuardError(err); mapped != nil {
		apperror.WriteError(w, r, mapped)
		return
	}
	if errors.Is(err, ErrShiftSwapNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrShiftSwapConflict) || errors.Is(err, ErrScheduleLockedPeriod) || errors.Is(err, ErrScheduleNoCurrentRevision) || errors.Is(err, ErrScheduleConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, ErrShiftSwapValidation) || errors.Is(err, ErrNoActiveEmployment) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrShiftSwapForbidden) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func validDate(value string) bool {
	_, err := time.Parse("2006-01-02", value)
	return err == nil
}

func validSwapStatus(value ShiftSwapStatus) bool {
	return value == ShiftSwapPendingTarget || value == ShiftSwapPendingAdmin || value == ShiftSwapApproved || value == ShiftSwapRejected || value == ShiftSwapCancelled
}

var (
	ErrShiftSwapNotFound   = errors.New("shift swap: not found")
	ErrShiftSwapConflict   = errors.New("shift swap: conflict")
	ErrShiftSwapValidation = errors.New("shift swap: validation failed")
	ErrShiftSwapForbidden  = errors.New("shift swap: forbidden")
)
