import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import RecruitmentSourceChart from "../recruitment-source-chart";

vi.mock("@/hooks/use-dashboard", () => ({
    useRecruitmentEffectiveness: vi.fn(),
}));

vi.mock("@/hooks/use-workspace", () => ({
    useWorkspace: () => ({ workspace: { id: "ws-1" } }),
}));

vi.mock("@/hooks/use-departments", () => ({
    useDepartments: () => ({ data: [{ id: "dept-1", name: "Engineering" }] }),
}));

vi.mock("@/hooks/use-datetime", () => ({
    useDateTimeFormatter: () => ({
        formatDate: (d: string | Date | null | undefined) => String(d ?? ""),
    }),
}));

// Templates for the few keys the widget interpolates with variables. Identity
// for everything else keeps existing assertions on raw keys working.
const I18N_TEMPLATES: Record<string, string> = {
    "dashboard.hr.effective_comparison_delta": "{{delta}} vs previous",
    "dashboard.hr.effective_previous_hires": "Previous: {{count}} hires",
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

const rate = (value: string | null) => value;

type EffectivenessData = dashboardHooks.RecruitmentEffectivenessAnalytics;

function fullMockData(overrides: Partial<EffectivenessData> = {}): EffectivenessData {
    return {
        period_start: "2025-08-14",
        period_end: "2026-02-14",
        total_hires: 50,
        overview: {
            hires: 50,
            average_cost_per_hire: "2500.00",
            cost_available: true,
            retained_90d: 42,
            retention_90d: "84.0",
            turnover_in_range: 3,
            comparison_hires: 40,
            comparison_hires_delta: 10,
        },
        funnel: [
            { stage: "applicants", count: null, data_source: null, conversion_rate: null },
            { stage: "screening", count: null, data_source: null, conversion_rate: null },
            { stage: "interview", count: null, data_source: null, conversion_rate: null },
            { stage: "offer", count: null, data_source: null, conversion_rate: null },
            { stage: "hired", count: 50, data_source: "hires", conversion_rate: null },
            { stage: "retained_90d", count: 42, data_source: "hires", conversion_rate: "84.0" },
        ],
        by_source: [
            { source_id: "src-1", source_type: "direct", source_name: "Direct Hire", hires: 30, hires_ratio: "60.0", retained_90d: 27, retention_90d: "90.0", cost_per_hire: null, cost_per_hire_available: false, turnover_in_range: 1 },
            { source_id: "src-2", source_type: "referral", source_name: "Employee Referral", hires: 15, hires_ratio: "30.0", retained_90d: 11, retention_90d: "73.3", cost_per_hire: "2000.00", cost_per_hire_available: true, turnover_in_range: 2 },
            { source_id: "src-3", source_type: "agency", source_name: "ABC Recruitment", hires: 5, hires_ratio: "10.0", retained_90d: 4, retention_90d: "80.0", cost_per_hire: "5000.00", cost_per_hire_available: true, turnover_in_range: 0 },
        ],
        hires_trend: [
            { month: "2025-09", hires: 12 },
            { month: "2025-10", hires: 18 },
            { month: "2025-11", hires: 20 },
        ],
        data_gaps: [
            { metric: "applicants", reason: "dashboard.hr.effective_gap_no_applicant_data" },
        ],
        warnings: [],
        ...overrides,
    };
}

const emptyData: EffectivenessData = {
    period_start: "2025-08-14",
    period_end: "2026-02-14",
    total_hires: 0,
    overview: {
        hires: 0,
        average_cost_per_hire: null,
        cost_available: false,
        retained_90d: 0,
        retention_90d: null,
        turnover_in_range: 0,
        comparison_hires: null,
        comparison_hires_delta: null,
    },
    funnel: [
        { stage: "applicants", count: null, data_source: null, conversion_rate: null },
        { stage: "hired", count: null, data_source: null, conversion_rate: null },
    ],
    by_source: [],
    hires_trend: [],
    data_gaps: [
        { metric: "applicants", reason: "dashboard.hr.effective_gap_no_applicant_data" },
    ],
    warnings: [],
};

describe("RecruitmentSourceChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading skeleton when data is loading", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.queryByTestId("line-chart")).not.toBeInTheDocument();
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: emptyData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders KPI header with hires + retention + comparison", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("dashboard.hr.effective_total_hires")).toBeInTheDocument();
        expect(screen.getByText("50")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.effective_retained_90d")).toBeInTheDocument();
        expect(screen.getByText("42")).toBeInTheDocument();
        // Comparison badge: delta 10 → "10 vs previous"
        expect(screen.getByText(/10 vs previous/)).toBeInTheDocument();
    });

    it("renders line trend chart on the overview tab (not a pie)", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByTestId("line-chart")).toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
    });

    it("funnel tab shows 'No data source' for pre-hire stages, never a fabricated count", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        fireEvent.click(screen.getByText("dashboard.hr.effective_funnel_tab"));

        expect(screen.getByText("dashboard.hr.effective_funnel_honesty_note")).toBeInTheDocument();
        fireEvent.click(screen.getByText("dashboard.hr.table_view"));
        expect(screen.getAllByText("dashboard.hr.effective_no_data_source").length).toBeGreaterThan(0);
    });

    it("ranking tab uses stable per-source-type colors (direct is green, not array-index)", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        const { unmount } = renderWithProviders(<RecruitmentSourceChart />);
        fireEvent.click(screen.getByText("dashboard.hr.effective_ranking_tab"));

        const cells = screen.getAllByTestId("cell");
        expect(cells.length).toBeGreaterThan(0);
        // Direct sources always map to #22c55e via colorByEntityWithKnown.
        const directCells = cells.filter((c) => c.getAttribute("fill") === "#22c55e");
        expect(directCells.length).toBeGreaterThan(0);

        unmount();
        renderWithProviders(<RecruitmentSourceChart />);
        fireEvent.click(screen.getByText("dashboard.hr.effective_ranking_tab"));
        const cellsAfter = screen.getAllByTestId("cell");
        const directCellsAfter = cellsAfter.filter((c) => c.getAttribute("fill") === "#22c55e");
        expect(directCellsAfter.length).toBe(directCells.length);
    });

    it("renders warning strip when warnings non-empty", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: {
                ...fullMockData(),
                warnings: [{ code: "LOW_DENOMINATOR", message: "dashboard.hr.effective_warning_low_sample" }],
            },
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("dashboard.hr.effective_warning_low_sample")).toBeInTheDocument();
    });

    it("shows tab navigation with 4 tabs", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("dashboard.hr.effective_overview_tab")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.effective_funnel_tab")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.effective_ranking_tab")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.effective_table_tab")).toBeInTheDocument();
    });

    it("renders the table-view twin", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        // Overview tab has a ChartTableTwin for the trend.
        expect(screen.getAllByText("dashboard.hr.table_view").length).toBeGreaterThan(0);
    });

    it("has a period selector dropdown defaulting to 6m", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        const select = screen.getByLabelText("dashboard.hr.turnover_period");
        expect(select).toHaveValue("6m");
    });

    it("passes comparison param to the hook", () => {
        const mockHook = vi.mocked(dashboardHooks.useRecruitmentEffectiveness);
        mockHook.mockReturnValue({
            data: fullMockData(),
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        const lastCall = mockHook.mock.calls[mockHook.mock.calls.length - 1][0];
        expect(lastCall?.comparison).toBe("previous_period");
    });

    it("handles undefined data (not loading) with empty state", () => {
        vi.mocked(dashboardHooks.useRecruitmentEffectiveness).mockReturnValue({
            data: undefined,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentEffectiveness>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });
});
