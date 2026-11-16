package api

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrScheduleLifecycleUnavailable = errors.New("schedule lifecycle is unavailable")
	ErrSchedulePreviewStale         = errors.New("schedule lifecycle preview is stale")
	ErrScheduleAlreadyPublished     = errors.New("schedule lifecycle day is already published")
	ErrScheduleRevisionStale        = errors.New("schedule lifecycle revision is stale")
	ErrScheduleNoCurrentRevision    = errors.New("schedule lifecycle has no current revision")
	ErrScheduleLockedPeriod         = errors.New("schedule lifecycle period is locked")
	ErrSchedulePreviewScope         = errors.New("schedule lifecycle preview scope is invalid")
	ErrScheduleTimezoneMismatch     = errors.New("schedule lifecycle timezone does not match resolver")
	ErrScheduleIdempotencyConflict  = errors.New("schedule lifecycle idempotency key conflicts with stored request")
	ErrScheduleForbidden            = errors.New("schedule lifecycle replay is forbidden")
	ErrScheduleValidation           = errors.New("schedule lifecycle validation failed")
	ErrScheduleApprovalInvalid      = errors.New("schedule lifecycle approval is invalid")
	ErrScheduleConflict              = errors.New("schedule lifecycle schedule conflict")
)

const (
	scheduleLifecyclePermissionPreview  = "schedules:preview"
	scheduleLifecyclePermissionPublish  = "schedules:publish:office"
	scheduleLifecyclePermissionAmend    = "schedules:amend"
	scheduleLifecyclePermissionActivate = "schedules:activate"
	scheduleLifecycleResolverVersion    = "schedule-resolver-v1"
)

type SchedulePreviewRequest struct {
	EmploymentIDs      []uuid.UUID      `json:"employment_ids"`
	StartDate          string           `json:"start_date"`
	EndDate            string           `json:"end_date"`
	Timezone           string           `json:"timezone"`
	Mode               string           `json:"mode"`
	ApprovalInstanceID *uuid.UUID       `json:"approval_instance_id"`
	Reason             string           `json:"reason"`
	Retroactive        bool             `json:"retroactive"`
	ExpectedRevisions  map[string]int64 `json:"expected_revisions"`
}

type SchedulePublishRequest struct {
	Mode               string           `json:"mode"`
	EmploymentIDs      []uuid.UUID      `json:"employment_ids"`
	StartDate          string           `json:"start_date"`
	EndDate            string           `json:"end_date"`
	Timezone           string           `json:"timezone"`
	ApprovalInstanceID *uuid.UUID       `json:"approval_instance_id"`
	PreviewID          uuid.UUID        `json:"preview_id"`
	PreviewHash        string           `json:"preview_hash"`
	Reason             string           `json:"reason"`
	Retroactive        bool             `json:"retroactive"`
	ExpectedRevisions  map[string]int64 `json:"expected_revisions"`
}

type ScheduleReplanReplacement struct {
	Status         string                   `json:"status"`
	Source         string                   `json:"source"`
	ShiftVersionID *uuid.UUID               `json:"shift_version_id"`
	Intervals      []ScheduleReplanInterval `json:"intervals"`
}

type ScheduleReplanInterval struct {
	SortOrder int                   `json:"sort_order"`
	Start     string                `json:"start"`
	End       string                `json:"end"`
	Breaks    []ScheduleReplanBreak `json:"breaks"`
}

type ScheduleReplanBreak struct {
	Start string `json:"start"`
	End   string `json:"end"`
}

type ScheduleReplanPreviewRequest struct {
	ExpectedRevision int64                     `json:"expected_revision"`
	Reason           string                    `json:"reason"`
	Replacement      ScheduleReplanReplacement `json:"replacement"`
}

type ScheduleReplanRequest struct {
	ExpectedRevision int64                     `json:"expected_revision"`
	Reason           string                    `json:"reason"`
	Replacement      ScheduleReplanReplacement `json:"replacement"`
	PreviewID        uuid.UUID                 `json:"preview_id"`
	PreviewHash      string                    `json:"preview_hash"`
	ConfirmDiffHash  string                    `json:"confirm_diff_hash"`
}

