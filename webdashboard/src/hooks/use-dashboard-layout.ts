import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

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

interface LocalOverrides {
    activeView: DashboardView | null;
    collapsedSections: Record<string, boolean> | null;
    visibleWidgets: Record<WidgetId, boolean> | null;
    widgetOrder: WidgetId[] | null;
}

function getNewVisibleForView(
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

function getNewOrderForView(
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

function getActiveView(
    localActiveView: DashboardView | null,
    preferences: DashboardLayoutPreferences | undefined
): DashboardView {
    return localActiveView ?? preferences?.active_view ?? "overview";
}

function getCollapsedSections(
    localCollapsedSections: Record<string, boolean> | null,
    preferences: DashboardLayoutPreferences | undefined
): Record<string, boolean> {
    return localCollapsedSections ?? preferences?.collapsed_sections ?? {};
}

function getCurrentVisibleWidgets(
    localVisible: Record<WidgetId, boolean> | null,
    activeView: DashboardView,
    preferences: DashboardLayoutPreferences | undefined
): Record<WidgetId, boolean> {
    return localVisible ?? getNewVisibleForView(activeView, preferences);
}

function getCurrentWidgetOrder(
    localOrder: WidgetId[] | null,
    activeView: DashboardView,
    preferences: DashboardLayoutPreferences | undefined
): WidgetId[] {
    return localOrder ?? getNewOrderForView(activeView, preferences);
}

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
    const wsId = workspace ? workspace.id : undefined;
    const { data: preferences, isLoading, error } = useDashboardPreferences(wsId);
    const mutation = useDashboardMutation(wsId);

    const [local, setLocal] = useState<LocalOverrides>({
        activeView: null,
        collapsedSections: null,
        visibleWidgets: null,
        widgetOrder: null,
    });
    const [prevWorkspaceId, setPrevWorkspaceId] = useState<string | undefined>(wsId);

    if (wsId !== prevWorkspaceId) {
        setPrevWorkspaceId(wsId);
        setLocal({
            activeView: null,
            collapsedSections: null,
            visibleWidgets: null,
            widgetOrder: null,
        });
    }

    const activeView = getActiveView(local.activeView, preferences);
    const visibleWidgets = getCurrentVisibleWidgets(local.visibleWidgets, activeView, preferences);
    const widgetOrder = getCurrentWidgetOrder(local.widgetOrder, activeView, preferences);
    const collapsedSections = getCollapsedSections(local.collapsedSections, preferences);

    const setActiveView = (view: DashboardView) => {
        const newVisible = getNewVisibleForView(view, preferences);
        const newOrder = getNewOrderForView(view, preferences);
        setLocal((p) => ({ ...p, activeView: view, visibleWidgets: newVisible, widgetOrder: newOrder }));
        mutation.mutate({
            active_view: view,
            visible_widgets: newVisible,
            widget_order: newOrder,
            collapsed_sections: collapsedSections,
        });
    };

    const toggleWidget = (widgetId: WidgetId) => {
        const updated = { ...visibleWidgets, [widgetId]: !visibleWidgets[widgetId] };
        const order = getNewOrderForView("custom", preferences);
        setLocal((p) => ({ ...p, activeView: "custom", visibleWidgets: updated, widgetOrder: order }));
        mutation.mutate({
            active_view: "custom",
            visible_widgets: updated,
            widget_order: order,
            collapsed_sections: collapsedSections,
        });
    };

    const setWidgetOrderAction = (order: WidgetId[]) => {
        setLocal((p) => ({ ...p, activeView: "custom", widgetOrder: order }));
        mutation.mutate({
            active_view: "custom",
            visible_widgets: visibleWidgets,
            widget_order: order,
            collapsed_sections: collapsedSections,
        });
    };

    const toggleSection = (sectionId: string) => {
        const updated = { ...collapsedSections, [sectionId]: !collapsedSections[sectionId] };
        setLocal((p) => ({ ...p, collapsedSections: updated }));
        mutation.mutate({
            active_view: activeView,
            visible_widgets: visibleWidgets,
            widget_order: widgetOrder,
            collapsed_sections: updated,
        });
    };

    const resetToDefault = () => {
        const defaultCollapsed = { summary: false, charts: false, actions: true };
        const visible = getNewVisibleForView("overview", preferences);
        const order = getNewOrderForView("overview", preferences);
        setLocal({
            activeView: "overview",
            visibleWidgets: visible,
            widgetOrder: order,
            collapsedSections: defaultCollapsed,
        });
        mutation.mutate({
            active_view: "overview",
            visible_widgets: visible,
            widget_order: order,
            collapsed_sections: defaultCollapsed,
        });
    };

    return {
        activeView,
        visibleWidgets,
        widgetOrder,
        collapsedSections,
        isLoading,
        error,
        isSaving: mutation.isPending,
        setActiveView,
        toggleWidget,
        setWidgetOrder: setWidgetOrderAction,
        toggleSection,
        resetToDefault,
        allWidgets: ALL_WIDGETS,
    };
}

