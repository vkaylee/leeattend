import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useDailyDrawerQueries } from "../use-daily-drawer-queries";

// Mock i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({ t: (key: string) => key }),
}));

// Mock the API module — use vi.hoisted for vitest's mock hoisting
const {
    mockFetchAttendanceReport,
    mockFetchWorkspaceAttendanceAdjustments,
    mockFetchRawAttendanceLogs,
    mockFetchShifts,
} = vi.hoisted(() => ({
    mockFetchAttendanceReport: vi.fn(),
    mockFetchWorkspaceAttendanceAdjustments: vi.fn(),
    mockFetchRawAttendanceLogs: vi.fn(),
    mockFetchShifts: vi.fn(),
}));

vi.mock("@/lib/api", () => ({
    fetchAttendanceReport: mockFetchAttendanceReport,
    fetchAttendanceAdjustments: mockFetchWorkspaceAttendanceAdjustments,
    fetchRawAttendanceLogs: mockFetchRawAttendanceLogs,
    fetchShifts: mockFetchShifts,
}));

describe("useDailyDrawerQueries — Adjustment Map", () => {
    let queryClient: QueryClient;

    const defaultProps = { employeeId: "emp-1",
    dateRange: { start: "2024-03-01", end: "2024-03-31" },
    isOpen: true,
    viewMode: "daily" as const, };

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: {
                queries: {
                    retry: false,
                    gcTime: 0,
                    staleTime: 0,
                },
            },
        });

        mockFetchAttendanceReport.mockResolvedValue({
            data: [],
            total: 0,
        });
        mockFetchRawAttendanceLogs.mockResolvedValue({
            data: [],
            total: 0,
            page: 1,
            limit: 100,
        });
        mockFetchShifts.mockResolvedValue([]);
        mockFetchWorkspaceAttendanceAdjustments.mockResolvedValue({
            data: [],
            total: 0,
        });
    });

    const wrapper = ({ children }: { children: React.ReactNode }) => (
  // eslint-disable-next-line import-x/no-named-as-default-member
        React.createElement(QueryClientProvider, { client: queryClient }, children)
    );

    it("queries daily rows by employee and employer group", async () => {
        renderHook(
            () => useDailyDrawerQueries({ ...defaultProps, employerId: "employer-1" }),
            { wrapper }
        );

        await waitFor(() => {
            expect(mockFetchAttendanceReport).toHaveBeenCalled();
        });

        expect(mockFetchAttendanceReport).toHaveBeenCalledWith(expect.objectContaining({
            employeeIds: ["emp-1"],
            employerId: "employer-1",
        }));
        expect(mockFetchAttendanceReport.mock.calls[0][0]).not.toHaveProperty("employmentId");
    });

    it("combines totals from adjacent temporal employment rows", async () => {
        mockFetchAttendanceReport.mockResolvedValue({
            data: [
                { computed_std_hours: 8, computed_ot_hours: 1, employment_id: "old-employment" },
                { computed_std_hours: 7.5, computed_ot_hours: 0.5, employment_id: "new-employment" },
            ],
            total: 1,
        });

        const { result } = renderHook(
            () => useDailyDrawerQueries({ ...defaultProps, employerId: "employer-1" }),
            { wrapper }
        );

        await waitFor(() => {
            expect(result.current.reportData).toHaveLength(2);
        });

        expect(result.current.reportData.map(row => row.employment_id)).toEqual([
            "old-employment",
            "new-employment",
        ]);
        expect(result.current.totalStdHours).toBe("15.5");
        expect(result.current.totalOtHours).toBe("1.5");
    });

    it("indexes adjustments by employment_id", async () => {
        mockFetchWorkspaceAttendanceAdjustments.mockResolvedValue({
            data: [
                {
                    id: "adj-1",
                    employment_id: "emp-001",
                    requested_date: "2024-03-20",
                },
            ],
            total: 1,
        });

        const { result } = renderHook(
            () => useDailyDrawerQueries(defaultProps),
            { wrapper }
        );

        // Wait for the query to settle
        await waitFor(() => {
            expect(result.current.adjustmentMap).toBeDefined();
            expect(Object.keys(result.current.adjustmentMap).length).toBeGreaterThan(0);
        });

        // Should be indexed by employment_id-date
        expect(result.current.adjustmentMap["emp-001-2024-03-20"]).toBeDefined();
        expect(result.current.adjustmentMap["emp-001-2024-03-20"].id).toBe("adj-1");
    });

    it("double-indexes adjustments by employee_id when it differs from employment_id", async () => {
        mockFetchWorkspaceAttendanceAdjustments.mockResolvedValue({
            data: [
                {
                    id: "adj-2",
                    employment_id: "emp-001",
                    employee_id: "user-99",
                    requested_date: "2024-03-20",
                },
            ],
            total: 1,
        });

        const { result } = renderHook(
            () => useDailyDrawerQueries(defaultProps),
            { wrapper }
        );

        await waitFor(() => {
            expect(result.current.adjustmentMap).toBeDefined();
            expect(result.current.adjustmentMap["emp-001-2024-03-20"]).toBeDefined();
        });

        // Should be indexed by both keys
        expect(result.current.adjustmentMap["user-99-2024-03-20"]).toBeDefined();
        expect(result.current.adjustmentMap["user-99-2024-03-20"].id).toBe("adj-2");
    });

    it("does not overwrite existing entry when employee_id matches employment_id", async () => {
        mockFetchWorkspaceAttendanceAdjustments.mockResolvedValue({
            data: [
                {
                    id: "adj-3",
                    employment_id: "emp-001",
                    employee_id: "emp-001",  // same as employment_id
                    requested_date: "2024-03-20",
                },
            ],
            total: 1,
        });

        const { result } = renderHook(
            () => useDailyDrawerQueries(defaultProps),
            { wrapper }
        );

        await waitFor(() => {
            expect(result.current.adjustmentMap).toBeDefined();
            expect(Object.keys(result.current.adjustmentMap).length).toBe(1);
        });

        // Should only have one entry (not double-indexed since ids are same)
        expect(result.current.adjustmentMap["emp-001-2024-03-20"]).toBeDefined();
    });

    it("returns empty map when no adjustments available", async () => {
        mockFetchWorkspaceAttendanceAdjustments.mockResolvedValue({
            data: [],
            total: 0,
        });

        const { result } = renderHook(
            () => useDailyDrawerQueries(defaultProps),
            { wrapper }
        );

        await waitFor(() => {
            expect(result.current.adjustmentMap).toBeDefined();
        });

        expect(Object.keys(result.current.adjustmentMap)).toHaveLength(0);
    });

    it("does not overwrite existing entry with employee_id key if map already has it", async () => {
        mockFetchWorkspaceAttendanceAdjustments.mockResolvedValue({
            data: [
                {
                    id: "adj-4",
                    employment_id: "emp-001",
                    employee_id: "user-88",
                    requested_date: "2024-03-20",
                },
                {
                    id: "adj-5",
                    employment_id: "user-88",  // this would conflict with prev's employee_id
                    employee_id: "emp-001",
                    requested_date: "2024-03-20",
                },
            ],
            total: 2,
        });

        const { result } = renderHook(
            () => useDailyDrawerQueries(defaultProps),
            { wrapper }
        );

        await waitFor(() => {
            expect(result.current.adjustmentMap).toBeDefined();
            expect(result.current.adjustmentMap["emp-001-2024-03-20"]).toBeDefined();
        });

        // The first adjustment indexed by emp-001 should remain (not overwritten)
        expect(result.current.adjustmentMap["emp-001-2024-03-20"].id).toBe("adj-4");
        // user-88 key from adj-4 also exists
        expect(result.current.adjustmentMap["user-88-2024-03-20"]).toBeDefined();
        expect(result.current.adjustmentMap["user-88-2024-03-20"].id).toBe("adj-5");
    });
});
