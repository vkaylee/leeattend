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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrWorkPolicyGroupValidation    = errors.New("work policy group: validation failed")
	ErrWorkPolicyGroupNotFound      = errors.New("work policy group: not found")
	ErrWorkPolicyReferenceNotFound  = errors.New("work policy group: reference not found")
	ErrWorkPolicyResolutionNotFound = errors.New("work policy resolution: not found")
	ErrWorkPolicyMembershipConflict = errors.New("work policy membership: conflict")
	ErrWorkPolicyVersionConflict    = errors.New("work policy version: conflict")
	ErrWorkPolicyLockedPeriod       = errors.New("work policy: locked payroll period")
)

type WorkPolicyVersion struct {
	ID                               uuid.UUID                      `json:"id"`
	GroupID                          uuid.UUID                      `json:"group_id"`
	GroupCode                        string                         `json:"group_code"`
	GroupName                        string                         `json:"group_name"`
	VersionNo                        int                            `json:"version_no"`
	Status                           workpolicy.VersionStatus       `json:"status"`
	RowVersion                       int64                          `json:"row_version"`
	EffectiveFrom                    string                         `json:"effective_from"`
	EffectiveTo                      *string                        `json:"effective_to,omitempty"`
	MinutesPerCredit                 int                            `json:"minutes_per_credit"`
	WorkUnitMinutes                  int                            `json:"work_unit_minutes"`
	RoundingPolicy                   string                         `json:"rounding_policy"`
	EarlyArrivalStrategy             string                         `json:"early_arrival_strategy"`
	LateArrivalGraceMinutes          int                            `json:"late_arrival_grace_minutes"`
	EarlyOTEnabled                   bool                           `json:"early_ot_enabled"`
	EarlyOTQualificationGraceMinutes int                            `json:"early_ot_qualification_grace_minutes"`
	EarlyOTMaxMinutes                *int                           `json:"early_ot_max_minutes,omitempty"`
	BreakAutoDeduct                  bool                           `json:"break_auto_deduct"`
	BreakIsOvertime                  bool                           `json:"break_is_overtime"`
	OTAggregationMode                string                         `json:"ot_aggregation_mode"`
	WorkingPatternID                 *uuid.UUID                     `json:"working_pattern_id,omitempty"`
	PayMethod                        workpolicy.PayMethod           `json:"pay_method"`
	MonthlyDenominator               *workpolicy.MonthlyDenominator `json:"monthly_denominator,omitempty"`
	MonthlyStandardDays              *decimal.Decimal               `json:"monthly_standard_days,omitempty"`
	RateComponentDefinitionID        *uuid.UUID                     `json:"rate_component_definition_id,omitempty"`
	PaidLeaveCreditMode              workpolicy.PaidLeaveCreditMode `json:"paid_leave_credit_mode"`
	OvertimePolicyVersionID          uuid.UUID                      `json:"overtime_policy_version_id"`
	CreatedBy                        uuid.UUID                      `json:"created_by"`
	CreatedAt                        string                         `json:"created_at"`
	UpdatedAt                        string                         `json:"updated_at"`
	PublishedBy                      *uuid.UUID                     `json:"published_by,omitempty"`
	PublishedAt                      *string                        `json:"published_at,omitempty"`
}

type WorkPolicyGroupSummary struct {
	ID                    uuid.UUID          `json:"id"`
	Code                  string             `json:"code"`
	Name                  string             `json:"name"`
	Description           *string            `json:"description,omitempty"`
	ArchivedAt            *string            `json:"archived_at,omitempty"`
	ActiveMembershipCount int64              `json:"active_membership_count"`
	CurrentPublished      *WorkPolicyVersion `json:"current_published,omitempty"`
	LatestDraft           *WorkPolicyVersion `json:"latest_draft,omitempty"`
	CreatedAt             string             `json:"created_at"`
	UpdatedAt             string             `json:"updated_at"`
}

type WorkPolicyGroupListResponse struct {
	Data  []WorkPolicyGroupSummary `json:"data"`
	Total int64                    `json:"total"`
	Page  int                      `json:"page"`
	Limit int                      `json:"limit"`
}

type WorkPolicyVersionListResponse struct {
	Data  []WorkPolicyVersion `json:"data"`
	Total int64               `json:"total"`
	Page  int                 `json:"page"`
	Limit int                 `json:"limit"`
}

