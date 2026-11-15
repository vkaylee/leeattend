package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrWorkPolicyGroupValidation    = errors.New("work policy group: validation failed")
	ErrWorkPolicyGroupNotFound      = errors.New("work policy group: not found")
	ErrWorkPolicyReferenceNotFound  = errors.New("work policy group: reference not found")
	ErrWorkPolicyResolutionNotFound = errors.New("work policy resolution: not found")
	ErrWorkPolicyMembershipConflict = errors.New("work policy membership: conflict")
)

type WorkPolicyGroupVersion struct {
	ID                        uuid.UUID                      `json:"id"`
	GroupID                   uuid.UUID                      `json:"group_id"`
	Name                      string                         `json:"name"`
	WorkCreditPolicyID        uuid.UUID                      `json:"work_credit_policy_id"`
	WorkCreditPolicyName      string                         `json:"work_credit_policy_name"`
	MinutesPerCredit          int                            `json:"minutes_per_credit"`
	AttendancePolicyID        uuid.UUID                      `json:"attendance_policy_id"`
	WorkingPatternID          uuid.UUID                      `json:"working_pattern_id"`
	PayBasisPolicyID          uuid.UUID                      `json:"pay_basis_policy_id"`
	PayBasisPolicyName        string                         `json:"pay_basis_policy_name"`
	PayMethod                 workpolicy.PayMethod           `json:"pay_method"`
	MonthlyDenominator        *workpolicy.MonthlyDenominator `json:"monthly_denominator,omitempty"`
	MonthlyStandardDays       *decimal.Decimal               `json:"monthly_standard_days,omitempty"`
	RateComponentDefinitionID *uuid.UUID                     `json:"rate_component_definition_id,omitempty"`
	OvertimePolicyID          uuid.UUID                      `json:"overtime_policy_id"`
	EffectiveFrom             string                         `json:"effective_from"`
	EffectiveTo               *string                        `json:"effective_to,omitempty"`
}

type WorkPolicyGroupListResponse struct {
	Data  []WorkPolicyGroupVersion `json:"data"`
	Total int64                    `json:"total"`
	Page  int                      `json:"page"`
	Limit int                      `json:"limit"`
}

type WorkPolicyMembership struct {
	ID            uuid.UUID `json:"id"`
	EmploymentID  uuid.UUID `json:"employment_id"`
	GroupID       uuid.UUID `json:"group_id"`
	EffectiveFrom string    `json:"effective_from"`
	EffectiveTo   *string   `json:"effective_to,omitempty"`
}

type CreateWorkPolicyMembershipRequest struct {
	EmploymentID  uuid.UUID `json:"employment_id"`
	EffectiveFrom string    `json:"effective_from"`
	EffectiveTo   *string   `json:"effective_to,omitempty"`
}

type CreateWorkPolicyGroupRequest struct {
	Name                      string                         `json:"name"`
	MinutesPerCredit          int                            `json:"minutes_per_credit"`
	AttendancePolicyID        uuid.UUID                      `json:"attendance_policy_id"`
	WorkingPatternID          uuid.UUID                      `json:"working_pattern_id"`
	PayMethod                 workpolicy.PayMethod           `json:"pay_method"`
	MonthlyDenominator        *workpolicy.MonthlyDenominator `json:"monthly_denominator,omitempty"`
	MonthlyStandardDays       *decimal.Decimal               `json:"monthly_standard_days,omitempty"`
	RateComponentDefinitionID *uuid.UUID                     `json:"rate_component_definition_id,omitempty"`
	OvertimePolicyID          uuid.UUID                      `json:"overtime_policy_id"`
	EffectiveFrom             string                         `json:"effective_from"`
	EffectiveTo               *string                        `json:"effective_to,omitempty"`
}

