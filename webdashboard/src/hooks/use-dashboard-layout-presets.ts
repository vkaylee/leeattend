import { z } from "zod";

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
    visible: Record<WidgetId, boolean>;
}

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

const WIDGET_ID_SET = new Set<WidgetId>(ALL_WIDGETS.map((widget) => widget.id));
const DEFAULT_WIDGET_ORDER: WidgetId[] = [
    "kpi_cards",
    "donut_chart",
    "quick_stats",
    "attendance_trend",
    "department_chart",
    "overtime_trend",
    "peak_times",
    "leave_types",
    "lateness_by_day",
    "recruitment_sources",
    "turnover_analytics",
    "unexcused_absences",
    "promo_cards",
];

export const DEFAULT_VISIBLE_WIDGETS: Record<WidgetId, boolean> = {
    kpi_cards: true,
    donut_chart: true,
    quick_stats: true,
    attendance_trend: true,
    department_chart: true,
    overtime_trend: false,
    peak_times: false,
    leave_types: false,
    lateness_by_day: false,
    recruitment_sources: true,
    turnover_analytics: false,
    unexcused_absences: false,
    promo_cards: false,
};

export const DEFAULT_COLLAPSED_SECTIONS: Record<string, boolean> = {
    summary: false,
    charts: false,
    actions: true,
};

export const PRESET_VIEWS: Record<Exclude<DashboardView, "custom">, PresetViewConfig> = {
    overview: {
        visible: { ...DEFAULT_VISIBLE_WIDGETS },
        order: [...DEFAULT_WIDGET_ORDER],
    },
    attendance: {
        visible: {
            ...DEFAULT_VISIBLE_WIDGETS,
            donut_chart: false,
            quick_stats: false,
            department_chart: true,
            peak_times: true,
            recruitment_sources: false,
        },
        order: ["kpi_cards", "attendance_trend", "department_chart", "peak_times"],
    },
    trends: {
        visible: {
            ...DEFAULT_VISIBLE_WIDGETS,
            kpi_cards: false,
            donut_chart: false,
            quick_stats: false,
            overtime_trend: true,
            leave_types: true,
            lateness_by_day: true,
            recruitment_sources: true,
            turnover_analytics: true,
            unexcused_absences: true,
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

const dashboardPreferencesResponseSchema = z.object({
    active_view: z.enum(["overview", "attendance", "trends", "custom"]),
    collapsed_sections: z.record(z.string(), z.boolean()),
    id: z.string().min(1),
    user_id: z.string().min(1),
    visible_widgets: z.record(z.string(), z.boolean()),
    widget_order: z.array(z.string()),
    workspace_id: z.string().min(1),
}).strict();

export class DashboardPreferencesContractError extends Error {
    constructor() {
        super("Dashboard preferences response does not match the API contract.");
        this.name = "DashboardPreferencesContractError";
    }
}

export function sanitizeWidgetOrder(value: unknown): WidgetId[] {
    const seen = new Set<WidgetId>();
    const order: WidgetId[] = [];
    if (Array.isArray(value)) {
        value.forEach((candidate) => {
            if (typeof candidate !== "string" || !WIDGET_ID_SET.has(candidate as WidgetId)) return;
            const id = candidate as WidgetId;
            if (seen.has(id)) return;
            seen.add(id);
            order.push(id);
        });
    }
    DEFAULT_WIDGET_ORDER.forEach((id) => {
        if (!seen.has(id)) order.push(id);
    });
    return order;
}

export function sanitizeVisibleWidgets(value: unknown): Record<WidgetId, boolean> {
    const visible = { ...DEFAULT_VISIBLE_WIDGETS };
    if (!value || typeof value !== "object" || Array.isArray(value)) return visible;
    Object.entries(value).forEach(([candidate, enabled]) => {
        if (WIDGET_ID_SET.has(candidate as WidgetId) && typeof enabled === "boolean") {
            visible[candidate as WidgetId] = enabled;
        }
    });
    return visible;
}

/**
 * Validates the current API envelope, then removes legacy/unknown widget IDs
 * while merging the backend's defaults for omitted fields.
 */
export function sanitizeDashboardPreferences(value: unknown): DashboardLayoutPreferences {
    const parsed = dashboardPreferencesResponseSchema.safeParse(value);
    if (!parsed.success) {
        throw new DashboardPreferencesContractError();
    }

    return {
        active_view: parsed.data.active_view,
        collapsed_sections: { ...DEFAULT_COLLAPSED_SECTIONS, ...parsed.data.collapsed_sections },
        id: parsed.data.id,
        user_id: parsed.data.user_id,
        visible_widgets: sanitizeVisibleWidgets(parsed.data.visible_widgets),
        widget_order: sanitizeWidgetOrder(parsed.data.widget_order),
        workspace_id: parsed.data.workspace_id,
    };
}

// ── Helpers for resolving view-dependent widget state ─────────────────

export function getNewVisibleForView(
    view: DashboardView,
    preferences: DashboardLayoutPreferences | undefined,
): Record<WidgetId, boolean> {
    if (preferences?.active_view === view) return sanitizeVisibleWidgets(preferences.visible_widgets);
    if (view === "custom") return sanitizeVisibleWidgets(undefined);
    return { ...PRESET_VIEWS[view].visible };
}

export function getNewOrderForView(
    view: DashboardView,
    preferences: DashboardLayoutPreferences | undefined,
): WidgetId[] {
    if (preferences?.active_view === view) return sanitizeWidgetOrder(preferences.widget_order);
    if (view === "custom") return [...DEFAULT_WIDGET_ORDER];
    return [...PRESET_VIEWS[view].order];
}

export function getActiveView(
    localActiveView: DashboardView | null,
    preferences: DashboardLayoutPreferences | undefined,
): DashboardView {
    const candidate = localActiveView ?? preferences?.active_view;
    return candidate === "attendance" || candidate === "trends" || candidate === "custom" ? candidate : "overview";
}

export function getCollapsedSections(
    localCollapsedSections: Record<string, boolean> | null,
    preferences: DashboardLayoutPreferences | undefined,
): Record<string, boolean> {
    return localCollapsedSections ?? preferences?.collapsed_sections ?? { ...DEFAULT_COLLAPSED_SECTIONS };
}

export function getCurrentVisibleWidgets(
    localVisible: Record<WidgetId, boolean> | null,
    activeView: DashboardView,
    preferences: DashboardLayoutPreferences | undefined,
): Record<WidgetId, boolean> {
    return localVisible ? sanitizeVisibleWidgets(localVisible) : getNewVisibleForView(activeView, preferences);
}

export function getCurrentWidgetOrder(
    localOrder: WidgetId[] | null,
    activeView: DashboardView,
    preferences: DashboardLayoutPreferences | undefined,
): WidgetId[] {
    return localOrder ? sanitizeWidgetOrder(localOrder) : getNewOrderForView(activeView, preferences);
}