type WorkPolicyValidationIssue = workpolicy.ValidationIssue

type WorkPolicyValidationResponse struct {
	VersionID uuid.UUID                   `json:"version_id"`
	Valid     bool                        `json:"valid"`
	Blockers  []WorkPolicyValidationIssue `json:"blockers"`
	Warnings  []WorkPolicyValidationIssue `json:"warnings"`
	CheckedAt string                      `json:"checked_at"`
}

type WorkPolicyMembership struct {
	ID                       uuid.UUID `json:"id"`
	EmploymentRelationshipID uuid.UUID `json:"employment_relationship_id"`
	GroupID                  uuid.UUID `json:"group_id"`
	EffectiveFrom            string    `json:"effective_from"`
	EffectiveTo              *string   `json:"effective_to,omitempty"`
	CreatedAt                string    `json:"created_at,omitempty"`
}

type WorkPolicyVersionFieldsPayload struct {
	EffectiveFrom                    string                         `json:"effective_from"`
	EffectiveTo                      *string                        `json:"effective_to,omitempty"`
	MinutesPerCredit                 int                            `json:"minutes_per_credit"`
	WorkUnitMinutes                  int                            `json:"work_unit_minutes"`
	RoundingPolicy                   string                         `json:"rounding_policy"`
	EarlyArrivalStrategy             string                         `json:"early_arrival_strategy"`
	LateArrivalGraceMinutes          int                            `json:"late_arrival_grace_minutes"`
	EarlyOTEnabled                   bool                           `json:"early_ot_enabled"`
	EarlyOTQualificationGraceMinutes int                            `json:"early_ot_qualification_grace_minutes"`
	EarlyOTMaxMinutes                *int                           `json:"early_ot_max_minutes,omitempty"`
	BreakAutoDeduct                  bool                           `json:"break_auto_deduct"`
	BreakIsOvertime                  bool                           `json:"break_is_overtime"`
	OTAggregationMode                string                         `json:"ot_aggregation_mode"`
	WorkingPatternID                 *uuid.UUID                     `json:"working_pattern_id,omitempty"`
	PayMethod                        workpolicy.PayMethod           `json:"pay_method"`
	MonthlyDenominator               *workpolicy.MonthlyDenominator `json:"monthly_denominator,omitempty"`
	MonthlyStandardDays              *decimal.Decimal               `json:"monthly_standard_days,omitempty"`
	RateComponentDefinitionID        *uuid.UUID                     `json:"rate_component_definition_id,omitempty"`
	PaidLeaveCreditMode              workpolicy.PaidLeaveCreditMode `json:"paid_leave_credit_mode"`
	OvertimePolicyVersionID          uuid.UUID                      `json:"overtime_policy_version_id"`
}

type CreateWorkPolicyGroupRequest struct {
	Code        string  `json:"code"`
	Name        string  `json:"name"`
	Description *string `json:"description,omitempty"`
	WorkPolicyVersionFieldsPayload
}

type CreateWorkPolicyVersionRequest struct {
	CloneFromVersionID *uuid.UUID `json:"clone_from_version_id,omitempty"`
	*WorkPolicyVersionFieldsPayload
}

type PatchWorkPolicyVersionRequest struct {
	ExpectedRowVersion int64 `json:"expected_row_version"`
	WorkPolicyVersionFieldsPayload
}

type PublishWorkPolicyVersionRequest struct {
	ExpectedRowVersion int64  `json:"expected_row_version"`
	AutoClosePrevious  bool   `json:"auto_close_previous"`
	IdempotencyKey     string `json:"idempotency_key,omitempty"`
}

type CreateWorkPolicyMembershipRequest struct {
	EmploymentRelationshipID uuid.UUID `json:"employment_relationship_id"`
	EffectiveFrom            string    `json:"effective_from"`
	EffectiveTo              *string   `json:"effective_to,omitempty"`
}

type TransferWorkPolicyMembershipRequest struct {
	EmploymentRelationshipID uuid.UUID `json:"employment_relationship_id"`
	TargetGroupID            uuid.UUID `json:"target_group_id"`
	EffectiveDate            string    `json:"effective_date"`
}

type TransferWorkPolicyMembershipResponse struct {
	ClosedMembership *WorkPolicyMembership `json:"closed_membership,omitempty"`
	NewMembership    WorkPolicyMembership  `json:"new_membership"`
}

