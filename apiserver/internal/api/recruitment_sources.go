package api

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrRecruitmentSourceNotFound = errors.New("recruitment source: not found")

type RecruitmentSource struct {
	ID                  uuid.UUID        `json:"id"`
	WorkspaceID         uuid.UUID        `json:"workspace_id"`
	SourceType          string           `json:"source_type"`
	Name                string           `json:"name"`
	ReferrerEmployeeID  *uuid.UUID       `json:"referrer_employee_id"`
	ReferralBonusAmount *decimal.Decimal `json:"referral_bonus_amount"`
	ReferralBonusPaid   bool             `json:"referral_bonus_paid"`
	AgencyContactName   *string          `json:"agency_contact_name"`
	AgencyContactEmail  *string          `json:"agency_contact_email"`
	AgencyFeeAmount     *decimal.Decimal `json:"agency_fee_amount"`
	AgencyFeePaid       bool             `json:"agency_fee_paid"`
	Notes               *string          `json:"notes"`
	CreatedAt           string           `json:"created_at"`
	UpdatedAt           string           `json:"updated_at"`
}

type CreateRecruitmentSourceRequest struct {
	SourceType          string           `json:"source_type"`
	Name                string           `json:"name"`
	ReferrerEmployeeID  *uuid.UUID       `json:"referrer_employee_id"`
	ReferralBonusAmount *decimal.Decimal `json:"referral_bonus_amount"`
	AgencyContactName   *string          `json:"agency_contact_name"`
	AgencyContactEmail  *string          `json:"agency_contact_email"`
	AgencyFeeAmount     *decimal.Decimal `json:"agency_fee_amount"`
	Notes               *string          `json:"notes"`
}

type UpdateRecruitmentSourceRequest struct {
	Name                *string          `json:"name"`
	ReferrerEmployeeID  *uuid.UUID       `json:"referrer_employee_id"`
	ReferralBonusAmount *decimal.Decimal `json:"referral_bonus_amount"`
	ReferralBonusPaid   *bool            `json:"referral_bonus_paid"`
	AgencyContactName   *string          `json:"agency_contact_name"`
	AgencyContactEmail  *string          `json:"agency_contact_email"`
	AgencyFeeAmount     *decimal.Decimal `json:"agency_fee_amount"`
	AgencyFeePaid       *bool            `json:"agency_fee_paid"`
	Notes               *string          `json:"notes"`
}

type RecruitmentSourceListResponse struct {
	Data      []RecruitmentSource `json:"data"`
	Total     int64               `json:"total"`
	Page      int                 `json:"page"`
	Limit     int                 `json:"limit"`
	Truncated *bool               `json:"truncated"`
}

type RecruitmentSourceBreakdown struct {
	SourceType string `json:"source_type"`
	SourceName string `json:"source_name"`
	Count      int64  `json:"count"`
}

type RecruitmentSourceRepository interface {
	ListRecruitmentSources(context.Context, uuid.UUID, int, int) (RecruitmentSourceListResponse, error)
	CreateRecruitmentSource(context.Context, uuid.UUID, CreateRecruitmentSourceRequest) (RecruitmentSource, error)
	UpdateRecruitmentSource(context.Context, uuid.UUID, uuid.UUID, UpdateRecruitmentSourceRequest) (RecruitmentSource, error)
	DeleteRecruitmentSource(context.Context, uuid.UUID, uuid.UUID) error
	RecruitmentSourceBreakdown(context.Context, uuid.UUID) ([]RecruitmentSourceBreakdown, error)
}

type RecruitmentSourceAuthorizer interface {
	WorkspaceAuthorizer
	WorkspaceMemberAuthorizer
}

type RecruitmentSourceHandler struct {
	Repository RecruitmentSourceRepository
	Authorizer RecruitmentSourceAuthorizer
}

func (h RecruitmentSourceHandler) List(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	if !ok || !h.isMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListRecruitmentSources(r.Context(), wsID, p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h RecruitmentSourceHandler) Create(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, wsID, "recruitment:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateRecruitmentSourceRequest](r, 1<<20)
	if err != nil || !validRecruitmentRequest(req.SourceType, req.Name) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	item, err := h.Repository.CreateRecruitmentSource(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, item)
}

func (h RecruitmentSourceHandler) Update(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "recruitment:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateRecruitmentSourceRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Name != nil {
		name := strings.TrimSpace(*req.Name)
		if name == "" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		req.Name = &name
	}
	item, err := h.Repository.UpdateRecruitmentSource(r.Context(), wsID, id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h RecruitmentSourceHandler) Delete(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	id, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "recruitment:manage") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteRecruitmentSource(r.Context(), wsID, id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h RecruitmentSourceHandler) Dashboard(w http.ResponseWriter, r *http.Request) {
	wsID, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, wsID, "dashboard:read") {
		h.forbidden(w, r)
		return
	}
	result, err := h.Repository.RecruitmentSourceBreakdown(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h RecruitmentSourceHandler) isMember(r *http.Request, wsID uuid.UUID) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID, wsID)
	return err == nil && member
}

func (h RecruitmentSourceHandler) allowed(r *http.Request, wsID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, wsID, permission)
}

func (h RecruitmentSourceHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h RecruitmentSourceHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrRecruitmentSourceNotFound) || errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func validRecruitmentRequest(sourceType, name string) bool {
	if sourceType != "direct" && sourceType != "referral" && sourceType != "agency" {
		return false
	}
	return strings.TrimSpace(name) != ""
}
