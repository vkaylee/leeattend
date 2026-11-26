package api

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
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

type OrganizationLeaveBalance struct {
	EmployeeID     uuid.UUID       `json:"employee_id"`
	EmployeeName   string          `json:"employee_name"`
	LeaveTypeID    uuid.UUID       `json:"leave_type_id"`
	LeaveTypeCode  string          `json:"leave_type_code"`
	LeaveTypeName  string          `json:"leave_type_name"`
	IsPaid         bool            `json:"is_paid"`
	Entitlement    int             `json:"entitlement"`
	AccruedAmount  decimal.Decimal `json:"accrued_amount"`
	CarriedOver    int             `json:"carried_over"`
	Used           int             `json:"used"`
	Remaining      decimal.Decimal `json:"remaining"`
	PolicyName     *string         `json:"policy_name"`
	AssignmentType *string         `json:"assignment_type"`
}

type PaginatedOrganizationLeaveBalances struct {
	Data  []OrganizationLeaveBalance `json:"data"`
	Total int64                      `json:"total"`
	Page  int                        `json:"page"`
	Limit int                        `json:"limit"`
}
type LeaveHistoryItem struct {
	ID              uuid.UUID       `json:"id"`
	Date            string          `json:"date"`
	TransactionType string          `json:"transaction_type"`
	Amount          decimal.Decimal `json:"amount"`
	Reason          *string         `json:"reason"`
	ReferenceID     *uuid.UUID      `json:"reference_id"`
	CreatedAt       string          `json:"created_at"`
}
type AdjustBalanceRequest struct {
	LeaveTypeID     uuid.UUID       `json:"leave_type_id"`
	Amount          decimal.Decimal `json:"amount"`
	Reason          string          `json:"reason"`
	TransactionDate string          `json:"transaction_date"`
}

// LeaveRequestRepository manages leave request submission, listing, and approval.
type LeaveRequestRepository interface {
	SubmitLeaveRequest(context.Context, uuid.UUID, SubmitLeaveRequest, bool) (LeaveRequestRecord, error)
	ListOrganizationLeaveRequests(context.Context, *string) ([]LeaveRequestRecord, error)
	ApproveLeaveRequest(context.Context, uuid.UUID, uuid.UUID, string) error
	ResolveEmployeeForUser(context.Context, uuid.UUID) (uuid.UUID, error)
	ListEmployeeLeaveRequests(context.Context, uuid.UUID) ([]LeaveRequestRecord, error)
}

// LeaveBalanceRepository manages leave balances, history, adjustments, and accrual jobs.
type LeaveBalanceRepository interface {
	GetEmployeeLeaveBalances(context.Context, uuid.UUID, int) ([]EmployeeLeaveBalanceRecord, error)
	GetOrganizationLeaveBalances(context.Context, int, int, int, string) ([]OrganizationLeaveBalance, int64, error)
	GetEmployeeLeaveHistory(context.Context, uuid.UUID, int, *uuid.UUID) ([]LeaveHistoryItem, error)
	AdjustLeaveBalance(context.Context, uuid.UUID, AdjustBalanceRequest) error
	CreateAccrualJob(context.Context, uuid.UUID, string) (uuid.UUID, error)
	GetJobStatus(context.Context, uuid.UUID) (LeaveJobStatusResponse, error)
}

// LeaveRepository composes the segregated leave sub-domain repository interfaces.
type LeaveRepository interface {
	LeaveTypeRepository
	LeavePolicyRepository
	LeaveRequestRepository
	LeaveBalanceRepository
}

type LeaveHandler struct {
	Repository       LeaveRepository
	BehalfRepository BehalfAssignmentRepository
	ImportRepository LeaveImportRepository
	Authorizer       Authorizer
	StorageDir       string
}

func (h *LeaveHandler) Submit(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:requests:submit") {
		writeForbidden(w, r)
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[SubmitLeaveRequest](r, 1<<20)
	if err != nil || validateSubmitLeaveRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	autoApprove := h.hasAnyPermission(r, "leave:requests:approve", "leave:approve:l1", "leave:approve:l2")
	record, err := h.Repository.SubmitLeaveRequest(r.Context(), userID, req, autoApprove)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, record)
}

func (h *LeaveHandler) ListRequests(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:requests:read") {
		writeForbidden(w, r)
		return
	}
	var statusFilter *string
	if st := strings.TrimSpace(r.URL.Query().Get("status")); st != "" {
		statusFilter = &st
	}
	records, err := h.Repository.ListOrganizationLeaveRequests(r.Context(), statusFilter)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, records)
}

