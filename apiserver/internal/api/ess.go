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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
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

type EssLeaveBalance struct {
	LeaveType     string  `json:"leave_type"`
	Entitlement   int     `json:"entitlement"`
	AccruedAmount float64 `json:"accrued_amount"`
	CarriedOver   int     `json:"carried_over"`
	Used          int64   `json:"used"`
	Remaining     float64 `json:"remaining"`
}

type ESSRepository interface {
	ResolveEmployeeForUser(ctx context.Context, userID uuid.UUID) (uuid.UUID, error)
	GetMyProfile(ctx context.Context, employeeID uuid.UUID) (MyProfileResponse, error)
	UpsertProfile(ctx context.Context, employeeID uuid.UUID, req UpdateEmployeeProfileRequest) (EmployeeProfileRecord, error)
	GetEmergencyContacts(ctx context.Context, employeeID uuid.UUID) ([]EmployeeEmergencyContactRecord, error)
	AddEmergencyContact(ctx context.Context, employeeID uuid.UUID, req CreateEmergencyContactRequest) (EmployeeEmergencyContactRecord, error)
	DeleteEmergencyContact(ctx context.Context, employeeID, contactID uuid.UUID) error
	GetPendingDocuments(ctx context.Context, employeeID uuid.UUID) ([]AcknowledgementRecord, error)
	AcknowledgeDocument(ctx context.Context, employeeID, docID uuid.UUID) error
	GetEmployeeDocuments(ctx context.Context, employeeID uuid.UUID) ([]EmployeeDocumentRecord, error)
}

type ESSHandler struct {
	Repository ESSRepository
	Attendance AttendanceRepository
	Authorizer Authorizer
	Payroll    PayrollRepository
	Leaves     LeaveRepository
	ShiftSwaps ShiftSwapRepository
}

func (h *ESSHandler) allowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, permission)
}

func (h *ESSHandler) resolveContext(r *http.Request) (uuid.UUID, uuid.UUID, error) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || userID == uuid.Nil {
		return uuid.Nil, uuid.Nil, ErrESSNoEmployeeRecord
	}
	employeeID, err := h.Repository.ResolveEmployeeForUser(r.Context(), userID)
	if err != nil || employeeID == uuid.Nil {
		return uuid.Nil, uuid.Nil, ErrESSNoEmployeeRecord
	}
	return employeeID, userID, nil
}

func (h *ESSHandler) GetProfile(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	profile, err := h.Repository.GetMyProfile(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, profile)
}

func (h *ESSHandler) UpdateProfile(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
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
	profile, err := h.Repository.UpsertProfile(r.Context(), empID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, profile)
}

func (h *ESSHandler) GetContacts(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	contacts, err := h.Repository.GetEmergencyContacts(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, contacts)
}

func (h *ESSHandler) AddContact(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	req, err := DecodeJSON[CreateEmergencyContactRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.ContactName) == "" || strings.TrimSpace(req.Relationship) == "" || strings.TrimSpace(req.Phone) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	contact, err := h.Repository.AddEmergencyContact(r.Context(), empID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, contact)
}

func (h *ESSHandler) DeleteContact(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	contactID, ok := pathUUID(r, "contact_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.DeleteEmergencyContact(r.Context(), empID, contactID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *ESSHandler) GetPendingDocuments(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	pending, err := h.Repository.GetPendingDocuments(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if pending == nil {
		pending = []AcknowledgementRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, pending)
}

func (h *ESSHandler) AcknowledgeDocument(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	id, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.AcknowledgeDocument(r.Context(), empID, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]string{"status": "acknowledged"})
}

func (h *ESSHandler) GetDocuments(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	docs, err := h.Repository.GetEmployeeDocuments(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if docs == nil {
		docs = []EmployeeDocumentRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, docs)
}

func (h *ESSHandler) GetPayslips(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	payslips, err := h.Payroll.ListEmployeePayslips(r.Context(), ids.EmployeeID(empID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if payslips == nil {
		payslips = []PayslipRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, payslips)
}

func (h *ESSHandler) GetAttendance(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
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
	entries, err := h.Attendance.ListTimeHistory(r.Context(), empID, start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if entries == nil {
		entries = []TimeEntryRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, entries)
}

func (h *ESSHandler) AttendanceAction(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "ess:clock") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	key := strings.TrimSpace(r.Header.Get("Idempotency-Key"))
	if key == "" || len(key) > 128 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[AttendanceActionRequest](r, 1<<20)
	if err != nil || !validAttendanceAction(req.Action) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	response, err := h.Attendance.ESSAttendanceAction(r.Context(), empID, req.Action, req.Note, key, time.Now().UTC())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	status := http.StatusOK
	if !response.Deduplicated { status = http.StatusCreated }
	apperror.WriteJSON(w, status, response)
}

func (h *ESSHandler) AttendanceState(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "ess:clock") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	state, err := h.Attendance.GetESSAttendanceState(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, state)
}

func validAttendanceAction(action string) bool {
	return action == AttendanceActionWorkIn || action == AttendanceActionWorkOut || action == AttendanceActionBreakStart || action == AttendanceActionBreakEnd
}

func (h *ESSHandler) GetTimeEntries(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
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
	entries, err := h.Attendance.ListTimeHistory(r.Context(), empID, start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if entries == nil {
		entries = []TimeEntryRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, entries)
}

func (h *ESSHandler) GetLeaveBalances(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	year := time.Now().UTC().Year()
	balances, err := h.Leaves.GetEmployeeLeaveBalances(r.Context(), empID, year)
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

func (h *ESSHandler) GetLeaveRequests(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	requests, err := h.Leaves.ListEmployeeLeaveRequests(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if requests == nil {
		requests = []LeaveRequestRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, requests)
}

func (h *ESSHandler) GetShiftSwaps(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	swaps, err := h.ShiftSwaps.ListEmployeeSwaps(r.Context(), empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if swaps == nil {
		swaps = []ShiftSwapRequestWithNames{}
	}
	apperror.WriteJSON(w, http.StatusOK, swaps)
}

func (h *ESSHandler) CreateShiftSwap(w http.ResponseWriter, r *http.Request) {
	empID, userID, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	req, err := DecodeJSON[EssCreateShiftSwapRequest](r, 1<<20)
	if err != nil || req.TargetID == uuid.Nil || req.TargetID == empID || !validDate(req.SwapDate) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	swap, err := h.ShiftSwaps.CreateESSSwap(r.Context(), empID, userID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, swap)
}

func (h *ESSHandler) RespondShiftSwap(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
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
	if err := h.ShiftSwaps.RespondSwap(r.Context(), empID, swapID, req.Accept); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h *ESSHandler) CancelShiftSwap(w http.ResponseWriter, r *http.Request) {
	empID, _, err := h.resolveContext(r)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	swapID, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.ShiftSwaps.CancelSwap(r.Context(), empID, swapID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *ESSHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrESSNotFound) || errors.Is(err, ErrShiftSwapNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrESSValidation) || errors.Is(err, ErrNoActiveEmployment) || errors.Is(err, ErrShiftSwapValidation) || errors.Is(err, ErrAttendanceBreakNotRequired) || errors.Is(err, ErrAttendanceBreakNotOpen) {
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
	if errors.Is(err, ErrShiftSwapConflict) || errors.Is(err, ErrAttendanceAlreadyWorking) || errors.Is(err, ErrAttendanceNotWorking) || errors.Is(err, ErrAttendanceInvalidTransition) || errors.Is(err, ErrAttendanceIdempotencyConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
