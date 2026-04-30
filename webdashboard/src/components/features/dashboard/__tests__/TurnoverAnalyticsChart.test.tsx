import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import TurnoverAnalyticsChart from "../TurnoverAnalyticsChart";

vi.mock("@/hooks/use-dashboard", () => ({
    useTurnoverAnalytics: vi.fn(),
}));

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

vi.mock("recharts", () => ({
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    BarChart: ({ children, ...props }: any) => (
        <div data-testid="bar-chart" {...props}>{children as React.ReactNode}</div>
    ),
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    Bar: (props: any) => <div data-testid="bar" {...props} />,
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    XAxis: (props: any) => <div data-testid="x-axis" {...props} />,
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    YAxis: (props: any) => <div data-testid="y-axis" {...props} />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    PieChart: ({ children, ...props }: any) => (
        <div data-testid="pie-chart" {...props}>{children as React.ReactNode}</div>
    ),
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    Pie: ({ children, ...props }: any) => (
        <div data-testid="pie" {...props}>{children as React.ReactNode}</div>
    ),
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    Cell: (props: any) => <div data-testid="cell" {...props} />,
// eslint-disable-next-line @typescript-eslint/no-explicit-any
    ResponsiveContainer: ({ children }: any) => (
        <div data-testid="responsive-container">{children as React.ReactNode}</div>
    ),
    Tooltip: () => <div data-testid="tooltip" />,
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

const mockData = {
    tenure_distribution: [
        { bucket: "< 3 months", count: 5 },
        { bucket: "3-6 months", count: 3 },
        { bucket: "6-12 months", count: 8 },
    ],
    by_source: [
        { source_type: "direct", source_name: "Direct Hire", count: 10 },
        { source_type: "referral", source_name: "Referral", count: 6 },
    ],
    by_department: [
        { department_name: "Engineering", count: 7 },
        { department_name: "Sales", count: 5 },
        { department_name: "Marketing", count: 4 },
    ],
    total_terminated: 16,
    period_start: "2025-09-03",
    period_end: "2026-03-03",
};

describe("TurnoverAnalyticsChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading skeleton when data is loading", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: undefined,
            isLoading: true,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Should not render any chart
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: {
                tenure_distribution: [],
                by_source: [],
                by_department: [],
                total_terminated: 0,
                period_start: "2025-09-03",
                period_end: "2026-03-03",
            },
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders tenure bar chart by default when data is available", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("displays the correct chart title", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("dashboard.hr.turnover_title")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_subtitle")).toBeInTheDocument();
    });

    it("displays total terminated badge", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText(/dashboard.hr.turnover_total.*16/)).toBeInTheDocument();
    });

    it("shows tab navigation with 3 tabs", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("dashboard.hr.turnover_tenure")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_by_source")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_by_dept")).toBeInTheDocument();
    });

    it("switches to source pie chart when source tab clicked", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Click source tab
        fireEvent.click(screen.getByText("dashboard.hr.turnover_by_source"));

        // Should render pie chart
        expect(screen.getByTestId("pie-chart")).toBeInTheDocument();
    });

    it("switches to department chart when dept tab clicked", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Click department tab
        fireEvent.click(screen.getByText("dashboard.hr.turnover_by_dept"));

        // Should render bar chart (horizontal)
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("has a period selector dropdown with options", () => {
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        const select = screen.getByRole("combobox");
        expect(select).toBeInTheDocument();
        expect(select).toHaveValue("6m");
    });
});
