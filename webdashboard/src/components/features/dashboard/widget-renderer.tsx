import React, { lazy, Suspense, memo } from "react";

import type { WidgetId } from "@/hooks/use-dashboard-layout";

// ── Lazy-loaded chart components ───────────────────────────────────

const AttendanceTrendChart = lazy(
    () => import("@/components/features/dashboard/attendance-trend-chart")
);
const DepartmentAttendanceChart = lazy(
    () => import("@/components/features/dashboard/department-attendance-chart")
);
const OvertimeTrendChart = lazy(
    () => import("@/components/features/dashboard/overtime-trend-chart")
);
const PeakTimesChart = lazy(
    () => import("@/components/features/dashboard/peak-times-chart")
);
const LeaveTypesChart = lazy(
    () => import("@/components/features/dashboard/leave-types-chart")
);
const LatenessByDayChart = lazy(
    () => import("@/components/features/dashboard/lateness-by-day-chart")
);
const RecruitmentSourceChart = lazy(
    () => import("@/components/features/dashboard/recruitment-source-chart")
);
const TurnoverAnalyticsChart = lazy(
    () => import("@/components/features/dashboard/turnover-analytics-chart")
);
const UnexcusedAbsencesChart = lazy(
    () => import("@/components/features/dashboard/unexcused-absences-chart")
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
 
const WidgetRenderer = memo(function WidgetRenderer({ widgetId }: WidgetRendererProps) {
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

