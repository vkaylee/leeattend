import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";

import { useAttendanceTrend } from "../use-dashboard";

// Mock the API module
vi.mock("@/lib/api", async () => {
    const actual = await vi.importActual("@/lib/api");
    return {
        ...actual,
        apiFetch: vi.fn(),
    };
});

// Mock the workspace hook
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

const rate = (value: string | null) => ({ value, numerator: 1, denominator: 10, unit: "employee_days" });

const mockTrendData = [
    { date: "2026-02-01", present: 40, late: 3, absent: 5, on_leave: 2, eligible: 50, attendance_rate: rate("80"), absence_rate: rate("10"), late_rate: rate("6") },
    { date: "2026-02-02", present: 42, late: 1, absent: 4, on_leave: 3, eligible: 50, attendance_rate: rate("84"), absence_rate: rate("8"), late_rate: rate("2") },
    { date: "2026-02-03", present: 38, late: 5, absent: 6, on_leave: 1, eligible: 50, attendance_rate: rate("76"), absence_rate: rate("12"), late_rate: rate("10") },
];

describe("useAttendanceTrend hook", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("fetches attendance trend data successfully", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve(mockTrendData),
        } as unknown as never);

        const { result } = renderHook(() => useAttendanceTrend(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });

        expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/attendance-trend");
        expect(result.current.data).toEqual(mockTrendData);
        expect(result.current.data).toHaveLength(3);
    });

    it("returns correct data shape with all fields", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve(mockTrendData),
        } as unknown as never);

        const { result } = renderHook(() => useAttendanceTrend(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.data).toBeDefined(); });

        const firstPoint = result.current.data![0];
        expect(firstPoint).toHaveProperty("date");
        expect(firstPoint).toHaveProperty("present");
        expect(firstPoint).toHaveProperty("late");
        expect(firstPoint).toHaveProperty("absent");
        expect(firstPoint).toHaveProperty("on_leave");
    });

    it("handles API error gracefully", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValue({
            ok: false,
            status: 500,
        } as unknown as never);

        const { result } = renderHook(() => useAttendanceTrend(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isError).toBe(true); });
        expect(result.current.error).toBeDefined();
    });

    it("handles empty data response", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve([]),
        } as unknown as never);

        const { result } = renderHook(() => useAttendanceTrend(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });
        expect(result.current.data).toEqual([]);
    });
});
