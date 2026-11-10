import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import {
    fetchAttendanceReport,
    DailyAttendanceReport,
    fetchAttendanceAdjustments,
    AttendanceAdjustmentRequest,
    fetchRawAttendanceLogs,
    fetchShifts,
} from "@/lib/api";
import { formatHours } from "@/lib/utils/attendance";
import { maybeNull } from "@/types/api-overrides";

interface UseDailyDrawerQueriesProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    employerId?: string;
    isOpen: boolean;
    viewMode: "daily" | "raw";
}

interface DrawerDayTotals {
    computed_ot_hours?: number;
    computed_std_hours?: number;
}

export function useDailyDrawerQueries({employeeId,
    employerId,
    dateRange,
    isOpen,
    viewMode,
}: UseDailyDrawerQueriesProps) {
    const { data: reportResponse, isLoading } = useQuery({
        queryKey: ["attendance-report",dateRange, "daily", 1, 100, undefined, [employeeId], employerId],
        queryFn: () =>
            fetchAttendanceReport({
                start: dateRange.start,
                end: dateRange.end,
                type: "daily",
                page: 1,
                limit: 100,
                employeeIds: [employeeId],
                employerId,
            }),
        enabled: isOpen  && !!employeeId,
    });

    const { data: pendingAdjustmentsResponse } = useQuery({ queryKey: ["attendance-adjustments", "pending"], queryFn: () => fetchAttendanceAdjustments({ status: "pending" }),
    enabled: isOpen , });

    const { data: shifts } = useQuery({ queryKey: ["shifts"], queryFn: () => fetchShifts(),
    enabled: isOpen , });

    const {
        data: rawLogsResponse,
        isLoading: rawLogsLoading,
        fetchNextPage,
        hasNextPage,
        isFetchingNextPage
    } = useInfiniteQuery({
        queryKey: ["raw-logs",dateRange, [employeeId]],
        queryFn: ({ pageParam }) => {
            const from = dateRange.start ? new Date(`${dateRange.start}T00:00:00`).toISOString() : undefined;
            const to = dateRange.end ? new Date(`${dateRange.end}T23:59:59`).toISOString() : undefined;
            return fetchRawAttendanceLogs({ employee_id: employeeId, from, to, page: pageParam, limit: 100 });
        },
        getNextPageParam: (lastPage, allPages) => {
            if (lastPage.data.length < 100) return undefined;
            return allPages.length + 1;
        },
        initialPageParam: 1,
        enabled: isOpen  && !!employeeId && viewMode === "raw",
    });

    const rawLogsData = useMemo(() => {
        if (!rawLogsResponse) return [];
        return rawLogsResponse.pages.flatMap((p) => p.data);
    }, [rawLogsResponse]);

    const adjustmentMap = useMemo(() => {
        const map: Record<string, AttendanceAdjustmentRequest> = {};
        const adjustments = pendingAdjustmentsResponse?.data ?? [];
        adjustments.forEach((request) => {
            const key = `${request.employment_id}-${request.requested_date}`;
            map[key] = request;
            // Also index by employee_id if available (handles id mismatch between endpoints).
            // The API may include employee_id alongside employment_id even though the
            // generated type doesn't declare it yet.
            const empId = (request as { employee_id?: string }).employee_id;
            if (empId && empId !== request.employment_id) {
                const empKey = `${empId}-${request.requested_date}`;
                // Record indexed access without noUncheckedIndexedAccess —
                // type says Record<K,V> always returns V, but runtime can be undefined
                if (!maybeNull(map[empKey])) map[empKey] = request;
            }
        });
        return map;
    }, [pendingAdjustmentsResponse]);

    const reportData = useMemo(
        () => (reportResponse?.data ?? (Array.isArray(reportResponse) ? reportResponse : [])) as DailyAttendanceReport[],
        [reportResponse]
    );

    const { totalStdHours, totalOtHours } = useMemo(() => {
        const authoritativeTotals = new Map<string, DrawerDayTotals>();

        reportData.forEach((row: DailyAttendanceReport) => {
            const metadata = row.generation_metadata as { day_totals?: DrawerDayTotals } | undefined;
            const dayTotals = metadata?.day_totals;
            if (!dayTotals) return;

            const key = `${row.employment_id}-${row.date}`;
            // Every segment row for a day carries the same backend-calculated
            // total. Keep one copy so split segments cannot be counted twice.
            if (!authoritativeTotals.has(key)) authoritativeTotals.set(key, dayTotals);
        });

        const { stdHours, otHours } = authoritativeTotals.size > 0
            ? Array.from(authoritativeTotals.values()).reduce(
                (totals, dayTotals) => ({
                    stdHours: totals.stdHours + (dayTotals.computed_std_hours ?? 0),
                    otHours: totals.otHours + (dayTotals.computed_ot_hours ?? 0),
                }),
                { stdHours: 0, otHours: 0 },
            )
            : reportData.reduce(
                (totals, row: DailyAttendanceReport) => ({
                    stdHours: totals.stdHours + (row.computed_std_hours || 0),
                    otHours: totals.otHours + (row.computed_ot_hours || 0),
                }),
                { stdHours: 0, otHours: 0 },
            );

        return {
            totalStdHours: formatHours(stdHours),
            totalOtHours: formatHours(otHours),
        };
    }, [reportData]);

    return {
        isLoading,
        reportData,
        adjustmentMap,
        shifts,
        rawLogsData,
        rawLogsLoading,
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
        totalStdHours,
        totalOtHours,
    };
}
