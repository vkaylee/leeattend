import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import AttendanceTrendChart from "../AttendanceTrendChart";

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
    AreaChart: ({ children, ...props }: any) => (
        <div data-testid="area-chart" {...props}>{children}</div>
    ),
    Area: (props: any) => <div data-testid={`area-${props.dataKey}`} />,
    XAxis: () => <div data-testid="x-axis" />,
    YAxis: () => <div data-testid="y-axis" />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
    Tooltip: () => <div data-testid="tooltip" />,
    ResponsiveContainer: ({ children }: any) => (
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

const mockTrendData = [
    { date: "2026-02-01", present: 40, late: 3, absent: 5, on_leave: 2 },
    { date: "2026-02-02", present: 42, late: 1, absent: 4, on_leave: 3 },
    { date: "2026-02-03", present: 38, late: 5, absent: 6, on_leave: 1 },
];

describe("AttendanceTrendChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: undefined,
            isLoading: true,
        });

        renderWithProviders(<AttendanceTrendChart />);

        // Should show spinner, not chart or empty state
        expect(screen.queryByTestId("area-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: [],
            isLoading: false,
        });

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("area-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all values are zero", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: [
                { date: "2026-02-01", present: 0, late: 0, absent: 0, on_leave: 0 },
            ],
            isLoading: false,
        });

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders chart with data when trend data is available", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        });

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("area-chart")).toBeInTheDocument();
    });

    it("renders all four area series (present, late, absent, on_leave)", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        });

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByTestId("area-present")).toBeInTheDocument();
        expect(screen.getByTestId("area-late")).toBeInTheDocument();
        expect(screen.getByTestId("area-absent")).toBeInTheDocument();
        expect(screen.getByTestId("area-on_leave")).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        });

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByText("dashboard.hr.attendance_trend")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.last_30_days")).toBeInTheDocument();
    });

    it("renders axes and grid alongside the chart", () => {
        (dashboardHooks.useAttendanceTrend as any).mockReturnValue({
            data: mockTrendData,
            isLoading: false,
        });

        renderWithProviders(<AttendanceTrendChart />);

        expect(screen.getByTestId("x-axis")).toBeInTheDocument();
        expect(screen.getByTestId("y-axis")).toBeInTheDocument();
        expect(screen.getByTestId("cartesian-grid")).toBeInTheDocument();
        expect(screen.getByTestId("legend")).toBeInTheDocument();
    });
});