type SchedulePreviewResponse struct {
	Provisional     bool              `json:"provisional"`
	PreviewID       uuid.UUID         `json:"preview_id"`
	PreviewHash     string            `json:"preview_hash"`
	ExpiresAt       time.Time         `json:"expires_at"`
	ResolverVersion string            `json:"resolver_version"`
	Data            []json.RawMessage `json:"data"`
}

type SchedulePublicationResponse struct {
	BatchID      uuid.UUID   `json:"batch_id"`
	Status       string      `json:"status"`
	PublishedAt  time.Time   `json:"published_at"`
	RevisionIDs  []uuid.UUID `json:"revision_ids"`
	AffectedDays int         `json:"affected_days"`
}

type ScheduleReplanPreviewResponse struct {
	PreviewID          uuid.UUID `json:"preview_id"`
	PreviewHash        string    `json:"preview_hash"`
	ExpiresAt          time.Time `json:"expires_at"`
	OriginalRevisionID uuid.UUID `json:"original_revision_id"`
	Replacement        any       `json:"replacement"`
	Diff               any       `json:"diff"`
	DiffHash           string    `json:"diff_hash"`
}

type ScheduleReplanResponse struct {
	BatchID               uuid.UUID `json:"batch_id"`
	OriginalRevisionID    uuid.UUID `json:"original_revision_id"`
	ReplacementRevisionID uuid.UUID `json:"replacement_revision_id"`
	RevisionNumber        int64     `json:"revision_number"`
	Reason                string    `json:"reason"`
	ActualTime            time.Time `json:"actual_time"`
	Retroactive           bool      `json:"retroactive"`
	Diff                  any       `json:"diff"`
}

type ScheduleActivationRepository interface {
	ActivateScheduleEmployer(context.Context, uuid.UUID, string, string, string, uuid.UUID) error
	DeactivateScheduleEmployer(context.Context, uuid.UUID, string, string, string, uuid.UUID) error
}

type ScheduleLifecycleRepository interface {
	CreatePreview(context.Context, uuid.UUID, SchedulePreviewRequest) (SchedulePreviewResponse, error)
	Publish(context.Context, uuid.UUID, string, SchedulePublishRequest) (SchedulePublicationResponse, error)
	PreviewReplan(context.Context, uuid.UUID, uuid.UUID, string, ScheduleReplanPreviewRequest) (ScheduleReplanPreviewResponse, error)
	CommitReplan(context.Context, uuid.UUID, uuid.UUID, string, string, ScheduleReplanRequest) (ScheduleReplanResponse, error)
}

type OrganizationSchedulesQuery struct {
	StartDate     string
	EndDate       string
	EmploymentIDs []uuid.UUID
	Page          int
	Limit         int
	View          string
}

type ScheduleTrace struct {
	RuleID   *uuid.UUID `json:"rule_id"`
	RuleName string     `json:"rule_name"`
	Priority int        `json:"priority"`
	Status   string     `json:"status"`
	Reason   *string    `json:"reason"`
}

