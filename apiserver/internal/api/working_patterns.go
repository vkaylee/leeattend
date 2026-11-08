package api

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type WorkingPattern struct {
	ID              uuid.UUID `json:"id"`
	Name            string    `json:"name"`
	Description     *string   `json:"description"`
	IsActive        bool      `json:"is_active"`
	CreatedAt       string    `json:"created_at"`
	UpdatedAt       string    `json:"updated_at"`
	PatternType     string    `json:"pattern_type"`
	CycleLengthDays *int      `json:"cycle_length_days"`
}

type WorkingPatternDetail struct {
	ID          uuid.UUID  `json:"id"`
	PatternID   uuid.UUID  `json:"pattern_id"`
	ScheduleDay int        `json:"schedule_day"`
	ShiftID     *uuid.UUID `json:"shift_id"`
}

type EmployeeWorkingPattern struct {
	ID           uuid.UUID `json:"id"`
	EmploymentID uuid.UUID `json:"employment_id"`
	PatternID    uuid.UUID `json:"pattern_id"`
	StartDate    string    `json:"start_date"`
	EndDate      *string   `json:"end_date"`
	CreatedAt    string    `json:"created_at"`
	UpdatedAt    string    `json:"updated_at"`
	AnchorDate   *string   `json:"anchor_date"`
}

type WorkingPatternDetailRequest struct {
	ScheduleDay int        `json:"schedule_day"`
	ShiftID     *uuid.UUID `json:"shift_id"`
}

type CreateWorkingPatternRequest struct {
	Name            string                        `json:"name"`
	Description     *string                       `json:"description"`
	PatternType     string                        `json:"pattern_type"`
	CycleLengthDays *int                          `json:"cycle_length_days"`
	Details         []WorkingPatternDetailRequest `json:"details"`
}

type AssignPatternRequest struct {
	PatternID      uuid.UUID `json:"pattern_id"`
	StartDate      string    `json:"start_date"`
	EndDate        *string   `json:"end_date"`
	AnchorDate     *string   `json:"anchor_date"`
	AllowWarnings  bool      `json:"allow_warnings"`
	OverrideReason *string   `json:"override_reason"`
}

type BulkAssignPatternRequest struct {
	EmploymentIDs  []uuid.UUID `json:"employment_ids"`
	PatternID      uuid.UUID   `json:"pattern_id"`
	StartDate      string      `json:"start_date"`
	EndDate        *string     `json:"end_date"`
	AnchorDate     *string     `json:"anchor_date"`
	AllowWarnings  bool        `json:"allow_warnings"`
	OverrideReason *string     `json:"override_reason"`
}

type BulkAssignPatternResponse struct {
	SuccessCount    int         `json:"success_count"`
	FailedEmployees []uuid.UUID `json:"failed_employees"`
}

var (
	ErrPatternValidation = errors.New("working pattern: validation failed")
	ErrPatternNotFound   = errors.New("working pattern: not found")
	ErrPatternConflict   = errors.New("working pattern: assignment conflict")
)

type PatternComplianceChecker interface {
	CheckPatternCompliance(context.Context, uuid.UUID, AssignPatternRequest) error
	CheckBulkPatternCompliance(context.Context, BulkAssignPatternRequest) error
}

type WorkingPatternRepository interface {
	CreatePattern(ctx context.Context, req CreateWorkingPatternRequest) (WorkingPattern, error)
	ListPatterns(ctx context.Context) ([]WorkingPattern, error)
	GetPatternDetails(ctx context.Context, patternID uuid.UUID) ([]WorkingPatternDetail, error)
	UpdatePattern(ctx context.Context, patternID uuid.UUID, req CreateWorkingPatternRequest) (WorkingPattern, error)
	DeletePattern(ctx context.Context, patternID uuid.UUID) error
	AssignPattern(ctx context.Context, employmentID uuid.UUID, req AssignPatternRequest) (EmployeeWorkingPattern, error)
	BulkAssignPattern(ctx context.Context, req BulkAssignPatternRequest) (BulkAssignPatternResponse, error)
	EmploymentPatternHistory(ctx context.Context, employmentID uuid.UUID) ([]EmployeeWorkingPattern, error)
}

type WorkingPatternHandler struct {
	Repository WorkingPatternRepository
	Authorizer Authorizer
	Compliance PatternComplianceChecker
}

