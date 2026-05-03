import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import RecruitmentSourceChart from "../recruitment-source-chart";

vi.mock("@/hooks/use-dashboard", () => ({
    useRecruitmentSourceBreakdown: vi.fn(),
}));

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

vi.mock("recharts", () => ({
 
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

const mockData = [
    { source_type: "direct", source_name: "Direct Hire", count: 30 },
    { source_type: "referral", source_name: "Employee Referral", count: 15 },
    { source_type: "agency", source_name: "ABC Recruitment", count: 5 },
];

describe("RecruitmentSourceChart", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("renders loading spinner when data is loading", () => {
 
        vi.mocked(dashboardHooks.useRecruitmentSourceBreakdown).mockReturnValue({
            data: undefined,
            isLoading: true,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentSourceBreakdown>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
        expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    });

    it("renders empty state when no data available", () => {
 
        vi.mocked(dashboardHooks.useRecruitmentSourceBreakdown).mockReturnValue({
            data: [],
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentSourceBreakdown>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
    });

    it("renders empty state when all counts are zero", () => {
 
        vi.mocked(dashboardHooks.useRecruitmentSourceBreakdown).mockReturnValue({
            data: [{ source_type: "direct", source_name: "Direct", count: 0 }],
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentSourceBreakdown>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("common.no_data")).toBeInTheDocument();
    });

    it("renders pie chart when data is available", () => {
 
        vi.mocked(dashboardHooks.useRecruitmentSourceBreakdown).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentSourceBreakdown>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
        expect(screen.getByTestId("pie-chart")).toBeInTheDocument();
    });

    it("displays inline legend with source names and counts", () => {
 
        vi.mocked(dashboardHooks.useRecruitmentSourceBreakdown).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentSourceBreakdown>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText(/Direct Hire: 30/)).toBeInTheDocument();
        expect(screen.getByText(/Employee Referral: 15/)).toBeInTheDocument();
        expect(screen.getByText(/ABC Recruitment: 5/)).toBeInTheDocument();
    });

    it("displays the correct chart title and subtitle", () => {
 
        vi.mocked(dashboardHooks.useRecruitmentSourceBreakdown).mockReturnValue({
            data: mockData,
            isLoading: false,
        } as unknown as ReturnType<typeof dashboardHooks.useRecruitmentSourceBreakdown>);

        renderWithProviders(<RecruitmentSourceChart />);

        expect(screen.getByText("dashboard.hr.recruitment_sources")).toBeInTheDocument();
        expect(screen.getByText("dashboard.hr.recruitment_sources_subtitle")).toBeInTheDocument();
    });
});
