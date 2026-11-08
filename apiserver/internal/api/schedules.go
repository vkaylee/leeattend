package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type OrganizationSchedulesQuery struct {
	StartDate     string
	EndDate       string
	EmploymentIDs []uuid.UUID
	Page          int
	Limit         int
}

type ScheduleTrace struct {
	RuleID   *uuid.UUID `json:"rule_id"`
	RuleName string     `json:"rule_name"`
	Priority int        `json:"priority"`
	Status   string     `json:"status"`
	Reason   *string    `json:"reason"`
}

type EffectiveSchedule struct {
	Date                           string          `json:"date"`
	ShiftID                        *uuid.UUID      `json:"shift_id"`
	Source                         string          `json:"source"`
	ShiftName                      *string         `json:"shift_name"`
	ShiftSymbol                    *string         `json:"shift_symbol"`
	StartTime                      *string         `json:"start_time"`
	EndTime                        *string         `json:"end_time"`
	BreakDurationMinutes           *int            `json:"break_duration_minutes"`
	BreakStart                     *string         `json:"break_start"`
	BreakEnd                       *string         `json:"break_end"`
	IsBreakAutoDeduct              *bool           `json:"is_break_auto_deduct"`
	EarlyPunchInGracePeriodMinutes *int            `json:"early_punch_in_grace_period_minutes"`
	AllowEarlyOT                   *bool           `json:"allow_early_ot"`
	BreakIsOvertime                *bool           `json:"break_is_overtime"`
	MaxEarlyOTMinutes              *int            `json:"max_early_ot_minutes"`
	Traces                         []ScheduleTrace `json:"traces"`
	IsHoliday                      bool            `json:"is_holiday"`
	IsHolidayPaid                  bool            `json:"is_holiday_paid"`
	HolidayName                    *string         `json:"holiday_name"`
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

type ScheduleHandler struct {
	Repository ScheduleRepository
	Authorizer Authorizer
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
	result, err := h.Repository.GetOrganizationSchedules(r.Context(), query.start, query.end, query.employmentIDs, query.page, query.limit)
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
	return parsedOrganizationSchedulesQuery{start: start, end: end, employmentIDs: ids, page: pagination.Page, limit: minInt(pagination.Limit, MaxLimit)}, nil
}
