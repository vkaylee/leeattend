package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type WorkSegment struct {
	ID                        uuid.UUID       `json:"id"`
	WorkspaceID               uuid.UUID       `json:"workspace_id"`
	EmploymentID              uuid.UUID       `json:"employment_id"`
	Date                      string          `json:"date"`
	TemplateID                uuid.UUID       `json:"template_id"`
	IntervalID                *uuid.UUID      `json:"interval_id"`
	SortOrder                 int             `json:"sort_order"`
	ScheduledStart            string          `json:"scheduled_start"`
	ScheduledEnd              string          `json:"scheduled_end"`
	SegmentType               string          `json:"segment_type"`
	SegmentStatus             string          `json:"segment_status"`
	CompensationRate          decimal.Decimal `json:"compensation_rate"`
	GapCompensationBefore     string          `json:"gap_compensation_before"`
	GapCompensationRateBefore decimal.Decimal `json:"gap_compensation_rate_before"`
	BreakConfigOverride       json.RawMessage `json:"break_config_override,omitempty"`
	OTConfigOverride          json.RawMessage `json:"ot_config_override,omitempty"`
	Source                    string          `json:"source"`
	SourceDetail              json.RawMessage `json:"source_detail"`
	Notes                     *string         `json:"notes"`
	GenerationMetadata        json.RawMessage `json:"generation_metadata,omitempty"`
	CreatedAt                 string          `json:"created_at"`
	UpdatedAt                 string          `json:"updated_at"`
}

type CreateWorkSegmentRequest struct {
	EmploymentID              uuid.UUID       `json:"employment_id"`
	Date                      string          `json:"date"`
	TemplateID                uuid.UUID       `json:"template_id"`
	IntervalID                *uuid.UUID      `json:"interval_id"`
	SortOrder                 int             `json:"sort_order"`
	ScheduledStart            string          `json:"scheduled_start"`
	ScheduledEnd              string          `json:"scheduled_end"`
	SegmentType               string          `json:"segment_type"`
	SegmentStatus             string          `json:"segment_status"`
	CompensationRate          decimal.Decimal `json:"compensation_rate"`
	GapCompensationBefore     string          `json:"gap_compensation_before"`
	GapCompensationRateBefore decimal.Decimal `json:"gap_compensation_rate_before"`
	BreakConfigOverride       json.RawMessage `json:"break_config_override"`
	OTConfigOverride          json.RawMessage `json:"ot_config_override"`
	Source                    string          `json:"source"`
	SourceDetail              json.RawMessage `json:"source_detail"`
	Notes                     *string         `json:"notes"`
}

type UpdateWorkSegmentRequest struct {
	TemplateID                *uuid.UUID       `json:"template_id"`
	IntervalID                *uuid.UUID       `json:"interval_id"`
	SortOrder                 *int             `json:"sort_order"`
	ScheduledStart            *string          `json:"scheduled_start"`
	ScheduledEnd              *string          `json:"scheduled_end"`
	SegmentType               *string          `json:"segment_type"`
	SegmentStatus             *string          `json:"segment_status"`
	CompensationRate          *decimal.Decimal `json:"compensation_rate"`
	GapCompensationBefore     *string          `json:"gap_compensation_before"`
	GapCompensationRateBefore *decimal.Decimal `json:"gap_compensation_rate_before"`
	BreakConfigOverride       json.RawMessage  `json:"break_config_override"`
	OTConfigOverride          json.RawMessage  `json:"ot_config_override"`
	Source                    *string          `json:"source"`
	SourceDetail              json.RawMessage  `json:"source_detail"`
	Notes                     *string          `json:"notes"`
}

type BulkCreateWorkSegmentsRequest struct {
	Segments        []CreateWorkSegmentRequest `json:"segments"`
	ReplaceExisting bool                       `json:"replace_existing"`
}

type WorkSegmentFilter struct {
	EmploymentIDs []uuid.UUID
	StartDate     *string
	EndDate       *string
	SegmentType   string
	SegmentStatus string
	Source        string
}

type WorkSegmentListResponse struct {
	Segments   []WorkSegment `json:"segments"`
	TotalCount int64         `json:"total_count"`
}

var (
	ErrWorkSegmentValidation = errors.New("work segment: validation failed")
	ErrWorkSegmentNotFound   = errors.New("work segment: not found")
	ErrWorkSegmentConflict   = errors.New("work segment: conflict")
)

type WorkSegmentRepository interface {
	ListSegments(context.Context, uuid.UUID, WorkSegmentFilter, int, int) (WorkSegmentListResponse, error)
	GetSegment(context.Context, uuid.UUID, uuid.UUID) (WorkSegment, error)
	CreateSegment(context.Context, uuid.UUID, CreateWorkSegmentRequest) (WorkSegment, error)
	UpdateSegment(context.Context, uuid.UUID, uuid.UUID, UpdateWorkSegmentRequest) (WorkSegment, error)
	DeleteSegment(context.Context, uuid.UUID, uuid.UUID) error
	BulkCreateSegments(context.Context, uuid.UUID, BulkCreateWorkSegmentsRequest) ([]WorkSegment, error)
}

type WorkSegmentHandler struct {
	Repository WorkSegmentRepository
	Authorizer AttendanceAuthorizer
}

