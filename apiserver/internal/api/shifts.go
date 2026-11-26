package api

import (
	"context"
	"errors"
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/compliance"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	"net/http"
	"time"
)

var ErrShiftValidation = errors.New("shift: validation failed")
var ErrEmploymentNotFound = errors.New("shift: employment not found")
var ErrShiftNotFound = errors.New("shift: template not found")
var ErrShiftAssignmentConflict = errors.New("shift: assignment conflict")

// ShiftComplianceError carries typed compliance violations from the domain package.
// Hard violations map to 400; soft violations map to 409 with override details.
type ShiftComplianceError struct {
	Hard       bool
	Violations []compliance.Violation
}

func (e *ShiftComplianceError) Error() string {
	if e.Hard {
		return "shift: compliance hard violation"
	}
	return "shift: compliance override required"
}

type ShiftIntervalRequest struct {
	IntervalName          *string `json:"interval_name,omitempty"`
	StartTime             string  `json:"start_time"`
	EndTime               string  `json:"end_time"`
	StartDayOffset        int     `json:"start_day_offset"`
	EndDayOffset          int     `json:"end_day_offset"`
	StartUTCOffsetMinutes *int    `json:"start_utc_offset_minutes,omitempty"`
	EndUTCOffsetMinutes   *int    `json:"end_utc_offset_minutes,omitempty"`
	BreakDurationMinutes  int     `json:"break_duration_minutes"`
	BreakStart            *string `json:"break_start,omitempty"`
	BreakEnd              *string `json:"break_end,omitempty"`
	BreakPunchMode        string  `json:"break_punch_mode,omitempty"`
}

func (r ShiftIntervalRequest) BreakPunchModeValue() string {
	if r.BreakPunchMode == "" {
		return "none"
	}
	return r.BreakPunchMode
}

type CreateShiftRequest struct {
	Name         string                 `json:"name"`
	ReportSymbol *string                `json:"report_symbol,omitempty"`
	LocationID   *uuid.UUID             `json:"location_id,omitempty"`
	EmployerID   *uuid.UUID             `json:"employer_id,omitempty"`
	Intervals    []ShiftIntervalRequest `json:"intervals"`
}

type UpdateShiftRequest struct {
	Name          string                 `json:"name"`
	ReportSymbol  *string                `json:"report_symbol,omitempty"`
	LocationID    *uuid.UUID             `json:"location_id,omitempty"`
	EmployerID    *uuid.UUID             `json:"employer_id,omitempty"`
	Intervals     []ShiftIntervalRequest `json:"intervals"`
	IsCorrection  bool                   `json:"is_correction"`
	EffectiveDate *string                `json:"effective_date,omitempty"`
	ChangeReason  *string                `json:"change_reason,omitempty"`
}

type ShiftVersionRecord struct {
	ID            uuid.UUID             `json:"id"`
	TemplateID    uuid.UUID             `json:"template_id"`
	VersionNumber int                   `json:"version_number"`
	Name          string                `json:"name"`
	ReportSymbol  *string               `json:"report_symbol"`
	LocationID    *uuid.UUID            `json:"location_id"`
	EmployerID    *uuid.UUID            `json:"employer_id"`
	EffectiveFrom string                `json:"effective_from"`
	EffectiveTo   *string               `json:"effective_to"`
	ChangeType    string                `json:"change_type"`
	ChangeReason  *string               `json:"change_reason"`
	SupersededBy  *uuid.UUID            `json:"superseded_by"`
	CreatedBy     uuid.UUID             `json:"created_by"`
	CreatedAt     string                `json:"created_at"`
	Intervals     []ShiftIntervalRecord `json:"intervals"`
}

