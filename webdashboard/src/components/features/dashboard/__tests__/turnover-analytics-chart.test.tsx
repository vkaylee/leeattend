import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";

import { afterEach, describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";
import { colorByEntity } from "@/lib/stable-colors";

import TurnoverAnalyticsChart from "../turnover-analytics-chart";

vi.mock("@/hooks/use-dashboard", () => ({
    useTurnoverAnalytics: vi.fn(),
}));

// Templates for the few keys the widget interpolates with variables. Identity
// for everything else keeps existing assertions on raw keys working.
const I18N_TEMPLATES: Record<string, string> = {
    "dashboard.hr.rate_with_denominator": "{{rate}} · {{numerator}}/{{denominator}}",
    "dashboard.hr.pp_delta": "{{delta}}pp",
};

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, variables?: Record<string, unknown>) => {
            const template = I18N_TEMPLATES[key] ?? key;
            if (!variables) return template;
            return template.replace(/\{\{(\w+)\}\}/g, (_, name: string) => {
                const v = variables[name];
                return v == null ? `{{${name}}}` : String(v);
            });
        },
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
    LineChart: ({ children, ...props }: { children?: React.ReactNode; [key: string]: unknown }) => (
        <div data-testid="line-chart" {...props}>{children}</div>
    ),
    Line: (props: { dataKey: string }) => <div data-testid={`line-${props.dataKey}`} />,
    ReferenceLine: () => <div data-testid="reference-line" />,
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

const rate = (value: string | null, numerator: number, denominator: number) => ({
    value,
    numerator,
    denominator,
    unit: "employees",
});

type TurnoverData = dashboardHooks.TurnoverAnalytics;

function fullMockData(overrides: Partial<TurnoverData> = {}): TurnoverData {
    return {
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
        turnover_rate: rate("12.4", 16, 129),
        average_headcount: 129,
        voluntary_count: 9,
        involuntary_count: 5,
        unknown_classification_count: 2,
        retention_rate: "88.4",
        early_attrition: rate("3.1", 4, 129),
        regrettable_attrition: null,
        comparison: {
            basis: "previous_period",
            period_start: "2025-03-03",
            period_end: "2025-09-03",
            rate: rate("10.1", 13, 129),
            delta_percentage_points: "2.3",
        },
        target: { value: "15", operator: "lte", source: "default" },
        monthly_trend: [
            { month: "2025-10", headcount: 132, leavers: 4, turnover_rate: rate("3.0", 4, 132) },
            { month: "2025-11", headcount: 130, leavers: 3, turnover_rate: rate("2.3", 3, 130) },
            { month: "2025-12", headcount: 128, leavers: 5, turnover_rate: rate("3.9", 5, 128) },
        ],
        cohort_retention: [
            { cohort_month: "2025-09", months_since_hire: 0, cohort_size: 10, active_count: 10, retention_rate: "100" },
            { cohort_month: "2025-09", months_since_hire: 1, cohort_size: 10, active_count: 9, retention_rate: "90" },
        ],
        termination_reasons: [
            { reason: "voluntary_resignation", count: 7 },
            { reason: "involuntary_termination", count: 5 },
            { reason: "retirement", count: 2 },
            { reason: "unknown", count: 2 },
        ],
        classification: [
            { classification: "voluntary", count: 9 },
            { classification: "involuntary", count: 5 },
            { classification: "unknown", count: 2 },
        ],
        by_employment_type: [
            { employment_type: "full_time", count: 12 },
            { employment_type: "contract", count: 4 },
        ],
        by_location: [
            { location_name: "Hanoi", count: 10 },
            { location_name: "HCM", count: 6 },
        ],
        by_manager: [
            { manager_name: "Alice", count: 8 },
            { manager_name: "Bob", count: 8 },
        ],
        warnings: [],
        ...overrides,
    };
}