func (h WorkSegmentHandler) List(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, workspaceID, "rosters:read") {
		h.forbidden(w, r)
		return
	}
	page, limit, err := parseSegmentPagination(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	filter, err := parseSegmentFilter(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListSegments(r.Context(), workspaceID, filter, page, limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h WorkSegmentHandler) Get(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	segmentID, segmentOK := pathUUID(r, "segment_id")
	if !ok || !segmentOK || !h.allowed(r, workspaceID, "rosters:read") {
		h.forbidden(w, r)
		return
	}
	segment, err := h.Repository.GetSegment(r.Context(), workspaceID, segmentID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, segment)
}

func (h WorkSegmentHandler) Create(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, workspaceID, "rosters:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateWorkSegmentRequest](r, 1<<20)
	if err != nil || validateCreateSegment(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Source == "" {
		req.Source = "manual"
	}
	segment, err := h.Repository.CreateSegment(r.Context(), workspaceID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, segment)
}

func (h WorkSegmentHandler) Update(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	segmentID, segmentOK := pathUUID(r, "segment_id")
	if !ok || !segmentOK || !h.allowed(r, workspaceID, "rosters:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateWorkSegmentRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	segment, err := h.Repository.UpdateSegment(r.Context(), workspaceID, segmentID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, segment)
}

func (h WorkSegmentHandler) Delete(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	segmentID, segmentOK := pathUUID(r, "segment_id")
	if !ok || !segmentOK || !h.allowed(r, workspaceID, "rosters:manage") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteSegment(r.Context(), workspaceID, segmentID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h WorkSegmentHandler) BulkCreate(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, workspaceID, "rosters:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[BulkCreateWorkSegmentsRequest](r, 4<<20)
	if err != nil || len(req.Segments) > 1000 || len(req.Segments) == 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	for index := range req.Segments {
		if validateCreateSegment(req.Segments[index]) != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		if req.Segments[index].Source == "" {
			req.Segments[index].Source = "manual"
		}
	}
	segments, err := h.Repository.BulkCreateSegments(r.Context(), workspaceID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, segments)
}

func validateCreateSegment(req CreateWorkSegmentRequest) error {
	if req.EmploymentID == uuid.Nil || req.TemplateID == uuid.Nil || req.Date == "" || req.ScheduledStart == "" || req.ScheduledEnd == "" {
		return ErrWorkSegmentValidation
	}
	if _, err := time.Parse("2006-01-02", req.Date); err != nil {
		return ErrWorkSegmentValidation
	}
	if _, err := time.Parse("15:04:05", req.ScheduledStart); err != nil {
		return ErrWorkSegmentValidation
	}
	if _, err := time.Parse("15:04:05", req.ScheduledEnd); err != nil {
		return ErrWorkSegmentValidation
	}
	if req.SortOrder < 0 || req.CompensationRate.IsNegative() || req.GapCompensationRateBefore.IsNegative() {
		return ErrWorkSegmentValidation
	}
	if req.SegmentType == "" {
		req.SegmentType = "work"
	}
	if req.SegmentStatus == "" {
		req.SegmentStatus = "scheduled"
	}
	if req.GapCompensationBefore == "" {
		req.GapCompensationBefore = "unpaid"
	}
	return nil
}

func parseSegmentPagination(r *http.Request) (int, int, error) {
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		return 0, 0, err
	}
	return pagination.Page, minInt(pagination.Limit, 3000), nil
}

func parseSegmentFilter(r *http.Request) (WorkSegmentFilter, error) {
	query := r.URL.Query()
	filter := WorkSegmentFilter{
		StartDate:     optionalDateString(query.Get("start_date")),
		EndDate:       optionalDateString(query.Get("end_date")),
		SegmentType:   query.Get("segment_type"),
		SegmentStatus: query.Get("segment_status"),
		Source:        query.Get("source"),
	}
	if filter.StartDate == nil && query.Get("start_date") != "" || filter.EndDate == nil && query.Get("end_date") != "" {
		return WorkSegmentFilter{}, ErrWorkSegmentValidation
	}
	if raw := query.Get("employment_ids"); raw != "" {
		for _, value := range splitComma(raw) {
			id, err := uuid.Parse(value)
			if err != nil {
				return WorkSegmentFilter{}, ErrWorkSegmentValidation
			}
			filter.EmploymentIDs = append(filter.EmploymentIDs, id)
		}
	}
	return filter, nil
}

func optionalDateString(value string) *string {
	if value == "" {
		return nil
	}
	if _, err := time.Parse("2006-01-02", value); err != nil {
		return nil
	}
	return &value
}

func (h WorkSegmentHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}

func (h WorkSegmentHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h WorkSegmentHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrWorkSegmentNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrWorkSegmentValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrWorkSegmentConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func minInt(left, right int) int {
	if left < right {
		return left
	}
	return right
}

func splitComma(value string) []string {
	parts := make([]string, 0)
	start := 0
	for index := 0; index <= len(value); index++ {
		if index == len(value) || value[index] == ',' {
			if index > start {
				parts = append(parts, value[start:index])
			}
			start = index + 1
		}
	}
	return parts
}
