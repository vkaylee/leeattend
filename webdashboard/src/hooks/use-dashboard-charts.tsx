import { useCallback, useMemo, useState } from "react";

import WidgetRenderer from "@/components/features/dashboard/widget-renderer";
import { type WidgetId } from "@/hooks/use-dashboard-layout";

const CHART_WIDGET_IDS: WidgetId[] = [
  "attendance_trend",
  "department_chart",
  "overtime_trend",
  "peak_times",
  "leave_types",
  "lateness_by_day",
  "recruitment_sources",
  "turnover_analytics",
  "unexcused_absences",
];

export function useDashboardCharts(widgetOrder: WidgetId[], visibleWidgets: Record<string, boolean>) {
  const [chartErrors, setChartErrors] = useState<Set<WidgetId>>(new Set());

  const reportChartError = useCallback((widgetId: WidgetId, hasError: boolean) => {
    setChartErrors((current) => {
      const next = new Set(current);
      if (hasError) next.add(widgetId);
      else next.delete(widgetId);
      return next.size === current.size && [...next].every((id) => current.has(id)) ? current : next;
    });
  }, []);

  const visibleCharts = useMemo(() =>
    widgetOrder.filter((id) => CHART_WIDGET_IDS.includes(id) && visibleWidgets[id]),
    [widgetOrder, visibleWidgets]
  );

  const pairedCharts = useMemo(() => {
    const set = new Set<WidgetId>();
    if (visibleWidgets.leave_types && visibleWidgets.lateness_by_day) {
      set.add("leave_types");
      set.add("lateness_by_day");
    }
    return set;
  }, [visibleWidgets.leave_types, visibleWidgets.lateness_by_day]);

  const chartErrorHandlers = useMemo(() => {
    const handlers = new Map<WidgetId, (hasError: boolean) => void>();
    CHART_WIDGET_IDS.forEach((id) => {
      handlers.set(id, (hasError) => { reportChartError(id, hasError); });
    });
    return handlers;
  }, [reportChartError]);

  const chartElements = useMemo(() => {
    const rendered = new Set<WidgetId>();
    const elements: React.ReactNode[] = [];

    visibleCharts.forEach((id) => {
      if (rendered.has(id)) return;

      if (pairedCharts.has(id)) {
        const otherId = id === "leave_types" ? "lateness_by_day" : "leave_types";
        if (visibleCharts.includes(otherId) && !rendered.has(otherId)) {
          rendered.add(id);
          rendered.add(otherId);
          elements.push(
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8" key={`pair-${id}`}>
              <WidgetRenderer onErrorChange={chartErrorHandlers.get("leave_types")} widgetId="leave_types" />
              <WidgetRenderer onErrorChange={chartErrorHandlers.get("lateness_by_day")} widgetId="lateness_by_day" />
            </div>
          );
          return;
        }
      }

      rendered.add(id);
      elements.push(<WidgetRenderer key={id} onErrorChange={chartErrorHandlers.get(id)} widgetId={id} />);
    });

    return elements;
  }, [chartErrorHandlers, pairedCharts, visibleCharts]);

  return {
    chartElements,
    chartErrorCount: chartErrors.size,
  };
}
