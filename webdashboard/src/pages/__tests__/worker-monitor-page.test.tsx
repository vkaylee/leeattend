import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useWorkerMonitor, useWorkerJobs } from "@/hooks/use-worker-monitor";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, options?: Record<string, unknown>) => (options?.defaultValue as string) || key,
    }),
}));

vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({
        success: vi.fn(),
        error: vi.fn(),
        info: vi.fn(),
        warning: vi.fn(),
    }),
}));

// Mock chart components to avoid recharts SVG rendering in jsdom
vi.mock("@/components/features/monitor/worker-utilization-gauge", () => ({
    default: () => <div data-testid="utilization-gauge" />,
}));
vi.mock("@/components/features/monitor/queue-pressure-chart", () => ({
    default: () => <div data-testid="queue-pressure-chart" />,
}));
vi.mock("@/components/features/monitor/job-status-distribution-chart", () => ({
    default: () => <div data-testid="job-distribution-chart" />,
}));

// Mock data transformation hooks
vi.mock("@/hooks/use-monitor-charts", () => ({
    useWorkerUtilization: () => ({ active: 2, idle: 1, total: 3, percent: 67 }),
    useQueuePressure: () => ({ high: 3, low: 2, total: 5 }),
    useJobStatusDistribution: () => [
        { status: "completed", count: 1, color: "#10b981" },
        { status: "failed", count: 1, color: "#ef4444" },
    ],
}));

const testMetric = (unit = "jobs") => ({
    value: 0,
    unit,
    formula: "test fixture",
    time_window: {
        start: "2026-01-01T00:00:00.000Z",
        end: "2026-01-01T00:15:00.000Z",
        duration_seconds: 900,
    },
    observed_at: "2026-01-01T00:15:00.000Z",
    source: "test_fixture",
    unknown: null,
});

const mockMonitorData = {
    pending_tasks: 5,
    pending_high: 3,
    pending_low: 2,
    total_workers: 3,
    active_workers: 2,
    idle_workers: 1,
    completed_today: 42,
    failed_today: 1,
    queue_health: {
        dead_letter_depth: testMetric(),
        depth_high: testMetric(),
        depth_low: testMetric(),
        depth_normal: testMetric(),
        depth_total: testMetric(),
        enqueue_rate_per_second: testMetric("jobs/second"),
        estimated_drain_seconds: testMetric("seconds"),
        growth_rate_per_second: testMetric("jobs/second"),
        oldest_queued_age_seconds: testMetric("seconds"),
        processing_rate_per_second: testMetric("jobs/second"),
        queue_wait_seconds_p50: testMetric("seconds"),
        queue_wait_seconds_p95: testMetric("seconds"),
    },
    reliability: {
        consecutive_failures: testMetric(),
        dead_letter_count: testMetric(),
        execution_seconds_p50: testMetric("seconds"),
        execution_seconds_p95: testMetric("seconds"),
        execution_seconds_p99: testMetric("seconds"),
        failure_rate: testMetric("ratio"),
        lost_count: testMetric(),
        retryable_count: testMetric(),
        success_rate: testMetric("ratio"),
        timeout_count: testMetric(),
    },
    oldest_queued_job: null,
    oldest_running_job: null,
    task_type_reliability: [],
    job_type_stats: [
        {
            task_type: "attendance_report",
            total: 2,
            completed: 1,
            processing: 1,
            failed: 0,
            pending: 0,
        },
        {
            task_type: "payroll_report",
            total: 1,
            completed: 0,
            processing: 0,
            failed: 1,
            pending: 0,
        },
    ],
    workers: [
        {
            id: "aabbccdd-1111-2222-3333-444455556666",
            host: "worker-pod-1",
            status: "PROCESSING" as const,
            current_task_type: "LeaveAccrual",
            last_seen_at: new Date().toISOString(),
        },
        {
            id: "eeffaabb-5555-6666-7777-888899990000",
            host: "worker-pod-2",
            status: "IDLE" as const,
            current_task_type: null,
            last_seen_at: new Date().toISOString(),
        },
    ],
    recent_jobs: [
        {
            job_id: "job-1111-2222-3333-444455556666",
            status: "completed" as const,
            total_batches: 4,
            completed_batches: 4,
            failed_batches: 0,
            workspace_id: "ws-1",
            requester_id: "user-1",
            report_type: "summary",
            task_type: "attendance_report",
            payload: { start_date: "2026-01-01", end_date: "2026-01-31" },
            created_at: new Date(Date.now() - 60000).toISOString(),
            completed_at: new Date().toISOString(),
            error_message: null,
        },
        {
            job_id: "job-aaaa-bbbb-cccc-ddddeeeeffff",
            status: "failed" as const,
            total_batches: 1,
            completed_batches: 0,
            failed_batches: 1,
            workspace_id: "ws-1",
            requester_id: "user-1",
            report_type: "summary",
            task_type: "payroll_report",
            payload: { employee_ids: ["emp-1"] },
            created_at: new Date(Date.now() - 120000).toISOString(),
            completed_at: new Date().toISOString(),
            error_message: "DB connection timeout",
        },
        {
            job_id: "job-ffff-eeee-dddd-ccccbbbbaaaa",
            status: "processing" as const,
            total_batches: 3,
            completed_batches: 1,
            failed_batches: 0,
            workspace_id: "ws-1",
            requester_id: "user-1",
            report_type: "summary",
            task_type: "attendance_report",
            payload: null,
            created_at: new Date().toISOString(),
            completed_at: null,
            error_message: null,
        },
    ],
};

