import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import AttendanceTrendChart from "../attendance-trend-chart";

// Mock the dashboard hook
vi.mock("@/hooks/use-dashboard", () => ({
    useAttendanceTrend: vi.fn(),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

// Mock Recharts to avoid rendering SVG in tests
vi.mock("recharts", () => ({
    LineChart: ({ children, ...props }: { children: React.ReactNode }) => (
        <div data-testid="line-chart" {...props}>{children}</div>
    ),
    Line: (props: { dataKey: string }) => <div data-testid={`line-${props.dataKey}`} />,
    XAxis: () => <div data-testid="x-axis" />,
    YAxis: () => <div data-testid="y-axis" />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
    Tooltip: () => <div data-testid="tooltip" />,
    ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
        <div data-testid="responsive-container">{children}</div>
    ),
    Legend: () => <div data-testid="legend" />,
}));

const renderWithProviders = (component: React.ReactElement) => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
        },
    });
    return render(
        <QueryClientProvider client={queryClient}>
            {component}
        </QueryClientProvider>
    );
};

const rate = (value: string | null) => ({ value, numerator: 1, denominator: 10, unit: "employee_days" });

const mockTrendData = [
    { date: "2026-02-01", present: 40, late: 3, absent: 5, on_leave: 2, eligible: 50, attendance_rate: rate("80"), absence_rate: rate("10"), late_rate: rate("6") },
    { date: "2026-02-02", present: 42, late: 1, absent: 4, on_leave: 3, eligible: 50, attendance_rate: rate("84"), absence_rate: rate("8"), late_rate: rate("2") },
    { date: "2026-02-03", present: 38, late: 5, absent: 6, on_leave: 1, eligible: 50, attendance_rate: rate("76"), absence_rate: rate("12"), late_rate: rate("10") },
];

describe("AttendanceTrendChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
        vi.mocked(dashboardHooks.useAttendanceTrend).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as never);

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.queryByTestId("line-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        vi.mocked(dashboardHooks.useAttendanceTrend).mockReturnValue({
            data: [],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("line-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all eligible are zero", () => {
        vi.mocked(dashboardHooks.useAttendanceTrend).mockReturnValue({
            data: [
                { date: "2026-02-01", present: 0, late: 0, absent: 0, on_leave: 0, eligible: 0, attendance_rate: rate(null), absence_rate: rate(null), late_rate: rate(null) },
            ],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders a single line chart (no overlapping area fill) with data", () => {
        vi.mocked(dashboardHooks.useAttendanceTrend).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("line-chart")).toBeInTheDocument();
        // Single series — the selected metric ("present" by default).
        expect(screen.getByTestId("line-present")).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
        vi.mocked(dashboardHooks.useAttendanceTrend).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByText("dashboard.hr.attendance_trend")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.last_30_days")).toBeInTheDocument();
    });

    it("renders axes and grid alongside the chart", () => {
        vi.mocked(dashboardHooks.useAttendanceTrend).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useAttendanceTrend>);

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByTestId("x-axis")).toBeInTheDocument();
        expect(screen.getByTestId("y-axis")).toBeInTheDocument();
        expect(screen.getByTestId("cartesian-grid")).toBeInTheDocument();
        expect(screen.getByTestId("legend")).toBeInTheDocument();
    });
});
