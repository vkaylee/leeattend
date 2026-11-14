package api

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const (
	scheduleHistoryPermission   = "schedules:history:read"
	scheduleEvidencePermission  = "schedules:evidence:export"
	scheduleHistoryDefaultLimit = 50
	scheduleHistoryMaxLimit     = 100
)

var (
	ErrScheduleEvidenceUnavailable = errors.New("schedule evidence schema unavailable")
	ErrScheduleHistoryNotFound     = errors.New("schedule history not found")
	ErrScheduleEvidenceNotFound    = errors.New("schedule evidence not found")
)

type ScheduleHistoryCursor struct {
	BusinessDate   string    `json:"business_date"`
	EmploymentID   uuid.UUID `json:"employment_id"`
	RevisionNumber int64     `json:"revision_number"`
	RevisionID     uuid.UUID `json:"revision_id"`
}

type ScheduleHistoryQuery struct {
	EmploymentID uuid.UUID
	StartDate    *string
	EndDate      *string
	Cursor       *ScheduleHistoryCursor
	Limit        int
}

type ScheduleHistoryEntry struct {
	RevisionID         uuid.UUID       `json:"revision_id"`
	EmploymentID       uuid.UUID       `json:"employment_id"`
	EmployerID         uuid.UUID       `json:"employer_id"`
	BusinessDate       string          `json:"business_date"`
	Timezone           string          `json:"timezone"`
	RevisionNumber     int64           `json:"revision_number"`
	State              string          `json:"state"`
	PreviousRevisionID *uuid.UUID      `json:"previous_revision_id"`
	PublicationBatchID *uuid.UUID      `json:"publication_batch_id"`
	Source             string          `json:"source"`
	SourceSnapshot     json.RawMessage `json:"source_snapshot"`
	ResolverVersion    string          `json:"resolver_version"`
	Reason             string          `json:"reason"`
	PublishedBy        *uuid.UUID      `json:"published_by"`
	PublishedActorType string          `json:"published_actor_type"`
	PublishedActorName *string         `json:"published_actor_name"`
	PublishedAt        *time.Time      `json:"published_at"`
	CreatedAt          time.Time       `json:"created_at"`
}

type ScheduleHistoryResponse struct {
	Data       []ScheduleHistoryEntry `json:"data"`
	NextCursor *string                `json:"next_cursor"`
}

type ScheduleEvidenceSegment struct {
	ID                     uuid.UUID       `json:"id"`
	EmploymentID           uuid.UUID       `json:"employment_id"`
	BusinessDate           string          `json:"business_date"`
	ScheduleRevisionID     *uuid.UUID      `json:"schedule_revision_id"`
	SelectedShiftVersionID *uuid.UUID      `json:"selected_shift_version_id"`
	ScheduledStartAt       *time.Time      `json:"scheduled_start_at"`
	ScheduledEndAt         *time.Time      `json:"scheduled_end_at"`
	BreaksSnapshot         json.RawMessage `json:"breaks_snapshot"`
	SortOrder              int             `json:"sort_order"`
}

type ScheduleEvidenceClosureDay struct {
	ID                 uuid.UUID `json:"id"`
	ClosureID          uuid.UUID `json:"closure_id"`
	PeriodID           uuid.UUID `json:"period_id"`
	EmployerID         uuid.UUID `json:"employer_id"`
	EmploymentID       uuid.UUID `json:"employment_id"`
	BusinessDate       string    `json:"business_date"`
	ScheduleRevisionID uuid.UUID `json:"schedule_revision_id"`
	ScheduleState      string    `json:"schedule_state"`
}

type ScheduleEvidenceAuditAction struct {
	ID        uuid.UUID  `json:"id"`
	Action    string     `json:"action"`
	ActorID   *uuid.UUID `json:"actor_id"`
	ActorType *string    `json:"actor_type"`
	ActorName *string    `json:"actor_name"`
	CreatedAt time.Time  `json:"created_at"`
}

type ScheduleEvidenceEvent struct {
	ID        uuid.UUID  `json:"id"`
	Type      string     `json:"type"`
	Action    string     `json:"action"`
	ActorID   *uuid.UUID `json:"actor_id"`
	ActorType *string    `json:"actor_type"`
	ActorName *string    `json:"actor_name"`
	CreatedAt time.Time  `json:"created_at"`
}

type ScheduleEvidenceDiffLink struct {
	RevisionID         uuid.UUID  `json:"revision_id"`
	PreviousRevisionID *uuid.UUID `json:"previous_revision_id"`
}

