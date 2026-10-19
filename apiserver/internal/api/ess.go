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

type ESSRepository interface {
	ResolveEmployeeForUser(ctx context.Context, workspaceID, userID uuid.UUID) (uuid.UUID, error)
	GetMyProfile(ctx context.Context, workspaceID, employeeID uuid.UUID) (MyProfileResponse, error)
	UpsertProfile(ctx context.Context, workspaceID, employeeID uuid.UUID, req UpdateEmployeeProfileRequest) (EmployeeProfileRecord, error)
	GetEmergencyContacts(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]EmployeeEmergencyContactRecord, error)
	AddEmergencyContact(ctx context.Context, workspaceID, employeeID uuid.UUID, req CreateEmergencyContactRequest) (EmployeeEmergencyContactRecord, error)
	DeleteEmergencyContact(ctx context.Context, workspaceID, employeeID, contactID uuid.UUID) error
}

type ESSHandler struct {
	Repository ESSRepository
}

func (h ESSHandler) resolveContext(r *http.Request) (uuid.UUID, uuid.UUID, error) {
	workspaceID, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok || workspaceID == uuid.Nil {
		return uuid.Nil, uuid.Nil, ErrESSValidation
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return uuid.Nil, uuid.Nil, ErrESSNoEmployeeRecord
	}
	employeeID, err := h.Repository.ResolveEmployeeForUser(r.Context(), workspaceID, userID)
	if err != nil {
		return uuid.Nil, uuid.Nil, ErrESSNoEmployeeRecord
	}
	return workspaceID, employeeID, nil
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

func (h ESSHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrESSNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrESSValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrESSNoEmployeeRecord) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
