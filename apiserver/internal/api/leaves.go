package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrLeaveValidation   = errors.New("leave: validation failed")
	ErrLeaveNotFound     = errors.New("leave: not found")
	ErrLeaveConflict     = errors.New("leave: conflict")
	ErrLeaveTypeNotFound = errors.New("leave: type not found")
	ErrLeaveNoEmployment = errors.New("leave: employment not found")
)

type SubmitLeaveRequest struct {
	EmploymentID      uuid.UUID   `json:"employment_id"`
	LeaveTypeID       uuid.UUID   `json:"leave_type_id"`
	StartDate         string      `json:"start_date"`
	EndDate           string      `json:"end_date"`
	Reason            *string     `json:"reason"`
	DurationType      string      `json:"duration_type"`
	StartTime         *string     `json:"start_time"`
	EndTime           *string     `json:"end_time"`
	AttachmentFileIDs []uuid.UUID `json:"attachment_file_ids"`
}

type ApproveLeaveRequest struct {
	Status string `json:"status"`
}

type LeaveRequestRecord struct {
	ID                uuid.UUID   `json:"id"`
	EmploymentID      uuid.UUID   `json:"employment_id"`
	LeaveTypeID       uuid.UUID   `json:"leave_type_id"`
	StartDate         string      `json:"start_date"`
	EndDate           string      `json:"end_date"`
	Reason            *string     `json:"reason"`
	Status            string      `json:"status"`
	DurationType      string      `json:"duration_type"`
	StartTime         *string     `json:"start_time"`
	EndTime           *string     `json:"end_time"`
	RequestedByUserID *uuid.UUID  `json:"requested_by_user_id"`
	ApprovedByUserID  *uuid.UUID  `json:"approved_by_user_id"`
	CreatedAt         string      `json:"created_at"`
	UpdatedAt         string      `json:"updated_at"`
	AttachmentFileIDs []uuid.UUID `json:"attachment_file_ids"`
}

type EmployeeLeaveBalanceRecord struct {
	LeaveTypeID   uuid.UUID       `json:"leave_type_id"`
	LeaveTypeCode string          `json:"leave_type_code"`
	LeaveTypeName string          `json:"leave_type_name"`
	IsPaid        bool            `json:"is_paid"`
	Entitlement   int             `json:"entitlement"`
	AccruedAmount decimal.Decimal `json:"accrued_amount"`
	CarriedOver   int             `json:"carried_over"`
	Used          int             `json:"used"`
	Remaining     decimal.Decimal `json:"remaining"`
}

type LeaveTypeRecord struct {
	ID           uuid.UUID       `json:"id"`
	WorkspaceID  uuid.UUID       `json:"workspace_id"`
	Code         string          `json:"code"`
	Name         string          `json:"name"`
	IsPaid       bool            `json:"is_paid"`
	PolicyConfig json.RawMessage `json:"policy_config"`
	CreatedAt    string          `json:"created_at"`
}

type CreateLeaveTypeRequest struct {
	Code         string          `json:"code"`
	Name         string          `json:"name"`
	IsPaid       *bool           `json:"is_paid"`
	PolicyConfig json.RawMessage `json:"policy_config"`
}

type LeaveRepository interface {
	CreateLeaveType(context.Context, uuid.UUID, CreateLeaveTypeRequest) (LeaveTypeRecord, error)
	ListLeaveTypes(context.Context, uuid.UUID) ([]LeaveTypeRecord, error)
	SubmitLeaveRequest(context.Context, uuid.UUID, uuid.UUID, SubmitLeaveRequest, bool) (LeaveRequestRecord, error)
	ListWorkspaceLeaveRequests(context.Context, uuid.UUID, *string) ([]LeaveRequestRecord, error)
	ApproveLeaveRequest(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, string) error
	GetEmployeeLeaveBalances(context.Context, uuid.UUID, uuid.UUID, int) ([]EmployeeLeaveBalanceRecord, error)
	ResolveEmployeeForUser(context.Context, uuid.UUID, uuid.UUID) (uuid.UUID, error)
}

type LeaveAuthorizer interface {
	HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool
}

type LeaveHandler struct {
	Repository LeaveRepository
	Authorizer LeaveAuthorizer
}