type ScheduleActivationReadiness struct {
	EmployerID            uuid.UUID `json:"employer_id"`
	GlobalFlagEnabled     bool      `json:"global_flag_enabled"`
	EmployerActivation    bool      `json:"employer_activation"`
	OpenMismatchCount     int64     `json:"open_mismatch_count"`
	DeadLetterCount       int64     `json:"dead_letter_count"`
	LegacyMutationCount   int64     `json:"legacy_mutation_count"`
	CurrentDuplicateCount int64     `json:"current_duplicate_count"`
	ExpectedCoverageDays  int64     `json:"expected_coverage_days"`
	CoveredOfficialDays   int64     `json:"covered_official_days"`
	MissingCoverageDays   int64     `json:"missing_coverage_days"`
	Ready                 bool      `json:"ready"`
}

type ScheduleShadowMismatch struct {
	ID               uuid.UUID       `json:"id"`
	EmployerID       uuid.UUID       `json:"employer_id"`
	EmploymentID     uuid.UUID       `json:"employment_id"`
	BusinessDate     string          `json:"business_date"`
	MismatchKind     string          `json:"mismatch_kind"`
	LegacySnapshot   json.RawMessage `json:"legacy_snapshot"`
	ResolverSnapshot json.RawMessage `json:"resolver_snapshot"`
	ResolverVersion  string          `json:"resolver_version"`
	Status           string          `json:"status"`
	FirstSeenAt      time.Time       `json:"first_seen_at"`
	LastSeenAt       time.Time       `json:"last_seen_at"`
	ResolvedAt       *time.Time      `json:"resolved_at"`
	ResolutionNote   *string         `json:"resolution_note"`
}

type ScheduleEvidenceResponse struct {
	EmploymentID uuid.UUID                     `json:"employment_id"`
	BusinessDate string                        `json:"business_date"`
	Revisions    []ScheduleHistoryEntry        `json:"revisions"`
	Segments     []ScheduleEvidenceSegment     `json:"segments"`
	ClosureDays  []ScheduleEvidenceClosureDay  `json:"closure_days"`
	DiffChain    []ScheduleEvidenceDiffLink    `json:"diff_chain"`
	AuditActions []ScheduleEvidenceAuditAction `json:"audit_actions"`
	Events       []ScheduleEvidenceEvent       `json:"events"`
}

type ScheduleShadowMismatchRepository interface {
	ListScheduleShadowMismatches(context.Context, uuid.UUID, string, string) ([]ScheduleShadowMismatch, error)
	ResolveScheduleShadowMismatch(context.Context, uuid.UUID, string, string, uuid.UUID) error
	GetScheduleActivationReadiness(context.Context, uuid.UUID, string, string) (ScheduleActivationReadiness, error)
}

