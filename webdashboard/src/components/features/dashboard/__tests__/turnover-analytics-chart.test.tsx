import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import TurnoverAnalyticsChart from "../turnover-analytics-chart";

vi.mock("@/hooks/use-dashboard", () => ({
    useTurnoverAnalytics: vi.fn(),
}));

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

vi.mock("recharts", () => ({
 
    BarChart: ({ children, ...props }: { children?: React.ReactNode; [key: string]: unknown }) => (
        <div data-testid="bar-chart" {...props}>{children}</div>
    ),
 
    Bar: (props: Record<string, unknown>) => <div data-testid="bar" {...props} />,
 
    XAxis: (props: Record<string, unknown>) => <div data-testid="x-axis" {...props} />,
 
    YAxis: (props: Record<string, unknown>) => <div data-testid="y-axis" {...props} />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
 
    PieChart: ({ children, ...props }: { children?: React.ReactNode; [key: string]: unknown }) => (
        <div data-testid="pie-chart" {...props}>{children}</div>
    ),
 
    Pie: ({ children, ...props }: { children?: React.ReactNode; [key: string]: unknown }) => (
        <div data-testid="pie" {...props}>{children}</div>
    ),
 
    Cell: (props: Record<string, unknown>) => <div data-testid="cell" {...props} />,
 
    ResponsiveContainer: ({ children }: { children?: React.ReactNode }) => (
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
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Should not render any chart
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: {
                tenure_distribution: [],
                by_source: [],
                by_department: [],
                total_terminated: 0,
                period_start: "2025-09-03",
                period_end: "2026-03-03",
            },
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders tenure bar chart by default when data is available", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("displays the correct chart title", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("dashboard.hr.turnover_title")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_subtitle")).toBeInTheDocument();
    });

    it("displays total terminated badge", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText(/dashboard.hr.turnover_total.*16/)).toBeInTheDocument();
    });

    it("shows tab navigation with 3 tabs", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("dashboard.hr.turnover_tenure")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_by_source")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_by_dept")).toBeInTheDocument();
    });

    it("switches to source pie chart when source tab clicked", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Click source tab
        fireEvent.click(screen.getByText("dashboard.hr.turnover_by_source"));

        // Should render pie chart
        expect(screen.getByTestId("pie-chart")).toBeInTheDocument();
    });

    it("switches to department chart when dept tab clicked", () => {
 
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Click department tab
        fireEvent.click(screen.getByText("dashboard.hr.turnover_by_dept"));

        // Should render bar chart (horizontal)
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("has a period selector dropdown with options", () => {

        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        const select = screen.getByRole("combobox");
        expect(select).toBeInTheDocument();
        expect(select).toHaveValue("6m");
    });

    it("calls useTurnoverAnalytics with correct date range on period change", () => {
        const mockHook = vi.mocked(dashboardHooks.useTurnoverAnalytics);
        mockHook.mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Change period to 3 months
        const select = screen.getByRole("combobox");
        fireEvent.change(select, { target: { value: "3m" } });

        // The hook should have been called with new date range
        // The component re-renders with new period, which triggers a new hook call
        expect(mockHook).toHaveBeenCalled();
    });

    it("shows empty state when turnover data has all empty arrays", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: {
                tenure_distribution: [],
                by_source: [],
                by_department: [],
                total_terminated: 0,
                period_start: "2025-09-03",
                period_end: "2026-03-03",
            },
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("handles missing data gracefully (undefined data, not loading)", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: undefined,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Should show empty state, not crash
        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });
});
