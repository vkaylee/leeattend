package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrHolidayNotFound      = errors.New("holiday: not found")
	ErrHolidayAlreadyExists = errors.New("holiday: already exists")
	ErrHolidayValidation    = errors.New("holiday: validation error")
)

type HolidayRecord struct {
	ID                   uuid.UUID        `json:"id"`
	WorkspaceID          uuid.UUID        `json:"workspace_id"`
	Name                 string           `json:"name"`
	Date                 string           `json:"date"`
	HolidayType          string           `json:"holiday_type"`
	IsPaid               bool             `json:"is_paid"`
	LeaveTypeID          *uuid.UUID       `json:"leave_type_id"`
	OTMultiplier         *decimal.Decimal `json:"ot_multiplier"`
	Description          *string          `json:"description"`
	WorkThresholdPercent *int32           `json:"work_threshold_percent"`
	IsCreditOnWork       bool             `json:"is_credit_on_work"`
	CreatedAt            string           `json:"created_at"`
	UpdatedAt            string           `json:"updated_at"`
	LocationID           *uuid.UUID       `json:"location_id"`
}

type CreateHolidayRequest struct {
	Name                 string           `json:"name"`
	Date                 string           `json:"date"`
	HolidayType          string           `json:"holiday_type"`
	IsPaid               *bool            `json:"is_paid"`
	LeaveTypeID          *uuid.UUID       `json:"leave_type_id"`
	OTMultiplier         *decimal.Decimal `json:"ot_multiplier"`
	Description          *string          `json:"description"`
	WorkThresholdPercent *int32           `json:"work_threshold_percent"`
	IsCreditOnWork       *bool            `json:"is_credit_on_work"`
	LocationID           *uuid.UUID       `json:"location_id"`
}

type UpdateHolidayRequest struct {
	Name                 *string          `json:"name"`
	Date                 *string          `json:"date"`
	HolidayType          *string          `json:"holiday_type"`
	IsPaid               *bool            `json:"is_paid"`
	LeaveTypeID          *uuid.UUID       `json:"leave_type_id"`
	OTMultiplier         *decimal.Decimal `json:"ot_multiplier"`
	Description          *string          `json:"description"`
	WorkThresholdPercent *int32           `json:"work_threshold_percent"`
	IsCreditOnWork       *bool            `json:"is_credit_on_work"`
	LocationID           *uuid.UUID       `json:"location_id"`
}

type PaginatedHolidays struct {
	Data      []HolidayRecord `json:"data"`
	Total     int64           `json:"total"`
	Page      int64           `json:"page"`
	Limit     int64           `json:"limit"`
	Truncated *bool           `json:"truncated,omitempty"`
}

type HolidayLeaveResult struct {
	Type           string
	EmployeeID     *uuid.UUID
	LeaveRequestID *uuid.UUID
	LeaveTypeName  *string
}

func (r HolidayLeaveResult) MarshalJSON() ([]byte, error) {
	switch r.Type {
	case "Paid":
		return json.Marshal("Paid")
	case "LeaveDeducted":
		payload := map[string]any{
			"LeaveDeducted": map[string]any{
				"employee_id":      r.EmployeeID,
				"leave_request_id": r.LeaveRequestID,
				"leave_type_name":  r.LeaveTypeName,
			},
		}
		return json.Marshal(payload)
	case "Unpaid":
		payload := map[string]any{
			"Unpaid": map[string]any{
				"employee_id": r.EmployeeID,
			},
		}
		return json.Marshal(payload)
	default:
		return json.Marshal(r.Type)
	}
}

func (r *HolidayLeaveResult) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err == nil {
		r.Type = s
		return nil
	}
	var obj map[string]json.RawMessage
	if err := json.Unmarshal(data, &obj); err != nil {
		return err
	}
	if raw, ok := obj["LeaveDeducted"]; ok {
		r.Type = "LeaveDeducted"
		var p struct {
			EmployeeID     uuid.UUID `json:"employee_id"`
			LeaveRequestID uuid.UUID `json:"leave_request_id"`
			LeaveTypeName  string    `json:"leave_type_name"`
		}
		if err := json.Unmarshal(raw, &p); err != nil {
			return err
		}
		r.EmployeeID = &p.EmployeeID
		r.LeaveRequestID = &p.LeaveRequestID
		r.LeaveTypeName = &p.LeaveTypeName
		return nil
	}
	if raw, ok := obj["Unpaid"]; ok {
		r.Type = "Unpaid"
		var p struct {
			EmployeeID uuid.UUID `json:"employee_id"`
		}
		if err := json.Unmarshal(raw, &p); err != nil {
			return err
		}
		r.EmployeeID = &p.EmployeeID
		return nil
	}
	return errors.New("invalid holiday leave result payload")
}

