package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrESSValidation       = errors.New("ess: validation failed")
	ErrESSNotFound         = errors.New("ess: not found")
	ErrESSNoEmployeeRecord = errors.New("ess: no employee record linked to user")
)

type EmployeeProfileRecord struct {
	ID                uuid.UUID  `json:"id"`
	EmployeeID        uuid.UUID  `json:"employee_id"`
	Phone             *string    `json:"phone"`
	Address           *string    `json:"address"`
	DateOfBirth       *string    `json:"date_of_birth"`
	Gender            *string    `json:"gender"`
	NationalID        *string    `json:"national_id"`
	BankName          *string    `json:"bank_name"`
	BankAccountNumber *string    `json:"bank_account_number"`
	BankBranch        *string    `json:"bank_branch"`
	AvatarFileID      *uuid.UUID `json:"avatar_file_id"`
	CreatedAt         string     `json:"created_at"`
	UpdatedAt         string     `json:"updated_at"`
}

type EmployeeEmergencyContactRecord struct {
	ID           uuid.UUID `json:"id"`
	EmployeeID   uuid.UUID `json:"employee_id"`
	ContactName  string    `json:"contact_name"`
	Relationship string    `json:"relationship"`
	Phone        string    `json:"phone"`
	IsPrimary    bool      `json:"is_primary"`
	CreatedAt    string    `json:"created_at"`
}

type MyProfileResponse struct {
	Employee          EmployeeRecord                   `json:"employee"`
	Profile           *EmployeeProfileRecord           `json:"profile"`
	EmergencyContacts []EmployeeEmergencyContactRecord `json:"emergency_contacts"`
}

type UpdateEmployeeProfileRequest struct {
	Phone             *string    `json:"phone"`
	Address           *string    `json:"address"`
	DateOfBirth       *string    `json:"date_of_birth"`
	Gender            *string    `json:"gender"`
	NationalID        *string    `json:"national_id"`
	BankName          *string    `json:"bank_name"`
	BankAccountNumber *string    `json:"bank_account_number"`
	BankBranch        *string    `json:"bank_branch"`
	AvatarFileID      *uuid.UUID `json:"avatar_file_id"`
}

type CreateEmergencyContactRequest struct {
	ContactName  string `json:"contact_name"`
	Relationship string `json:"relationship"`
	Phone        string `json:"phone"`
	IsPrimary    *bool  `json:"is_primary"`
}

type AcknowledgementRecord struct {
	ID         uuid.UUID `json:"id"`
	DocumentID uuid.UUID `json:"document_id"`
	EmployeeID uuid.UUID `json:"employee_id"`
	Status     string    `json:"status"`
}

type EmployeeDocumentRecord struct {
	ID           uuid.UUID  `json:"id"`
	EmployeeID   uuid.UUID  `json:"employee_id"`
	WorkspaceID  uuid.UUID  `json:"workspace_id"`
	DocumentType string     `json:"document_type"`
	FileName     string     `json:"file_name"`
	FilePath     string     `json:"file_path"`
	FileSize     int32      `json:"file_size"`
	MimeType     string     `json:"mime_type"`
	ExpiresAt    *string    `json:"expires_at"`
	IsActive     bool       `json:"is_active"`
	UploadedBy   *uuid.UUID `json:"uploaded_by"`
	CreatedAt    string     `json:"created_at"`
}

type EssPunchRequest struct {
	Note      *string `json:"note,omitempty"`
	EntryType string  `json:"entry_type,omitempty"`
}

type EssLeaveBalance struct {
	LeaveType     string  `json:"leave_type"`
	Entitlement   int     `json:"entitlement"`
	AccruedAmount float64 `json:"accrued_amount"`
	CarriedOver   int     `json:"carried_over"`
	Used          int64   `json:"used"`
	Remaining     float64 `json:"remaining"`
}