vi.mock("@/hooks/use-worker-monitor", () => ({
    MONITOR_REFETCH_INTERVAL_S: 2,
    useWorkerMonitor: vi.fn(() => ({
        stats: mockMonitorData,
        workers: mockMonitorData.workers,
        recentJobs: mockMonitorData.recent_jobs,
        isLoading: false,
        isFetching: false,
        error: null,
        refetch: vi.fn(),
    })),
    useWorkerJobs: vi.fn(() => ({
        data: { pages: [{ jobs: mockMonitorData.recent_jobs, next_cursor: null }] },
        jobs: mockMonitorData.recent_jobs,
        hasNextPage: false,
        fetchNextPage: vi.fn(),
        isFetchingNextPage: false,
        isLoading: false,
    })),
}));

import WorkerMonitorPage from "../worker-monitor-page";

function renderPage() {
    const queryClient = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });
    return render(
        <QueryClientProvider client={queryClient}>
            <WorkerMonitorPage />
        </QueryClientProvider>
    );
}

describe("WorkerMonitorPage", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    // --- Stats Cards ---
    it("renders stat cards with correct values", () => {
        renderPage();
        expect(screen.getByText("42")).toBeInTheDocument(); // completed_today (unique)
        expect(screen.getAllByText("3").length).toBeGreaterThanOrEqual(1); // pending_high
        // Stat labels are rendered
        expect(screen.getByText("monitor.pending_high")).toBeInTheDocument();
        expect(screen.getByText("monitor.pending_low")).toBeInTheDocument();
        expect(screen.getByText("monitor.active_workers")).toBeInTheDocument();
        expect(screen.getByText("monitor.completed_today")).toBeInTheDocument();
        expect(screen.getByText("monitor.failed_today")).toBeInTheDocument();
    });

    // --- Workers Table ---
    it("renders worker table with worker entries", () => {
        renderPage();
        // ResponsiveTable renders both desktop and mobile views
        expect(screen.getAllByText("worker-pod-1").length).toBeGreaterThan(0);
        expect(screen.getAllByText("worker-pod-2").length).toBeGreaterThan(0);
        expect(screen.getAllByText("LeaveAccrual").length).toBeGreaterThan(0);
    });

    it("shows processing and idle status badges", () => {
        renderPage();
        expect(screen.getAllByText("PROCESSING").length).toBeGreaterThan(0);
        expect(screen.getAllByText("IDLE").length).toBeGreaterThan(0);
    });

    it("renders truncated worker IDs", () => {
        renderPage();
        expect(screen.getAllByText("aabbccdd...").length).toBeGreaterThan(0);
        expect(screen.getAllByText("eeffaabb...").length).toBeGreaterThan(0);
    });

    // --- Recent Jobs Table ---
    it("renders recent jobs section", () => {
        renderPage();
        expect(screen.getByText("Recent Jobs")).toBeInTheDocument();
    });

    it("renders recent jobs with truncated IDs", () => {
        renderPage();
        // ResponsiveTable renders both desktop and mobile views
        expect(screen.getAllByText("job-1111…").length).toBeGreaterThan(0);
        expect(screen.getAllByText("job-aaaa…").length).toBeGreaterThan(0);
        expect(screen.getAllByText("job-ffff…").length).toBeGreaterThan(0);
    });

    it("shows color-coded status badges for jobs", () => {
        renderPage();
        const statusBadges = screen.getAllByText(/completed|failed|processing/i);
        // Should find at least 3 (from the 3 recent jobs)
        expect(statusBadges.length).toBeGreaterThanOrEqual(3);
    });

    it("shows batch progress for each job", () => {
        renderPage();
        // ResponsiveTable renders both desktop and mobile views
        expect(screen.getAllByText("4/4").length).toBeGreaterThan(0); // completed job
        expect(screen.getAllByText("1/3").length).toBeGreaterThan(0); // processing job
    });

    it("shows failed batch count when present", () => {
        renderPage();
        expect(screen.getAllByText("(1 failed)").length).toBeGreaterThan(0);
    });

    it("does not render raw legacy error payloads", () => {
        renderPage();
        expect(screen.queryByText("DB connection timeout")).not.toBeInTheDocument();
    });

    it("shows dash for jobs without errors", () => {
        renderPage();
        // There should be dashes for successful/processing jobs
        const dashes = screen.getAllByText("—");
        expect(dashes.length).toBeGreaterThanOrEqual(2);
    });

    // --- Job Type Breakdown Panel ---
    it("renders job type breakdown panel with stats", () => {
        renderPage();
        expect(screen.getByText("Breakdown by Type")).toBeInTheDocument();
        // We have 2 attendance_reports (1 completed, 1 processing) -> total 2
        // We have 1 payroll_report (failed) -> total 1
        expect(screen.getAllByText("attendance_report").length).toBeGreaterThanOrEqual(1);
        expect(screen.getAllByText("payroll_report").length).toBeGreaterThanOrEqual(1);
    });

    // --- Job Details Drawer / Sheet ---
    it("opens details drawer on clicking a job row", async () => {
        renderPage();

        // ResponsiveTable renders both views; use getAllByText
        const rows = screen.getAllByText("job-1111…");
        expect(rows.length).toBeGreaterThan(0);

        // Click the desktop table row (tr) which is the clickable element
        const tableRow = rows[0].closest("tr");
        if (tableRow) {
            fireEvent.click(tableRow);
        } else {
            // Mobile card — click the card div
            fireEvent.click(rows[0].closest('[role="button"]') || rows[0]);
        }
        
        // Drawer should display job details
        expect(screen.getByText("Job Details")).toBeInTheDocument();
        expect(screen.getByText("job-1111-2222-3333-444455556666")).toBeInTheDocument();
        expect(screen.getByText("Sensitive diagnostic payloads are not displayed in the monitor.")).toBeInTheDocument();
    });

    // --- Empty State ---
    it("shows empty state when no workers", async () => {
        vi.mocked(useWorkerMonitor).mockReturnValue({
            stats: { ...mockMonitorData, total_workers: 0 },
            workers: [],
            recentJobs: [],
            isLoading: false,
            isFetching: false,
            error: null,
            refetch: vi.fn(),
        });
        vi.mocked(useWorkerJobs).mockReturnValue({
            data: { pages: [{ jobs: [], next_cursor: null }] },
            jobs: [],
            hasNextPage: false,
            fetchNextPage: vi.fn(),
            isFetchingNextPage: false,
            isLoading: false,
        } as any);

        renderPage();
        expect(screen.getByText("monitor.no_workers")).toBeInTheDocument();
        expect(screen.getByText("No jobs match your search filters")).toBeInTheDocument();
    });

    // --- Loading State ---
    it("shows loading indicator for stats", async () => {
        vi.mocked(useWorkerMonitor).mockReturnValue({
            stats: undefined,
            workers: [],
            recentJobs: [],
            isLoading: true,
            isFetching: false,
            error: null,
            refetch: vi.fn(),
        });

        renderPage();
        // Stats should show "..." when loading
        const loadingIndicators = screen.getAllByText("...");
        expect(loadingIndicators.length).toBeGreaterThanOrEqual(1);
    });

    // --- Error State ---
    it("shows error banner when fetch fails", async () => {
        vi.mocked(useWorkerMonitor).mockReturnValue({
            stats: undefined,
            workers: [],
            recentJobs: [],
            isLoading: false,
            isFetching: false,
            error: new Error("Redis connection failed"),
            refetch: vi.fn(),
        });

        renderPage();
        expect(screen.getByText("Failed to load monitoring data. Retrying...")).toBeInTheDocument();
    });
});