type AssignShiftRequest struct {
	EmploymentID   uuid.UUID `json:"employment_id"`
	ShiftID        uuid.UUID `json:"shift_id"`
	StartDate      string    `json:"start_date"`
	EndDate        *string   `json:"end_date,omitempty"`
	AllowWarnings  bool      `json:"allow_warnings"`
	OverrideReason *string   `json:"override_reason,omitempty"`
}
type ShiftIntervalRecord struct {
	ID                    uuid.UUID `json:"id"`
	VersionID             uuid.UUID `json:"version_id"`
	SortOrder             int       `json:"sort_order"`
	IntervalName          *string   `json:"interval_name"`
	StartTime             string    `json:"start_time"`
	EndTime               string    `json:"end_time"`
	StartDayOffset        int       `json:"start_day_offset"`
	EndDayOffset          int       `json:"end_day_offset"`
	StartUTCOffsetMinutes *int      `json:"start_utc_offset_minutes,omitempty"`
	EndUTCOffsetMinutes   *int      `json:"end_utc_offset_minutes,omitempty"`
	BreakDurationMinutes  int       `json:"break_duration_minutes"`
	BreakStart            *string   `json:"break_start"`
	BreakEnd              *string   `json:"break_end"`
	BreakPunchMode        string    `json:"break_punch_mode"`
}

type ShiftRecord struct {
	ID            uuid.UUID             `json:"id"`
	Name          string                `json:"name"`
	ReportSymbol  *string               `json:"report_symbol"`
	IsActive      bool                  `json:"is_active"`
	ParentShiftID *uuid.UUID            `json:"parent_shift_id"`
	LocationID    *uuid.UUID            `json:"location_id"`
	EmployerID    *uuid.UUID            `json:"employer_id"`
	CreatedAt     string                `json:"created_at"`
	UpdatedAt     string                `json:"updated_at"`
	Intervals     []ShiftIntervalRecord `json:"intervals,omitempty"`
}

type PaginatedShifts struct {
	Data  []ShiftRecord `json:"data"`
	Total int64         `json:"total"`
	Page  int           `json:"page"`
	Limit int           `json:"limit"`
}

type ShiftRepository interface {
	CreateShift(ctx context.Context, actorID uuid.UUID, req CreateShiftRequest) (ShiftRecord, error)
	ListShifts(ctx context.Context, locationID, employerID *uuid.UUID, page, limit int) (PaginatedShifts, error)
	UpdateShift(ctx context.Context, templateID, actorID uuid.UUID, req UpdateShiftRequest) (ShiftRecord, error)
	ListShiftVersions(ctx context.Context, templateID uuid.UUID) ([]ShiftVersionRecord, error)
	AssignShift(ctx context.Context, actorID uuid.UUID, req AssignShiftRequest) error
}

type ShiftHandler struct {
	Repository ShiftRepository
	Authorizer Authorizer
}

func (h *ShiftHandler) Create(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "shifts:create") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[CreateShiftRequest](r, 1<<20)
	if err != nil || request.Name == "" || len(request.Intervals) == 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := validateIntervals(request.Intervals); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	shift, err := h.Repository.CreateShift(r.Context(), actorID, request)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, shift)
}

