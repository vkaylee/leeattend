import { useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";

import {
  DashboardPreferencesError,
  useDashboardMutation,
  useDashboardPreferences,
  type DashboardPreferencesUpdate,
} from "./dashboard-layout-api";
import {
  ALL_WIDGETS,
  getActiveView,
  getCollapsedSections,
  getCurrentVisibleWidgets,
  getCurrentWidgetOrder,
} from "./use-dashboard-layout-presets";
import {
  useDashboardLayoutActions,
  type LocalOverrides,
} from "./use-dashboard-layout-actions";

export type { DashboardView, WidgetId } from "./use-dashboard-layout-presets";
export { ALL_WIDGETS } from "./use-dashboard-layout-presets";
export { DashboardPreferencesError } from "./dashboard-layout-api";

interface PendingDashboardWrite {
  rollbackLocal: LocalOverrides;
  sequence: number;
  update: DashboardPreferencesUpdate;
}

const EMPTY_LOCAL_OVERRIDES: LocalOverrides = {
  activeView: null,
  collapsedSections: null,
  visibleWidgets: null,
  widgetOrder: null,
};

export function useDashboardLayout() {
  const { data: preferences, isLoading, error } = useDashboardPreferences();
  const mutation = useDashboardMutation();
  const queryClient = useQueryClient();
  const [local, setLocal] = useState<LocalOverrides>(EMPTY_LOCAL_OVERRIDES);
  const localRef = useRef<LocalOverrides>(EMPTY_LOCAL_OVERRIDES);
  const mutationSequence = useRef(0);
  const activeWriteRef = useRef<PendingDashboardWrite | null>(null);
  const queuedWriteRef = useRef<PendingDashboardWrite | null>(null);
  const [saveError, setSaveError] = useState<DashboardPreferencesError | null>(null);

  const activeView = getActiveView(local.activeView, preferences);
  const visibleWidgets = getCurrentVisibleWidgets(local.visibleWidgets, activeView, preferences);
  const widgetOrder = getCurrentWidgetOrder(local.widgetOrder, activeView, preferences);
  const collapsedSections = getCollapsedSections(local.collapsedSections, preferences);

  const getEffectiveState = () => ({
    activeView: getActiveView(localRef.current.activeView, preferences),
    visibleWidgets: getCurrentVisibleWidgets(
      localRef.current.visibleWidgets,
      getActiveView(localRef.current.activeView, preferences),
      preferences,
    ),
    widgetOrder: getCurrentWidgetOrder(
      localRef.current.widgetOrder,
      getActiveView(localRef.current.activeView, preferences),
      preferences,
    ),
    collapsedSections: getCollapsedSections(localRef.current.collapsedSections, preferences),
  });

  const startWrite = (write: PendingDashboardWrite): void => {
    activeWriteRef.current = write;
    void mutation.mutateAsync(write.update).then(
      (data) => handleWriteSuccess(write, data),
      (error: unknown) => handleWriteFailure(write, error),
    );
  };

  const handleWriteSuccess = (write: PendingDashboardWrite, data: unknown): void => {
    if (activeWriteRef.current !== write) return;
    queryClient.setQueryData(["dashboard-preferences"], data);
    const queued = queuedWriteRef.current;
    if (queued) {
      queuedWriteRef.current = null;
      startWrite({ ...queued, rollbackLocal: EMPTY_LOCAL_OVERRIDES });
      return;
    }
    activeWriteRef.current = null;
    if (write.sequence !== mutationSequence.current) return;
    localRef.current = EMPTY_LOCAL_OVERRIDES;
    setLocal(EMPTY_LOCAL_OVERRIDES);
    setSaveError(null);
  };

  const handleWriteFailure = (write: PendingDashboardWrite, error: unknown): void => {
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
  };

  const persist = (nextLocal: LocalOverrides, update: DashboardPreferencesUpdate): void => {
    const previousLocal = localRef.current;
    const sequence = mutationSequence.current + 1;
    mutationSequence.current = sequence;
    setSaveError(null);
    localRef.current = nextLocal;
    setLocal(nextLocal);
    const write: PendingDashboardWrite = {
      sequence,
      update,
      rollbackLocal: activeWriteRef.current?.rollbackLocal ?? previousLocal,
    };
    if (activeWriteRef.current) queuedWriteRef.current = write;
    else startWrite(write);
  };

  const actions = useDashboardLayoutActions(preferences, localRef, persist, getEffectiveState);

  return {
    activeView,
    visibleWidgets,
    widgetOrder,
    collapsedSections,
    isLoading,
    error,
    saveError,
    isSaving: mutation.isPending,
    ...actions,
    allWidgets: ALL_WIDGETS,
  };
}