type EffectiveSchedule struct {
	Date                           string            `json:"date"`
	RevisionID                     *uuid.UUID        `json:"revision_id,omitempty"`
	ScheduleStatus                 string            `json:"schedule_status"`
	Provisional                    bool              `json:"provisional"`
	Timezone                       *string           `json:"timezone,omitempty"`
	PublishedAt                    *time.Time        `json:"published_at,omitempty"`
	Intervals                      []json.RawMessage `json:"intervals,omitempty"`
	ShiftID                        *uuid.UUID        `json:"shift_id"`
	Source                         string            `json:"source"`
	ShiftName                      *string           `json:"shift_name"`
	ShiftSymbol                    *string           `json:"shift_symbol"`
	StartTime                      *string           `json:"start_time"`
	EndTime                        *string           `json:"end_time"`
	BreakDurationMinutes           *int              `json:"break_duration_minutes"`
	BreakStart                     *string           `json:"break_start"`
	BreakEnd                       *string           `json:"break_end"`
	IsBreakAutoDeduct              *bool             `json:"is_break_auto_deduct"`
	EarlyPunchInGracePeriodMinutes *int              `json:"early_punch_in_grace_period_minutes"`
	AllowEarlyOT                   *bool             `json:"allow_early_ot"`
	BreakIsOvertime                *bool             `json:"break_is_overtime"`
	MaxEarlyOTMinutes              *int              `json:"max_early_ot_minutes"`
	Traces                         []ScheduleTrace   `json:"traces"`
	IsHoliday                      bool              `json:"is_holiday"`
	IsHolidayPaid                  bool              `json:"is_holiday_paid"`
	HolidayName                    *string           `json:"holiday_name"`
}

type EmployeeSchedule struct {
	EmploymentID     uuid.UUID           `json:"employment_id"`
	EmployeeID       uuid.UUID           `json:"employee_id"`
	EmployeeIDNumber string              `json:"employee_id_number"`
	FullName         string              `json:"full_name"`
	Schedules        []EffectiveSchedule `json:"schedules"`
}

type PaginatedEmployeeSchedules struct {
	Data  []EmployeeSchedule `json:"data"`
	Total int64              `json:"total"`
	Page  int                `json:"page"`
	Limit int                `json:"limit"`
}

type ScheduleRepository interface {
	GetOrganizationSchedules(context.Context, time.Time, time.Time, []uuid.UUID, int, int) (PaginatedEmployeeSchedules, error)
}

type OfficialScheduleRepository interface {
	GetOfficialSchedules(context.Context, time.Time, time.Time, []uuid.UUID, int, int) (PaginatedEmployeeSchedules, error)
}

type ActivationAwareScheduleRepository interface {
	GetActivationAwareSchedules(context.Context, time.Time, time.Time, []uuid.UUID, int, int) (PaginatedEmployeeSchedules, error)
}

type ScheduleHandler struct {
	Repository ScheduleRepository
	Lifecycle  ScheduleLifecycleRepository
	Shadow     ScheduleShadowMismatchRepository
	Activation ScheduleActivationRepository
	Authorizer Authorizer
}

