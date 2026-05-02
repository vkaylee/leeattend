import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import LeaveTypesChart from "../leave-types-chart";

vi.mock("@/hooks/use-dashboard", () => ({
    useLeaveTypesBreakdown: vi.fn(),
}));

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

vi.mock("recharts", () => ({
 
    PieChart: ({ children, ...props }: { children: React.ReactNode }) => (
        <div data-testid="pie-chart" {...props}>{children}</div>
    ),
 
    Pie: ({ children, ...props }: { children: React.ReactNode }) => (
        <div data-testid="pie" {...props}>{children}</div>
    ),
 
    Cell: (props: { fill: string }) => <div data-testid="cell" {...props} />,
 
    ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
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

const mockData = [
    { leave_type: "Sick", count: 15 },
    { leave_type: "Annual", count: 25 },
    { leave_type: "Maternity", count: 3 },
    { leave_type: "Unpaid", count: 7 },
];

describe("LeaveTypesChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
 
        vi.mocked(dashboardHooks.useLeaveTypesBreakdown).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as never);

        renderWithProviders(<LeaveTypesChart />);

        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
 
        vi.mocked(dashboardHooks.useLeaveTypesBreakdown).mockReturnValue({
            data: [],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<LeaveTypesChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all counts are zero", () => {
 
        vi.mocked(dashboardHooks.useLeaveTypesBreakdown).mockReturnValue({
            data: [{ leave_type: "Sick", count: 0 }],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<LeaveTypesChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders pie chart when data is available", () => {
 
        vi.mocked(dashboardHooks.useLeaveTypesBreakdown).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<LeaveTypesChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("pie-chart")).toBeInTheDocument();
    });

    it("displays inline legend with leave type names and counts", () => {
 
        vi.mocked(dashboardHooks.useLeaveTypesBreakdown).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<LeaveTypesChart />);

        expect(screen.getByText(/Sick: 15/)).toBeInTheDocument();
        expect(screen.getByText(/Annual: 25/)).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
 
        (dashboardHooks.useLeaveTypesBreakdown as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<LeaveTypesChart />);

        expect(screen.getByText("dashboard.hr.leave_types")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.leave_types_subtitle")).toBeInTheDocument();
    });
});