type HolidayRepository interface {
	ListHolidays(ctx context.Context, workspaceID uuid.UUID, year *int32, locationID *uuid.UUID, page, limit int64) (PaginatedHolidays, error)
	GetHolidayByID(ctx context.Context, workspaceID, holidayID uuid.UUID) (HolidayRecord, error)
	CreateHoliday(ctx context.Context, workspaceID uuid.UUID, req CreateHolidayRequest) (HolidayRecord, error)
	UpdateHoliday(ctx context.Context, workspaceID, holidayID uuid.UUID, req UpdateHolidayRequest) (HolidayRecord, error)
	DeleteHoliday(ctx context.Context, workspaceID, holidayID uuid.UUID) error
	ProcessHolidayLeave(ctx context.Context, workspaceID, holidayID uuid.UUID) ([]HolidayLeaveResult, error)
}

type HolidayAuthorizer interface {
	WorkspaceAuthorizer
	WorkspaceMemberAuthorizer
}

type HolidayHandler struct {
	Repository HolidayRepository
	Authorizer HolidayAuthorizer
}

func (h HolidayHandler) List(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.authenticateAndAuthorizeMember(w, r, wsID) {
		return
	}

	page := int64(1)
	limit := int64(50)
	var year *int32
	var locationID *uuid.UUID

	query := r.URL.Query()
	if rawPage := query.Get("page"); rawPage != "" {
		p, err := strconv.ParseInt(rawPage, 10, 64)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		if p < 1 {
			p = 1
		}
		page = p
	}
	if rawLimit := query.Get("limit"); rawLimit != "" {
		l, err := strconv.ParseInt(rawLimit, 10, 64)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		if l < 1 {
			l = 1
		} else if l > 3000 {
			l = 3000
		}
		limit = l
	}
	if rawYear := query.Get("year"); rawYear != "" {
		y, err := strconv.ParseInt(rawYear, 10, 32)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		y32 := int32(y)
		year = &y32
	}
	if rawLoc := query.Get("location_id"); rawLoc != "" {
		locUUID, err := uuid.Parse(rawLoc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		locationID = &locUUID
	}

	res, err := h.Repository.ListHolidays(r.Context(), wsID, year, locationID, page, limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, res)
}

func (h HolidayHandler) Get(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	holidayID, idOK := pathUUID(r, "id")
	if !ok || !idOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.authenticateAndAuthorizeMember(w, r, wsID) {
		return
	}

	record, err := h.Repository.GetHolidayByID(r.Context(), wsID, holidayID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func (h HolidayHandler) Create(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.checkManagePermission(w, r, wsID) {
		return
	}

	req, err := DecodeJSON[CreateHolidayRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	if req.Name == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.Date); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.HolidayType == "" {
		req.HolidayType = "national"
	}
	req.HolidayType = strings.TrimSpace(req.HolidayType)
	if req.HolidayType != "national" && req.HolidayType != "company" && req.HolidayType != "half_day" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.WorkThresholdPercent != nil && (*req.WorkThresholdPercent < 0 || *req.WorkThresholdPercent > 100) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	record, err := h.Repository.CreateHoliday(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, record)
}

func (h HolidayHandler) Update(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	holidayID, idOK := pathUUID(r, "id")
	if !ok || !idOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.checkManagePermission(w, r, wsID) {
		return
	}

	req, err := DecodeJSON[UpdateHolidayRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Name != nil {
		trimmed := strings.TrimSpace(*req.Name)
		if trimmed == "" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		req.Name = &trimmed
	}
	if req.Date != nil {
		if _, err := time.Parse("2006-01-02", *req.Date); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	if req.HolidayType != nil {
		t := strings.TrimSpace(*req.HolidayType)
		if t != "national" && t != "company" && t != "half_day" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		req.HolidayType = &t
	}
	if req.WorkThresholdPercent != nil && (*req.WorkThresholdPercent < 0 || *req.WorkThresholdPercent > 100) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	record, err := h.Repository.UpdateHoliday(r.Context(), wsID, holidayID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func (h HolidayHandler) Delete(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	holidayID, idOK := pathUUID(r, "id")
	if !ok || !idOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.checkManagePermission(w, r, wsID) {
		return
	}

	if err := h.Repository.DeleteHoliday(r.Context(), wsID, holidayID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h HolidayHandler) ProcessLeave(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	holidayID, idOK := pathUUID(r, "id")
	if !ok || !idOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.checkManagePermission(w, r, wsID) {
		return
	}

	results, err := h.Repository.ProcessHolidayLeave(r.Context(), wsID, holidayID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, results)
}

func (h HolidayHandler) authenticateAndAuthorizeMember(w http.ResponseWriter, r *http.Request, wsID uuid.UUID) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return false
	}
	if h.Authorizer == nil {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotWorkspaceMember))
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID, wsID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return false
	}
	if !member {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotWorkspaceMember))
		return false
	}
	return true
}

func (h HolidayHandler) checkManagePermission(w http.ResponseWriter, r *http.Request, wsID uuid.UUID) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return false
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, wsID, string(auth.PermissionHolidaysManage)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return false
	}
	return true
}

func (h HolidayHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrHolidayNotFound) || errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyHolidayNotFound))
		return
	}
	if errors.Is(err, ErrLeaveTypeNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyLeaveTypeNotFound))
		return
	}
	if errors.Is(err, ErrHolidayAlreadyExists) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyHolidayAlreadyExists))
		return
	}
	if errors.Is(err, ErrHolidayValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
