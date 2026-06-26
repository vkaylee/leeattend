import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { ResilientWidget } from "@/components/common/resilient-widget";
import ChartCatalogSidebar from "@/components/features/dashboard/chart-catalog-sidebar";
import CustomizePanel from "@/components/features/dashboard/customize-panel";
import { DashboardErrorBanner } from "@/components/features/dashboard/dashboard-error-banner";
import { DashboardHeader } from "@/components/features/dashboard/dashboard-header";
import { DashboardHrTab } from "@/components/features/dashboard/dashboard-hr-tab";
import { DashboardSystemTab } from "@/components/features/dashboard/dashboard-system-tab";
import PendingTasksWidget from "@/components/features/dashboard/pending-tasks-widget";
import { useAgentsStatus } from "@/hooks/use-agents";
import { useCurrency } from "@/hooks/use-currency";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { useDashboardCharts } from "@/hooks/use-dashboard-charts";
import { useDashboardLayout } from "@/hooks/use-dashboard-layout";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDevices } from "@/hooks/use-devices";

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
  const { agents, isLoading: agentsLoading, error: agentsError } = useAgentsStatus();
  const { devices, isLoading: devicesLoading, error: devicesError } = useDevices();
  const { data: hrStats, isLoading: hrLoading, error: hrError } = useDashboardStats();

  const {
    activeView, visibleWidgets, widgetOrder, collapsedSections, isSaving,
    setActiveView, toggleWidget, setWidgetOrder, toggleSection, resetToDefault, allWidgets,
  } = useDashboardLayout();

  const isLoading = agentsLoading || devicesLoading || hrLoading;
  const error = (agentsError ?? devicesError) ?? hrError;

  const onlineAgents = agents.filter((a) => a.is_online).length;
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
  const chartElements = useDashboardCharts(widgetOrder, visibleWidgets);

  return (
    <div className="space-y-6 animate-in fade-in duration-700">
      <DashboardHeader
        onReset={resetToDefault}
        setShowSystemTab={setShowSystemTab}
        showSystemTab={showSystemTab}
        t={t}
      />

      <ResilientWidget>
        <PendingTasksWidget />
      </ResilientWidget>

      {!showSystemTab ? (
        <div className="flex gap-6 items-start animate-in slide-in-from-bottom-4 duration-700">
          {/* LEFT: Chart Catalog Sidebar */}
          <ChartCatalogSidebar
            allWidgets={allWidgets}
            visibleWidgets={visibleWidgets}
            onToggleWidget={toggleWidget}
          />

          {/* RIGHT: Dashboard content */}
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
              activeDevices={activeDevices} agents={agents}
              devices={devices} formatTime={formatTime}
              isLoading={isLoading} onlineAgents={onlineAgents} t={t}
            />
          </ResilientWidget>
        </div>
      )}

      {error ? <DashboardErrorBanner /> : null}

      <CustomizePanel
        allWidgets={allWidgets} isOpen={customizeOpen} isSaving={isSaving}
        onClose={() => { setCustomizeOpen(false); }} onReorder={setWidgetOrder}
        onReset={resetToDefault} onToggleWidget={toggleWidget}
        visibleWidgets={visibleWidgets} widgetOrder={widgetOrder}
      />
    </div>
  );
}