func (h *LeaveHandler) Approve(w http.ResponseWriter, r *http.Request) {

	requestID, reqOK := pathUUID(r, "id")
	if !reqOK || !h.hasAnyPermission(r, "leave:requests:approve", "leave:approve:l1", "leave:approve:l2") {
		writeForbidden(w, r)
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[ApproveLeaveRequest](r, 1<<20)
	if err != nil || (req.Status != "approved" && req.Status != "rejected") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.ApproveLeaveRequest(r.Context(), userID, requestID, req.Status); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h *LeaveHandler) Balances(w http.ResponseWriter, r *http.Request) {

	employeeID, empOK := pathUUID(r, "id")
	if !empOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	// Self or leave:balances:read
	allowed := hasPermission(r, h.Authorizer, "leave:balances:read")
	if !allowed {
		linkedEmpID, err := h.Repository.ResolveEmployeeForUser(r.Context(), userID)
		if err == nil && linkedEmpID == employeeID {
			allowed = true
		}
	}
	if !allowed {
		writeForbidden(w, r)
		return
	}
	year := time.Now().UTC().Year()
	if yStr := r.URL.Query().Get("year"); yStr != "" {
		if yParsed, err := strconv.Atoi(yStr); err == nil && yParsed > 2000 && yParsed < 2100 {
			year = yParsed
		}
	}
	balances, err := h.Repository.GetEmployeeLeaveBalances(r.Context(), employeeID, year)
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

func (h *LeaveHandler) hasAnyPermission(r *http.Request, permissions ...string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	for _, permission := range permissions {
		if h.Authorizer.HasPermission(r.Context(), userID, permission) {
			return true
		}
	}
	return false
}

func (h *LeaveHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	writeMappedError(w, r, err, func(err error) *apperror.AppError {
		switch {
		case errors.Is(err, ErrNotFound),
			errors.Is(err, ErrLeaveNotFound),
			errors.Is(err, ErrLeaveTypeNotFound),
			errors.Is(err, ErrLeaveNoEmployment):
			return apperror.NotFound(apperror.KeyNotFound)
		case errors.Is(err, ErrLeaveValidation):
			return apperror.Validation(apperror.KeyValidation)
		case errors.Is(err, ErrLeaveConflict):
			return apperror.Conflict(apperror.KeyConflict)
		default:
			return nil
		}
	})
}

func (h *LeaveHandler) importRepo() LeaveImportRepository {
	if h.ImportRepository != nil {
		return h.ImportRepository
	}
	if ir, ok := h.Repository.(LeaveImportRepository); ok {
		return ir
	}
	return nil
}

func (h *LeaveHandler) behalfRepo() BehalfAssignmentRepository {
	if h.BehalfRepository != nil {
		return h.BehalfRepository
	}
	if br, ok := h.Repository.(BehalfAssignmentRepository); ok {
		return br
	}
	return nil
}

func (h *LeaveHandler) OrganizationBalances(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:balances:read") {
		writeForbidden(w, r)
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year := time.Now().UTC().Year()
	if raw := r.URL.Query().Get("year"); raw != "" {
		if y, e := strconv.Atoi(raw); e == nil && y >= 2000 && y <= 2100 {
			year = y
		}
	}
	data, total, err := h.Repository.GetOrganizationLeaveBalances(r.Context(), year, p.Page, p.Limit, r.URL.Query().Get("q"))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, PaginatedOrganizationLeaveBalances{Data: data, Total: total, Page: p.Page, Limit: p.Limit})
}

func (h *LeaveHandler) EmployeeHistory(w http.ResponseWriter, r *http.Request) {

	emp, empOK := pathUUID(r, "id")
	if !empOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		writeForbidden(w, r)
		return
	}
	allowed := hasPermission(r, h.Authorizer, "leave:balances:read")
	if !allowed {
		linked, err := h.Repository.ResolveEmployeeForUser(r.Context(), user)
		if err == nil && linked == emp {
			allowed = true
		}
	}
	if !allowed {
		writeForbidden(w, r)
		return
	}
	year := time.Now().UTC().Year()
	if raw := r.URL.Query().Get("year"); raw != "" {
		if y, e := strconv.Atoi(raw); e == nil {
			year = y
		}
	}
	var typeID *uuid.UUID
	if raw := r.URL.Query().Get("leave_type_id"); raw != "" {
		id, e := uuid.Parse(raw)
		if e != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		typeID = &id
	}
	data, err := h.Repository.GetEmployeeLeaveHistory(r.Context(), emp, year, typeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h *LeaveHandler) AdjustBalance(w http.ResponseWriter, r *http.Request) {

	emp, empOK := pathUUID(r, "id")
	if !empOK || !hasPermission(r, h.Authorizer, "leave:balances:adjust") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[AdjustBalanceRequest](r, 1<<20)
	if err != nil || req.LeaveTypeID == uuid.Nil || req.Amount.IsZero() || strings.TrimSpace(req.Reason) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err = time.Parse("2006-01-02", req.TransactionDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err = h.Repository.AdjustLeaveBalance(r.Context(), emp, req); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h *LeaveHandler) MyRequests(w http.ResponseWriter, r *http.Request) {

	emp, empOK := pathUUID(r, "id")
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !empOK || !userOK {
		writeForbidden(w, r)
		return
	}
	allowed := hasPermission(r, h.Authorizer, "leave:requests:read")
	if !allowed {
		linked, err := h.Repository.ResolveEmployeeForUser(r.Context(), user)
		if err == nil && linked == emp {
			allowed = true
		}
	}
	if !allowed {
		writeForbidden(w, r)
		return
	}
	data, err := h.Repository.ListEmployeeLeaveRequests(r.Context(), emp)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h *LeaveHandler) ListBehalfAssignments(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		writeForbidden(w, r)
		return
	}
	items, err := br.ListBehalfAssignments(r.Context(), "leave_request")
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h *LeaveHandler) CreateBehalfAssignment(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateBehalfAssignmentRequest](r, 1<<20)
	if err != nil || req.AssigneeUserID == uuid.Nil || (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := br.CreateBehalfAssignment(r.Context(), req, "leave_request")
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *LeaveHandler) GetMyBehalfTargets(w http.ResponseWriter, r *http.Request) {

	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		writeForbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		apperror.WriteJSON(w, http.StatusOK, []EmployeeRecord{})
		return
	}
	targets, err := br.ListMyBehalfTargets(r.Context(), user, "leave_request")
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, targets)
}

func (h *LeaveHandler) DeleteBehalfAssignment(w http.ResponseWriter, r *http.Request) {

	id, idOK := pathUUID(r, "id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		writeForbidden(w, r)
		return
	}
	if err := br.DeleteBehalfAssignment(r.Context(), id, "leave_request"); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}
