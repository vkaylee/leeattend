import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import PeakTimesChart from "../PeakTimesChart";

vi.mock("@/hooks/use-dashboard", () => ({
    usePeakTimes: vi.fn(),
}));

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

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

const mockData = [
    { time_slot: "07:30-08:00", check_in: 50, check_out: 5 },
    { time_slot: "08:00-08:30", check_in: 120, check_out: 10 },
    { time_slot: "17:00-17:30", check_in: 2, check_out: 80 },
];

describe("PeakTimesChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
 
        vi.mocked(dashboardHooks.usePeakTimes).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as never);

        renderWithProviders(<PeakTimesChart />);

        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
 
        vi.mocked(dashboardHooks.usePeakTimes).mockReturnValue({
            data: [],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<PeakTimesChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all values are zero", () => {
 
        vi.mocked(dashboardHooks.usePeakTimes).mockReturnValue({
            data: [{ time_slot: "07:30-08:00", check_in: 0, check_out: 0 }],
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<PeakTimesChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders chart with data when peak times data is available", () => {
 
        vi.mocked(dashboardHooks.usePeakTimes).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<PeakTimesChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("renders both bar series (check_in, check_out)", () => {
 
        vi.mocked(dashboardHooks.usePeakTimes).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as never);

        renderWithProviders(<PeakTimesChart />);

        expect(screen.getByTestId("bar-check_in")).toBeInTheDocument();
        expect(screen.getByTestId("bar-check_out")).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
 
        (dashboardHooks.usePeakTimes as any).mockReturnValue({
            data: mockData,
            isLoading: false,
        });

        renderWithProviders(<PeakTimesChart />);

        expect(screen.getByText("dashboard.hr.peak_times")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.peak_times_subtitle")).toBeInTheDocument();
    });
});
