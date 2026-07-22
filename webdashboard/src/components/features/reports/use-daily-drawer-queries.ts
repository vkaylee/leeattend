import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import {
    fetchAttendanceReport,
    DailyAttendanceReport,
    fetchWorkspaceAttendanceAdjustments,
    AttendanceAdjustmentRequest,
    fetchRawAttendanceLogs,
    fetchShifts,
} from "@/lib/api";
import { formatHours } from "@/lib/utils/attendance";

interface UseDailyDrawerQueriesProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    employerId?: string;
    isOpen: boolean;
    viewMode: "daily" | "raw";
    workspaceId: string;
}

export function useDailyDrawerQueries({
    workspaceId,
    employeeId,
    employerId,
    dateRange,
    isOpen,
    viewMode,
}: UseDailyDrawerQueriesProps) {
    const { data: reportResponse, isLoading } = useQuery({
        queryKey: ["attendance-report", workspaceId, dateRange, "daily", 1, 100, undefined, [employeeId], employerId],
        queryFn: () =>
            fetchAttendanceReport({
                wsId: workspaceId,
                start: dateRange.start,
                end: dateRange.end,
                type: "daily",
                page: 1,
                limit: 100,
                employeeIds: [employeeId],
                employerId,
            }),
        enabled: isOpen && !!workspaceId && !!employeeId,
    });

    const { data: pendingAdjustmentsResponse } = useQuery({
        queryKey: ["attendance-adjustments", workspaceId, "pending"],
        queryFn: () => fetchWorkspaceAttendanceAdjustments(workspaceId, { status: "pending" }),
        enabled: isOpen && !!workspaceId,
    });

    const { data: shifts } = useQuery({
        queryKey: ["shifts", workspaceId],
        queryFn: () => fetchShifts(workspaceId),
        enabled: isOpen && !!workspaceId,
    });

    const {
        data: rawLogsResponse,
        isLoading: rawLogsLoading,
        fetchNextPage,
        hasNextPage,
        isFetchingNextPage
    } = useInfiniteQuery({
        queryKey: ["raw-logs", workspaceId, dateRange, [employeeId]],
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
        enabled: isOpen && !!workspaceId && !!employeeId && viewMode === "raw",
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
                // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- Record indexed access without noUncheckedIndexedAccess
                if (!map[empKey]) map[empKey] = request;
            }
        });
        return map;
    }, [pendingAdjustmentsResponse]);

    const reportData = useMemo(
        () => (reportResponse?.data ?? (Array.isArray(reportResponse) ? reportResponse : [])) as DailyAttendanceReport[],
        [reportResponse]
    );

    const { totalStdHours, totalOtHours } = useMemo(() => {
        const stdHours = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.computed_std_hours || 0), 0);
        const otHours = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.computed_ot_hours || 0), 0);
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