type ESSRepository interface {
	ResolveEmployeeForUser(ctx context.Context, workspaceID, userID uuid.UUID) (uuid.UUID, error)
	GetMyProfile(ctx context.Context, workspaceID, employeeID uuid.UUID) (MyProfileResponse, error)
	UpsertProfile(ctx context.Context, workspaceID, employeeID uuid.UUID, req UpdateEmployeeProfileRequest) (EmployeeProfileRecord, error)
	GetEmergencyContacts(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]EmployeeEmergencyContactRecord, error)
	AddEmergencyContact(ctx context.Context, workspaceID, employeeID uuid.UUID, req CreateEmergencyContactRequest) (EmployeeEmergencyContactRecord, error)
	DeleteEmergencyContact(ctx context.Context, workspaceID, employeeID, contactID uuid.UUID) error
	GetPendingDocuments(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]AcknowledgementRecord, error)
	AcknowledgeDocument(ctx context.Context, workspaceID, employeeID, docID uuid.UUID) error
	GetEmployeeDocuments(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]EmployeeDocumentRecord, error)
}

type ESSHandler struct {
	Repository ESSRepository
	Attendance AttendanceRepository
	Payroll    PayrollRepository
	Leaves     LeaveRepository
	ShiftSwaps ShiftSwapRepository
}

func (h ESSHandler) resolveContext(r *http.Request) (uuid.UUID, uuid.UUID, error) {
	wsID, empID, _, err := h.resolveContextWithUser(r)
	return wsID, empID, err
}

func (h ESSHandler) resolveContextWithUser(r *http.Request) (uuid.UUID, uuid.UUID, uuid.UUID, error) {
	workspaceID, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok || workspaceID == uuid.Nil {
		return uuid.Nil, uuid.Nil, uuid.Nil, ErrESSValidation
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return uuid.Nil, uuid.Nil, uuid.Nil, ErrESSNoEmployeeRecord
	}
	employeeID, err := h.Repository.ResolveEmployeeForUser(r.Context(), workspaceID, userID)
	if err != nil {
		return uuid.Nil, uuid.Nil, uuid.Nil, ErrESSNoEmployeeRecord
	}
	return workspaceID, employeeID, userID, nil
}

func (h ESSHandler) GetProfile(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	profile, err := h.Repository.GetMyProfile(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, profile)
}

