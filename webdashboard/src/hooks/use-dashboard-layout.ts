import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";

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

const EMPTY_LOCAL_OVERRIDES: LocalOverrides = {
    activeView: null,
    collapsedSections: null,
    visibleWidgets: null,
    widgetOrder: null,
};

type DashboardPreferencesUpdate = Partial<Omit<DashboardLayoutPreferences, "id" | "user_id">>;

interface PendingDashboardWrite {
    sequence: number;
    update: DashboardPreferencesUpdate;
    rollbackLocal: LocalOverrides;
}

// ── Hook: useDashboardLayout ───────────────────────────────────────

function useDashboardPreferences() {
    return useQuery<DashboardLayoutPreferences>({
        queryKey: ["dashboard-preferences"],
        queryFn: async () => {
            const response = await apiFetch("/api/v1/dashboard/preferences");
            if (!response.ok) {
                throw new DashboardPreferencesError("Dashboard preferences could not be loaded.", response.status);
            }
            try {
                return sanitizeDashboardPreferences(await response.json());
            } catch {
                throw new DashboardPreferencesError("Dashboard preferences returned an invalid response.", response.status);
            }
        },
    });
}

function useDashboardMutation() {
    return useMutation({
        mutationFn: async (update: DashboardPreferencesUpdate) => {
            const response = await apiFetch("/api/v1/dashboard/preferences", {
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
    });
}

export function useDashboardLayout() {
    const { data: preferences, isLoading, error } = useDashboardPreferences();
    const mutation = useDashboardMutation();
    const queryClient = useQueryClient();
    const queryKey = ["dashboard-preferences"];
    const [local, setLocal] = useState<LocalOverrides>(EMPTY_LOCAL_OVERRIDES);
    const localRef = useRef<LocalOverrides>(EMPTY_LOCAL_OVERRIDES);
    const mutationSequence = useRef(0);
    const activeWriteRef = useRef<PendingDashboardWrite | null>(null);
    const queuedWriteRef = useRef<PendingDashboardWrite | null>(null);
    const [saveError, setSaveError] = useState<DashboardPreferencesError | null>(null);

    const getEffectiveState = () => {
        const currentActiveView = getActiveView(localRef.current.activeView, preferences);
        return {
            activeView: currentActiveView,
            visibleWidgets: getCurrentVisibleWidgets(
                localRef.current.visibleWidgets,
                currentActiveView,
                preferences,
            ),
            widgetOrder: getCurrentWidgetOrder(
                localRef.current.widgetOrder,
                currentActiveView,
                preferences,
            ),
            collapsedSections: getCollapsedSections(localRef.current.collapsedSections, preferences),
        };
    };

    const activeView = getActiveView(local.activeView, preferences);
    const visibleWidgets = getCurrentVisibleWidgets(local.visibleWidgets, activeView, preferences);
    const widgetOrder = getCurrentWidgetOrder(local.widgetOrder, activeView, preferences);
    const collapsedSections = getCollapsedSections(local.collapsedSections, preferences);

    const startWrite = (write: PendingDashboardWrite) => {
        activeWriteRef.current = write;
        void mutation.mutateAsync(write.update)
            .then((data) => {
                if (activeWriteRef.current !== write) return;
                queryClient.setQueryData(queryKey, data);
                const queued = queuedWriteRef.current;
                if (queued) {
                    queuedWriteRef.current = null;
                    startWrite({
                        ...queued,
                        rollbackLocal: EMPTY_LOCAL_OVERRIDES,
                    });
                    return;
                }
                activeWriteRef.current = null;
                if (write.sequence !== mutationSequence.current) return;
                localRef.current = EMPTY_LOCAL_OVERRIDES;
                setLocal(EMPTY_LOCAL_OVERRIDES);
                setSaveError(null);
            })
            .catch((error: unknown) => {
                if (activeWriteRef.current !== write) return;
                const queued = queuedWriteRef.current;
                if (queued) {
                    queuedWriteRef.current = null;
                    startWrite(queued);
                    return;
                }
                activeWriteRef.current = null;
                if (write.sequence !== mutationSequence.current) return;
                localRef.current = write.rollbackLocal;
                setLocal(write.rollbackLocal);
                setSaveError(
                    error instanceof DashboardPreferencesError
                        ? error
                        : new DashboardPreferencesError("Dashboard preferences could not be saved."),
                );
            });
    };

    const persist = (nextLocal: LocalOverrides, update: DashboardPreferencesUpdate) => {
        const previousLocal = localRef.current;
        const sequence = mutationSequence.current + 1;
        mutationSequence.current = sequence;
        setSaveError(null);
        localRef.current = nextLocal;
        setLocal(nextLocal);

        const activeWrite = activeWriteRef.current;
        const write: PendingDashboardWrite = {
            sequence,
            update,
            rollbackLocal: activeWrite?.rollbackLocal ?? previousLocal,
        };
        if (activeWrite) {
            queuedWriteRef.current = write;
        } else {
            startWrite(write);
        }
    };

    const setActiveView = (view: DashboardView) => {
        const current = getEffectiveState();
        const newVisible = getNewVisibleForView(view, preferences);
        const newOrder = getNewOrderForView(view, preferences);
        const nextLocal: LocalOverrides = {
            ...localRef.current,
            activeView: view,
            visibleWidgets: newVisible,
            widgetOrder: newOrder,
        };
        persist(nextLocal, {
            active_view: view,
            visible_widgets: newVisible,
            widget_order: newOrder,
            collapsed_sections: current.collapsedSections,
        });
    };

    const toggleWidget = (widgetId: WidgetId) => {
        const current = getEffectiveState();
        const updated = { ...current.visibleWidgets, [widgetId]: !current.visibleWidgets[widgetId] };
        const order = getNewOrderForView("custom", preferences);
        const nextLocal: LocalOverrides = {
            ...localRef.current,
            activeView: "custom",
            visibleWidgets: updated,
            widgetOrder: order,
        };
        persist(nextLocal, {
            active_view: "custom",
            visible_widgets: updated,
            widget_order: order,
            collapsed_sections: current.collapsedSections,
        });
    };

    const setWidgetOrderAction = (order: WidgetId[]) => {
        const current = getEffectiveState();
        const nextLocal: LocalOverrides = {
            ...localRef.current,
            activeView: "custom",
            widgetOrder: order,
        };
        persist(nextLocal, {
            active_view: "custom",
            visible_widgets: current.visibleWidgets,
            widget_order: order,
            collapsed_sections: current.collapsedSections,
        });
    };

    const toggleSection = (sectionId: string) => {
        const current = getEffectiveState();
        const updated = { ...current.collapsedSections, [sectionId]: !current.collapsedSections[sectionId] };
        const nextLocal = { ...localRef.current, collapsedSections: updated };
        persist(nextLocal, {
            active_view: current.activeView,
            visible_widgets: current.visibleWidgets,
            widget_order: current.widgetOrder,
            collapsed_sections: updated,
        });
    };

    const resetToDefault = () => {
        const defaultCollapsed = { summary: false, charts: false, actions: true };
        const visible = getNewVisibleForView("overview", undefined);
        const order = getNewOrderForView("overview", undefined);
        const nextLocal: LocalOverrides = {
            activeView: "overview",
            visibleWidgets: visible,
            widgetOrder: order,
            collapsedSections: defaultCollapsed,
        };
        persist(nextLocal, {
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
        saveError,
        isSaving: mutation.isPending,
        setActiveView,
        toggleWidget,
        setWidgetOrder: setWidgetOrderAction,
        toggleSection,
        resetToDefault,
        allWidgets: ALL_WIDGETS,
    };
}

