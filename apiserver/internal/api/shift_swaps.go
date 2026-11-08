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
	ID                uuid.UUID       `json:"id"`
	WorkspaceID       uuid.UUID       `json:"workspace_id"`
	RequesterID       uuid.UUID       `json:"requester_id"`
	TargetID          uuid.UUID       `json:"target_id"`
	SwapDate          string          `json:"swap_date"`
	RequesterShiftID  *uuid.UUID      `json:"requester_shift_id"`
	TargetShiftID     *uuid.UUID      `json:"target_shift_id"`
	Status            ShiftSwapStatus `json:"status"`
	Reason            *string         `json:"reason"`
	RequestedByUserID *uuid.UUID      `json:"requested_by_user_id"`
	ApprovedByUserID  *uuid.UUID      `json:"approved_by_user_id"`
	CreatedAt         string          `json:"created_at"`
	UpdatedAt         string          `json:"updated_at"`
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
	CreateSwap(context.Context, uuid.UUID, uuid.UUID, CreateShiftSwapRequest) (ShiftSwapRequest, error)
	ListSwaps(context.Context, uuid.UUID, *ShiftSwapStatus) ([]ShiftSwapRequestWithNames, error)
	ApproveSwap(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, ShiftSwapStatus) error
	Stats(context.Context, uuid.UUID) (SwapStats, error)
	ListEmployeeSwaps(context.Context, uuid.UUID, uuid.UUID) ([]ShiftSwapRequestWithNames, error)
	CreateESSSwap(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, EssCreateShiftSwapRequest) (ShiftSwapRequest, error)
	RespondSwap(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, bool) error
	CancelSwap(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
}

type ShiftSwapHandler struct {
	Repository ShiftSwapRepository
	Authorizer AttendanceAuthorizer
}

func (h ShiftSwapHandler) Create(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.allowed(r, workspaceID, "shift_swaps:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateShiftSwapRequest](r, 1<<20)
	if err != nil || req.RequesterID == uuid.Nil || req.TargetID == uuid.Nil || req.RequesterID == req.TargetID || !validDate(req.SwapDate) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	swap, err := h.Repository.CreateSwap(r.Context(), workspaceID, userID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, swap)
}

func (h ShiftSwapHandler) List(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, workspaceID, "shift_swaps:manage") {
		h.forbidden(w, r)
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
	data, err := h.Repository.ListSwaps(r.Context(), workspaceID, status)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h ShiftSwapHandler) Approve(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	requestID, requestOK := pathUUID(r, "id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !requestOK || !userOK || !h.allowed(r, workspaceID, "shift_swaps:approve") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[ApproveShiftSwapRequest](r, 1<<20)
	if err != nil || (req.Status != ShiftSwapApproved && req.Status != ShiftSwapRejected) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.ApproveSwap(r.Context(), workspaceID, requestID, userID, req.Status); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h ShiftSwapHandler) Stats(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, workspaceID, "shift_swaps:manage") {
		h.forbidden(w, r)
		return
	}
	stats, err := h.Repository.Stats(r.Context(), workspaceID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, stats)
}

func (h ShiftSwapHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}

func (h ShiftSwapHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h ShiftSwapHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrShiftSwapNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrShiftSwapConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, ErrShiftSwapValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
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