func (h *ShiftHandler) List(w http.ResponseWriter, r *http.Request) {
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	locationID, err := optionalUUID(r.URL.Query().Get("location_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employerID, err := optionalUUID(r.URL.Query().Get("employer_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	shifts, err := h.Repository.ListShifts(r.Context(), locationID, employerID, pagination.Page, pagination.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, shifts)
}

func (h *ShiftHandler) Update(w http.ResponseWriter, r *http.Request) {
	templateID, templateOK := pathUUID(r, "shift_id")
	if !templateOK || !hasPermission(r, h.Authorizer, "shifts:update") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[UpdateShiftRequest](r, 1<<20)
	if err != nil || request.Name == "" || len(request.Intervals) == 0 || validateIntervals(request.Intervals) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if request.EffectiveDate != nil && *request.EffectiveDate != "" {
		if _, err := time.Parse("2006-01-02", *request.EffectiveDate); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	shift, err := h.Repository.UpdateShift(r.Context(), templateID, actorID, request)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, shift)
}

func (h *ShiftHandler) Versions(w http.ResponseWriter, r *http.Request) {
	templateID, templateOK := pathUUID(r, "shift_id")
	if !templateOK || !hasPermission(r, h.Authorizer, "shifts:read") {
		writeForbidden(w, r)
		return
	}
	versions, err := h.Repository.ListShiftVersions(r.Context(), templateID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, versions)
}

func (h *ShiftHandler) Assign(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "shifts:assign") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[AssignShiftRequest](r, 1<<20)
	if err != nil || request.EmploymentID == uuid.Nil || request.ShiftID == uuid.Nil || request.StartDate == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", request.StartDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if request.EndDate != nil && *request.EndDate != "" {
		end, err := time.Parse("2006-01-02", *request.EndDate)
		start, _ := time.Parse("2006-01-02", request.StartDate)
		if err != nil || end.Before(start) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.AssignShift(r.Context(), actorID, request); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func validateIntervals(intervals []ShiftIntervalRequest) error {
	for _, interval := range intervals {
		if _, err := parseClock(interval.StartTime); err != nil {
			return ErrShiftValidation
		}
		if _, err := parseClock(interval.EndTime); err != nil {
			return ErrShiftValidation
		}
		if interval.BreakDurationMinutes < 0 {
			return ErrShiftValidation
		}
		if interval.BreakPunchModeValue() != "none" && interval.BreakPunchModeValue() != "required" {
			return ErrShiftValidation
		}
		if interval.StartDayOffset < 0 || interval.EndDayOffset < interval.StartDayOffset {
			return ErrShiftValidation
		}
		startClock, _ := parseClock(interval.StartTime)
		endClock, _ := parseClock(interval.EndTime)
		intervalStart := interval.StartDayOffset*24*60 + startClock.Hour()*60 + startClock.Minute()
		intervalEnd := interval.EndDayOffset*24*60 + endClock.Hour()*60 + endClock.Minute()
		if intervalEnd <= intervalStart {
			return ErrShiftValidation
		}
		if (interval.BreakStart == nil) != (interval.BreakEnd == nil) {
			return ErrShiftValidation
		}
		if interval.BreakStart != nil {
			breakStart, err := breakClockInInterval(*interval.BreakStart, intervalStart, intervalEnd)
			if err != nil {
				return ErrShiftValidation
			}
			breakEnd, err := breakClockInInterval(*interval.BreakEnd, intervalStart, intervalEnd)
			if err != nil || breakEnd <= breakStart {
				return ErrShiftValidation
			}
		}
		if interval.BreakDurationMinutes < 0 || (interval.BreakPunchModeValue() == "none" && interval.BreakDurationMinutes > intervalEnd-intervalStart) {
			return ErrShiftValidation
		}
		if interval.StartUTCOffsetMinutes != nil && !validUTCOffset(*interval.StartUTCOffsetMinutes) || interval.EndUTCOffsetMinutes != nil && !validUTCOffset(*interval.EndUTCOffsetMinutes) {
			return ErrShiftValidation
		}
	}
	return nil
}

func parseClock(value string) (time.Time, error) {
	return time.Parse("15:04:05", value)
}

func breakClockInInterval(value string, intervalStart, intervalEnd int) (int, error) {
	clock, err := parseClock(value)
	if err != nil {
		return 0, err
	}
	minutes := clock.Hour()*60 + clock.Minute()
	firstDay := intervalStart / (24 * 60)
	lastDay := intervalEnd / (24 * 60)
	for day := firstDay; day <= lastDay; day++ {
		candidate := day*24*60 + minutes
		if candidate >= intervalStart && candidate <= intervalEnd {
			return candidate, nil
		}
	}
	return 0, ErrShiftValidation
}

func validUTCOffset(minutes int) bool {
	return minutes >= -14*60 && minutes <= 14*60 && minutes%15 == 0
}

func optionalUUID(value string) (*uuid.UUID, error) {
	if value == "" {
		return nil, nil
	}
	parsed, err := uuid.Parse(value)
	if err != nil {
		return nil, err
	}
	return &parsed, nil
}

func (h *ShiftHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	var complianceErr *ShiftComplianceError
	if errors.As(err, &complianceErr) {
		if complianceErr.Hard {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyShiftOverlapDetected))
		} else {
			apperror.WriteError(w, r, apperror.Conflict(apperror.KeyComplianceOverrideRequired))
		}
		return
	}
	if errors.Is(err, ErrNotFound) || errors.Is(err, ErrEmploymentNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrShiftNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyShiftNotFound))
		return
	}
	if errors.Is(err, ErrShiftValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrShiftAssignmentConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
