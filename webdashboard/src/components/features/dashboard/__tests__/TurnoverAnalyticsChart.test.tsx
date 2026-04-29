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
    BarChart: ({ children, ...props }: any) => (
        <div data-testid="bar-chart" {...props}>{children}</div>
    ),
    Bar: (props: any) => <div data-testid="bar" {...props} />,
    XAxis: (props: any) => <div data-testid="x-axis" {...props} />,
    YAxis: (props: any) => <div data-testid="y-axis" {...props} />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
    PieChart: ({ children, ...props }: any) => (
        <div data-testid="pie-chart" {...props}>{children}</div>
    ),
    Pie: ({ children, ...props }: any) => (
        <div data-testid="pie" {...props}>{children}</div>
    ),
    Cell: (props: any) => <div data-testid="cell" {...props} />,
    ResponsiveContainer: ({ children }: any) => (
        <div data-testid="responsive-container">{children}</div>
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
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("displays the correct chart title", () => {
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("dashboard.hr.turnover_title")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_subtitle")).toBeInTheDocument();
    });

    it("displays total terminated badge", () => {
        (dashboardHooks.useTurnoverAnalytics as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText(/dashboard.hr.turnover_total.*16/)).toBeInTheDocument();
    });

    it("shows tab navigation with 3 tabs", () => {
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
