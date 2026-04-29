import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import LatenessByDayChart from "../LatenessByDayChart";

vi.mock("@/hooks/use-dashboard", () => ({
    useLatenessByDay: vi.fn(),
}));

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

vi.mock("recharts", () => ({
    ComposedChart: ({ children, ...props }: any) => (
        <div data-testid="composed-chart" {...props}>{children}</div>
    ),
    Bar: (props: any) => <div data-testid={`bar-${props.dataKey}`} />,
    Line: (props: any) => <div data-testid={`line-${props.dataKey}`} />,
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

const mockData = [
    { day: "Mon", late_count: 15, avg_late_minutes: 12.5 },
    { day: "Tue", late_count: 8, avg_late_minutes: 7.2 },
    { day: "Wed", late_count: 5, avg_late_minutes: 4.0 },
    { day: "Thu", late_count: 6, avg_late_minutes: 5.5 },
    { day: "Fri", late_count: 12, avg_late_minutes: 10.1 },
];

describe("LatenessByDayChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
        (dashboardHooks.useLatenessByDay as any).mockReturnValue({
            data: undefined,
            isLoading: true,
        });

        renderWithProviders(<LatenessByDayChart />);

        expect(screen.queryByTestId("composed-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        (dashboardHooks.useLatenessByDay as any).mockReturnValue({
            data: [],
            isLoading: false,
        });

        renderWithProviders(<LatenessByDayChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("composed-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all late counts are zero", () => {
        (dashboardHooks.useLatenessByDay as any).mockReturnValue({
            data: [{ day: "Mon", late_count: 0, avg_late_minutes: 0 }],
            isLoading: false,
        });

        renderWithProviders(<LatenessByDayChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders chart with data when lateness data is available", () => {
        (dashboardHooks.useLatenessByDay as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<LatenessByDayChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("composed-chart")).toBeInTheDocument();
    });

    it("renders bar and line series", () => {
        (dashboardHooks.useLatenessByDay as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<LatenessByDayChart />);

        expect(screen.getByTestId("bar-late_count")).toBeInTheDocument();
        expect(screen.getByTestId("line-avg_late_minutes")).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
        (dashboardHooks.useLatenessByDay as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<LatenessByDayChart />);

        expect(screen.getByText("dashboard.hr.lateness_by_day")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.lateness_by_day_subtitle")).toBeInTheDocument();
    });
});
