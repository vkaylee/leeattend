import type { DashboardLayoutPreferences, DashboardView, WidgetId } from "./use-dashboard-layout-presets";
import {
  getNewOrderForView,
  getNewVisibleForView,
} from "./use-dashboard-layout-presets";

export type DashboardPreferencesUpdate = Partial<Omit<DashboardLayoutPreferences, "id" | "user_id">>;

export interface LocalOverrides {
  activeView: DashboardView | null;
  collapsedSections: Record<string, boolean> | null;
  visibleWidgets: Record<WidgetId, boolean> | null;
  widgetOrder: WidgetId[] | null;
}

interface EffectiveState {
  activeView: DashboardView;
  collapsedSections: Record<string, boolean>;
  visibleWidgets: Record<WidgetId, boolean>;
  widgetOrder: WidgetId[];
}

interface DashboardLayoutActions {
  setActiveView: (view: DashboardView) => void;
  setWidgetOrder: (order: WidgetId[]) => void;
  toggleSection: (sectionId: string) => void;
  toggleWidget: (widgetId: WidgetId) => void;
  resetToDefault: () => void;
}

export function useDashboardLayoutActions(
  preferences: DashboardLayoutPreferences | undefined,
  localRef: { current: LocalOverrides },
  persist: (nextLocal: LocalOverrides, update: DashboardPreferencesUpdate) => void,
  getEffectiveState: () => EffectiveState,
): DashboardLayoutActions {
  const setActiveView = (view: DashboardView) => {
    const current = getEffectiveState();
    const visibleWidgets = getNewVisibleForView(view, preferences);
    const widgetOrder = getNewOrderForView(view, preferences);
    const nextLocal = { ...localRef.current, activeView: view, visibleWidgets, widgetOrder };
    persist(nextLocal, {
      active_view: view,
      visible_widgets: visibleWidgets,
      widget_order: widgetOrder,
      collapsed_sections: current.collapsedSections,
    });
  };

  const toggleWidget = (widgetId: WidgetId) => {
    const current = getEffectiveState();
    const visibleWidgets = { ...current.visibleWidgets, [widgetId]: !current.visibleWidgets[widgetId] };
    const widgetOrder = getNewOrderForView("custom", preferences);
    const nextLocal: LocalOverrides = { ...localRef.current, activeView: "custom", visibleWidgets, widgetOrder };
    persist(nextLocal, {
      active_view: "custom",
      visible_widgets: visibleWidgets,
      widget_order: widgetOrder,
      collapsed_sections: current.collapsedSections,
    });
  };

  const setWidgetOrder = (widgetOrder: WidgetId[]) => {
    const current = getEffectiveState();
    const nextLocal: LocalOverrides = { ...localRef.current, activeView: "custom", widgetOrder };
    persist(nextLocal, {
      active_view: "custom",
      visible_widgets: current.visibleWidgets,
      widget_order: widgetOrder,
      collapsed_sections: current.collapsedSections,
    });
  };

  const toggleSection = (sectionId: string) => {
    const current = getEffectiveState();
    const collapsedSections = {
      ...current.collapsedSections,
      [sectionId]: !current.collapsedSections[sectionId],
    };
    const nextLocal = { ...localRef.current, collapsedSections };
    persist(nextLocal, {
      active_view: current.activeView,
      visible_widgets: current.visibleWidgets,
      widget_order: current.widgetOrder,
      collapsed_sections: collapsedSections,
    });
  };

  const resetToDefault = () => {
    const collapsedSections = { summary: false, charts: false, actions: true };
    const visibleWidgets = getNewVisibleForView("overview", undefined);
    const widgetOrder = getNewOrderForView("overview", undefined);
    const nextLocal: LocalOverrides = { activeView: "overview", visibleWidgets, widgetOrder, collapsedSections };
    persist(nextLocal, {
      active_view: "overview",
      visible_widgets: visibleWidgets,
      widget_order: widgetOrder,
      collapsed_sections: collapsedSections,
    });
  };

  return { setActiveView, setWidgetOrder, toggleSection, toggleWidget, resetToDefault };
}
