package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrEmployerNotFound = errors.New("employer: not found")
	ErrEmployerInUse    = errors.New("employer: in use by active employments")
)

type Employer struct {
	ID                 uuid.UUID  `json:"id"`
	Name               string     `json:"name"`
	TaxCode            *string    `json:"tax_code"`
	LegalAddress       *string    `json:"legal_address"`
	CreatedAt          string     `json:"created_at"`
	UpdatedAt          string     `json:"updated_at"`
	ParentID           *uuid.UUID `json:"parent_id"`
	RelationshipType   *string    `json:"relationship_type"`
	BusinessType       *string    `json:"business_type"`
	RegistrationNumber *string    `json:"registration_number"`
	Phone              *string    `json:"phone"`
	Email              *string    `json:"email"`
	Website            *string    `json:"website"`
	IncorporationDate  *string    `json:"incorporation_date"`
	CountryCode        *string    `json:"country_code"`
	CurrencyCode       *string    `json:"currency_code"`
}

type EmployerListResponse struct {
	Data      []Employer `json:"data"`
	Total     int64      `json:"total"`
	Page      int        `json:"page"`
	Limit     int        `json:"limit"`
	Truncated *bool      `json:"truncated"`
}

type CreateEmployerRequest struct {
	Name               string     `json:"name"`
	TaxCode            *string    `json:"tax_code"`
	LegalAddress       *string    `json:"legal_address"`
	ParentID           *uuid.UUID `json:"parent_id"`
	RelationshipType   *string    `json:"relationship_type"`
	BusinessType       *string    `json:"business_type"`
	RegistrationNumber *string    `json:"registration_number"`
	Phone              *string    `json:"phone"`
	Email              *string    `json:"email"`
	Website            *string    `json:"website"`
	IncorporationDate  *string    `json:"incorporation_date"`
	CountryCode        *string    `json:"country_code"`
	CurrencyCode       *string    `json:"currency_code"`
}

// UpdateEmployerRequest uses **T so JSON null is distinct from an omitted
// field, and the *Present flags record whether each key appeared.
type UpdateEmployerRequest struct {
	Name                    *string     `json:"name"`
	TaxCode                 *string     `json:"tax_code"`
	LegalAddress            *string     `json:"legal_address"`
	ParentID                **uuid.UUID `json:"parent_id"`
	RelationshipType        **string    `json:"relationship_type"`
	BusinessType            *string     `json:"business_type"`
	RegistrationNumber      *string     `json:"registration_number"`
	Phone                   *string     `json:"phone"`
	Email                   *string     `json:"email"`
	Website                 *string     `json:"website"`
	IncorporationDate       *string     `json:"incorporation_date"`
	CountryCode             *string     `json:"country_code"`
	CurrencyCode            *string     `json:"currency_code"`
	ParentIDPresent         bool        `json:"-"`
	RelationshipTypePresent bool        `json:"-"`
}

var updateEmployerFields = map[string]bool{
	"name": true, "tax_code": true, "legal_address": true, "parent_id": true,
	"relationship_type": true, "business_type": true, "registration_number": true,
	"phone": true, "email": true, "website": true, "incorporation_date": true,
	"country_code": true, "currency_code": true,
}

// UnmarshalJSON records field presence for the nullable nested fields and keeps
// DecodeJSON's DisallowUnknownFields parity by rejecting unknown keys here.
func (r *UpdateEmployerRequest) UnmarshalJSON(data []byte) error {
	type plain UpdateEmployerRequest
	var value plain
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}
	for key := range fields {
		if !updateEmployerFields[key] {
			return fmt.Errorf("json: unknown field %q", key)
		}
	}
	*r = UpdateEmployerRequest(value)
	_, r.ParentIDPresent = fields["parent_id"]
	_, r.RelationshipTypePresent = fields["relationship_type"]
	return nil
}

type EmployerRepository interface {
	ListEmployers(context.Context, int, int) (EmployerListResponse, error)
	CreateEmployer(context.Context, CreateEmployerRequest) (Employer, error)
	GetEmployer(context.Context, uuid.UUID) (Employer, error)
	UpdateEmployer(context.Context, uuid.UUID, UpdateEmployerRequest) (Employer, error)
	DeleteEmployer(context.Context, uuid.UUID) error
}

type EmployerHandler struct {
	Repository EmployerRepository
	Authorizer Authorizer
	Audit      AuditEmitter
}