type CreateWorkPolicyGroupVersionRequest struct {
	Name                      string                         `json:"name"`
	MinutesPerCredit          int                            `json:"minutes_per_credit"`
	AttendancePolicyID        uuid.UUID                      `json:"attendance_policy_id"`
	WorkingPatternID          uuid.UUID                      `json:"working_pattern_id"`
	PayMethod                 workpolicy.PayMethod           `json:"pay_method"`
	MonthlyDenominator        *workpolicy.MonthlyDenominator `json:"monthly_denominator,omitempty"`
	MonthlyStandardDays       *decimal.Decimal               `json:"monthly_standard_days,omitempty"`
	RateComponentDefinitionID *uuid.UUID                     `json:"rate_component_definition_id,omitempty"`
	OvertimePolicyID          uuid.UUID                      `json:"overtime_policy_id"`
	EffectiveFrom             string                         `json:"effective_from"`
	EffectiveTo               *string                        `json:"effective_to,omitempty"`
}

type WorkPolicyGroupRepository interface {
	ListWorkPolicyGroupVersions(context.Context, int, int) (WorkPolicyGroupListResponse, error)
	ListWorkPolicyGroupHistory(context.Context, uuid.UUID, int, int) (WorkPolicyGroupListResponse, error)
	CreateWorkPolicyGroup(context.Context, CreateWorkPolicyGroupRequest, uuid.UUID) (WorkPolicyGroupVersion, error)
	CreateWorkPolicyGroupVersion(context.Context, uuid.UUID, CreateWorkPolicyGroupVersionRequest, uuid.UUID) (WorkPolicyGroupVersion, error)
	ResolveWorkPolicy(context.Context, uuid.UUID, time.Time) (WorkPolicyGroupVersion, error)
	ListWorkPolicyMemberships(context.Context, uuid.UUID) ([]WorkPolicyMembership, error)
	CreateWorkPolicyMembership(context.Context, uuid.UUID, CreateWorkPolicyMembershipRequest, uuid.UUID) (WorkPolicyMembership, error)
}

type WorkPolicyGroupAudit interface {
	Emit(context.Context, audit.Event) error
}

type WorkPolicyGroupHandler struct {
	Repository WorkPolicyGroupRepository
	Authorizer Authorizer
	Audit      WorkPolicyGroupAudit
}

func (h WorkPolicyGroupHandler) allowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, permission)
}

