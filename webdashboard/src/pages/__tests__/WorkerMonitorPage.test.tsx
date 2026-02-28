import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, opts?: any) => opts?.defaultValue || key,
    }),
}));

const mockMonitorData = {
    pending_tasks: 5,
    total_workers: 3,
    active_workers: 2,
    idle_workers: 1,
    completed_today: 42,
    failed_today: 1,
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
            created_at: new Date().toISOString(),
            completed_at: null,
            error_message: null,
        },
    ],
};

vi.mock("@/hooks/use-worker-monitor", () => ({
    useWorkerMonitor: vi.fn(() => ({
        stats: mockMonitorData,
        workers: mockMonitorData.workers,
        recentJobs: mockMonitorData.recent_jobs,
        isLoading: false,
        isFetching: false,
        error: null,
        refetch: vi.fn(),
    })),
}));

import WorkerMonitorPage from "../WorkerMonitorPage";

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
        expect(screen.getByText("5")).toBeInTheDocument(); // pending_tasks (unique)
        // Stat labels are rendered
        expect(screen.getByText("monitor.pending_tasks")).toBeInTheDocument();
        expect(screen.getByText("monitor.active_workers")).toBeInTheDocument();
        expect(screen.getByText("monitor.completed_today")).toBeInTheDocument();
        expect(screen.getByText("monitor.failed_today")).toBeInTheDocument();
    });

    // --- Workers Table ---
    it("renders worker table with worker entries", () => {
        renderPage();
        expect(screen.getByText("worker-pod-1")).toBeInTheDocument();
        expect(screen.getByText("worker-pod-2")).toBeInTheDocument();
        expect(screen.getByText("LeaveAccrual")).toBeInTheDocument();
    });

    it("shows processing and idle status badges", () => {
        renderPage();
        expect(screen.getByText("PROCESSING")).toBeInTheDocument();
        expect(screen.getByText("IDLE")).toBeInTheDocument();
    });

    it("renders truncated worker IDs", () => {
        renderPage();
        expect(screen.getByText("aabbccdd...")).toBeInTheDocument();
        expect(screen.getByText("eeffaabb...")).toBeInTheDocument();
    });

    // --- Recent Jobs Table ---
    it("renders recent jobs section", () => {
        renderPage();
        expect(screen.getByText("Recent Jobs")).toBeInTheDocument();
    });

    it("renders recent jobs with truncated IDs", () => {
        renderPage();
        expect(screen.getByText("job-1111...")).toBeInTheDocument();
        expect(screen.getByText("job-aaaa...")).toBeInTheDocument();
        expect(screen.getByText("job-ffff...")).toBeInTheDocument();
    });

    it("shows color-coded status badges for jobs", () => {
        renderPage();
        const statusBadges = screen.getAllByText(/completed|failed|processing/i);
        // Should find at least 3 (from the 3 recent jobs)
        expect(statusBadges.length).toBeGreaterThanOrEqual(3);
    });

    it("shows batch progress for each job", () => {
        renderPage();
        expect(screen.getByText("4/4")).toBeInTheDocument(); // completed job
        expect(screen.getByText("1/3")).toBeInTheDocument(); // processing job
    });

    it("shows failed batch count when present", () => {
        renderPage();
        expect(screen.getByText("(1 failed)")).toBeInTheDocument();
    });

    it("displays error messages for failed jobs", () => {
        renderPage();
        expect(screen.getByText("DB connection timeout")).toBeInTheDocument();
    });

    it("shows dash for jobs without errors", () => {
        renderPage();
        // There should be dashes for successful/processing jobs
        const dashes = screen.getAllByText("—");
        expect(dashes.length).toBeGreaterThanOrEqual(2);
    });

    // --- Empty State ---
    it("shows empty state when no workers", async () => {
        const useWorkerMonitor = (await import("@/hooks/use-worker-monitor")).useWorkerMonitor;
        (useWorkerMonitor as any).mockReturnValue({
            stats: { ...mockMonitorData, total_workers: 0 },
            workers: [],
            recentJobs: [],
            isLoading: false,
            isFetching: false,
            error: null,
            refetch: vi.fn(),
        });

        renderPage();
        expect(screen.getByText("monitor.no_workers")).toBeInTheDocument();
        expect(screen.getByText("No recent jobs")).toBeInTheDocument();
    });

    // --- Loading State ---
    it("shows loading indicator for stats", async () => {
        const useWorkerMonitor = (await import("@/hooks/use-worker-monitor")).useWorkerMonitor;
        (useWorkerMonitor as any).mockReturnValue({
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
        const useWorkerMonitor = (await import("@/hooks/use-worker-monitor")).useWorkerMonitor;
        (useWorkerMonitor as any).mockReturnValue({
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
