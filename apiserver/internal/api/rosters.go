package api

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

type ShiftRoster struct {
	ID             uuid.UUID  `json:"id"`
	EmploymentID   uuid.UUID  `json:"employment_id"`
	ShiftID        *uuid.UUID `json:"shift_id"`
	ShiftVersionID *uuid.UUID `json:"shift_version_id,omitempty"`
	Date           string     `json:"date"`
	Status         string     `json:"status"`
	Notes          *string    `json:"notes"`
	CreatedAt      string     `json:"created_at"`
	UpdatedAt      string     `json:"updated_at"`
}

type CreateRosterRequest struct {
	EmploymentID   uuid.UUID  `json:"employment_id"`
	ShiftID        *uuid.UUID `json:"shift_id"`
	ShiftVersionID *uuid.UUID `json:"shift_version_id,omitempty"`
	Date           string     `json:"date"`
	Notes          *string    `json:"notes"`
	AllowWarnings  bool       `json:"allow_warnings"`
	OverrideReason *string    `json:"override_reason"`
}

type RosterItem struct {
	Date           string     `json:"date"`
	ShiftID        *uuid.UUID `json:"shift_id"`
	ShiftVersionID *uuid.UUID `json:"shift_version_id,omitempty"`
	Notes          *string    `json:"notes"`
}

type BulkRosterRequest struct {
	Items          []RosterItem `json:"items"`
	EmploymentID   *uuid.UUID   `json:"employment_id"`
	AllowWarnings  bool         `json:"allow_warnings"`
	OverrideReason *string      `json:"override_reason"`
}

type PaginatedShiftRosters struct {
	Data  []ShiftRoster `json:"data"`
	Total int64         `json:"total"`
	Page  int           `json:"page"`
	Limit int           `json:"limit"`
}

type RosterRepository interface {
	ResolveActiveEmployment(context.Context, uuid.UUID) (uuid.UUID, error)
	ListRosters(context.Context, uuid.UUID, string, string, int, int) (PaginatedShiftRosters, error)
	UpsertRoster(context.Context, CreateRosterRequest) (ShiftRoster, error)
	BulkUpsertRoster(context.Context, uuid.UUID, []RosterItem) error
	DeleteRoster(context.Context, uuid.UUID, string) error
}

type RosterHandler struct {
	Repository RosterRepository
	Authorizer Authorizer
}

func (h *RosterHandler) ListEmployee(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := pathUUID(r, "id")
	if !ok || !hasPermission(r, h.Authorizer, "rosters:read") {
		writeForbidden(w, r)
		return
	}
	start, end, page, limit, err := parseRosterQuery(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	result, err := h.Repository.ListRosters(r.Context(), employmentID, start, end, page, limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *RosterHandler) Upsert(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := pathUUID(r, "id")
	if !ok || !hasPermission(r, h.Authorizer, "rosters:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateRosterRequest](r, 1<<20)
	if err != nil || req.Date == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, parseErr := time.Parse("2006-01-02", req.Date); parseErr != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	req.EmploymentID = employmentID
	roster, err := h.Repository.UpsertRoster(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, roster)
}

func (h *RosterHandler) BulkUpsert(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := pathUUID(r, "id")
	if !ok || !hasPermission(r, h.Authorizer, "rosters:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[BulkRosterRequest](r, 4<<20)
	if err != nil || len(req.Items) > 366 || len(req.Items) == 0 || duplicateRosterDates(req.Items) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if err := h.Repository.BulkUpsertRoster(r.Context(), employmentID, req.Items); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusAccepted)
}

func (h *RosterHandler) Delete(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := pathUUID(r, "id")
	if !ok || !hasPermission(r, h.Authorizer, "rosters:manage") {
		writeForbidden(w, r)
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	date := r.PathValue("date")
	if _, err := time.Parse("2006-01-02", date); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.DeleteRoster(r.Context(), employmentID, date); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func parseRosterQuery(r *http.Request) (string, string, int, int, error) {
	query := r.URL.Query()
	start := query.Get("start_date")
	if start == "" {
		start = time.Now().UTC().Format("2006-01-02")
	}
	if _, err := time.Parse("2006-01-02", start); err != nil {
		return "", "", 0, 0, err
	}
	end := query.Get("end_date")
	if end == "" {
		parsed, _ := time.Parse("2006-01-02", start)
		end = parsed.AddDate(0, 0, 30).Format("2006-01-02")
	}
	if _, err := time.Parse("2006-01-02", end); err != nil {
		return "", "", 0, 0, err
	}
	pagination, err := ParsePagination(query)
	if err != nil {
		return "", "", 0, 0, err
	}
	return start, end, pagination.Page, minInt(pagination.Limit, 3000), nil
}

func duplicateRosterDates(items []RosterItem) bool {
	seen := make(map[string]struct{}, len(items))
	for _, item := range items {
		if _, err := time.Parse("2006-01-02", item.Date); err != nil {
			return true
		}
		if _, exists := seen[item.Date]; exists {
			return true
		}
		seen[item.Date] = struct{}{}
	}
	return false
}

func (h *RosterHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrEmploymentNotFound) || errors.Is(err, ErrRosterNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrShiftNotFound) || errors.Is(err, ErrRosterConflict) || errors.Is(err, ErrRosterVersionConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

var (
	ErrRosterNotFound        = errors.New("roster: not found")
	ErrRosterConflict        = errors.New("roster: conflict")
	ErrRosterVersionConflict = errors.New("roster: shift version conflict")
)
