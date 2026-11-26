package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

type DashboardPreferencesRepository struct {
	DB DBTX
}

var defaultVisibleWidgetsJSON = []byte(`{"kpi_cards":true,"donut_chart":true,"quick_stats":true,"attendance_trend":true,"department_chart":true,"overtime_trend":false,"peak_times":false,"leave_types":false,"lateness_by_day":false,"recruitment_sources":true,"turnover_analytics":false,"unexcused_absences":false,"promo_cards":false}`)
var defaultWidgetOrderJSON = []byte(`["kpi_cards","donut_chart","quick_stats","attendance_trend","department_chart","overtime_trend","peak_times","leave_types","lateness_by_day","recruitment_sources","turnover_analytics","unexcused_absences","promo_cards"]`)
var defaultCollapsedSectionsJSON = []byte(`{"summary":false,"charts":false,"actions":true}`)
var defaultWindowLayoutJSON = []byte(`{}`)

func scanDashboardPreferences(row interface{ Scan(...any) error }) (api.DashboardPreferences, error) {
	var p api.DashboardPreferences
	var created, updated time.Time
	err := row.Scan(&p.ID, &p.UserID, &p.ActiveView, &p.VisibleWidgets, &p.WidgetOrder, &p.CollapsedSections, &p.WindowLayout, &created, &updated)
	return p, err
}
func (r DashboardPreferencesRepository) GetDashboardPreferences(ctx context.Context, userID ids.UserID) (api.DashboardPreferences, error) {
	db, err := requireDB(r.DB, "dashboard preferences")
	if err != nil {
		return api.DashboardPreferences{}, err
	}
	p, err := scanDashboardPreferences(db.QueryRow(ctx, `SELECT id,user_id,active_view,visible_widgets,widget_order,collapsed_sections,window_layout,created_at,updated_at FROM dashboard_preferences WHERE user_id=$1 LIMIT 1`, userID.UUID()))
	if err == nil {
		return p, nil
	}
	if !errors.Is(err, pgx.ErrNoRows) {
		return api.DashboardPreferences{}, err
	}
	return api.DashboardPreferences{UserID: userID.UUID(), VisibleWidgets: append([]byte(nil), defaultVisibleWidgetsJSON...), WidgetOrder: append([]byte(nil), defaultWidgetOrderJSON...), CollapsedSections: append([]byte(nil), defaultCollapsedSectionsJSON...), WindowLayout: append([]byte(nil), defaultWindowLayoutJSON...), ActiveView: "overview"}, nil
}
func (r DashboardPreferencesRepository) UpdateDashboardPreferences(ctx context.Context, userID ids.UserID, req api.UpdateDashboardPreferences) (api.DashboardPreferences, error) {
	db, err := requireDB(r.DB, "dashboard preferences")
	if err != nil {
		return api.DashboardPreferences{}, err
	}
	_, err = db.Exec(ctx, `INSERT INTO dashboard_preferences (id,user_id,active_view,visible_widgets,widget_order,collapsed_sections,window_layout,created_at,updated_at) VALUES ($1,$2,'overview',$3,$4,$5,$6,NOW(),NOW()) ON CONFLICT (user_id) DO NOTHING`, uuid.New(), userID.UUID(), defaultVisibleWidgetsJSON, defaultWidgetOrderJSON, defaultCollapsedSectionsJSON, defaultWindowLayoutJSON)
	if err != nil {
		return api.DashboardPreferences{}, err
	}
	sets := []string{}
	args := []any{}
	add := func(name string, v any) {
		args = append(args, v)
		sets = append(sets, fmt.Sprintf("%s=$%d", name, len(args)))
	}
	if req.ActiveView != nil {
		add("active_view", *req.ActiveView)
	}
	if req.VisibleWidgets != nil {
		add("visible_widgets", *req.VisibleWidgets)
	}
	if req.WidgetOrder != nil {
		add("widget_order", *req.WidgetOrder)
	}
	if req.CollapsedSections != nil {
		add("collapsed_sections", *req.CollapsedSections)
	}
	if req.WindowLayout != nil {
		add("window_layout", *req.WindowLayout)
	}
	if len(sets) > 0 {
		args = append(args, userID.UUID())
		_, err = db.Exec(ctx, fmt.Sprintf("UPDATE dashboard_preferences SET %s, updated_at=NOW() WHERE user_id=$%d", joinStrings(sets, ", "), len(args)), args...)
		if err != nil {
			return api.DashboardPreferences{}, err
		}
	}
	return r.GetDashboardPreferences(ctx, userID)
}