func (h LeaveHandler) CreateType(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, "leave:types:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateLeaveTypeRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Code) == "" || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lt, err := h.Repository.CreateLeaveType(r.Context(), workspaceID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, lt)
}

func (h LeaveHandler) ListTypes(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	types, err := h.Repository.ListLeaveTypes(r.Context(), workspaceID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, types)
}

func (h LeaveHandler) Submit(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, "leave:requests:submit") {
		h.forbidden(w, r)
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[SubmitLeaveRequest](r, 1<<20)
	if err != nil || validateSubmitLeaveRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	autoApprove := h.hasAnyPermission(r, workspaceID, "leave:requests:approve", "leave:approve:l1", "leave:approve:l2")
	record, err := h.Repository.SubmitLeaveRequest(r.Context(), workspaceID, userID, req, autoApprove)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, record)
}

func (h LeaveHandler) ListRequests(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, "leave:requests:read") {
		h.forbidden(w, r)
		return
	}
	var statusFilter *string
	if st := strings.TrimSpace(r.URL.Query().Get("status")); st != "" {
		statusFilter = &st
	}
	records, err := h.Repository.ListWorkspaceLeaveRequests(r.Context(), workspaceID, statusFilter)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, records)
}

func (h LeaveHandler) Approve(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	requestID, reqOK := pathUUID(r, "id")
	if !ok || !reqOK || !h.hasAnyPermission(r, workspaceID, "leave:requests:approve", "leave:approve:l1", "leave:approve:l2") {
		h.forbidden(w, r)
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[ApproveLeaveRequest](r, 1<<20)
	if err != nil || (req.Status != "approved" && req.Status != "rejected") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.ApproveLeaveRequest(r.Context(), workspaceID, userID, requestID, req.Status); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h LeaveHandler) Balances(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	employeeID, empOK := pathUUID(r, "id")
	if !ok || !empOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	// Self or leave:balances:read
	allowed := h.allowed(r, workspaceID, "leave:balances:read")
	if !allowed {
		linkedEmpID, err := h.Repository.ResolveEmployeeForUser(r.Context(), workspaceID, userID)
		if err == nil && linkedEmpID == employeeID {
			allowed = true
		}
	}
	if !allowed {
		h.forbidden(w, r)
		return
	}
	year := time.Now().UTC().Year()
	if yStr := r.URL.Query().Get("year"); yStr != "" {
		if yParsed, err := strconv.Atoi(yStr); err == nil && yParsed > 2000 && yParsed < 2100 {
			year = yParsed
		}
	}
	balances, err := h.Repository.GetEmployeeLeaveBalances(r.Context(), workspaceID, employeeID, year)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, balances)
}

func validateSubmitLeaveRequest(req SubmitLeaveRequest) error {
	if req.EmploymentID == uuid.Nil || req.LeaveTypeID == uuid.Nil {
		return ErrLeaveValidation
	}
	start, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return ErrLeaveValidation
	}
	end, err := time.Parse("2006-01-02", req.EndDate)
	if err != nil || end.Before(start) {
		return ErrLeaveValidation
	}
	duration := req.DurationType
	if duration == "" {
		duration = "full_day"
	}
	switch duration {
	case "full_day":
	case "first_half", "second_half":
		if req.StartDate != req.EndDate {
			return ErrLeaveValidation
		}
	case "custom_hours":
		if req.StartDate != req.EndDate {
			return ErrLeaveValidation
		}
		if req.StartTime == nil || req.EndTime == nil || *req.StartTime == "" || *req.EndTime == "" {
			return ErrLeaveValidation
		}
		if *req.StartTime >= *req.EndTime {
			return ErrLeaveValidation
		}
	default:
		return ErrLeaveValidation
	}
	return nil
}

func (h LeaveHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}

func (h LeaveHandler) hasAnyPermission(r *http.Request, workspaceID uuid.UUID, permissions ...string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	for _, perm := range permissions {
		if h.Authorizer.HasPermission(r.Context(), userID, workspaceID, perm) {
			return true
		}
	}
	return false
}

func (h LeaveHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h LeaveHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrLeaveNotFound) || errors.Is(err, ErrLeaveTypeNotFound) || errors.Is(err, ErrLeaveNoEmployment) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrLeaveValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrLeaveConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