func (h WorkPolicyGroupHandler) List(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListWorkPolicyGroupVersions(r.Context(), pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h WorkPolicyGroupHandler) History(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, ok := pathUUID(r, "group_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListWorkPolicyGroupHistory(r.Context(), groupID, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h WorkPolicyGroupHandler) Resolve(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	employmentID, ok := pathUUID(r, "employment_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	asOf := strings.TrimSpace(r.URL.Query().Get("as_of"))
	asOfTime, err := time.Parse("2006-01-02", asOf)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
		return
	}
	result, err := h.Repository.ResolveWorkPolicy(r.Context(), employmentID, asOfTime)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h WorkPolicyGroupHandler) CreateVersion(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || !h.allowed(r, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	if !groupOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CreateWorkPolicyGroupVersionRequest](r, 1<<20)
	if err != nil || validateWorkPolicyGroupVersionRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.CreateWorkPolicyGroupVersion(r.Context(), groupID, req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.Audit != nil {
		_ = h.Audit.Emit(r.Context(), audit.Event{
			Name: "payroll.work_policy_group.version_created", ActorID: userID.String(), Resource: result.GroupID.String(), Outcome: "success",
			Details: map[string]string{"effective_from": result.EffectiveFrom, "pay_method": string(result.PayMethod)}, At: time.Now().UTC(),
		})
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h WorkPolicyGroupHandler) ListMemberships(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, ok := pathUUID(r, "group_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListWorkPolicyMemberships(r.Context(), groupID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h WorkPolicyGroupHandler) CreateMembership(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || !h.allowed(r, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	if !groupOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CreateWorkPolicyMembershipRequest](r, 1<<20)
	if err != nil || validateWorkPolicyMembershipRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.CreateWorkPolicyMembership(r.Context(), groupID, req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.Audit != nil {
		_ = h.Audit.Emit(r.Context(), audit.Event{
			Name: "payroll.work_policy_membership.created", ActorID: userID.String(), Resource: result.ID.String(), Outcome: "success",
			Details: map[string]string{"group_id": result.GroupID.String(), "employment_id": result.EmploymentID.String(), "effective_from": result.EffectiveFrom}, At: time.Now().UTC(),
		})
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h WorkPolicyGroupHandler) Create(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || !h.allowed(r, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[CreateWorkPolicyGroupRequest](r, 1<<20)
	if err != nil || validateWorkPolicyGroupRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.CreateWorkPolicyGroup(r.Context(), req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.Audit != nil {
		_ = h.Audit.Emit(r.Context(), audit.Event{
			Name: "payroll.work_policy_group.created", ActorID: userID.String(), Resource: result.GroupID.String(), Outcome: "success",
			Details: map[string]string{"effective_from": result.EffectiveFrom, "pay_method": string(result.PayMethod)},
			At:      time.Now().UTC(),
		})
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func validateWorkPolicyMembershipRequest(req CreateWorkPolicyMembershipRequest) error {
	if req.EmploymentID == uuid.Nil {
		return ErrWorkPolicyGroupValidation
	}
	from, to, err := workPolicyEffectiveRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil || to != nil && !to.After(from) {
		return ErrWorkPolicyGroupValidation
	}
	return nil
}

func validateWorkPolicyGroupVersionRequest(req CreateWorkPolicyGroupVersionRequest) error {
	from, to, err := workPolicyEffectiveRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil || req.OvertimePolicyID == uuid.Nil || strings.TrimSpace(req.Name) == "" || to != nil && !to.After(from) {
		return ErrWorkPolicyGroupValidation
	}
	return validateWorkPolicyDefinitions(CreateWorkPolicyGroupRequest{
		Name: req.Name, MinutesPerCredit: req.MinutesPerCredit, AttendancePolicyID: req.AttendancePolicyID,
		WorkingPatternID: req.WorkingPatternID, PayMethod: req.PayMethod, MonthlyDenominator: req.MonthlyDenominator,
		MonthlyStandardDays: req.MonthlyStandardDays, RateComponentDefinitionID: req.RateComponentDefinitionID, OvertimePolicyID: req.OvertimePolicyID,
		EffectiveFrom: req.EffectiveFrom, EffectiveTo: req.EffectiveTo,
	})
}

func validateWorkPolicyGroupRequest(req CreateWorkPolicyGroupRequest) error {
	from, to, err := workPolicyEffectiveRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil || req.OvertimePolicyID == uuid.Nil || to != nil && !to.After(from) {
		return ErrWorkPolicyGroupValidation
	}
	return validateWorkPolicyDefinitions(req)
}

func workPolicyEffectiveRange(effectiveFrom string, effectiveTo *string) (time.Time, *time.Time, error) {
	from, err := time.Parse("2006-01-02", strings.TrimSpace(effectiveFrom))
	if err != nil {
		return time.Time{}, nil, err
	}
	if effectiveTo == nil || strings.TrimSpace(*effectiveTo) == "" {
		return from, nil, nil
	}
	to, err := time.Parse("2006-01-02", strings.TrimSpace(*effectiveTo))
	if err != nil {
		return time.Time{}, nil, err
	}
	return from, &to, nil
}

func validateWorkPolicyDefinitions(req CreateWorkPolicyGroupRequest) error {
	workCredit := workpolicy.WorkCreditPolicy{
		Name: req.Name, MinutesPerCredit: req.MinutesPerCredit,
		AttendancePolicyID: req.AttendancePolicyID, WorkingPatternID: req.WorkingPatternID,
	}
	if err := workCredit.Validate(); err != nil {
		return ErrWorkPolicyGroupValidation
	}
	payBasis := workpolicy.PayBasisPolicy{
		Name: req.Name, Method: req.PayMethod,
		MonthlyDenominator: req.MonthlyDenominator, MonthlyStandardDays: req.MonthlyStandardDays,
		RateComponentDefinitionID: req.RateComponentDefinitionID,
	}
	if err := payBasis.Validate(); err != nil {
		return ErrWorkPolicyGroupValidation
	}
	return nil
}

func (h WorkPolicyGroupHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrWorkPolicyGroupNotFound), errors.Is(err, ErrWorkPolicyReferenceNotFound), errors.Is(err, ErrWorkPolicyResolutionNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrWorkPolicyGroupValidation):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	case errors.Is(err, ErrConflict), errors.Is(err, ErrWorkPolicyMembershipConflict):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
