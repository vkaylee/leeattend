package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type DashboardPreferences struct {
	ID                uuid.UUID       `json:"id"`
	UserID            uuid.UUID       `json:"user_id"`
	ActiveView        string          `json:"active_view"`
	VisibleWidgets    json.RawMessage `json:"visible_widgets"`
	WidgetOrder       json.RawMessage `json:"widget_order"`
	CollapsedSections json.RawMessage `json:"collapsed_sections"`
	WindowLayout      json.RawMessage `json:"window_layout"`
}
type UpdateDashboardPreferences struct {
	ActiveView        *string          `json:"active_view"`
	VisibleWidgets    *json.RawMessage `json:"visible_widgets"`
	WidgetOrder       *json.RawMessage `json:"widget_order"`
	CollapsedSections *json.RawMessage `json:"collapsed_sections"`
	WindowLayout      *json.RawMessage `json:"window_layout"`
}
type DashboardPreferencesRepository interface {
	GetDashboardPreferences(context.Context, ids.UserID) (DashboardPreferences, error)
	UpdateDashboardPreferences(context.Context, ids.UserID, UpdateDashboardPreferences) (DashboardPreferences, error)
}
type DashboardPreferencesHandler struct {
	Repository DashboardPreferencesRepository
	Authorizer Authorizer
}

var defaultWidgetOrder = []string{"kpi_cards", "donut_chart", "quick_stats", "attendance_trend", "department_chart", "overtime_trend", "peak_times", "leave_types", "lateness_by_day", "recruitment_sources", "turnover_analytics", "unexcused_absences", "promo_cards"}
var defaultWidgets = map[string]bool{"kpi_cards": true, "donut_chart": true, "quick_stats": true, "attendance_trend": true, "department_chart": true, "overtime_trend": false, "peak_times": false, "leave_types": false, "lateness_by_day": false, "recruitment_sources": true, "turnover_analytics": false, "unexcused_absences": false, "promo_cards": false}

func normalizePreferences(req UpdateDashboardPreferences) (UpdateDashboardPreferences, error) {
	if req.ActiveView != nil && *req.ActiveView != "overview" && *req.ActiveView != "attendance" && *req.ActiveView != "trends" && *req.ActiveView != "custom" {
		return req, errors.New("invalid active view")
	}
	if req.VisibleWidgets != nil {
		var raw map[string]json.RawMessage
		if err := json.Unmarshal(*req.VisibleWidgets, &raw); err != nil || raw == nil {
			return req, errors.New("invalid visible widgets")
		}
		out := map[string]bool{}
		for k, v := range defaultWidgets {
			out[k] = v
			if value, ok := raw[k]; ok {
				var enabled bool
				if err := json.Unmarshal(value, &enabled); err != nil {
					return req, errors.New("invalid visible widget value")
				}
				out[k] = enabled
			}
		}
		b, _ := json.Marshal(out)
		encoded := json.RawMessage(b)
		req.VisibleWidgets = &encoded
	}
	if req.WidgetOrder != nil {
		var values []json.RawMessage
		if err := json.Unmarshal(*req.WidgetOrder, &values); err != nil || values == nil {
			return req, errors.New("invalid widget order")
		}
		seen := map[string]bool{}
		out := []string{}
		for _, value := range values {
			var widgetID string
			if err := json.Unmarshal(value, &widgetID); err != nil {
				return req, errors.New("invalid widget order item")
			}
			if _, ok := defaultWidgets[widgetID]; ok && !seen[widgetID] {
				seen[widgetID] = true
				out = append(out, widgetID)
			}
		}
		for _, widgetID := range defaultWidgetOrder {
			if !seen[widgetID] {
				out = append(out, widgetID)
			}
		}
		b, _ := json.Marshal(out)
		encoded := json.RawMessage(b)
		req.WidgetOrder = &encoded
	}
	if req.CollapsedSections != nil {
		var raw map[string]json.RawMessage
		if err := json.Unmarshal(*req.CollapsedSections, &raw); err != nil || raw == nil {
			return req, errors.New("invalid collapsed sections")
		}
		out := map[string]bool{"summary": false, "charts": false, "actions": true}
		for key, value := range raw {
			var collapsed bool
			if err := json.Unmarshal(value, &collapsed); err != nil {
				return req, errors.New("invalid collapsed section value")
			}
			out[key] = collapsed
		}
		b, _ := json.Marshal(out)
		encoded := json.RawMessage(b)
		req.CollapsedSections = &encoded
	}
	if req.WindowLayout != nil {
		var raw map[string]json.RawMessage
		if err := json.Unmarshal(*req.WindowLayout, &raw); err != nil || raw == nil {
			return req, errors.New("invalid window layout")
		}
	}
	return req, nil
}

func normalizedPreferences(p DashboardPreferences) (DashboardPreferences, error) {
	active := p.ActiveView
	if active == "" {
		active = "overview"
	}
	visible := p.VisibleWidgets
	if len(visible) == 0 || string(visible) == "null" {
		visible = json.RawMessage(`{}`)
	}
	order := p.WidgetOrder
	if len(order) == 0 || string(order) == "null" {
		order = json.RawMessage(`[]`)
	}
	collapsed := p.CollapsedSections
	if len(collapsed) == 0 || string(collapsed) == "null" {
		collapsed = json.RawMessage(`{}`)
	}
	layout := p.WindowLayout
	if len(layout) == 0 || string(layout) == "null" {
		layout = json.RawMessage(`{}`)
	}
	req, err := normalizePreferences(UpdateDashboardPreferences{
		ActiveView:        &active,
		VisibleWidgets:    &visible,
		WidgetOrder:       &order,
		CollapsedSections: &collapsed,
		WindowLayout:      &layout,
	})
	if err != nil {
		return DashboardPreferences{}, err
	}
	p.ActiveView = *req.ActiveView
	p.VisibleWidgets = *req.VisibleWidgets
	p.WidgetOrder = *req.WidgetOrder
	p.CollapsedSections = *req.CollapsedSections
	if req.WindowLayout != nil {
		p.WindowLayout = *req.WindowLayout
	}
	return p, nil
}
func (h *DashboardPreferencesHandler) Get(w http.ResponseWriter, r *http.Request) {
	h.handle(w, r, false)
}
func (h *DashboardPreferencesHandler) Put(w http.ResponseWriter, r *http.Request) {
	h.handle(w, r, true)
}
func (h *DashboardPreferencesHandler) handle(w http.ResponseWriter, r *http.Request, put bool) {
	uid, uok := httpinfra.UserIDValueFromContext(r.Context())
	if !uok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), uid.UUID(), string(auth.PermissionDashboardPreferences)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if !put {
		p, e := h.Repository.GetDashboardPreferences(r.Context(), uid)
		if e != nil {
			apperror.WriteError(w, r, apperror.Internal(e))
			return
		}
		p, e = normalizedPreferences(p)
		if e != nil {
			apperror.WriteError(w, r, apperror.Internal(e))
			return
		}
		apperror.WriteJSON(w, 200, p)
		return
	}
	req, e := DecodeJSON[UpdateDashboardPreferences](r, 1<<20)
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, e = normalizePreferences(req)
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	p, e := h.Repository.UpdateDashboardPreferences(r.Context(), uid, req)
	if e != nil {
		apperror.WriteError(w, r, apperror.Internal(e))
		return
	}
	apperror.WriteJSON(w, 200, p)
}