type ResolvedWorkPolicyResponse struct {
	MembershipID                     uuid.UUID                      `json:"membership_id"`
	EmploymentRelationshipID         uuid.UUID                      `json:"employment_relationship_id"`
	GroupID                          uuid.UUID                      `json:"group_id"`
	GroupCode                        string                         `json:"group_code"`
	GroupName                        string                         `json:"group_name"`
	VersionID                        uuid.UUID                      `json:"version_id"`
	VersionNo                        int                            `json:"version_no"`
	Status                           workpolicy.VersionStatus       `json:"status"`
	EffectiveFrom                    string                         `json:"effective_from"`
	EffectiveTo                      *string                        `json:"effective_to,omitempty"`
	MinutesPerCredit                 int                            `json:"minutes_per_credit"`
	WorkUnitMinutes                  int                            `json:"work_unit_minutes"`
	RoundingPolicy                   string                         `json:"rounding_policy"`
	EarlyArrivalStrategy             string                         `json:"early_arrival_strategy"`
	LateArrivalGraceMinutes          int                            `json:"late_arrival_grace_minutes"`
	EarlyOTEnabled                   bool                           `json:"early_ot_enabled"`
	EarlyOTQualificationGraceMinutes int                            `json:"early_ot_qualification_grace_minutes"`
	EarlyOTMaxMinutes                *int                           `json:"early_ot_max_minutes,omitempty"`
	BreakAutoDeduct                  bool                           `json:"break_auto_deduct"`
	BreakIsOvertime                  bool                           `json:"break_is_overtime"`
	OTAggregationMode                string                         `json:"ot_aggregation_mode"`
	WorkingPatternID                 *uuid.UUID                     `json:"working_pattern_id,omitempty"`
	PayMethod                        workpolicy.PayMethod           `json:"pay_method"`
	MonthlyDenominator               *workpolicy.MonthlyDenominator `json:"monthly_denominator,omitempty"`
	MonthlyStandardDays              *decimal.Decimal               `json:"monthly_standard_days,omitempty"`
	RateComponentDefinitionID        *uuid.UUID                     `json:"rate_component_definition_id,omitempty"`
	PaidLeaveCreditMode              workpolicy.PaidLeaveCreditMode `json:"paid_leave_credit_mode"`
	OvertimePolicyVersionID          uuid.UUID                      `json:"overtime_policy_version_id"`
	OvertimePolicyID                 uuid.UUID                      `json:"overtime_policy_id"`
	OvertimePolicyVersionNo          int                            `json:"overtime_policy_version_no"`
	DailyCapMinutes                  *int                           `json:"daily_cap_minutes,omitempty"`
	MonthlyCapHours                  *decimal.Decimal               `json:"monthly_cap_hours,omitempty"`
	YearlyCapHours                   *decimal.Decimal               `json:"yearly_cap_hours,omitempty"`
	OvertimeRules                    []workpolicy.OvertimeRule      `json:"overtime_rules"`
}

type WorkPolicyGroupRepository interface {
	ListWorkPolicyGroups(context.Context, int, int) (WorkPolicyGroupListResponse, error)
	ListWorkPolicyGroupVersions(context.Context, workpolicy.GroupID, string, int, int) (WorkPolicyVersionListResponse, error)
	GetWorkPolicyVersion(context.Context, workpolicy.GroupID, workpolicy.VersionID) (WorkPolicyVersion, error)
	ValidateWorkPolicyVersion(context.Context, workpolicy.GroupID, workpolicy.VersionID) (WorkPolicyValidationResponse, error)
	CreateWorkPolicyGroup(context.Context, workpolicy.CreateGroupInput, ids.UserID) (WorkPolicyVersion, error)
	CreateWorkPolicyVersion(context.Context, workpolicy.CreateVersionInput, ids.UserID) (WorkPolicyVersion, error)
	PatchWorkPolicyVersion(context.Context, workpolicy.PatchVersionInput) (WorkPolicyVersion, error)
	PublishWorkPolicyVersion(context.Context, workpolicy.PublishVersionInput) (WorkPolicyVersion, error)
	ResolveWorkPolicyByRelationship(context.Context, uuid.UUID, time.Time) (ResolvedWorkPolicyResponse, error)
	ListWorkPolicyMemberships(context.Context, workpolicy.GroupID) ([]WorkPolicyMembership, error)
	CreateWorkPolicyMembership(context.Context, workpolicy.CreateMembershipInput) (WorkPolicyMembership, error)
	TransferWorkPolicyMembership(context.Context, workpolicy.TransferMembershipInput) (TransferWorkPolicyMembershipResponse, error)
}

