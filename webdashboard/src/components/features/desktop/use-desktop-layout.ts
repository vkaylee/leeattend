import { useState, useEffect, useCallback, useMemo } from "react";

import type { DesktopLayoutState } from "./types";

import {
  getDefaultEnabledWidgetIds,
  getDefaultWidgetOrder,
} from "./desktop-widget-registry";

const STORAGE_KEY = "leeattend-desktop-widgets-v1";

function loadSavedLayout(): DesktopLayoutState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return {
        enabledIds: getDefaultEnabledWidgetIds(),
        order: getDefaultWidgetOrder(),
      };
    }
    const parsed = JSON.parse(raw) as Partial<DesktopLayoutState>;
    return {
      enabledIds: Array.isArray(parsed.enabledIds)
        ? parsed.enabledIds
        : getDefaultEnabledWidgetIds(),
      order: Array.isArray(parsed.order) ? parsed.order : getDefaultWidgetOrder(),
    };
  } catch {
    return {
      enabledIds: getDefaultEnabledWidgetIds(),
      order: getDefaultWidgetOrder(),
    };
  }
}

export function useDesktopLayout() {
  const [layout, setLayout] = useState<DesktopLayoutState>(loadSavedLayout);
  const [isCustomizing, setIsCustomizing] = useState(false);

  // Sync to localStorage
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(layout));
    } catch {
      // Ignore storage errors in private browsing
    }
  }, [layout]);

  const toggleWidget = useCallback((widgetId: string) => {
    setLayout((prev) => {
      const isCurrentlyEnabled = prev.enabledIds.includes(widgetId);
      const nextEnabled = isCurrentlyEnabled
        ? prev.enabledIds.filter((id) => id !== widgetId)
        : [...prev.enabledIds, widgetId];

      // Ensure widget is in order array
      const nextOrder = prev.order.includes(widgetId)
        ? prev.order
        : [...prev.order, widgetId];

      return {
        enabledIds: nextEnabled,
        order: nextOrder,
      };
    });
  }, []);

  const isWidgetEnabled = useCallback(
    (widgetId: string) => layout.enabledIds.includes(widgetId),
    [layout.enabledIds],
  );

  const reorderWidgets = useCallback((newOrder: string[]) => {
    setLayout((prev) => ({
      ...prev,
      order: newOrder,
    }));
  }, []);

  const resetToDefault = useCallback(() => {
    const defaultState: DesktopLayoutState = {
      enabledIds: getDefaultEnabledWidgetIds(),
      order: getDefaultWidgetOrder(),
    };
    setLayout(defaultState);
  }, []);

  const openCustomizer = useCallback(() => setIsCustomizing(true), []);
  const closeCustomizer = useCallback(() => setIsCustomizing(false), []);

  return {
    enabledIds: layout.enabledIds,
    order: layout.order,
    isCustomizing,
    toggleWidget,
    isWidgetEnabled,
    reorderWidgets,
    resetToDefault,
    openCustomizer,
    closeCustomizer,
  };
}