func (h WorkingPatternHandler) Create(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "patterns:create") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateWorkingPatternRequest](r, 1<<20)
	if err != nil || validatePatternRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	pattern, err := h.Repository.CreatePattern(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, pattern)
}

func (h WorkingPatternHandler) List(w http.ResponseWriter, r *http.Request) {
	patterns, err := h.Repository.ListPatterns(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, patterns)
}

func (h WorkingPatternHandler) Details(w http.ResponseWriter, r *http.Request) {
	patternID, patternOK := pathUUID(r, "pattern_id")
	if !patternOK {
		h.forbidden(w, r)
		return
	}
	details, err := h.Repository.GetPatternDetails(r.Context(), patternID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, details)
}

func (h WorkingPatternHandler) Update(w http.ResponseWriter, r *http.Request) {
	patternID, patternOK := pathUUID(r, "pattern_id")
	if !patternOK || !h.allowed(r, "patterns:create") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateWorkingPatternRequest](r, 1<<20)
	if err != nil || validatePatternRequest(req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	pattern, err := h.Repository.UpdatePattern(r.Context(), patternID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, pattern)
}

func (h WorkingPatternHandler) Delete(w http.ResponseWriter, r *http.Request) {
	patternID, patternOK := pathUUID(r, "pattern_id")
	if !patternOK || !h.allowed(r, "patterns:delete") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeletePattern(r.Context(), patternID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h WorkingPatternHandler) Assign(w http.ResponseWriter, r *http.Request) {
	employmentID, employmentOK := pathUUID(r, "employment_id")
	if !employmentOK || !h.allowed(r, "patterns:assign") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignPatternRequest](r, 1<<20)
	if err != nil || validateAssignmentDates(req.StartDate, req.EndDate) != nil || req.PatternID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Compliance != nil {
		if err := h.Compliance.CheckPatternCompliance(r.Context(), employmentID, req); err != nil {
			h.writeError(w, r, err)
			return
		}
	}
	assignment, err := h.Repository.AssignPattern(r.Context(), employmentID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, assignment)
}

func (h WorkingPatternHandler) BulkAssign(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "patterns:assign") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[BulkAssignPatternRequest](r, 1<<20)
	if err != nil || len(req.EmploymentIDs) == 0 || len(req.EmploymentIDs) > 366 || validateAssignmentDates(req.StartDate, req.EndDate) != nil || req.PatternID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Compliance != nil {
		if err := h.Compliance.CheckBulkPatternCompliance(r.Context(), req); err != nil {
			h.writeError(w, r, err)
			return
		}
	}
	result, err := h.Repository.BulkAssignPattern(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h WorkingPatternHandler) History(w http.ResponseWriter, r *http.Request) {
	employmentID, employmentOK := pathUUID(r, "employment_id")
	if !employmentOK {
		h.forbidden(w, r)
		return
	}
	history, err := h.Repository.EmploymentPatternHistory(r.Context(), employmentID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, history)
}

func validatePatternRequest(req CreateWorkingPatternRequest) error {
	if req.Name == "" || len(req.Details) == 0 {
		return ErrPatternValidation
	}
	patternType := req.PatternType
	if patternType == "" {
		patternType = "weekly"
	}
	if patternType != "weekly" && patternType != "rotation" {
		return ErrPatternValidation
	}
	if patternType == "weekly" && req.CycleLengthDays != nil {
		return ErrPatternValidation
	}
	if patternType == "rotation" && (req.CycleLengthDays == nil || *req.CycleLengthDays <= 0) {
		return ErrPatternValidation
	}
	maxDay := 6
	if patternType == "rotation" {
		maxDay = *req.CycleLengthDays - 1
	}
	for _, detail := range req.Details {
		if detail.ScheduleDay < 0 || detail.ScheduleDay > maxDay {
			return ErrPatternValidation
		}
	}
	return nil
}

func validateAssignmentDates(start string, end *string) error {
	startDate, err := time.Parse("2006-01-02", start)
	if err != nil {
		return ErrPatternValidation
	}
	if end != nil && *end != "" {
		endDate, err := time.Parse("2006-01-02", *end)
		if err != nil || endDate.Before(startDate) {
			return ErrPatternValidation
		}
	}
	return nil
}

func (h WorkingPatternHandler) allowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, permission)
}

func (h WorkingPatternHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h WorkingPatternHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	var compErr *ShiftComplianceError
	if errors.As(err, &compErr) {
		if compErr.Hard {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrPatternNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrPatternValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrPatternConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
