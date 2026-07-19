// ── Dashboard hooks ─────────────────────────────────────────────────────

export { useDashboardStats, useAttendanceTrend, useDepartmentAttendance,
  useUnexcusedAbsences, useOvertimeTrend, usePeakTimes,
  useLeaveTypesBreakdown, useLatenessByDay,
  useRecruitmentSourceBreakdown, useTurnoverAnalytics } from "@/hooks/use-dashboard";
export { useDashboardCharts } from "@/hooks/use-dashboard-charts";
export { getNewVisibleForView, getNewOrderForView, getActiveView,
  getCollapsedSections, getCurrentVisibleWidgets, getCurrentWidgetOrder,
  PRESET_VIEWS, ALL_WIDGETS } from "@/hooks/use-dashboard-layout-presets";
export { useDashboardLayout } from "@/hooks/use-dashboard-layout";
export type { WidgetId, DashboardView } from "@/hooks/use-dashboard-layout";
