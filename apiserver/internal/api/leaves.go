package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"os"
	"path/filepath"
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

type UpdateLeaveTypeConfigRequest struct {
	PolicyConfig json.RawMessage `json:"policy_config"`
}

type SeniorityBonus struct {
	Months      int             `json:"months"`
	BonusAmount decimal.Decimal `json:"bonus_amount"`
}
type LeavePolicyConfig struct {
	Frequency            string           `json:"frequency"`
	AccrualAmount        decimal.Decimal  `json:"accrual_amount"`
	AccrualTiming        string           `json:"accrual_timing"`
	SeniorityBonus       []SeniorityBonus `json:"seniority_bonus"`
	CarryOverLimit       *int             `json:"carry_over_limit"`
	MaxBalance           *decimal.Decimal `json:"max_balance"`
	ResetMonth           *int             `json:"reset_month"`
	MinWorkDaysThreshold *decimal.Decimal `json:"min_work_days_threshold"`
}
type WorkspaceLeaveBalance struct {
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
type PaginatedWorkspaceLeaveBalances struct {
	Data  []WorkspaceLeaveBalance `json:"data"`
	Total int64                   `json:"total"`
	Page  int                     `json:"page"`
	Limit int                     `json:"limit"`
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
type LeavePolicyDefinition struct {
	ID          uuid.UUID         `json:"id"`
	WorkspaceID uuid.UUID         `json:"workspace_id"`
	Name        string            `json:"name"`
	Description *string           `json:"description"`
	LeaveTypeID uuid.UUID         `json:"leave_type_id"`
	RuleConfig  LeavePolicyConfig `json:"rule_config"`
	CreatedAt   string            `json:"created_at"`
	UpdatedAt   string            `json:"updated_at"`
}
type CreateLeavePolicyRequest struct {
	Name        string            `json:"name"`
	Description *string           `json:"description"`
	LeaveTypeID uuid.UUID         `json:"leave_type_id"`
	RuleConfig  LeavePolicyConfig `json:"rule_config"`
}
type UpdateLeavePolicyRequest struct {
	Name        *string            `json:"name"`
	Description *string            `json:"description"`
	LeaveTypeID *uuid.UUID         `json:"leave_type_id"`
	RuleConfig  *LeavePolicyConfig `json:"rule_config"`
}
type AssignLeavePolicyRequest struct {
	AssigneeType  string     `json:"assignee_type"`
	AssigneeID    *uuid.UUID `json:"assignee_id"`
	PolicyID      uuid.UUID  `json:"policy_id"`
	EffectiveFrom string     `json:"effective_from"`
	EffectiveTo   *string    `json:"effective_to"`
}
type LeavePolicyAssignment struct {
	ID            uuid.UUID  `json:"id"`
	WorkspaceID   uuid.UUID  `json:"workspace_id"`
	AssigneeType  string     `json:"assignee_type"`
	AssigneeID    *uuid.UUID `json:"assignee_id"`
	PolicyID      uuid.UUID  `json:"policy_id"`
	EffectiveFrom string     `json:"effective_from"`
	EffectiveTo   *string    `json:"effective_to"`
	CreatedAt     string     `json:"created_at"`
}
type LeavePolicyAssignmentDetail struct {
	LeavePolicyAssignment
	PolicyName    string `json:"policy_name"`
	LeaveTypeName string `json:"leave_type_name"`
}
type TriggerAccrualRequest struct {
	Date string `json:"date"`
}
type AccrualJobResponse struct {
	JobID   uuid.UUID `json:"job_id"`
	Status  string    `json:"status"`
	Message string    `json:"message,omitempty"`
}
type LeaveJobStatusResponse struct {
	JobID        uuid.UUID `json:"job_id"`
	Status       string    `json:"status"`
	CompletedAt  *string   `json:"completed_at,omitempty"`
	ErrorMessage *string   `json:"error_message,omitempty"`
}
type LeaveImportRow struct {
	LineNumber    int
	EmployeeCode  string
	LeaveTypeCode string
	StartDate     string
	EndDate       string
	Reason        *string
	DurationType  string
}
type LeaveImportJob struct {
	ID            uuid.UUID `json:"id"`
	WorkspaceID   uuid.UUID `json:"workspace_id"`
	UserID        uuid.UUID `json:"user_id"`
	Status        string    `json:"status"`
	TotalRows     int       `json:"total_rows"`
	ProcessedRows int       `json:"processed_rows"`
	FilePath      *string   `json:"file_path,omitempty"`
	ErrorLog      *string   `json:"error_log,omitempty"`
	CreatedAt     time.Time `json:"created_at"`
	UpdatedAt     time.Time `json:"updated_at"`
}
type LeaveImportRepository interface {
	CreateLeaveImportJob(context.Context, uuid.UUID, uuid.UUID, string) (LeaveImportJob, error)
	ProcessLeaveImport(context.Context, uuid.UUID, uuid.UUID, LeaveImportJob, []LeaveImportRow, bool) error
	GetLatestLeaveImport(context.Context, uuid.UUID) (*LeaveImportJob, error)
	GetLeaveImport(context.Context, uuid.UUID, uuid.UUID) (*LeaveImportJob, error)
	GetLeaveImportErrorFile(context.Context, uuid.UUID, uuid.UUID) ([]byte, error)
}
type LeaveRepository interface {
	CreateLeaveType(context.Context, uuid.UUID, CreateLeaveTypeRequest) (LeaveTypeRecord, error)
	ListLeaveTypes(context.Context, uuid.UUID) ([]LeaveTypeRecord, error)
	UpdateLeaveTypeConfig(context.Context, uuid.UUID, uuid.UUID, json.RawMessage) (LeaveTypeRecord, error)
	DeleteLeaveType(context.Context, uuid.UUID, uuid.UUID) error
	RestoreLeaveType(context.Context, uuid.UUID, uuid.UUID) (LeaveTypeRecord, error)
	SubmitLeaveRequest(context.Context, uuid.UUID, uuid.UUID, SubmitLeaveRequest, bool) (LeaveRequestRecord, error)
	ListWorkspaceLeaveRequests(context.Context, uuid.UUID, *string) ([]LeaveRequestRecord, error)
	ApproveLeaveRequest(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, string) error
	GetEmployeeLeaveBalances(context.Context, uuid.UUID, uuid.UUID, int) ([]EmployeeLeaveBalanceRecord, error)
	ResolveEmployeeForUser(context.Context, uuid.UUID, uuid.UUID) (uuid.UUID, error)
	GetWorkspaceLeaveBalances(context.Context, uuid.UUID, int, int, int, string) ([]WorkspaceLeaveBalance, int64, error)
	GetEmployeeLeaveHistory(context.Context, uuid.UUID, uuid.UUID, int, *uuid.UUID) ([]LeaveHistoryItem, error)
	AdjustLeaveBalance(context.Context, uuid.UUID, uuid.UUID, AdjustBalanceRequest) error
	ListLeavePolicies(context.Context, uuid.UUID) ([]LeavePolicyDefinition, error)
	CreateLeavePolicy(context.Context, uuid.UUID, CreateLeavePolicyRequest) (LeavePolicyDefinition, error)
	UpdateLeavePolicy(context.Context, uuid.UUID, uuid.UUID, UpdateLeavePolicyRequest) (LeavePolicyDefinition, error)
	DeleteLeavePolicy(context.Context, uuid.UUID, uuid.UUID) error
	AssignLeavePolicy(context.Context, uuid.UUID, AssignLeavePolicyRequest) (LeavePolicyAssignment, error)
	ListLeavePolicyAssignments(context.Context, uuid.UUID) ([]LeavePolicyAssignmentDetail, error)
	DeleteLeavePolicyAssignment(context.Context, uuid.UUID, uuid.UUID) error
	ListEmployeeLeaveRequests(context.Context, uuid.UUID, uuid.UUID) ([]LeaveRequestRecord, error)
	CreateAccrualJob(context.Context, uuid.UUID, uuid.UUID, string) (uuid.UUID, error)
	GetJobStatus(context.Context, uuid.UUID, uuid.UUID) (LeaveJobStatusResponse, error)
}

type LeaveAuthorizer interface {
	HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool
}

type LeaveHandler struct {
	Repository       LeaveRepository
	BehalfRepository BehalfAssignmentRepository
	ImportRepository LeaveImportRepository
	Authorizer       LeaveAuthorizer
	StorageDir       string
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

func (h LeaveHandler) UpdateTypeConfig(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	leaveTypeID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, workspaceID, "leave:types:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateLeaveTypeConfigRequest](r, 1<<20)
	if err != nil || !validLeaveTypeConfig(req.PolicyConfig) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	record, err := h.Repository.UpdateLeaveTypeConfig(r.Context(), workspaceID, leaveTypeID, req.PolicyConfig)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func (h LeaveHandler) DeleteType(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	leaveTypeID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, workspaceID, "leave:types:manage") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLeaveType(r.Context(), workspaceID, leaveTypeID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h LeaveHandler) RestoreType(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	leaveTypeID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, workspaceID, "leave:types:manage") {
		h.forbidden(w, r)
		return
	}
	record, err := h.Repository.RestoreLeaveType(r.Context(), workspaceID, leaveTypeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func validLeaveTypeConfig(value json.RawMessage) bool {
	var raw map[string]json.RawMessage
	if len(value) == 0 || json.Unmarshal(value, &raw) != nil || raw == nil {
		return false
	}
	for _, key := range []string{"requires_document", "allow_negative_balance", "approval_required"} {
		if item, ok := raw[key]; ok {
			var v bool
			if json.Unmarshal(item, &v) != nil {
				return false
			}
		}
	}
	if item, ok := raw["negative_balance_limit"]; ok && string(item) != "null" {
		var v float64
		if json.Unmarshal(item, &v) != nil {
			return false
		}
	}
	return true
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
func (h LeaveHandler) importRepo() LeaveImportRepository {
	if h.ImportRepository != nil {
		return h.ImportRepository
	}
	if ir, ok := h.Repository.(LeaveImportRepository); ok {
		return ir
	}
	return nil
}

func (h LeaveHandler) behalfRepo() BehalfAssignmentRepository {
	if h.BehalfRepository != nil {
		return h.BehalfRepository
	}
	if br, ok := h.Repository.(BehalfAssignmentRepository); ok {
		return br
	}
	return nil
}

func (h LeaveHandler) WorkspaceBalances(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:balances:read") {
		h.forbidden(w, r)
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
	data, total, err := h.Repository.GetWorkspaceLeaveBalances(r.Context(), ws, year, p.Page, p.Limit, r.URL.Query().Get("q"))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, PaginatedWorkspaceLeaveBalances{Data: data, Total: total, Page: p.Page, Limit: p.Limit})
}

func (h LeaveHandler) EmployeeHistory(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	emp, empOK := pathUUID(r, "id")
	if !ok || !empOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		h.forbidden(w, r)
		return
	}
	allowed := h.allowed(r, ws, "leave:balances:read")
	if !allowed {
		linked, err := h.Repository.ResolveEmployeeForUser(r.Context(), ws, user)
		if err == nil && linked == emp {
			allowed = true
		}
	}
	if !allowed {
		h.forbidden(w, r)
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
	data, err := h.Repository.GetEmployeeLeaveHistory(r.Context(), ws, emp, year, typeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h LeaveHandler) AdjustBalance(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	emp, empOK := pathUUID(r, "id")
	if !ok || !empOK || !h.allowed(r, ws, "leave:balances:adjust") {
		h.forbidden(w, r)
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
	if err = h.Repository.AdjustLeaveBalance(r.Context(), ws, emp, req); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h LeaveHandler) MyRequests(w http.ResponseWriter, r *http.Request) {
	ws, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok {
		ws, ok = pathUUID(r, "ws_id")
	}
	emp, empOK := pathUUID(r, "id")
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !empOK || !userOK {
		h.forbidden(w, r)
		return
	}
	allowed := h.allowed(r, ws, "leave:requests:read")
	if !allowed {
		linked, err := h.Repository.ResolveEmployeeForUser(r.Context(), ws, user)
		if err == nil && linked == emp {
			allowed = true
		}
	}
	if !allowed {
		h.forbidden(w, r)
		return
	}
	data, err := h.Repository.ListEmployeeLeaveRequests(r.Context(), ws, emp)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h LeaveHandler) ListPolicies(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK {
		h.forbidden(w, r)
		return
	}
	data, err := h.Repository.ListLeavePolicies(r.Context(), ws)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h LeaveHandler) CreatePolicy(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateLeavePolicyRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || req.LeaveTypeID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	data, err := h.Repository.CreateLeavePolicy(r.Context(), ws, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, data)
}

func (h LeaveHandler) UpdatePolicy(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "policy_id")
	if !ok || !idOK || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateLeavePolicyRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	data, err := h.Repository.UpdateLeavePolicy(r.Context(), ws, id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h LeaveHandler) DeletePolicy(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "policy_id")
	if !ok || !idOK || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLeavePolicy(r.Context(), ws, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h LeaveHandler) AssignPolicy(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignLeavePolicyRequest](r, 1<<20)
	if err != nil || (req.AssigneeType != "workspace" && req.AssigneeType != "department" && req.AssigneeType != "employee") || req.PolicyID == uuid.Nil || (req.AssigneeType == "workspace" && req.AssigneeID != nil) || (req.AssigneeType != "workspace" && req.AssigneeID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err = time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	data, err := h.Repository.AssignLeavePolicy(r.Context(), ws, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, data)
}

func (h LeaveHandler) ListPolicyAssignments(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	data, err := h.Repository.ListLeavePolicyAssignments(r.Context(), ws)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h LeaveHandler) DeletePolicyAssignment(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "assignment_id")
	if !ok || !idOK || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLeavePolicyAssignment(r.Context(), ws, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h LeaveHandler) TriggerAccrual(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:accrual:run") {
		h.forbidden(w, r)
		return
	}
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[TriggerAccrualRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, parseErr := time.Parse("2006-01-02", req.Date); parseErr != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	jobID, err := h.Repository.CreateAccrualJob(r.Context(), ws, user, req.Date)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, AccrualJobResponse{
		JobID:   jobID,
		Status:  "queued",
		Message: "Accrual job enqueued for background processing",
	})
}

func (h LeaveHandler) JobStatus(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	jobID, jobOK := pathUUID(r, "job_id")
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !jobOK || !userOK {
		h.forbidden(w, r)
		return
	}
	status, err := h.Repository.GetJobStatus(r.Context(), ws, jobID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, status)
}

func (h LeaveHandler) UploadExcel(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.allowed(r, ws, "leave:requests:submit") {
		h.forbidden(w, r)
		return
	}
	name, content, err := readAttendanceMultipart(r, false)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	dir := h.StorageDir
	if dir == "" {
		dir = "storage"
	}
	if err = os.MkdirAll(filepath.Join(dir, "imports"), 0o750); err != nil {
		h.writeError(w, r, err)
		return
	}
	jobID := uuid.New()
	path := filepath.Join(dir, "imports", jobID.String()+"_"+filepath.Base(name))
	if err = os.WriteFile(path, content, 0o640); err != nil {
		h.writeError(w, r, err)
		return
	}
	ir := h.importRepo()
	if ir == nil {
		apperror.WriteJSON(w, http.StatusOK, map[string]any{"job_id": jobID, "status": "enqueued"})
		return
	}
	job, err := ir.CreateLeaveImportJob(r.Context(), ws, user, path)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	workbook, err := parseEmployeeWorkbook(content, []string{"employee_code", "leave_type_code", "start_date", "end_date"})
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rows := make([]LeaveImportRow, 0, len(workbook))
	for _, row := range workbook {
		empCode := strings.TrimSpace(row["employee_code"])
		ltCode := strings.TrimSpace(row["leave_type_code"])
		startDate := strings.TrimSpace(row["start_date"])
		endDate := strings.TrimSpace(row["end_date"])
		if empCode == "" || ltCode == "" || startDate == "" || endDate == "" {
			continue
		}
		var reason *string
		if rVal, hasR := row["reason"]; hasR && strings.TrimSpace(rVal) != "" {
			trimmed := strings.TrimSpace(rVal)
			reason = &trimmed
		}
		duration := strings.TrimSpace(row["duration_type"])
		if duration == "" {
			duration = "full_day"
		}
		rows = append(rows, LeaveImportRow{
			LineNumber:    parseRowNumber(row["_row_number"]),
			EmployeeCode:  empCode,
			LeaveTypeCode: ltCode,
			StartDate:     startDate,
			EndDate:       endDate,
			Reason:        reason,
			DurationType:  duration,
		})
	}
	autoApprove := h.hasAnyPermission(r, ws, "leave:requests:approve", "leave:approve:l1", "leave:approve:l2")
	if err = ir.ProcessLeaveImport(r.Context(), ws, user, job, rows, autoApprove); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]any{"job_id": job.ID, "status": "enqueued"})
}

func (h LeaveHandler) LatestImport(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:requests:submit") {
		h.forbidden(w, r)
		return
	}
	ir := h.importRepo()
	if ir == nil {
		apperror.WriteJSON(w, http.StatusOK, nil)
		return
	}
	job, err := ir.GetLatestLeaveImport(r.Context(), ws)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, job)
}

func (h LeaveHandler) ImportErrors(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	jobID, jobOK := pathUUID(r, "job_id")
	if !ok || !jobOK || !h.allowed(r, ws, "leave:requests:submit") {
		h.forbidden(w, r)
		return
	}
	ir := h.importRepo()
	if ir == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}
	data, err := ir.GetLeaveImportErrorFile(r.Context(), ws, jobID)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, os.ErrNotExist) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyErrorFileNotFound))
			return
		}
		h.writeError(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", `attachment; filename="leave_import_`+jobID.String()+`_errors.xlsx"`)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(data)
}