func (h *EmployerHandler) List(w http.ResponseWriter, r *http.Request) {
	if !h.isMember(r) {
		writeForbidden(w, r)
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListEmployers(r.Context(), p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *EmployerHandler) Create(w http.ResponseWriter, r *http.Request) {
	if !h.isMember(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateEmployerRequest](r, 1<<20)
	if err != nil ||
		strings.TrimSpace(req.Name) == "" ||
		!h.validParent(r.Context(), uuid.Nil, req.ParentID) ||
		!validEmployerFields(req.Email, req.IncorporationDate, req.CountryCode, req.CurrencyCode) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employer, err := h.Repository.CreateEmployer(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r, "employer.created", employer)
	apperror.WriteJSON(w, http.StatusCreated, employer)
}

func (h *EmployerHandler) Get(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !h.isMember(r) {
		writeForbidden(w, r)
		return
	}
	employer, err := h.Repository.GetEmployer(r.Context(), id)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, employer)
}

func (h *EmployerHandler) Update(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !h.isMember(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateEmployerRequest](r, 1<<20)
	if err != nil ||
		(req.Name != nil && strings.TrimSpace(*req.Name) == "") ||
		!validEmployerFields(req.Email, req.IncorporationDate, req.CountryCode, req.CurrencyCode) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.ParentIDPresent {
		var parent *uuid.UUID
		if req.ParentID != nil {
			parent = *req.ParentID
		}
		if !h.validParent(r.Context(), id, parent) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	employer, err := h.Repository.UpdateEmployer(r.Context(), id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r, "employer.updated", employer)
	apperror.WriteJSON(w, http.StatusOK, employer)
}

func (h *EmployerHandler) Delete(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !h.isMember(r) {
		writeForbidden(w, r)
		return
	}
	existing, err := h.Repository.GetEmployer(r.Context(), id)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if err := h.Repository.DeleteEmployer(r.Context(), id); err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r, "employer.deleted", existing)
	w.WriteHeader(http.StatusNoContent)
}

// validParent checks parent existence on create and rejects self-parent or
// circular chains on update.
func (h *EmployerHandler) validParent(ctx context.Context, employerID uuid.UUID, parentID *uuid.UUID) bool {
	if parentID == nil {
		return true
	}
	if employerID == uuid.Nil {
		_, err := h.Repository.GetEmployer(ctx, *parentID)
		return err == nil
	}
	if employerID == *parentID {
		return false
	}
	visited := map[uuid.UUID]bool{employerID: true, *parentID: true}
	current := *parentID
	for {
		parent, err := h.Repository.GetEmployer(ctx, current)
		if err != nil {
			return false
		}
		if parent.ParentID == nil {
			return true
		}
		if visited[*parent.ParentID] {
			return false
		}
		visited[*parent.ParentID] = true
		current = *parent.ParentID
	}
}

func (h *EmployerHandler) isMember(r *http.Request) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID)
	return err == nil && member
}

func (h *EmployerHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrEmployerNotFound), errors.Is(err, ErrNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrEmployerInUse):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}

// emitAudit records employer.* events when an audit emitter is wired.
// The shared writer redacts phone/email keys.
func (h *EmployerHandler) emitAudit(r *http.Request, name string, employer Employer) {
	if h.Audit == nil {
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return
	}
	_ = h.Audit.Emit(r.Context(), audit.Event{
		Name:     name,
		ActorID:  userID.String(),
		Resource: employer.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"name":                employer.Name,
			"tax_code":            stringValue(employer.TaxCode),
			"business_type":       stringValue(employer.BusinessType),
			"registration_number": stringValue(employer.RegistrationNumber),
			"phone":               stringValue(employer.Phone),
			"email":               stringValue(employer.Email),
			"website":             stringValue(employer.Website),
			"incorporation_date":  stringValue(employer.IncorporationDate),
			"country_code":        stringValue(employer.CountryCode),
			"currency_code":       stringValue(employer.CurrencyCode),
		},
		At: time.Now().UTC(),
	})
}

func stringValue(value *string) string {
	if value == nil {
		return ""
	}
	return *value
}

func validEmployerFields(email, date, country, currency *string) bool {
	if email != nil && !validEmail(*email) {
		return false
	}
	if date != nil {
		if _, err := time.Parse("2006-01-02", *date); err != nil {
			return false
		}
	}
	for _, code := range []*string{country, currency} {
		if code != nil && !validCurrencyCode(*code) {
			return false
		}
	}
	return true
}

func validEmail(email string) bool {
	value := strings.TrimSpace(email)
	if len(value) == 0 || len(value) > 254 {
		return false
	}
	parts := strings.SplitN(value, "@", 2)
	if len(parts) != 2 || len(parts[0]) == 0 || len(parts[0]) > 64 {
		return false
	}
	domain := parts[1]
	return domain != "" && strings.Contains(domain, ".") && !strings.HasPrefix(domain, ".") && !strings.HasSuffix(domain, ".") && !strings.Contains(domain, "..")
}

func validCurrencyCode(code string) bool {
	value := strings.TrimSpace(code)
	if len(value) != 3 {
		return false
	}
	for _, c := range value {
		if (c < 'A' || c > 'Z') && (c < 'a' || c > 'z') {
			return false
		}
	}
	return true
}
