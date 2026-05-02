import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import UnexcusedAbsencesChart from "../UnexcusedAbsencesChart";

// Mock the dashboard hook
vi.mock("@/hooks/use-dashboard", () => ({
    useUnexcusedAbsences: vi.fn(),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

// Mock Recharts to avoid rendering SVG in tests
vi.mock("recharts", () => ({
 
    BarChart: ({ children, ...props }: any) => (
        <div data-layout={props.layout} data-testid="bar-chart" {...props}>{children as React.ReactNode}</div>
    ),
 
    Bar: (props: any) => <div data-testid={`bar-${props.dataKey}`} />,
 
    XAxis: (props: any) => <div data-testid="x-axis" data-type={props.type} />,
 
    YAxis: (props: any) => <div data-testid="y-axis" data-type={props.type} />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
 
    Tooltip: (_props: any) => <div data-testid="tooltip" />,
 
    ResponsiveContainer: ({ children }: any) => (
        <div data-testid="responsive-container">{children as React.ReactNode}</div>
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

const mockData = [
    { department_id: "1", department_name: "Engineering", unexcused_count: 5, consecutive_count: 3, total_absent_count: 8, total_leave_count: 3 },
    { department_id: "2", department_name: "Sales", unexcused_count: 2, consecutive_count: 1, total_absent_count: 4, total_leave_count: 2 },
    { department_id: "3", department_name: "HR", unexcused_count: 0, consecutive_count: 0, total_absent_count: 1, total_leave_count: 1 },
];

describe("UnexcusedAbsencesChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: undefined,
            isLoading: true,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: [],
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
    });

    it("renders chart with data when unexcused absences data is available", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("renders chart with horizontal (vertical) layout", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        const chart = screen.getByTestId("bar-chart");
        expect(chart).toHaveAttribute("data-layout", "vertical");
    });

    it("renders stacked bars: consecutive + remaining_unexcused (no excused leave)", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.getByTestId("bar-consecutive_count")).toBeInTheDocument();
        expect(screen.getByTestId("bar-remaining_unexcused")).toBeInTheDocument();
        expect(screen.queryByTestId("bar-total_leave_count")).not.toBeInTheDocument();
    });

    it("displays chart title and subtitle", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.getByText("dashboard.hr.unexcused_absences")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.unexcused_absences_subtitle")).toBeInTheDocument();
    });

    it("shows 7 Days and 30 Days toggle buttons", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.getByText("dashboard.hr.last_7_days")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.last_30_days_range")).toBeInTheDocument();
    });

    it("shows group-by toggle buttons (Department and Employee)", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        expect(screen.getByText("dashboard.hr.group_by_department")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.group_by_employee")).toBeInTheDocument();
    });

    it("calls useUnexcusedAbsences with groupBy parameter", () => {
 
        (dashboardHooks.useUnexcusedAbsences as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<UnexcusedAbsencesChart />);

        // Default groupBy should be "department"
        expect(dashboardHooks.useUnexcusedAbsences).toHaveBeenCalledWith(
            expect.any(String),
            expect.any(String),
            "department"
        );
    });
});