func (h ESSHandler) UpdateProfile(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	req, err := DecodeJSON[UpdateEmployeeProfileRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.DateOfBirth != nil && *req.DateOfBirth != "" {
		if _, parseErr := time.Parse("2006-01-02", *req.DateOfBirth); parseErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	profile, err := h.Repository.UpsertProfile(r.Context(), wsID, empID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, profile)
}

func (h ESSHandler) GetContacts(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	contacts, err := h.Repository.GetEmergencyContacts(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, contacts)
}

func (h ESSHandler) AddContact(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	req, err := DecodeJSON[CreateEmergencyContactRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.ContactName) == "" || strings.TrimSpace(req.Relationship) == "" || strings.TrimSpace(req.Phone) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	contact, err := h.Repository.AddEmergencyContact(r.Context(), wsID, empID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, contact)
}

func (h ESSHandler) DeleteContact(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	contactID, ok := pathUUID(r, "contact_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.DeleteEmergencyContact(r.Context(), wsID, empID, contactID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ESSHandler) GetPendingDocuments(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	pending, err := h.Repository.GetPendingDocuments(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if pending == nil {
		pending = []AcknowledgementRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, pending)
}

func (h ESSHandler) AcknowledgeDocument(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	id, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.AcknowledgeDocument(r.Context(), wsID, empID, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]string{"status": "acknowledged"})
}

func (h ESSHandler) GetDocuments(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	docs, err := h.Repository.GetEmployeeDocuments(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if docs == nil {
		docs = []EmployeeDocumentRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, docs)
}

func (h ESSHandler) GetPayslips(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	payslips, err := h.Payroll.ListEmployeePayslips(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if payslips == nil {
		payslips = []PayslipRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, payslips)
}

func (h ESSHandler) GetAttendance(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	startStr := r.URL.Query().Get("start_date")
	endStr := r.URL.Query().Get("end_date")

	now := time.Now().UTC()
	start := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, time.UTC).AddDate(0, 0, -30)
	end := time.Date(now.Year(), now.Month(), now.Day(), 23, 59, 59, 999999999, time.UTC)

	if startStr != "" {
		parsed, err := time.Parse("2006-01-02", startStr)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		start = time.Date(parsed.Year(), parsed.Month(), parsed.Day(), 0, 0, 0, 0, time.UTC)
	}
	if endStr != "" {
		parsed, err := time.Parse("2006-01-02", endStr)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		end = time.Date(parsed.Year(), parsed.Month(), parsed.Day(), 23, 59, 59, 999999999, time.UTC)
	}
	entries, err := h.Attendance.ListTimeHistory(r.Context(), wsID, empID, start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if entries == nil {
		entries = []TimeEntryRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, entries)
}

func (h ESSHandler) Punch(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	var req EssPunchRequest
	if r.Body != nil && r.ContentLength != 0 {
		decoded, err := DecodeJSON[EssPunchRequest](r, 1<<20)
		if err == nil {
			req = decoded
		}
	}
	entry, err := h.Attendance.ESSPunch(r.Context(), wsID, empID, req.EntryType, req.Note, time.Now().UTC())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, entry)
}

func (h ESSHandler) GetTimeEntries(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	startStr := r.URL.Query().Get("start_date")
	endStr := r.URL.Query().Get("end_date")

	now := time.Now().UTC()
	start := now.AddDate(0, 0, -30)
	end := now

	if startStr != "" {
		parsed, err := time.Parse(time.RFC3339, startStr)
		if err != nil {
			if parsedDate, dateErr := time.Parse("2006-01-02", startStr); dateErr == nil {
				parsed = time.Date(parsedDate.Year(), parsedDate.Month(), parsedDate.Day(), 0, 0, 0, 0, time.UTC)
			} else {
				apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
				return
			}
		}
		start = parsed
	}
	if endStr != "" {
		parsed, err := time.Parse(time.RFC3339, endStr)
		if err != nil {
			if parsedDate, dateErr := time.Parse("2006-01-02", endStr); dateErr == nil {
				parsed = time.Date(parsedDate.Year(), parsedDate.Month(), parsedDate.Day(), 23, 59, 59, 999999999, time.UTC)
			} else {
				apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
				return
			}
		}
		end = parsed
	}
	entries, err := h.Attendance.ListTimeHistory(r.Context(), wsID, empID, start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if entries == nil {
		entries = []TimeEntryRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, entries)
}

func (h ESSHandler) GetLeaveBalances(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	year := time.Now().UTC().Year()
	balances, err := h.Leaves.GetEmployeeLeaveBalances(r.Context(), wsID, empID, year)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	result := make([]EssLeaveBalance, 0, len(balances))
	for _, b := range balances {
		result = append(result, EssLeaveBalance{
			LeaveType:     b.LeaveTypeCode,
			Entitlement:   b.Entitlement,
			AccruedAmount: b.AccruedAmount.InexactFloat64(),
			CarriedOver:   b.CarriedOver,
			Used:          int64(b.Used),
			Remaining:     b.Remaining.InexactFloat64(),
		})
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ESSHandler) GetLeaveRequests(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	requests, err := h.Leaves.ListEmployeeLeaveRequests(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if requests == nil {
		requests = []LeaveRequestRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, requests)
}

func (h ESSHandler) GetShiftSwaps(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	swaps, err := h.ShiftSwaps.ListEmployeeSwaps(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if swaps == nil {
		swaps = []ShiftSwapRequestWithNames{}
	}
	apperror.WriteJSON(w, http.StatusOK, swaps)
}

func (h ESSHandler) CreateShiftSwap(w http.ResponseWriter, r *http.Request) {
	wsID, empID, userID, err := h.resolveContextWithUser(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	req, err := DecodeJSON[EssCreateShiftSwapRequest](r, 1<<20)
	if err != nil || req.TargetID == uuid.Nil || req.TargetID == empID || !validDate(req.SwapDate) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	swap, err := h.ShiftSwaps.CreateESSSwap(r.Context(), wsID, empID, userID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, swap)
}

func (h ESSHandler) RespondShiftSwap(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	swapID, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[RespondShiftSwapRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.ShiftSwaps.RespondSwap(r.Context(), wsID, empID, swapID, req.Accept); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h ESSHandler) CancelShiftSwap(w http.ResponseWriter, r *http.Request) {
	wsID, empID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	swapID, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.ShiftSwaps.CancelSwap(r.Context(), wsID, empID, swapID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ESSHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrESSNotFound) || errors.Is(err, ErrShiftSwapNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrESSValidation) || errors.Is(err, ErrNoActiveEmployment) || errors.Is(err, ErrShiftSwapValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrESSNoEmployeeRecord) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNoEmployeeRecord))
		return
	}
	if errors.Is(err, ErrShiftSwapForbidden) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if errors.Is(err, ErrShiftSwapConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