func (h ScheduleHandler) Deactivate(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, scheduleLifecyclePermissionActivate)
	if !ok {
		return
	}
	if h.Activation == nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	employerID, err := ParseUUID(r.PathValue("employer_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[struct {
		StartDate string `json:"start_date"`
		EndDate   string `json:"end_date"`
		Reason    string `json:"reason"`
	}](r, 1<<16)
	if err != nil || strings.TrimSpace(request.Reason) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := ParseDate(request.StartDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := ParseDate(request.EndDate); err != nil || request.StartDate > request.EndDate {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Activation.DeactivateScheduleEmployer(r.Context(), employerID, request.StartDate, request.EndDate, strings.TrimSpace(request.Reason), userID); err != nil {
		if errors.Is(err, ErrScheduleActivationEvidence) {
			apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		} else {
			apperror.WriteError(w, r, apperror.Internal(err))
		}
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ScheduleHandler) Activate(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, scheduleLifecyclePermissionActivate)
	if !ok {
		return
	}
	if h.Activation == nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	employerID, err := ParseUUID(r.PathValue("employer_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[struct {
		StartDate string `json:"start_date"`
		EndDate   string `json:"end_date"`
		Reason    string `json:"reason"`
	}](r, 1<<16)
	if err != nil || strings.TrimSpace(request.Reason) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := ParseDate(request.StartDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := ParseDate(request.EndDate); err != nil || request.StartDate > request.EndDate {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Activation.ActivateScheduleEmployer(r.Context(), employerID, request.StartDate, request.EndDate, strings.TrimSpace(request.Reason), userID); err != nil {
		if errors.Is(err, ErrScheduleActivationNotReady) {
			apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		} else {
			apperror.WriteError(w, r, apperror.Internal(err))
		}
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ScheduleHandler) PreviewRotating(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, "schedules:publish:rotating")
	if !ok {
		return
	}
	request, err := DecodeJSON[SchedulePreviewRequest](r, 1<<20)
	if err != nil || request.Mode != "rotating" || !validPreviewRequest(request) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lifecycle, ok := h.lifecycleRepository()
	if !ok {
		h.lifecycleUnavailable(w, r)
		return
	}
	result, err := lifecycle.CreatePreview(r.Context(), userID, request)
	if err != nil {
		h.writeLifecycleError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ScheduleHandler) Preview(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, scheduleLifecyclePermissionPreview)
	if !ok {
		return
	}
	request, err := DecodeJSON[SchedulePreviewRequest](r, 1<<20)
	if err != nil || !validPreviewRequest(request) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lifecycle, ok := h.lifecycleRepository()
	if !ok {
		h.lifecycleUnavailable(w, r)
		return
	}
	result, err := lifecycle.CreatePreview(r.Context(), userID, request)
	if err != nil {
		h.writeLifecycleError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ScheduleHandler) PublishRotating(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, "schedules:publish:rotating")
	if !ok {
		return
	}
	key, err := idempotencyKey(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[SchedulePublishRequest](r, 1<<20)
	if err != nil || !validPublishRequest(request) || request.Mode != "rotating" || request.ApprovalInstanceID != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lifecycle, ok := h.lifecycleRepository()
	if !ok {
		h.lifecycleUnavailable(w, r)
		return
	}
	result, err := lifecycle.Publish(r.Context(), userID, key, request)
	if err != nil {
		h.writeLifecycleError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h ScheduleHandler) Publish(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, scheduleLifecyclePermissionPublish)
	if !ok {
		return
	}
	key, err := idempotencyKey(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[SchedulePublishRequest](r, 1<<20)
	if err != nil || !validPublishRequest(request) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lifecycle, ok := h.lifecycleRepository()
	if !ok {
		h.lifecycleUnavailable(w, r)
		return
	}
	result, err := lifecycle.Publish(r.Context(), userID, key, request)
	if err != nil {
		h.writeLifecycleError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h ScheduleHandler) PreviewReplan(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, scheduleLifecyclePermissionAmend)
	if !ok {
		return
	}
	employmentID, date, ok := lifecyclePath(w, r)
	if !ok {
		return
	}
	request, err := DecodeJSON[ScheduleReplanPreviewRequest](r, 1<<20)
	if err != nil || !validReplanPreviewRequest(request) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lifecycle, ok := h.lifecycleRepository()
	if !ok {
		h.lifecycleUnavailable(w, r)
		return
	}
	result, err := lifecycle.PreviewReplan(r.Context(), userID, employmentID, date, request)
	if err != nil {
		h.writeLifecycleError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ScheduleHandler) CommitReplan(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.requirePermission(w, r, scheduleLifecyclePermissionAmend)
	if !ok {
		return
	}
	employmentID, date, ok := lifecyclePath(w, r)
	if !ok {
		return
	}
	key, err := idempotencyKey(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[ScheduleReplanRequest](r, 1<<20)
	if err != nil || !validReplanRequest(request) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lifecycle, ok := h.lifecycleRepository()
	if !ok {
		h.lifecycleUnavailable(w, r)
		return
	}
	result, err := lifecycle.CommitReplan(r.Context(), userID, employmentID, date, key, request)
	if err != nil {
		h.writeLifecycleError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h ScheduleHandler) requirePermission(w http.ResponseWriter, r *http.Request, permission string) (uuid.UUID, bool) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return uuid.Nil, false
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, permission) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return uuid.Nil, false
	}
	return userID, true
}

func (h ScheduleHandler) lifecycleRepository() (ScheduleLifecycleRepository, bool) {
	if h.Lifecycle != nil {
		return h.Lifecycle, true
	}
	lifecycle, ok := h.Repository.(ScheduleLifecycleRepository)
	return lifecycle, ok
}

func (h ScheduleHandler) lifecycleUnavailable(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
}

func (h ScheduleHandler) writeLifecycleError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrScheduleLifecycleUnavailable):
		h.lifecycleUnavailable(w, r)
	case errors.Is(err, ErrScheduleForbidden):
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
	case errors.Is(err, ErrSchedulePreviewStale), errors.Is(err, ErrScheduleRevisionStale), errors.Is(err, ErrScheduleAlreadyPublished), errors.Is(err, ErrScheduleLockedPeriod), errors.Is(err, ErrScheduleNoCurrentRevision), errors.Is(err, ErrScheduleIdempotencyConflict), errors.Is(err, ErrScheduleConflict):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	case errors.Is(err, ErrSchedulePreviewScope), errors.Is(err, ErrScheduleTimezoneMismatch), errors.Is(err, ErrScheduleValidation), errors.Is(err, ErrScheduleApprovalInvalid):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}

func idempotencyKey(r *http.Request) (string, error) {
	key := strings.TrimSpace(r.Header.Get("Idempotency-Key"))
	if key == "" || len(key) > 128 || !isASCII(key) {
		return "", errors.New("invalid idempotency key")
	}
	return key, nil
}

func isASCII(value string) bool {
	for i := 0; i < len(value); i++ {
		if value[i] < 0x20 || value[i] > 0x7e {
			return false
		}
	}
	return true
}

func lifecyclePath(w http.ResponseWriter, r *http.Request) (uuid.UUID, string, bool) {
	employmentID, err := ParseUUID(r.PathValue("employment_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return uuid.Nil, "", false
	}
	date := r.PathValue("business_date")
	if _, err := ParseDate(date); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return uuid.Nil, "", false
	}
	return employmentID, date, true
}

func validPreviewRequest(request SchedulePreviewRequest) bool {
	start, err := ParseDate(request.StartDate)
	if err != nil || request.EndDate == "" || len(request.EmploymentIDs) == 0 || len(request.EmploymentIDs) > 500 || (request.Mode != "office" && request.Mode != "rotating") {
		return false
	}
	if request.Mode == "rotating" && request.ApprovalInstanceID != nil {
		return false
	}
	end, err := ParseDate(request.EndDate)
	if err != nil || end.Before(start) || end.Sub(start).Hours()/24 > 61 {
		return false
	}
	if _, err := ParseTimezone(request.Timezone); err != nil || len(request.ExpectedRevisions) == 0 {
		return false
	}
	return validExpectedRevisions(request.EmploymentIDs, start, end, request.ExpectedRevisions)
}

func validPublishRequest(request SchedulePublishRequest) bool {
	mode := request.Mode
	if mode == "" {
		mode = "office"
	}
	if mode != "office" && mode != "rotating" {
		return false
	}
	return validPreviewRequest(SchedulePreviewRequest{EmploymentIDs: request.EmploymentIDs, StartDate: request.StartDate, EndDate: request.EndDate, Timezone: request.Timezone, Mode: mode, ApprovalInstanceID: request.ApprovalInstanceID, Reason: request.Reason, Retroactive: request.Retroactive, ExpectedRevisions: request.ExpectedRevisions}) && request.PreviewID != uuid.Nil && validHash(request.PreviewHash) && strings.TrimSpace(request.Reason) != ""
}

func validExpectedRevisions(ids []uuid.UUID, start, end time.Time, revisions map[string]int64) bool {
	for _, employmentID := range ids {
		for date := start; !date.After(end); date = date.AddDate(0, 0, 1) {
			value, ok := revisions[employmentID.String()+":"+date.Format("2006-01-02")]
			if !ok || value < 0 {
				return false
			}
		}
	}
	return true
}

func validReplanPreviewRequest(request ScheduleReplanPreviewRequest) bool {
	return request.ExpectedRevision > 0 && validReason(request.Reason) && validReplacement(request.Replacement)
}

func validReplanRequest(request ScheduleReplanRequest) bool {
	return validReplanPreviewRequest(ScheduleReplanPreviewRequest{ExpectedRevision: request.ExpectedRevision, Reason: request.Reason, Replacement: request.Replacement}) && request.PreviewID != uuid.Nil && validHash(request.PreviewHash) && validHash(request.ConfirmDiffHash)
}

func validReason(reason string) bool {
	value := strings.TrimSpace(reason)
	return len(value) >= 1 && len(value) <= 2000
}

func validReplacement(replacement ScheduleReplanReplacement) bool {
	if replacement.Status != "work" || replacement.Source == "" || replacement.ShiftVersionID == nil || len(replacement.Intervals) == 0 {
		return false
	}
	for _, interval := range replacement.Intervals {
		if interval.SortOrder < 0 || interval.Start == "" || interval.End == "" {
			return false
		}
	}
	return true
}

func validHash(value string) bool {
	if len(value) != 64 {
		return false
	}
	_, err := hex.DecodeString(value)
	return err == nil
}

func (h ScheduleHandler) List(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	if h.Authorizer == nil {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotOrganizationMember))
		return
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if !member {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotOrganizationMember))
		return
	}
	query, err := parseOrganizationSchedulesQuery(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var result PaginatedEmployeeSchedules
	if query.view == "official" {
		official, ok := h.Repository.(OfficialScheduleRepository)
		if !ok {
			apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
			return
		}
		result, err = official.GetOfficialSchedules(r.Context(), query.start, query.end, query.employmentIDs, query.page, query.limit)
	} else if query.view == "activation-aware" {
		activationAware, ok := h.Repository.(ActivationAwareScheduleRepository)
		if !ok {
			apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
			return
		}
		result, err = activationAware.GetActivationAwareSchedules(r.Context(), query.start, query.end, query.employmentIDs, query.page, query.limit)
	} else {
		result, err = h.Repository.GetOrganizationSchedules(r.Context(), query.start, query.end, query.employmentIDs, query.page, query.limit)
	}
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

type parsedOrganizationSchedulesQuery struct {
	start, end    time.Time
	employmentIDs []uuid.UUID
	page, limit   int
	view          string
}

func parseOrganizationSchedulesQuery(r *http.Request) (parsedOrganizationSchedulesQuery, error) {
	values := r.URL.Query()
	startValue, endValue := values.Get("start_date"), values.Get("end_date")
	start, err := time.Parse("2006-01-02", startValue)
	if err != nil || startValue == "" {
		return parsedOrganizationSchedulesQuery{}, errors.New("start_date is required")
	}
	end, err := time.Parse("2006-01-02", endValue)
	if err != nil || endValue == "" {
		return parsedOrganizationSchedulesQuery{}, errors.New("end_date is required")
	}
	if end.Before(start) {
		return parsedOrganizationSchedulesQuery{}, errors.New("end_date precedes start_date")
	}
	pagination, err := ParsePagination(values)
	if err != nil {
		return parsedOrganizationSchedulesQuery{}, err
	}
	view := strings.TrimSpace(values.Get("view"))
	if view == "" {
		view = "activation-aware"
	}
	if view != "legacy" && view != "official" && view != "activation-aware" {
		return parsedOrganizationSchedulesQuery{}, errors.New("unsupported schedule view")
	}
	ids := make([]uuid.UUID, 0)
	if raw := strings.TrimSpace(values.Get("employment_ids")); raw != "" {
		for _, value := range strings.Split(raw, ",") {
			id, parseErr := uuid.Parse(strings.TrimSpace(value))
			if parseErr != nil {
				return parsedOrganizationSchedulesQuery{}, parseErr
			}
			ids = append(ids, id)
		}
	}
	return parsedOrganizationSchedulesQuery{start: start, end: end, employmentIDs: ids, page: pagination.Page, limit: minInt(pagination.Limit, MaxLimit), view: view}, nil
}
