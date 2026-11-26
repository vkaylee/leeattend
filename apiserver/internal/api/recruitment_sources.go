package api

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var ErrRecruitmentSourceNotFound = errors.New("recruitment source: not found")

type RecruitmentSource struct {
	ID                  uuid.UUID        `json:"id"`
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
	ListRecruitmentSources(context.Context, int, int) (RecruitmentSourceListResponse, error)
	CreateRecruitmentSource(context.Context, CreateRecruitmentSourceRequest) (RecruitmentSource, error)
	UpdateRecruitmentSource(context.Context, uuid.UUID, UpdateRecruitmentSourceRequest) (RecruitmentSource, error)
	DeleteRecruitmentSource(context.Context, uuid.UUID) error
	RecruitmentSourceBreakdown(context.Context) ([]RecruitmentSourceBreakdown, error)
}

type RecruitmentSourceHandler struct {
	Repository RecruitmentSourceRepository
	Authorizer Authorizer
}

func (h *RecruitmentSourceHandler) List(w http.ResponseWriter, r *http.Request) {
	if !h.isMember(r) {
		writeForbidden(w, r)
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListRecruitmentSources(r.Context(), p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *RecruitmentSourceHandler) Create(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "recruitment:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateRecruitmentSourceRequest](r, 1<<20)
	if err != nil || !validRecruitmentRequest(req.SourceType, req.Name) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	item, err := h.Repository.CreateRecruitmentSource(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, item)
}

func (h *RecruitmentSourceHandler) Update(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !hasPermission(r, h.Authorizer, "recruitment:manage") {
		writeForbidden(w, r)
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
	item, err := h.Repository.UpdateRecruitmentSource(r.Context(), id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h *RecruitmentSourceHandler) Delete(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !hasPermission(r, h.Authorizer, "recruitment:manage") {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeleteRecruitmentSource(r.Context(), id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *RecruitmentSourceHandler) Dashboard(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "dashboard:read") {
		writeForbidden(w, r)
		return
	}
	result, err := h.Repository.RecruitmentSourceBreakdown(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *RecruitmentSourceHandler) isMember(r *http.Request) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID)
	return err == nil && member
}

func (h *RecruitmentSourceHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrRecruitmentSourceNotFound) || errors.Is(err, ErrNotFound) {
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