type WorkPolicyGroupHandler struct {
	Repository WorkPolicyGroupRepository
	Authorizer Authorizer
	Audit      AuditEmitter
}

func (h *WorkPolicyGroupHandler) List(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListWorkPolicyGroups(r.Context(), pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) ListVersions(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
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
	statusFilter := strings.TrimSpace(r.URL.Query().Get("status"))
	if statusFilter != "" && statusFilter != "draft" && statusFilter != "published" && statusFilter != "archived" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListWorkPolicyGroupVersions(r.Context(), workpolicy.GroupID(groupID), statusFilter, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) Detail(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	versionID, versionOK := pathUUID(r, "version_id")
	if !groupOK || !versionOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.GetWorkPolicyVersion(r.Context(), workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) Validate(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	versionID, versionOK := pathUUID(r, "version_id")
	if !groupOK || !versionOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ValidateWorkPolicyVersion(r.Context(), workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) Resolve(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	relationshipID, ok := pathUUID(r, "relationship_id")
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
	result, err := h.Repository.ResolveWorkPolicyByRelationship(r.Context(), relationshipID, asOfTime)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) Create(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !hasPermission(r, h.Authorizer, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[CreateWorkPolicyGroupRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Code) == "" || len(strings.TrimSpace(req.Code)) > 64 || strings.TrimSpace(req.Name) == "" || len(strings.TrimSpace(req.Name)) > 255 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	draft := normalizeDraftFields(req.WorkPolicyVersionFieldsPayload)
	if err := validateDraftPayload(draft); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.CreateWorkPolicyGroup(r.Context(), workpolicy.CreateGroupInput{
		Code:        strings.TrimSpace(req.Code),
		Name:        strings.TrimSpace(req.Name),
		Description: req.Description,
		Draft:       draft,
	}, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r.Context(), "payroll.work_policy_group.created", userID.String(), result.GroupID.String(), map[string]string{
		"version_id": result.ID.String(), "status": string(result.Status), "pay_method": string(result.PayMethod),
	})
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h *WorkPolicyGroupHandler) CreateVersion(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !hasPermission(r, h.Authorizer, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	if !groupOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CreateWorkPolicyVersionRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	input := workpolicy.CreateVersionInput{
		GroupID:            workpolicy.GroupID(groupID),
		CloneFromVersionID: req.CloneFromVersionID,
	}
	if req.WorkPolicyVersionFieldsPayload != nil {
		draft := normalizeDraftFields(*req.WorkPolicyVersionFieldsPayload)
		if err := validateDraftPayload(draft); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		input.Draft = &draft
	} else if req.CloneFromVersionID == nil || *req.CloneFromVersionID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.CreateWorkPolicyVersion(r.Context(), input, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r.Context(), "payroll.work_policy_version.created", userID.String(), result.GroupID.String(), map[string]string{
		"version_id": result.ID.String(), "status": string(result.Status), "effective_from": result.EffectiveFrom,
	})
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h *WorkPolicyGroupHandler) PatchVersion(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !hasPermission(r, h.Authorizer, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	versionID, versionOK := pathUUID(r, "version_id")
	if !groupOK || !versionOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[PatchWorkPolicyVersionRequest](r, 1<<20)
	if err != nil || req.ExpectedRowVersion < 1 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	draft := normalizeDraftFields(req.WorkPolicyVersionFieldsPayload)
	if err := validateDraftPayload(draft); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.PatchWorkPolicyVersion(r.Context(), workpolicy.PatchVersionInput{
		GroupID:            workpolicy.GroupID(groupID),
		VersionID:          workpolicy.VersionID(versionID),
		ExpectedRowVersion: req.ExpectedRowVersion,
		Draft:              draft,
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r.Context(), "payroll.work_policy_version.updated", userID.String(), result.ID.String(), map[string]string{
		"group_id": result.GroupID.String(), "effective_from": result.EffectiveFrom,
	})
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) PublishVersion(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !hasPermission(r, h.Authorizer, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	versionID, versionOK := pathUUID(r, "version_id")
	if !groupOK || !versionOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[PublishWorkPolicyVersionRequest](r, 1<<20)
	if err != nil || req.ExpectedRowVersion < 1 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	idempotencyKey := strings.TrimSpace(r.Header.Get("Idempotency-Key"))
	if idempotencyKey == "" {
		idempotencyKey = strings.TrimSpace(req.IdempotencyKey)
	}
	result, err := h.Repository.PublishWorkPolicyVersion(r.Context(), workpolicy.PublishVersionInput{
		GroupID:            workpolicy.GroupID(groupID),
		VersionID:          workpolicy.VersionID(versionID),
		ExpectedRowVersion: req.ExpectedRowVersion,
		IdempotencyKey:     idempotencyKey,
		AutoClosePrevious:  req.AutoClosePrevious,
		ActorID:            userID,
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r.Context(), "payroll.work_policy_version.published", userID.String(), result.ID.String(), map[string]string{
		"group_id": result.GroupID.String(), "effective_from": result.EffectiveFrom, "pay_method": string(result.PayMethod),
	})
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) ListMemberships(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	groupID, ok := pathUUID(r, "group_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListWorkPolicyMemberships(r.Context(), workpolicy.GroupID(groupID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) CreateMembership(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !hasPermission(r, h.Authorizer, "payroll:manage") {
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
	result, err := h.Repository.CreateWorkPolicyMembership(r.Context(), workpolicy.CreateMembershipInput{
		GroupID:                  workpolicy.GroupID(groupID),
		EmploymentRelationshipID: req.EmploymentRelationshipID,
		EffectiveFrom:            strings.TrimSpace(req.EffectiveFrom),
		EffectiveTo:              req.EffectiveTo,
		ActorID:                  userID,
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r.Context(), "payroll.work_policy_membership.created", userID.String(), result.ID.String(), map[string]string{
		"group_id": result.GroupID.String(), "employment_relationship_id": result.EmploymentRelationshipID.String(), "effective_from": result.EffectiveFrom,
	})
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h *WorkPolicyGroupHandler) TransferMembership(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !hasPermission(r, h.Authorizer, "payroll:manage") {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[TransferWorkPolicyMembershipRequest](r, 1<<20)
	if err != nil || req.EmploymentRelationshipID == uuid.Nil || req.TargetGroupID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, parseErr := time.Parse("2006-01-02", strings.TrimSpace(req.EffectiveDate)); parseErr != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
		return
	}
	result, err := h.Repository.TransferWorkPolicyMembership(r.Context(), workpolicy.TransferMembershipInput{
		EmploymentRelationshipID: req.EmploymentRelationshipID,
		TargetGroupID:            workpolicy.GroupID(req.TargetGroupID),
		EffectiveDate:            strings.TrimSpace(req.EffectiveDate),
		ActorID:                  userID,
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitAudit(r.Context(), "payroll.work_policy_membership.transferred", userID.String(), result.NewMembership.ID.String(), map[string]string{
		"target_group_id": result.NewMembership.GroupID.String(), "employment_relationship_id": result.NewMembership.EmploymentRelationshipID.String(), "effective_date": result.NewMembership.EffectiveFrom,
	})
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *WorkPolicyGroupHandler) emitAudit(ctx context.Context, name, actorID, resource string, details map[string]string) {
	if h.Audit == nil {
		return
	}
	_ = h.Audit.Emit(ctx, audit.Event{
		Name:     name,
		ActorID:  actorID,
		Resource: resource,
		Outcome:  "success",
		Details:  details,
		At:       time.Now().UTC(),
	})
}

func normalizeDraftFields(p WorkPolicyVersionFieldsPayload) workpolicy.VersionDraftFields {
	workUnit := p.WorkUnitMinutes
	if workUnit == 0 {
		workUnit = 1
	}
	rounding := strings.TrimSpace(p.RoundingPolicy)
	if rounding == "" {
		rounding = "none"
	}
	earlyArrival := strings.TrimSpace(p.EarlyArrivalStrategy)
	if earlyArrival == "" {
		earlyArrival = "clamp"
	}
	otAgg := strings.TrimSpace(p.OTAggregationMode)
	if otAgg == "" {
		otAgg = "aggregate"
	}
	paidLeave := p.PaidLeaveCreditMode
	if paidLeave == "" {
		paidLeave = workpolicy.PaidLeaveCreditStandardDay
	}
	return workpolicy.VersionDraftFields{
		EffectiveFrom:                    strings.TrimSpace(p.EffectiveFrom),
		EffectiveTo:                      p.EffectiveTo,
		MinutesPerCredit:                 p.MinutesPerCredit,
		WorkUnitMinutes:                  workUnit,
		RoundingPolicy:                   rounding,
		EarlyArrivalStrategy:             earlyArrival,
		LateArrivalGraceMinutes:          p.LateArrivalGraceMinutes,
		EarlyOTEnabled:                   p.EarlyOTEnabled,
		EarlyOTQualificationGraceMinutes: p.EarlyOTQualificationGraceMinutes,
		EarlyOTMaxMinutes:                p.EarlyOTMaxMinutes,
		BreakAutoDeduct:                  p.BreakAutoDeduct,
		BreakIsOvertime:                  p.BreakIsOvertime,
		OTAggregationMode:                otAgg,
		WorkingPatternID:                 p.WorkingPatternID,
		PayMethod:                        p.PayMethod,
		MonthlyDenominator:               p.MonthlyDenominator,
		MonthlyStandardDays:              p.MonthlyStandardDays,
		RateComponentDefinitionID:        p.RateComponentDefinitionID,
		PaidLeaveCreditMode:              paidLeave,
		OvertimePolicyVersionID:          p.OvertimePolicyVersionID,
	}
}

func validateDraftPayload(d workpolicy.VersionDraftFields) error {
	from, to, err := workPolicyEffectiveRange(d.EffectiveFrom, d.EffectiveTo)
	if err != nil {
		return ErrWorkPolicyGroupValidation
	}
	rules := workpolicy.VersionRules{
		EffectiveFrom:                    from,
		EffectiveTo:                      to,
		MinutesPerCredit:                 d.MinutesPerCredit,
		WorkUnitMinutes:                  d.WorkUnitMinutes,
		RoundingPolicy:                   d.RoundingPolicy,
		EarlyArrivalStrategy:             d.EarlyArrivalStrategy,
		LateArrivalGraceMinutes:          d.LateArrivalGraceMinutes,
		EarlyOTEnabled:                   d.EarlyOTEnabled,
		EarlyOTQualificationGraceMinutes: d.EarlyOTQualificationGraceMinutes,
		EarlyOTMaxMinutes:                d.EarlyOTMaxMinutes,
		BreakAutoDeduct:                  d.BreakAutoDeduct,
		BreakIsOvertime:                  d.BreakIsOvertime,
		OTAggregationMode:                d.OTAggregationMode,
		WorkingPatternID:                 d.WorkingPatternID,
		PayMethod:                        d.PayMethod,
		MonthlyDenominator:               d.MonthlyDenominator,
		MonthlyStandardDays:              d.MonthlyStandardDays,
		RateComponentDefinitionID:        d.RateComponentDefinitionID,
		PaidLeaveCreditMode:              d.PaidLeaveCreditMode,
		OvertimePolicyVersionID:          d.OvertimePolicyVersionID,
	}
	if err := rules.ValidateStrict(); err != nil {
		return ErrWorkPolicyGroupValidation
	}
	return nil
}

func validateWorkPolicyMembershipRequest(req CreateWorkPolicyMembershipRequest) error {
	if req.EmploymentRelationshipID == uuid.Nil {
		return ErrWorkPolicyGroupValidation
	}
	from, to, err := workPolicyEffectiveRange(req.EffectiveFrom, req.EffectiveTo)
	if err != nil || (to != nil && !to.After(from)) {
		return ErrWorkPolicyGroupValidation
	}
	return nil
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

func (h *WorkPolicyGroupHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrWorkPolicyGroupNotFound), errors.Is(err, ErrWorkPolicyReferenceNotFound), errors.Is(err, ErrWorkPolicyResolutionNotFound), errors.Is(err, ErrEmploymentRelationshipNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrWorkPolicyGroupValidation):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	case errors.Is(err, ErrConflict), errors.Is(err, ErrWorkPolicyMembershipConflict), errors.Is(err, ErrWorkPolicyVersionConflict), errors.Is(err, ErrWorkPolicyLockedPeriod):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
