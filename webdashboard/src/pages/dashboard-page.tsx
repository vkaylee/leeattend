import { Cpu, RotateCcw, Users } from "lucide-react";
import { useState, useMemo, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";

import { ResilientWidget } from "@/components/common/resilient-widget";
import ChartCatalogSidebar from "@/components/features/dashboard/chart-catalog-sidebar";
import CustomizePanel from "@/components/features/dashboard/customize-panel";
import { DashboardErrorBanner } from "@/components/features/dashboard/dashboard-error-banner";
import { DashboardHrTab } from "@/components/features/dashboard/dashboard-hr-tab";
import { DashboardSystemTab } from "@/components/features/dashboard/dashboard-system-tab";
import PendingTasksWidget from "@/components/features/dashboard/pending-tasks-widget";
import { DashboardFilterBar, useDashboardFilterState } from "@/components/features/dashboard/dashboard-filter-bar";
import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ExecutiveCostPanel } from "@/components/features/dashboard/executive-cost-panel";
import { ExecutiveKpiGrid } from "@/components/features/dashboard/executive-kpi-grid";
import { ExecutiveOvertimeBreakdown } from "@/components/features/dashboard/executive-overtime-breakdown";
import { useCurrency } from "@/hooks/use-currency";
import { useDashboardMetrics, useDashboardStats } from "@/hooks/use-dashboard";
import { useDashboardCharts } from "@/hooks/use-dashboard-charts";
import { useDashboardLayout } from "@/hooks/use-dashboard-layout";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDevices } from "@/hooks/use-devices";
import { useHeaderActions } from "@/hooks/use-header-actions";
import { cn } from "@/lib/utils";

const DONUT_COLORS = {
  on_time: "#10b981",
  late: "#f59e0b",
  absent: "#ef4444",
  on_leave: "#3b82f6",
};

