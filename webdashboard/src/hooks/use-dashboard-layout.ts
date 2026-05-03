import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import { apiFetch } from "@/lib/api";

import { useWorkspace } from "./use-workspace";

// ── Types ──────────────────────────────────────────────────────────

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

// ── Preset view configurations ─────────────────────────────────────

const PRESET_VIEWS: Record<
    Exclude<DashboardView, "custom">,
    {
        visible: Partial<Record<WidgetId, boolean>>;
        order: WidgetId[];
    }
> = {
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

// ── Hook: useDashboardLayout ───────────────────────────────────────

function useDashboardPreferences(workspaceId: string | undefined) {
    return useQuery<DashboardLayoutPreferences>({
        queryKey: ["dashboard-preferences", workspaceId],
        queryFn: async () => {
            const response = await apiFetch("/dashboard/preferences");
            if (!response.ok) throw new Error("Failed to fetch dashboard preferences");
            return (await response.json()) as DashboardLayoutPreferences;
        },
        enabled: !!workspaceId,
    });
}

function useDashboardMutation(workspaceId: string | undefined) {
    const queryClient = useQueryClient();
    const queryKey = ["dashboard-preferences", workspaceId];

    return useMutation({
        mutationFn: async (update: Partial<Omit<DashboardLayoutPreferences, "id" | "user_id" | "workspace_id">>) => {
            const response = await apiFetch("/dashboard/preferences", {
                method: "PUT",
                body: JSON.stringify(update),
            });
            if (!response.ok) throw new Error("Failed to save dashboard preferences");
            return (await response.json()) as DashboardLayoutPreferences;
        },
        onSuccess: (data) => {
            queryClient.setQueryData(queryKey, data);
        },
    });
}

export function useDashboardLayout() {
    const { workspace } = useWorkspace();
    
    const {
        data: preferences,
        isLoading,
        error,
    } = useDashboardPreferences(workspace?.id);

    const mutation = useDashboardMutation(workspace?.id);

    // ── Computed values ──────────────────────────────────────────────

    const activeView: DashboardView = preferences?.active_view ?? "overview";

    const visibleWidgets: Record<WidgetId, boolean> = (() => {
        if (activeView === "custom") {
            return (preferences?.visible_widgets ?? {}) as Record<WidgetId, boolean>;
        }
        return PRESET_VIEWS[activeView].visible as Record<WidgetId, boolean>;
    })();

    const widgetOrder: WidgetId[] = (() => {
        if (activeView === "custom" && preferences?.widget_order.length) {
            return preferences.widget_order;
        }
        return PRESET_VIEWS[activeView as Exclude<DashboardView, "custom">].order;
    })();

    const collapsedSections: Record<string, boolean> =
        preferences?.collapsed_sections ?? {};

    // ── Actions ──────────────────────────────────────────────────────

    const setActiveView = (view: DashboardView) => {
        mutation.mutate({
            active_view: view,
            visible_widgets: view === "custom" ? preferences?.visible_widgets : PRESET_VIEWS[view].visible as Record<string, boolean>,
            widget_order: view === "custom" ? preferences?.widget_order : PRESET_VIEWS[view].order,
            collapsed_sections: preferences?.collapsed_sections,
        });
    };

    const toggleWidget = (widgetId: WidgetId) => {
        const updated = { ...visibleWidgets, [widgetId]: !visibleWidgets[widgetId] };
        mutation.mutate({
            active_view: "custom",
            visible_widgets: updated,
            widget_order: preferences?.widget_order ?? ALL_WIDGETS.map((w) => w.id),
            collapsed_sections: preferences?.collapsed_sections,
        });
    };

    const setWidgetOrder = (order: WidgetId[]) => {
        mutation.mutate({
            active_view: "custom",
            visible_widgets: preferences?.visible_widgets,
            widget_order: order,
            collapsed_sections: preferences?.collapsed_sections,
        });
    };

    const toggleSection = (sectionId: string) => {
        const updated = { ...collapsedSections, [sectionId]: !collapsedSections[sectionId] };
        mutation.mutate({
            active_view: preferences?.active_view,
            visible_widgets: preferences?.visible_widgets,
            widget_order: preferences?.widget_order,
            collapsed_sections: updated,
        });
    };

    const resetToDefault = () => {
        mutation.mutate({
            active_view: "overview",
            visible_widgets: PRESET_VIEWS.overview.visible as Record<string, boolean>,
            widget_order: PRESET_VIEWS.overview.order,
            collapsed_sections: { summary: false, charts: false, actions: true },
        });
    };

    return {
        // State
        activeView,
        visibleWidgets,
        widgetOrder,
        collapsedSections,
        isLoading,
        error,
        isSaving: mutation.isPending,
        // Actions
        setActiveView,
        toggleWidget,
        setWidgetOrder,
        toggleSection,
        resetToDefault,
        // Raw data for CustomizePanel
        allWidgets: ALL_WIDGETS,
    };
}
