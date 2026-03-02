import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import OvertimeTrendChart from "../OvertimeTrendChart";
import * as dashboardHooks from "@/hooks/use-dashboard";

// Mock the dashboard hook
vi.mock("@/hooks/use-dashboard", () => ({
    useOvertimeTrend: vi.fn(),
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

const mockOtData = [
    { date: "2026-02-01", system_ot_hours: 5.5, approved_ot_hours: 3.0 },
    { date: "2026-02-02", system_ot_hours: 8.2, approved_ot_hours: 6.0 },
    { date: "2026-02-03", system_ot_hours: 3.0, approved_ot_hours: 2.5 },
];

describe("OvertimeTrendChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: undefined,
            isLoading: true,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.queryByTestId("area-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: [],
            isLoading: false,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("area-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all values are zero", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: [{ date: "2026-02-01", system_ot_hours: 0, approved_ot_hours: 0 }],
            isLoading: false,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders chart with data when OT data is available", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: mockOtData,
            isLoading: false,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("area-chart")).toBeInTheDocument();
    });

    it("renders both area series (system_ot_hours, approved_ot_hours)", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: mockOtData,
            isLoading: false,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.getByTestId("area-system_ot_hours")).toBeInTheDocument();
        expect(screen.getByTestId("area-approved_ot_hours")).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: mockOtData,
            isLoading: false,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.getByText("dashboard.hr.overtime_trend")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.overtime_trend_subtitle")).toBeInTheDocument();
    });

    it("renders axes and grid alongside the chart", () => {
        (dashboardHooks.useOvertimeTrend as any).mockReturnValue({
            data: mockOtData,
            isLoading: false,
        });

        renderWithProviders(<OvertimeTrendChart />);

        expect(screen.getByTestId("x-axis")).toBeInTheDocument();
        expect(screen.getByTestId("y-axis")).toBeInTheDocument();
        expect(screen.getByTestId("cartesian-grid")).toBeInTheDocument();
        expect(screen.getByTestId("legend")).toBeInTheDocument();
    });
});
