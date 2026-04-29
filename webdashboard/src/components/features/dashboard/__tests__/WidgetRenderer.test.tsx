import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import WidgetRenderer from "../WidgetRenderer";

import type { WidgetId } from "@/hooks/use-dashboard-layout";

// Mock all lazy-loaded chart components
vi.mock("@/components/features/dashboard/AttendanceTrendChart", () => ({
    default: () => <div data-testid="chart-attendance_trend" />,
}));
vi.mock("@/components/features/dashboard/DepartmentAttendanceChart", () => ({
    default: () => <div data-testid="chart-department_chart" />,
}));
vi.mock("@/components/features/dashboard/OvertimeTrendChart", () => ({
    default: () => <div data-testid="chart-overtime_trend" />,
}));
vi.mock("@/components/features/dashboard/PeakTimesChart", () => ({
    default: () => <div data-testid="chart-peak_times" />,
}));
vi.mock("@/components/features/dashboard/LeaveTypesChart", () => ({
    default: () => <div data-testid="chart-leave_types" />,
}));
vi.mock("@/components/features/dashboard/LatenessByDayChart", () => ({
    default: () => <div data-testid="chart-lateness_by_day" />,
}));
vi.mock("@/components/features/dashboard/RecruitmentSourceChart", () => ({
    default: () => <div data-testid="chart-recruitment_sources" />,
}));
vi.mock("@/components/features/dashboard/TurnoverAnalyticsChart", () => ({
    default: () => <div data-testid="chart-turnover_analytics" />,
}));
vi.mock("@/components/features/dashboard/UnexcusedAbsencesChart", () => ({
    default: () => <div data-testid="chart-unexcused_absences" />,
}));

describe("WidgetRenderer", () => {
    const chartWidgets: WidgetId[] = [
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

    it.each(chartWidgets)(
        "renders %s chart component",
        async (widgetId) => {
            render(<WidgetRenderer widgetId={widgetId} />);
            expect(
                await screen.findByTestId(`chart-${widgetId}`)
            ).toBeInTheDocument();
        }
    );

    it("returns null for non-chart widgets (kpi_cards, donut_chart, quick_stats, promo_cards)", () => {
        const nonChartWidgets: WidgetId[] = [
            "kpi_cards",
            "donut_chart",
            "quick_stats",
            "promo_cards",
        ];

        for (const id of nonChartWidgets) {
            const { container } = render(<WidgetRenderer widgetId={id} />);
            expect(container.innerHTML).toBe("");
        }
    });
});
