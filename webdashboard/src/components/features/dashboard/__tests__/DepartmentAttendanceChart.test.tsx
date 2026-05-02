import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import DepartmentAttendanceChart from "../DepartmentAttendanceChart";

// Mock the dashboard hook
vi.mock("@/hooks/use-dashboard", () => ({
    useDepartmentAttendance: vi.fn(),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

// Mock Recharts to avoid rendering SVG in tests
vi.mock("recharts", () => ({
 
    BarChart: ({ children, ...props }: { children: React.ReactNode }) => (
        <div data-testid="bar-chart" {...props}>{children}</div>
    ),
 
    Bar: (props: { dataKey: string }) => <div data-testid={`bar-${props.dataKey ?? ""}`} />,
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

const mockDeptData = [
    { department_id: "1", department_name: "Engineering", on_time: 15, late: 2, absent: 3, on_leave: 1 },
    { department_id: "2", department_name: "Sales", on_time: 8, late: 5, absent: 2, on_leave: 0 },
    { department_id: "3", department_name: "HR", on_time: 4, late: 0, absent: 1, on_leave: 1 },
];

describe("DepartmentAttendanceChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
 
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as never);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
 
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: [],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
    });

    it("renders chart with data when department data is available", () => {
 
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: mockDeptData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("renders all four bar series (on_time, late, absent, on_leave)", () => {
 
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: mockDeptData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByTestId("bar-on_time")).toBeInTheDocument();
        expect(screen.getByTestId("bar-late")).toBeInTheDocument();
        expect(screen.getByTestId("bar-absent")).toBeInTheDocument();
        expect(screen.getByTestId("bar-on_leave")).toBeInTheDocument();
    });

    it("displays chart title", () => {
 
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: mockDeptData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByText("dashboard.hr.dept_attendance")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.dept_attendance_subtitle")).toBeInTheDocument();
    });

    it("shows Today and This Week toggle buttons", () => {
 
        (dashboardHooks.useDepartmentAttendance as any).mockReturnValue({
            data: mockDeptData,
            isLoading: false,
        });

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByText("dashboard.hr.today")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.this_week")).toBeInTheDocument();
    });
});
