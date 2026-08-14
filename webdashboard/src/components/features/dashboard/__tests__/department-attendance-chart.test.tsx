import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import DepartmentAttendanceChart from "../department-attendance-chart";

// Mock the dashboard hook
vi.mock("@/hooks/use-dashboard", () => ({
    useDepartmentAttendance: vi.fn(),
}));

// Mock useWorkspace (provides workspace timezone)
vi.mock("@/hooks/use-workspace", () => ({
    useWorkspace: () => ({ workspace: { timezone: "Asia/Ho_Chi_Minh" } }),
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
    Bar: (props: { dataKey: string }) => <div data-testid={`bar-${props.dataKey}`} />,
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

const rate = (value: string | null) => ({ value, numerator: 1, denominator: 10, unit: "employees" });

const mockDeptData = [
    { department_id: "1", department_name: "Engineering", on_time: 15, late: 2, absent: 3, on_leave: 1, total_employees: 21, attendance_rate: rate("71.4"), absence_rate: rate("14.3"), late_rate: rate("9.5") },
    { department_id: "2", department_name: "Sales", on_time: 8, late: 5, absent: 2, on_leave: 0, total_employees: 15, attendance_rate: rate("53.3"), absence_rate: rate("13.3"), late_rate: rate("33.3") },
    { department_id: "3", department_name: "HR", on_time: 4, late: 0, absent: 1, on_leave: 1, total_employees: 6, attendance_rate: rate("66.7"), absence_rate: rate("16.7"), late_rate: rate("0") },
];

describe("DepartmentAttendanceChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as ReturnType<typeof dashboardHooks.useDepartmentAttendance>);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: [],
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useDepartmentAttendance>);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
    });

    it("renders a single attendance-rate bar series (not raw-count bars)", () => {
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: mockDeptData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useDepartmentAttendance>);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
        // Rate series only — no on_time/late/absent/on_leave bars.
        expect(screen.getByTestId("bar-attendance")).toBeInTheDocument();
        expect(screen.queryByTestId("bar-on_time")).not.toBeInTheDocument();
        expect(screen.queryByTestId("bar-late")).not.toBeInTheDocument();
    });

    it("displays chart title", () => {
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: mockDeptData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useDepartmentAttendance>);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByText("dashboard.hr.dept_attendance")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.dept_attendance_subtitle")).toBeInTheDocument();
    });

    it("handles missing data gracefully (undefined data, not loading)", () => {
        vi.mocked(dashboardHooks.useDepartmentAttendance).mockReturnValue({
            data: undefined,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useDepartmentAttendance>);

        renderWithProviders(<DepartmentAttendanceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });
});