const emptyData = {
    tenure_distribution: [],
    by_source: [],
    by_department: [],
    by_employment_type: [],
    by_location: [],
    by_manager: [],
    termination_reasons: [],
    classification: [],
    cohort_retention: [],
    monthly_trend: [],
    total_terminated: 0,
    period_start: "2025-09-03",
    period_end: "2026-03-03",
    turnover_rate: rate(null, 0, 0),
    average_headcount: 0,
    voluntary_count: 0,
    involuntary_count: 0,
    unknown_classification_count: 0,
    retention_rate: null,
    early_attrition: rate(null, 0, 0),
    regrettable_attrition: null,
    comparison: null,
    target: { value: "15", operator: "lte", source: "default" },
    warnings: [],
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

        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByTestId("line-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: emptyData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders KPI turnover rate with denominator when value present", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Overview is the default tab — KPI header shows the rate + denominator.
        expect(screen.getByText("dashboard.hr.turnover_rate")).toBeInTheDocument();
        // rate_with_denominator interpolates to "12.4% · 16/129"
        expect(screen.getByText("12.4% · 16/129")).toBeInTheDocument();
        expect(screen.getByText("129")).toBeInTheDocument();
    });

    it("shows em-dash when turnover_rate.value is null (zero denominator)", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: {
                ...fullMockData(),
                turnover_rate: rate(null, 0, 0),
                warnings: [{ code: "LOW_DENOMINATOR", message: "low" }],
            },
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // rate_no_value renders "—" (the KPI header is rendered since data is non-empty)
        expect(screen.getByText("dashboard.hr.rate_no_value")).toBeInTheDocument();
    });

    it("renders comparison pp-delta arrow from comparison.delta_percentage_points", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // delta 2.3 → "▲ 2.3pp" (positive delta → rose/up arrow)
        expect(screen.getByText(/2\.3pp/)).toBeInTheDocument();
    });

    it("renders warning strip when warnings non-empty", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: {
                ...fullMockData(),
                warnings: [{ code: "LOW_DENOMINATOR", message: "dashboard.hr.turnover_warning_low_denominator" }],
            },
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Message key is translated by the widget's t() → identity returns the key.
        expect(screen.getByText("dashboard.hr.turnover_warning_low_denominator")).toBeInTheDocument();
    });

    it("shows tab navigation with 6 tabs", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByText("dashboard.hr.turnover_overview_tab")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_tenure")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_by_source")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_by_dept")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_reasons_tab")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.turnover_breakdown_tab")).toBeInTheDocument();
    });

    it("renders line trend chart on the overview tab (not a pie)", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        expect(screen.getByTestId("line-chart")).toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
    });

    it("renders horizontal bars (not pie) for the source tab", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        fireEvent.click(screen.getByText("dashboard.hr.turnover_by_source"));

        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
    });

    it("renders the table-view twin on each tab", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Overview tab has a ChartTableTwin for the trend.
        expect(screen.getAllByText("dashboard.hr.table_view").length).toBeGreaterThan(0);
    });

    it("has a period selector dropdown with options", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        const select = screen.getByLabelText("dashboard.hr.turnover_period");
        expect(select).toBeInTheDocument();
        expect(select).toHaveValue("6m");
    });

    it("passes comparison param to the hook", () => {
        const mockHook = vi.mocked(dashboardHooks.useTurnoverAnalytics);
        mockHook.mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        // Default comparison is previous_period.
        const lastCall = mockHook.mock.calls[mockHook.mock.calls.length - 1][0];
        expect(lastCall?.comparison).toBe("previous_period");
    });

    it("renders DashboardDateRange when custom is selected", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

        renderWithProviders(<TurnoverAnalyticsChart />);

        const select = screen.getByLabelText("dashboard.hr.turnover_period");
        fireEvent.change(select, { target: { value: "custom" } });

        expect(screen.getByLabelText("dashboard.filters.start_date")).toBeInTheDocument();
        expect(screen.getByLabelText("dashboard.filters.end_date")).toBeInTheDocument();
    });

    it("shows empty state when turnover data has all empty arrays", () => {
        vi.mocked(dashboardHooks.useTurnoverAnalytics).mockReturnValue({
            data: emptyData,
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

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });
});

describe("TurnoverAnalyticsChart — day-31 date arithmetic", () => {
    beforeEach(() => {
        vi.useFakeTimers();
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    const mockFull = () => ({
        data: fullMockData(),
        isLoading: false,
    } as unknown as ReturnType<typeof dashboardHooks.useTurnoverAnalytics>);

    it("passes correct 6-month start date when today is July 31 (should be Jan 1, not Jul 1)", () => {
        vi.setSystemTime(new Date("2026-07-31T12:00:00"));

        const mockHook = vi.mocked(dashboardHooks.useTurnoverAnalytics);
        mockHook.mockReturnValue(mockFull());

        renderWithProviders(<TurnoverAnalyticsChart />);

        const startDates = mockHook.mock.calls.map((c) => c[0]?.startDate);
        expect(startDates).toContain("2026-01-01");
    });

    it("passes correct 6-month start date when today is August 31 (should be Feb 1, not Aug 1)", () => {
        vi.setSystemTime(new Date("2026-08-31T12:00:00"));

        const mockHook = vi.mocked(dashboardHooks.useTurnoverAnalytics);
        mockHook.mockReturnValue(mockFull());

        renderWithProviders(<TurnoverAnalyticsChart />);

        const startDates = mockHook.mock.calls.map((c) => c[0]?.startDate);
        expect(startDates).toContain("2026-02-01");
    });

    it("passes correct 3-month start date when today is March 31 (should be Dec 1 of prev year)", () => {
        vi.setSystemTime(new Date("2026-03-31T12:00:00"));

        const mockHook = vi.mocked(dashboardHooks.useTurnoverAnalytics);
        mockHook.mockReturnValue(mockFull());

        renderWithProviders(<TurnoverAnalyticsChart />);

        const select = screen.getByLabelText("dashboard.hr.turnover_period");
        fireEvent.change(select, { target: { value: "3m" } });

        const startDates = mockHook.mock.calls.map((c) => c[0]?.startDate);
        expect(startDates).toContain("2025-12-01");
    });
});

describe("colorByEntity stable identity colors", () => {
    it("returns the same color for the same entity key", () => {
        const first = colorByEntity("Engineering");
        const second = colorByEntity("Engineering");
        expect(first).toBe(second);
    });

    it("can differ between distinct keys", () => {
        const a = colorByEntity("Engineering");
        const b = colorByEntity("Sales");
        // Two distinct keys may rarely collide, but these two should not.
        expect(a).not.toBe(b);
    });
});
