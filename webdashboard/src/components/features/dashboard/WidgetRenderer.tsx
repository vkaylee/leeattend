import React, { lazy, Suspense } from "react";
import type { WidgetId } from "@/hooks/use-dashboard-layout";

// ── Lazy-loaded chart components ───────────────────────────────────

const AttendanceTrendChart = lazy(
    () => import("@/components/features/dashboard/AttendanceTrendChart")
);
const DepartmentAttendanceChart = lazy(
    () => import("@/components/features/dashboard/DepartmentAttendanceChart")
);
const OvertimeTrendChart = lazy(
    () => import("@/components/features/dashboard/OvertimeTrendChart")
);
const PeakTimesChart = lazy(
    () => import("@/components/features/dashboard/PeakTimesChart")
);
const LeaveTypesChart = lazy(
    () => import("@/components/features/dashboard/LeaveTypesChart")
);
const LatenessByDayChart = lazy(
    () => import("@/components/features/dashboard/LatenessByDayChart")
);
const RecruitmentSourceChart = lazy(
    () => import("@/components/features/dashboard/RecruitmentSourceChart")
);
const TurnoverAnalyticsChart = lazy(
    () => import("@/components/features/dashboard/TurnoverAnalyticsChart")
);
const UnexcusedAbsencesChart = lazy(
    () => import("@/components/features/dashboard/UnexcusedAbsencesChart")
);

// ── Static chart map (hoisted to module scope — never re-created) ──

const chartMap: Record<string, React.LazyExoticComponent<React.ComponentType>> = {
    attendance_trend: AttendanceTrendChart,
    department_chart: DepartmentAttendanceChart,
    overtime_trend: OvertimeTrendChart,
    peak_times: PeakTimesChart,
    leave_types: LeaveTypesChart,
    lateness_by_day: LatenessByDayChart,
    recruitment_sources: RecruitmentSourceChart,
    turnover_analytics: TurnoverAnalyticsChart,
    unexcused_absences: UnexcusedAbsencesChart,
};

// ── Loading skeleton ───────────────────────────────────────────────

function ChartSkeleton() {
    return (
        <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-10 border border-slate-100 dark:border-slate-800 shadow-sm animate-pulse">
            <div className="h-4 w-40 bg-slate-200 dark:bg-slate-700 rounded-full mb-8" />
            <div className="h-64 bg-slate-100 dark:bg-slate-800 rounded-3xl" />
        </div>
    );
}

// ── Widget Renderer ────────────────────────────────────────────────

interface WidgetRendererProps {
    widgetId: WidgetId;
}

/**
 * Renders a single dashboard widget by ID.
 * "kpi_cards", "donut_chart", "quick_stats", and "promo_cards" are rendered
 * directly in DashboardPage because they require data from the page scope.
 * This renderer handles chart-type widgets with lazy loading.
 */
const WidgetRenderer = React.memo(function WidgetRenderer({ widgetId }: WidgetRendererProps) {
    const ChartComponent = chartMap[widgetId];

    if (!ChartComponent) {
        // Non-chart widgets (kpi_cards, donut, quick_stats, promo_cards) 
        // are rendered directly in DashboardPage
        return null;
    }

    return (
        <Suspense fallback={<ChartSkeleton />}>
            <ChartComponent />
        </Suspense>
    );
});

export default WidgetRenderer;

