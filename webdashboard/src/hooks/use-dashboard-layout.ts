import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { apiFetch } from "@/lib/api/helpers";

import {
    ALL_WIDGETS,
    getActiveView,
    getCollapsedSections,
    getCurrentVisibleWidgets,
    getCurrentWidgetOrder,
    getNewOrderForView,
    getNewVisibleForView,
    sanitizeDashboardPreferences,
    type DashboardLayoutPreferences,
    type DashboardView,
    type WidgetId,
} from "./use-dashboard-layout-presets";

export type { DashboardView, WidgetId } from "./use-dashboard-layout-presets";
export { ALL_WIDGETS } from "./use-dashboard-layout-presets";
import { useWorkspace } from "./use-workspace";

export class DashboardPreferencesError extends Error {
    readonly status?: number;

    constructor(message: string, status?: number) {
        super(message);
        this.name = "DashboardPreferencesError";
        this.status = status;
    }
}

interface LocalOverrides {
    activeView: DashboardView | null;
    collapsedSections: Record<string, boolean> | null;
    visibleWidgets: Record<WidgetId, boolean> | null;
    widgetOrder: WidgetId[] | null;
}

// ── Hook: useDashboardLayout ───────────────────────────────────────

function useDashboardPreferences(workspaceId: string | undefined) {
    return useQuery<DashboardLayoutPreferences>({
        queryKey: ["dashboard-preferences", workspaceId],
        queryFn: async () => {
            const response = await apiFetch("/v1/dashboard/preferences");
            if (!response.ok) {
                throw new DashboardPreferencesError("Dashboard preferences could not be loaded.", response.status);
            }
            try {
                return sanitizeDashboardPreferences(await response.json());
            } catch {
                throw new DashboardPreferencesError("Dashboard preferences returned an invalid response.", response.status);
            }
        },
        enabled: !!workspaceId,
    });
}

function useDashboardMutation(workspaceId: string | undefined) {
    const queryClient = useQueryClient();
    const queryKey = ["dashboard-preferences", workspaceId];

    return useMutation({
        mutationFn: async (update: Partial<Omit<DashboardLayoutPreferences, "id" | "user_id" | "workspace_id">>) => {
            const response = await apiFetch("/v1/dashboard/preferences", {
                method: "PUT",
                body: JSON.stringify(update),
            });
            if (!response.ok) {
                throw new DashboardPreferencesError("Dashboard preferences could not be saved.", response.status);
            }
            try {
                return sanitizeDashboardPreferences(await response.json());
            } catch {
                throw new DashboardPreferencesError("Dashboard preferences returned an invalid response.", response.status);
            }
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

