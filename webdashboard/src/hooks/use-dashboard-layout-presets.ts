// Preset view configurations for dashboard layout.
// Split out from use-dashboard-layout.ts to keep the hook file under max-lines.

export type WidgetId =
    | "kpi_cards"
    | "donut_chart"
    | "quick_stats"
    | "attendance_trend"
    | "department_chart"
    | "overtime_trend"
    | "peak_times"
    | "leave_types"
    | "lateness_by_day"
    | "recruitment_sources"
    | "turnover_analytics"
    | "unexcused_absences"
    | "promo_cards";

export type DashboardView = "overview" | "attendance" | "trends" | "custom";

export interface DashboardLayoutPreferences {
    active_view: DashboardView;
    collapsed_sections: Record<string, boolean>;
    id: string;
    user_id: string;
    visible_widgets: Record<WidgetId, boolean>;
    widget_order: WidgetId[];
    workspace_id: string;
}

interface PresetViewConfig {
    order: WidgetId[];
    visible: Partial<Record<WidgetId, boolean>>;
}

export const PRESET_VIEWS: Record<Exclude<DashboardView, "custom">, PresetViewConfig> = {
    overview: {
        visible: {
            kpi_cards: true,
            donut_chart: true,
            quick_stats: true,
            attendance_trend: true,
            department_chart: false,
            overtime_trend: false,
            peak_times: false,
            leave_types: false,
            lateness_by_day: false,
            recruitment_sources: true,
            turnover_analytics: false,
            unexcused_absences: false,
            promo_cards: false,
        },
        order: [
            "kpi_cards",
            "donut_chart",
            "quick_stats",
            "attendance_trend",
            "recruitment_sources",
        ],
    },
    attendance: {
        visible: {
            kpi_cards: true,
            donut_chart: false,
            quick_stats: false,
            attendance_trend: true,
            department_chart: true,
            overtime_trend: false,
            peak_times: true,
            leave_types: false,
            lateness_by_day: false,
            recruitment_sources: false,
            turnover_analytics: false,
            unexcused_absences: false,
            promo_cards: false,
        },
        order: [
            "kpi_cards",
            "attendance_trend",
            "department_chart",
            "peak_times",
        ],
    },
    trends: {
        visible: {
            kpi_cards: false,
            donut_chart: false,
            quick_stats: false,
            attendance_trend: true,
            department_chart: false,
            overtime_trend: true,
            peak_times: false,
            leave_types: true,
            lateness_by_day: true,
            recruitment_sources: true,
            turnover_analytics: true,
            unexcused_absences: true,
            promo_cards: false,
        },
        order: [
            "attendance_trend",
            "overtime_trend",
            "leave_types",
            "lateness_by_day",
            "recruitment_sources",
            "turnover_analytics",
            "unexcused_absences",
        ],
    },
};

// ── All widgets for "custom" mode ────────────────────────────────

export const ALL_WIDGETS: { id: WidgetId; labelKey: string }[] = [
    { id: "kpi_cards", labelKey: "dashboard.widgets.kpi_cards" },
    { id: "donut_chart", labelKey: "dashboard.widgets.donut_chart" },
    { id: "quick_stats", labelKey: "dashboard.widgets.quick_stats" },
    { id: "attendance_trend", labelKey: "dashboard.widgets.attendance_trend" },
    { id: "department_chart", labelKey: "dashboard.widgets.department_chart" },
    { id: "overtime_trend", labelKey: "dashboard.widgets.overtime_trend" },
    { id: "peak_times", labelKey: "dashboard.widgets.peak_times" },
    { id: "leave_types", labelKey: "dashboard.widgets.leave_types" },
    { id: "lateness_by_day", labelKey: "dashboard.widgets.lateness_by_day" },
    { id: "recruitment_sources", labelKey: "dashboard.widgets.recruitment_sources" },
    { id: "turnover_analytics", labelKey: "dashboard.widgets.turnover_analytics" },
    { id: "unexcused_absences", labelKey: "dashboard.widgets.unexcused_absences" },
    { id: "promo_cards", labelKey: "dashboard.widgets.promo_cards" },
];

// ── Helpers for resolving view-dependent widget state ─────────────────

export function getNewVisibleForView(
    view: DashboardView,
    preferences: DashboardLayoutPreferences | undefined
): Record<WidgetId, boolean> {
    if (view === "custom") {
        const prefsVisible = preferences?.visible_widgets;
        if (prefsVisible && Object.keys(prefsVisible).length > 0) {
            return prefsVisible;
        }
        const defaults = {} as Record<WidgetId, boolean>;
        ALL_WIDGETS.forEach((w) => {
            defaults[w.id] = true;
        });
        return defaults;
    }
    return PRESET_VIEWS[view].visible as Record<WidgetId, boolean>;
}

export function getNewOrderForView(
    view: DashboardView,
    preferences: DashboardLayoutPreferences | undefined
): WidgetId[] {
    if (view === "custom") {
        const prefsOrder = preferences?.widget_order;
        if (prefsOrder && prefsOrder.length > 0) {
            return prefsOrder;
        }
        return ALL_WIDGETS.map((w) => w.id);
    }
    return PRESET_VIEWS[view].order;
}

export function getActiveView(
    localActiveView: DashboardView | null,
    preferences: DashboardLayoutPreferences | undefined
): DashboardView {
    return localActiveView ?? preferences?.active_view ?? "overview";
}

export function getCollapsedSections(
    localCollapsedSections: Record<string, boolean> | null,
    preferences: DashboardLayoutPreferences | undefined
): Record<string, boolean> {
    return localCollapsedSections ?? preferences?.collapsed_sections ?? {};
}

export function getCurrentVisibleWidgets(
    localVisible: Record<WidgetId, boolean> | null,
    activeView: DashboardView,
    preferences: DashboardLayoutPreferences | undefined
): Record<WidgetId, boolean> {
    return localVisible ?? getNewVisibleForView(activeView, preferences);
}

export function getCurrentWidgetOrder(
    localOrder: WidgetId[] | null,
    activeView: DashboardView,
    preferences: DashboardLayoutPreferences | undefined
): WidgetId[] {
    return localOrder ?? getNewOrderForView(activeView, preferences);
}
