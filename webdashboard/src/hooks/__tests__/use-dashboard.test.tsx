import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";

import {
    DashboardQueryError,
    useDashboardStats,
    useAttendanceTrend,
    useDepartmentAttendance,
    useUnexcusedAbsences,
    useTurnoverAnalytics,
    useDashboardMetrics,
} from "../use-dashboard";
import type { DashboardMetricsFilters } from "../dashboard-metrics-filters";

vi.mock("@/lib/api", async () => {
    const actual = await vi.importActual("@/lib/api");
    return {
        ...actual,
        apiFetch: vi.fn(),
    };
});

vi.mock("../use-workspace", () => ({
    useWorkspace: () => ({
        workspace: { id: "ws-test-123", name: "Test Workspace" },
    }),
}));

const createWrapper = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
        },
    });
    const Wrapper = ({ children }: { children: React.ReactNode }) => (
        <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    Wrapper.displayName = "QueryClientWrapper";
    return Wrapper;
};

describe("use-dashboard hooks", () => {
    beforeEach(() => {
        vi.mocked(api.apiFetch).mockReset();
    });

    describe("useDashboardStats", () => {
        it("fetches dashboard stats correctly", async () => {
            const mockStats = {
                total_employees: 100,
                present_today: 95,
                pending_leave_requests: 2,
                estimated_monthly_payroll: "100000.00",
                late_today: 5,
                absent_today: 3,
                on_leave_today: 2,
                early_departure_today: 1,
                avg_work_hours_today: 8.5
            };

            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve(mockStats),
            } as unknown as never);

            const { result } = renderHook(() => useDashboardStats(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isLoading).toBe(false); });

            expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/workspace-stats");
            expect(result.current.data).toEqual(mockStats);
        });

        it("handles error when fetch fails", async () => {
            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: false,
                status: 503,
            } as unknown as never);

            const { result } = renderHook(() => useDashboardStats(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isError).toBe(true); });
            expect(result.current.error).toBeInstanceOf(DashboardQueryError);
            expect(result.current.error).toMatchObject({
                code: "HTTP_ERROR",
                endpoint: "/v1/dashboard/workspace-stats",
                queryKey: "workspace-stats",
                status: 503,
            });
        });
    });

    describe("useAttendanceTrend", () => {
        it("fetches attendance trend correctly", async () => {
            const mockData = [{ date: "2023-01-01", present: 10, late: 1, absent: 0, on_leave: 0 }];

            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve(mockData),
            } as unknown as never);

            const { result } = renderHook(() => useAttendanceTrend(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isLoading).toBe(false); });

            expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/attendance-trend");
            expect(result.current.data).toEqual(mockData);
        });
    });

    describe("useDepartmentAttendance", () => {
        it("fetches department attendance correctly with range", async () => {
            const mockData = [{ department_id: "dept-1", department_name: "Engineering", on_time: 10, late: 0, absent: 0, on_leave: 0 }];

            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve(mockData),
            } as unknown as never);

            const { result } = renderHook(() => useDepartmentAttendance("week"), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isLoading).toBe(false); });

            expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/department-attendance?range=week");
            expect(result.current.data).toEqual(mockData);
        });
    });

    describe("useUnexcusedAbsences", () => {
        it("builds query params correctly and fetches unexcused absences", async () => {
            const mockData = [{ department_id: "dept-1", department_name: "HR", unexcused_count: 1, consecutive_count: 1, total_absent_count: 1, total_leave_count: 0 }];

            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve(mockData),
            } as unknown as never);

            const { result } = renderHook(() => useUnexcusedAbsences("2023-01-01", "2023-01-31", "employee"), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isLoading).toBe(false); });

            expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/unexcused-absences?start_date=2023-01-01&end_date=2023-01-31&group_by=employee");
            expect(result.current.data).toEqual(mockData);
        });
        
        it("omits query parameters when not provided", async () => {
            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve([]),
            } as unknown as never);

            const { result } = renderHook(() => useUnexcusedAbsences(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isLoading).toBe(false); });

            expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/unexcused-absences");
        });
    });

    describe("useDashboardMetrics", () => {
        const departmentA = "11111111-1111-4111-8111-111111111111";
        const departmentB = "22222222-2222-4222-8222-222222222222";
        const locationA = "33333333-3333-4333-8333-333333333333";
        const locationB = "44444444-4444-4444-8444-444444444444";

        const filters: DashboardMetricsFilters = {
            businessUnitIds: [],
            comparison: "previous_period",
            departmentIds: [departmentB, departmentA, departmentA],
            employeeStatuses: [],
            employmentTypes: [],
            endDate: "2026-08-13",
            locationIds: [locationB, locationA],
            managerIds: [],
            period: "custom",
            riskThreshold: "watch",
            shiftIds: [],
            startDate: "2026-08-01",
            topN: 10,
        };

        const renderMetrics = (metricsFilters: DashboardMetricsFilters) => {
            const queryClient = new QueryClient({
                defaultOptions: { queries: { retry: false } },
            });
            const Wrapper = ({ children }: { children: React.ReactNode }) => (
                <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
            );
            Wrapper.displayName = "QueryClientWrapper";
            return { queryClient, result: renderHook(() => useDashboardMetrics(metricsFilters), { wrapper: Wrapper }) };
        };

        it("uses canonical serialized query as stable cache key", async () => {
            vi.mocked(api.apiFetch).mockResolvedValue({ ok: false, status: 503 } as unknown as never);

            const first = renderMetrics(filters);
            await waitFor(() => { expect(api.apiFetch).toHaveBeenCalledTimes(1); });

            const second = renderMetrics({
                ...filters,
                departmentIds: [departmentA, departmentB],
                locationIds: [locationA, locationB],
            });
            await waitFor(() => { expect(api.apiFetch).toHaveBeenCalledTimes(2); });

            const expectedEndpoint = "/v1/dashboard/executive-overview?period=custom&comparison=previous_period&start_date=2026-08-01&end_date=2026-08-13&department_ids=11111111-1111-4111-8111-111111111111&department_ids=22222222-2222-4222-8222-222222222222&location_ids=33333333-3333-4333-8333-333333333333&location_ids=44444444-4444-4444-8444-444444444444";
            expect(first.queryClient.getQueryCache().getAll()[0]?.queryKey).toEqual([
                "executive-overview",
                "ws-test-123",
                expectedEndpoint,
            ]);
            expect(second.queryClient.getQueryCache().getAll()[0]?.queryKey).toEqual([
                "executive-overview",
                "ws-test-123",
                expectedEndpoint,
            ]);
        });

        it("requests every canonical filter parameter", async () => {
            vi.mocked(api.apiFetch).mockResolvedValue({ ok: false, status: 503 } as unknown as never);

            renderMetrics(filters);
            await waitFor(() => {
                expect(api.apiFetch).toHaveBeenCalledWith(
                    "/v1/dashboard/executive-overview?period=custom&comparison=previous_period&start_date=2026-08-01&end_date=2026-08-13&department_ids=11111111-1111-4111-8111-111111111111&department_ids=22222222-2222-4222-8222-222222222222&location_ids=33333333-3333-4333-8333-333333333333&location_ids=44444444-4444-4444-8444-444444444444",
                );
            });
        });
    });

    describe("useTurnoverAnalytics", () => {
        it("does not fetch if startDate or endDate is missing", () => {
            const { result } = renderHook(() => useTurnoverAnalytics("", "2023-01-31"), {
                wrapper: createWrapper(),
            });

            expect(result.current.fetchStatus).toBe("idle");
            expect(api.apiFetch).not.toHaveBeenCalled();
        });

        it("fetches turnover analytics correctly", async () => {
            const mockData = {
                tenure_distribution: [],
                by_source: [],
                by_department: [],
                total_terminated: 5,
                period_start: "2023-01-01",
                period_end: "2023-01-31"
            };

            vi.mocked(api.apiFetch).mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve(mockData),
            } as unknown as never);

            const { result } = renderHook(() => useTurnoverAnalytics("2023-01-01", "2023-01-31"), {
                wrapper: createWrapper(),
            });

            await waitFor(() => { expect(result.current.isLoading).toBe(false); });

            expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/turnover-analytics?start=2023-01-01&end=2023-01-31");
            expect(result.current.data).toEqual(mockData);
        });
    });
});