export default function DashboardPage() {
  const { t } = useTranslation();
  const [showSystemTab, setShowSystemTab] = useState(false);
  const [customizeOpen, setCustomizeOpen] = useState(false);
  const { formatTime } = useDateTimeFormatter();
  const { formatCurrency } = useCurrency();
  const { devices, isLoading: devicesLoading, error: devicesError } = useDevices();
  const { data: hrStats, isLoading: hrLoading, error: hrError } = useDashboardStats();
  const [dashboardFilters, setDashboardFilters] = useDashboardFilterState();
  const {
    data: executiveMetrics,
    isLoading: executiveLoading,
    error: executiveError,
    refetch: refetchExecutiveMetrics,
  } = useDashboardMetrics(dashboardFilters);
  const { setActions } = useHeaderActions();

  const {
    visibleWidgets, widgetOrder, collapsedSections, isSaving, saveError,
    toggleWidget, setWidgetOrder, toggleSection, resetToDefault, allWidgets,
  } = useDashboardLayout();

  // ── Stable refs for callbacks (prevents effect re-fire from non-memoized deps) ─
  const tRef = useRef(t);
  const resetRef = useRef(resetToDefault);
  const setActionsRef = useRef(setActions);
  useEffect(() => { tRef.current = t; }, [t]);
  useEffect(() => { resetRef.current = resetToDefault; }, [resetToDefault]);
  useEffect(() => { setActionsRef.current = setActions; }, [setActions]);

  // ── Inject header actions ─────────────────────────────────────
  useEffect(() => {
    const updateActions = () => {
      setActionsRef.current(
        <div className="flex min-w-0 max-w-full flex-wrap items-center gap-2">
          <Button
            aria-label={tRef.current("dashboard.customize.reset")}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:border-primary-200 dark:hover:border-primary-700 transition-all shadow-sm h-auto"
            onClick={resetRef.current}
            type="button"
            variant="outline"
          >
            <RotateCcw size={13} />
            <span className="hidden sm:inline">{tRef.current("dashboard.customize.reset")}</span>
          </Button>
          <div className="flex shrink-0 bg-slate-100 dark:bg-slate-700/50 p-0.5 rounded-full border border-slate-200 dark:border-slate-700/50 shadow-inner">
            <Button
              className={cn(
                "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent",
                !showSystemTab
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border-slate-100 dark:border-slate-800"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200",
              )}
              onClick={() => { setShowSystemTab(false); }}
              type="button"
              variant="ghost"
            >
              <Users size={13} /> {tRef.current("dashboard.tabs.hr")}
            </Button>
            <Button
              className={cn(
                "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent",
                showSystemTab
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border-slate-100 dark:border-slate-800"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200",
              )}
              onClick={() => { setShowSystemTab(true); }}
              type="button"
              variant="ghost"
            >
              <Cpu size={13} /> {tRef.current("dashboard.tabs.system")}
            </Button>
          </div>
          <Button
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 transition-all shadow-sm h-auto"
            onClick={() => { setCustomizeOpen(true); }}
            type="button"
            variant="outline"
          >
            {tRef.current("dashboard.customize.btn")}
          </Button>
        </div>,
      );
    };

    updateActions();
    return () => { setActionsRef.current(null); };
  }, [showSystemTab]);

  const isLoading = devicesLoading || hrLoading;
  const error = devicesError ?? hrError;
  const executiveDataGap = executiveMetrics?.data_quality.status === "data_gap";
  const executiveContext = executiveMetrics?.context;
  const executiveMetricCount = executiveMetrics?.metrics.length ?? 0;

  // The breakdown endpoint resolves dates itself from the workspace timezone; it
  // needs the *resolved* reporting window (not the raw filters) so the department
  // table matches the KPI grid exactly. Non-custom periods like `this_month` only
  // carry dates after the backend resolves them into context.period.
  const executiveRange = useMemo(() => {
    const period = executiveContext?.period;
    return period
      ? { startDate: period.start_date, endDate: period.end_date, timezone: period.timezone }
      : {};
  }, [executiveContext]);

  // The grid's overtime_cost metric status tells the drilldown table whether a
  // missing cost value means "no permission" (restricted) or "no paid payroll"
  // (data_gap) — the breakdown response itself cannot distinguish these.
  const overtimeCostStatus = useMemo(() => {
    const metric = (executiveMetrics?.metrics ?? []).find((m) => m.key === "overtime_cost");
    if (!metric || metric.status === "available" || metric.status === "not_applicable") return "available" as const;
    return metric.status === "restricted" ? ("restricted" as const) : ("data_gap" as const);
  }, [executiveMetrics]);

  const activeDevices = devices.filter((d) => d.is_enabled).length;

  const donutData = useMemo(() => {
    const { present_today = 0, late_today = 0, absent_today = 0, on_leave_today = 0 } = hrStats ?? {};
    const onTimeCount = Math.max(0, present_today - late_today);
    return [
      { name: t("dashboard.hr.on_time"), value: onTimeCount, fill: DONUT_COLORS.on_time },
      { name: t("dashboard.hr.late_today"), value: late_today, fill: DONUT_COLORS.late },
      { name: t("dashboard.hr.absent_today"), value: absent_today, fill: DONUT_COLORS.absent },
      { name: t("dashboard.hr.on_leave_today"), value: on_leave_today, fill: DONUT_COLORS.on_leave },
    ];
  }, [hrStats, t]);

  const hasDonutData = useMemo(() => donutData.some((d) => d.value > 0), [donutData]);
  const { chartElements, chartErrorCount } = useDashboardCharts(widgetOrder, visibleWidgets);

  return (
    <div className="space-y-6 animate-in fade-in duration-700">
      <ResilientWidget>
        <PendingTasksWidget />
      </ResilientWidget>

      <DashboardFilterBar filters={dashboardFilters} onChange={setDashboardFilters} />

      <section aria-labelledby="executive-dashboard-heading" className="space-y-4 rounded-3xl border border-slate-200 bg-white/70 p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900/70 sm:p-6">
        <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <h1 className="text-xl font-semibold tracking-tight text-slate-900 dark:text-slate-100" id="executive-dashboard-heading">
              {t("dashboard.executive.title")}
            </h1>
            <p className="text-sm text-slate-500 dark:text-slate-400">
              {executiveContext
                ? t("dashboard.executive.context", { period: t(`dashboard.filters.period.${executiveContext.period.key}`), timezone: executiveContext.period.timezone })
                : t("dashboard.executive.context_pending")}
            </p>
          </div>
          <span className="text-xs font-medium text-slate-500 dark:text-slate-400" aria-live="polite">
            {executiveLoading ? t("dashboard.executive.loading") : executiveError ? t("dashboard.executive.error") : t("dashboard.executive.metric_count", { count: executiveMetricCount })}
          </span>
        </div>
        <ExecutiveKpiGrid
          data={executiveMetrics}
          error={Boolean(executiveError)}
          loading={executiveLoading}
          onRetry={() => void refetchExecutiveMetrics()}
        />
        {!executiveLoading && !executiveError ? (
          <>
            <ExecutiveCostPanel
              currency={executiveMetrics?.workspace.currency ?? "USD"}
              data={executiveMetrics}
            />
            <ExecutiveOvertimeBreakdown
              costStatus={overtimeCostStatus}
              range={executiveRange}
              topN={dashboardFilters.topN}
            />
          </>
        ) : null}
      </section>

      {!showSystemTab ? (
        <div className="flex gap-6 items-start animate-in slide-in-from-bottom-4 duration-700">
          <ChartCatalogSidebar
            allWidgets={allWidgets}
            onToggleWidget={toggleWidget}
            visibleWidgets={visibleWidgets}
          />
          <div className="flex-1 min-w-0">
            <ResilientWidget>
              <DashboardHrTab
                chartElements={chartElements}
                collapsedSections={collapsedSections}
                donutData={donutData}
                formatCurrency={formatCurrency}
                hasDonutData={hasDonutData}
                hrStats={hrStats}
                isLoading={isLoading}
                t={t}
                toggleSection={toggleSection}
                visibleWidgets={visibleWidgets}
              />
            </ResilientWidget>
          </div>
        </div>
      ) : (
        <div className="animate-in slide-in-from-bottom-4 duration-700">
          <ResilientWidget>
            <DashboardSystemTab
              activeDevices={activeDevices}
              devices={devices}
              isLoading={isLoading}
              t={t}
            />
          </ResilientWidget>
        </div>
      )}

      {error || saveError || chartErrorCount > 0 ? <DashboardErrorBanner /> : null}

      <CustomizePanel
        allWidgets={allWidgets} isOpen={customizeOpen} isSaving={isSaving}
        onClose={() => { setCustomizeOpen(false); }} onReorder={setWidgetOrder}
        onReset={resetToDefault} onToggleWidget={toggleWidget}
        visibleWidgets={visibleWidgets} widgetOrder={widgetOrder}
      />
    </div>
  );
}