func (h LeaveHandler) UploadTemplate(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:requests:submit") {
		h.forbidden(w, r)
		return
	}
	types, err := h.Repository.ListLeaveTypes(r.Context(), ws)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	typeRows := [][]string{{"code", "name", "is_paid"}}
	for _, t := range types {
		paid := "false"
		if t.IsPaid {
			paid = "true"
		}
		typeRows = append(typeRows, []string{t.Code, t.Name, paid})
	}
	templateBytes, err := writeXLSX([]xlsxSheet{
		{
			Name:   "Leave Requests",
			Widths: []float64{20, 20, 15, 15, 30, 15},
			Rows: [][]string{
				{"employee_code", "leave_type_code", "start_date", "end_date", "reason", "duration_type"},
				{"EMP001", "annual", time.Now().Format("2006-01-02"), time.Now().Format("2006-01-02"), "Sample leave", "full_day"},
			},
		},
		{
			Name:   "Leave Types",
			Widths: []float64{20, 30, 15},
			Rows:   typeRows,
		},
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
	w.Header().Set("Content-Disposition", `attachment; filename="leave_import_template.xlsx"`)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(templateBytes)
}

func (h LeaveHandler) ListBehalfAssignments(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok {
		h.forbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		h.forbidden(w, r)
		return
	}
	items, err := br.ListBehalfAssignments(r.Context(), ws, "leave_request")
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h LeaveHandler) CreateBehalfAssignment(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateBehalfAssignmentRequest](r, 1<<20)
	if err != nil || req.AssigneeUserID == uuid.Nil || (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := br.CreateBehalfAssignment(r.Context(), ws, req, "leave_request")
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h LeaveHandler) GetMyBehalfTargets(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	user, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK {
		h.forbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		apperror.WriteJSON(w, http.StatusOK, []EmployeeRecord{})
		return
	}
	targets, err := br.ListMyBehalfTargets(r.Context(), ws, user, "leave_request")
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, targets)
}

func (h LeaveHandler) DeleteBehalfAssignment(w http.ResponseWriter, r *http.Request) {
	ws, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, ws, "leave:policies:manage") {
		h.forbidden(w, r)
		return
	}
	br := h.behalfRepo()
	if br == nil {
		h.forbidden(w, r)
		return
	}
	if err := br.DeleteBehalfAssignment(r.Context(), ws, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}
