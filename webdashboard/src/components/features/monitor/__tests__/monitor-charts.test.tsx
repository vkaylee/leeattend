import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";


// Mock react-i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
 
        t: (key: string, options?: any) => options?.defaultValue ?? key,
    }),
}));

// Mock recharts
vi.mock("recharts", () => ({
 
    RadialBarChart: ({ children, ...props }: any) => (
        <div data-testid="radial-chart" {...props}>{children}</div>
    ),
 
    RadialBar: (_props: any) => <div data-testid="radial-bar" />,
    PolarAngleAxis: () => <div data-testid="polar-axis" />,
 
    ResponsiveContainer: ({ children }: any) => (
        <div data-testid="responsive-container">{children}</div>
    ),
 
    BarChart: ({ children, ...props }: any) => (
        <div data-testid="bar-chart" {...props}>{children}</div>
    ),
 
    Bar: (_props: any) => <div data-testid="bar" />,
    XAxis: () => <div data-testid="x-axis" />,
    YAxis: () => <div data-testid="y-axis" />,
    CartesianGrid: () => <div data-testid="cartesian-grid" />,
    Tooltip: () => <div data-testid="tooltip" />,
    Cell: () => <div data-testid="cell" />,
 
    PieChart: ({ children, ...props }: any) => (
        <div data-testid="pie-chart" {...props}>{children}</div>
    ),
 
    Pie: ({ children, ...props }: any) => (
        <div data-testid="pie" {...props}>{children}</div>
    ),
}));

import JobStatusDistributionChart from "../job-status-distribution-chart";
import QueuePressureChart from "../queue-pressure-chart";
import WorkerUtilizationGauge from "../worker-utilization-gauge";

const renderWithProviders = (component: React.ReactElement) => {
    const queryClient = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });
    return render(
        <QueryClientProvider client={queryClient}>
            {component}
        </QueryClientProvider>
    );
};

// =============================================================================
// WorkerUtilizationGauge
// =============================================================================
describe("WorkerUtilizationGauge", () => {
    it("renders gauge with correct percentage", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 3, idle: 2, total: 5, percent: 60 }} isLoading={false} />
        );
        expect(screen.getByText("60%")).toBeInTheDocument();
    });

    it("shows loading state", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 0, idle: 0, total: 0, percent: 0 }} isLoading={true} />
        );
        expect(screen.queryByText("0%")).not.toBeInTheDocument();
        expect(screen.queryByTestId("radial-chart")).not.toBeInTheDocument();
    });

    it("shows scale-up warning when utilization > 80%", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 9, idle: 1, total: 10, percent: 90 }} isLoading={false} />
        );
        expect(screen.getByText(/monitor.charts.scale_up_warning/)).toBeInTheDocument();
    });

    it("shows scale-down hint when utilization < 30%", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 1, idle: 9, total: 10, percent: 10 }} isLoading={false} />
        );
        expect(screen.getByText(/monitor.charts.scale_down_hint/)).toBeInTheDocument();
    });

    it("does not show scaling hints for normal utilization", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 3, idle: 2, total: 5, percent: 60 }} isLoading={false} />
        );
        expect(screen.queryByText(/scale_up_warning/)).not.toBeInTheDocument();
        expect(screen.queryByText(/scale_down_hint/)).not.toBeInTheDocument();
    });

    it("renders chart title", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 0, idle: 0, total: 0, percent: 0 }} isLoading={false} />
        );
        expect(screen.getAllByText("monitor.charts.utilization").length).toBeGreaterThanOrEqual(1);
    });

    it("renders active and idle pills", () => {
        renderWithProviders(
            <WorkerUtilizationGauge data={{ active: 3, idle: 2, total: 5, percent: 60 }} isLoading={false} />
        );
        // Active pill
        expect(screen.getByText(/3/)).toBeInTheDocument();
        // Idle pill
        expect(screen.getByText(/2/)).toBeInTheDocument();
    });
});

// =============================================================================
// QueuePressureChart
// =============================================================================
describe("QueuePressureChart", () => {
    it("renders bars with correct total", () => {
        renderWithProviders(
            <QueuePressureChart data={{ high: 10, low: 5, total: 15, pressureLevel: "medium" }} isLoading={false} />
        );
        expect(screen.getByText("15")).toBeInTheDocument();
        expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    });

    it("shows loading state", () => {
        renderWithProviders(
            <QueuePressureChart data={{ high: 0, low: 0, total: 0, pressureLevel: "low" }} isLoading={true} />
        );
        expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
    });

    it("shows empty state when queue is empty", () => {
        renderWithProviders(
            <QueuePressureChart data={{ high: 0, low: 0, total: 0, pressureLevel: "low" }} isLoading={false} />
        );
        expect(screen.getByText("Queue clear")).toBeInTheDocument();
    });

    it("renders pressure level badge", () => {
        renderWithProviders(
            <QueuePressureChart data={{ high: 20, low: 5, total: 25, pressureLevel: "critical" }} isLoading={false} />
        );
        const badge = screen.getByTestId("pressure-badge");
        expect(badge).toBeInTheDocument();
        expect(badge).toHaveTextContent("Critical");
    });

    it("renders chart title", () => {
        renderWithProviders(
            <QueuePressureChart data={{ high: 0, low: 0, total: 0, pressureLevel: "low" }} isLoading={false} />
        );
        expect(screen.getByText("monitor.queue_pressure")).toBeInTheDocument();
    });
});

// =============================================================================
// JobStatusDistributionChart
// =============================================================================
describe("JobStatusDistributionChart", () => {
    const mockDistribution = [
        { status: "completed", count: 8, color: "#10b981", percent: 72.7 },
        { status: "failed", count: 2, color: "#ef4444", percent: 18.2 },
        { status: "processing", count: 1, color: "#f59e0b", percent: 9.1 },
    ];

    it("renders donut chart", () => {
        renderWithProviders(
            <JobStatusDistributionChart data={mockDistribution} isLoading={false} />
        );
        expect(screen.getByTestId("pie-chart")).toBeInTheDocument();
    });

    it("renders legend entries with percentages", () => {
        renderWithProviders(
            <JobStatusDistributionChart data={mockDistribution} isLoading={false} />
        );
        expect(screen.getByTestId("legend-completed")).toHaveTextContent("completed 72.7%");
        expect(screen.getByTestId("legend-failed")).toHaveTextContent("failed 18.2%");
        expect(screen.getByTestId("legend-processing")).toHaveTextContent("processing 9.1%");
    });

    it("shows loading state", () => {
        renderWithProviders(
            <JobStatusDistributionChart data={[]} isLoading={true} />
        );
        expect(screen.queryByTestId("pie-chart")).not.toBeInTheDocument();
    });

    it("shows empty state when no jobs", () => {
        renderWithProviders(
            <JobStatusDistributionChart data={[]} isLoading={false} />
        );
        expect(screen.getByText("No data")).toBeInTheDocument();
    });

    it("renders chart title", () => {
        renderWithProviders(
            <JobStatusDistributionChart data={mockDistribution} isLoading={false} />
        );
        expect(screen.getByText("monitor.job_distribution")).toBeInTheDocument();
    });
});
