import { type TFunction } from "i18next";

import { AttendanceBreakdown } from "@/components/features/dashboard/attendance-breakdown";
import CollapsibleSection from "@/components/features/dashboard/collapsible-section";
import { HrKpiCards } from "@/components/features/dashboard/hr-kpi-cards";
import { HrPromoCards } from "@/components/features/dashboard/hr-promo-cards";
import { HrQuickStats } from "@/components/features/dashboard/hr-quick-stats";
import { type WorkspaceStats } from "@/hooks/use-dashboard";

export interface DonutData {
  fill: string;
  name: string;
  value: number;
}

interface DashboardHrTabProps {
  chartElements: React.ReactNode[];
  collapsedSections: Record<string, boolean>;
  donutData: DonutData[];
  formatCurrency: (value: number) => string;
  hasDonutData: boolean;
  hrStats: WorkspaceStats | undefined;
  isLoading: boolean;
  t: TFunction;
  toggleSection: (id: string) => void;
  visibleWidgets: Record<string, boolean>;
}

export function DashboardHrTab({
  hrStats,
  isLoading,
  visibleWidgets,
  collapsedSections,
  toggleSection,
  donutData,
  hasDonutData,
  formatCurrency,
  chartElements,
  t
}: DashboardHrTabProps) {
  return (
    <div className="space-y-8">
      {visibleWidgets.kpi_cards ? (
        <HrKpiCards
          hrStats={hrStats}
          isCollapsed={collapsedSections.summary}
          isLoading={isLoading}
          onToggle={toggleSection}
          t={t}
        />
      ) : null}

      {visibleWidgets.donut_chart || visibleWidgets.quick_stats ? (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {visibleWidgets.donut_chart ? (
            <AttendanceBreakdown
              donutData={donutData}
              hasDonutData={hasDonutData}
              t={t}
            />
          ) : null}

          {visibleWidgets.quick_stats ? (
            <HrQuickStats
              formatCurrency={formatCurrency}
              hrStats={hrStats}
              isLoading={isLoading}
              t={t}
            />
          ) : null}
        </div>
      ) : null}

      {chartElements.length > 0 ? (
        <CollapsibleSection
          id="charts"
          isCollapsed={collapsedSections.charts}
          onToggle={toggleSection}
          title={t("dashboard.sections.charts")}
        >
          <div className="space-y-8">
            {chartElements}
          </div>
        </CollapsibleSection>
      ) : null}

      {visibleWidgets.promo_cards ? (
        <HrPromoCards
          isCollapsed={collapsedSections.actions}
          onToggle={toggleSection}
          t={t}
        />
      ) : null}
    </div>
  );
}