type ScheduleEvidenceRepository interface {
	AuthorizeScheduleHistory(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	AuthorizeScheduleEmployment(context.Context, uuid.UUID, uuid.UUID, string) (bool, error)
	ListScheduleHistory(context.Context, ScheduleHistoryQuery) (ScheduleHistoryResponse, error)
	GetScheduleEvidence(context.Context, uuid.UUID, uuid.UUID, string) (ScheduleEvidenceResponse, error)
}

type ScheduleShadowMismatchResolutionRequest struct {
	Status string `json:"status"`
	Note   string `json:"note"`
}

func (h ScheduleHandler) ActivationReadiness(w http.ResponseWriter, r *http.Request) {
	if !h.requireScheduleEvidencePermission(w, r, scheduleHistoryPermission) {
		return
	}
	if h.Shadow == nil {
		writeScheduleEvidenceUnavailable(w, r)
		return
	}
	employerID, err := ParseUUID(r.URL.Query().Get("employer_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	startDate, endDate := r.URL.Query().Get("start_date"), r.URL.Query().Get("end_date")
	if _, err := ParseDate(startDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := ParseDate(endDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Shadow.GetScheduleActivationReadiness(r.Context(), employerID, startDate, endDate)
	if err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ScheduleHandler) ResolveShadowMismatch(w http.ResponseWriter, r *http.Request) {
	if !h.requireScheduleEvidencePermission(w, r, scheduleEvidencePermission) {
		return
	}
	if h.Shadow == nil {
		writeScheduleEvidenceUnavailable(w, r)
		return
	}
	mismatchID, err := ParseUUID(r.PathValue("mismatch_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[ScheduleShadowMismatchResolutionRequest](r, 1<<16)
	if err != nil || (request.Status != "resolved" && request.Status != "waived") || strings.TrimSpace(request.Note) == "" || len(request.Note) > 2000 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, _ := httpinfra.UserIDFromContext(r.Context())
	if err := h.Shadow.ResolveScheduleShadowMismatch(r.Context(), mismatchID, request.Status, strings.TrimSpace(request.Note), userID); err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ScheduleHandler) ListShadowMismatches(w http.ResponseWriter, r *http.Request) {
	if !h.requireScheduleEvidencePermission(w, r, scheduleHistoryPermission) {
		return
	}
	if h.Shadow == nil {
		writeScheduleEvidenceUnavailable(w, r)
		return
	}
	employerID, err := ParseUUID(r.URL.Query().Get("employer_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	startDate := r.URL.Query().Get("start_date")
	endDate := r.URL.Query().Get("end_date")
	if _, err := ParseDate(startDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := ParseDate(endDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	mismatches, err := h.Shadow.ListScheduleShadowMismatches(r.Context(), employerID, startDate, endDate)
	if err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]any{"data": mismatches})
}

func (h ScheduleHandler) History(w http.ResponseWriter, r *http.Request) {
	if !h.requireScheduleEvidencePermission(w, r, scheduleHistoryPermission) {
		return
	}
	query, err := parseScheduleHistoryQuery(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	repository, ok := h.scheduleEvidenceRepository()
	if !ok {
		writeScheduleEvidenceUnavailable(w, r)
		return
	}
	userID, _ := httpinfra.UserIDFromContext(r.Context())
	allowed, err := repository.AuthorizeScheduleHistory(r.Context(), userID, query.EmploymentID)
	if err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	if !allowed {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	result, err := repository.ListScheduleHistory(r.Context(), query)
	if err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ScheduleHandler) Evidence(w http.ResponseWriter, r *http.Request) {
	if !h.requireScheduleEvidencePermission(w, r, scheduleEvidencePermission) {
		return
	}
	employmentID, err := ParseUUID(r.PathValue("employment_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	businessDate := r.PathValue("business_date")
	if _, err := ParseDate(businessDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	repository, ok := h.scheduleEvidenceRepository()
	if !ok {
		writeScheduleEvidenceUnavailable(w, r)
		return
	}
	userID, _ := httpinfra.UserIDFromContext(r.Context())
	allowed, err := repository.AuthorizeScheduleEmployment(r.Context(), userID, employmentID, businessDate)
	if err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	if !allowed {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	result, err := repository.GetScheduleEvidence(r.Context(), userID, employmentID, businessDate)
	if err != nil {
		writeScheduleEvidenceError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h ScheduleHandler) requireScheduleEvidencePermission(w http.ResponseWriter, r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return false
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, permission) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return false
	}
	return true
}

func (h ScheduleHandler) scheduleEvidenceRepository() (ScheduleEvidenceRepository, bool) {
	repository, ok := h.Repository.(ScheduleEvidenceRepository)
	return repository, ok && repository != nil
}

func parseScheduleHistoryQuery(r *http.Request) (ScheduleHistoryQuery, error) {
	values := r.URL.Query()
	employmentID, err := ParseUUID(values.Get("employment_id"))
	if err != nil {
		return ScheduleHistoryQuery{}, err
	}
	query := ScheduleHistoryQuery{EmploymentID: employmentID, Limit: scheduleHistoryDefaultLimit}
	for _, field := range []struct {
		name  string
		value *string
	}{{"start_date", nil}, {"end_date", nil}} {
		raw := strings.TrimSpace(values.Get(field.name))
		if raw == "" {
			continue
		}
		if _, err := ParseDate(raw); err != nil {
			return ScheduleHistoryQuery{}, err
		}
		if field.name == "start_date" {
			query.StartDate = &raw
		} else {
			query.EndDate = &raw
		}
	}
	if query.StartDate != nil && query.EndDate != nil && *query.StartDate > *query.EndDate {
		return ScheduleHistoryQuery{}, errors.New("history date range is invalid")
	}
	if raw := values.Get("limit"); raw != "" {
		limit, err := strconv.Atoi(raw)
		if err != nil || limit < 1 || limit > scheduleHistoryMaxLimit {
			return ScheduleHistoryQuery{}, errors.New("history limit is invalid")
		}
		query.Limit = limit
	}
	if raw := values.Get("cursor"); raw != "" {
		cursor, err := decodeScheduleHistoryCursor(raw)
		if err != nil || cursor.EmploymentID != employmentID {
			return ScheduleHistoryQuery{}, errors.New("history cursor is invalid")
		}
		if _, err := ParseDate(cursor.BusinessDate); err != nil || cursor.RevisionNumber < 1 || cursor.RevisionID == uuid.Nil {
			return ScheduleHistoryQuery{}, errors.New("history cursor is invalid")
		}
		query.Cursor = &cursor
	}
	return query, nil
}

func decodeScheduleHistoryCursor(raw string) (ScheduleHistoryCursor, error) {
	if len(raw) > 512 {
		return ScheduleHistoryCursor{}, errors.New("history cursor is too large")
	}
	payload, err := base64.RawURLEncoding.DecodeString(raw)
	if err != nil {
		return ScheduleHistoryCursor{}, err
	}
	var cursor ScheduleHistoryCursor
	if err := json.Unmarshal(payload, &cursor); err != nil {
		return ScheduleHistoryCursor{}, err
	}
	return cursor, nil
}

func writeScheduleEvidenceUnavailable(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
}

func writeScheduleEvidenceError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrScheduleEvidenceUnavailable):
		writeScheduleEvidenceUnavailable(w, r)
	case errors.Is(err, ErrScheduleHistoryNotFound), errors.Is(err, ErrScheduleEvidenceNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
