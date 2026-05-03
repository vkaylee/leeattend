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
import { formatUnitsToHours } from "@/lib/utils/attendance";

interface UseDailyDrawerQueriesProps {
    workspaceId: string;
    employeeId: string;
    dateRange: { start: string; end: string };
    isOpen: boolean;
    viewMode: "daily" | "raw";
    workUnitMinutes: number;
}

export function useDailyDrawerQueries({
    workspaceId,
    employeeId,
    dateRange,
    isOpen,
    viewMode,
    workUnitMinutes,
}: UseDailyDrawerQueriesProps) {
    const { data: reportResponse, isLoading } = useQuery({
        queryKey: ["attendance-report", workspaceId, dateRange, "daily", 1, 100, undefined, [employeeId]],
        queryFn: () =>
            fetchAttendanceReport({
                wsId: workspaceId,
                start: dateRange.start,
                end: dateRange.end,
                type: "daily",
                page: 1,
                limit: 100,
                employeeIds: [employeeId],
            }),
        enabled: isOpen && !!workspaceId && !!employeeId,
    });

    const { data: pendingAdjustments } = useQuery({
        queryKey: ["attendance-adjustments", workspaceId, "pending"],
        queryFn: () => fetchWorkspaceAttendanceAdjustments(workspaceId, "pending"),
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
        pendingAdjustments?.forEach((request) => {
            const key = `${request.employee_id}-${request.requested_date}`;
            map[key] = request;
        });
        return map;
    }, [pendingAdjustments]);

    const reportData = useMemo(
        () => (reportResponse?.data ?? (Array.isArray(reportResponse) ? reportResponse : [])) as DailyAttendanceReport[],
        [reportResponse]
    );

    const { totalStdHours, totalOtHours } = useMemo(() => {
        const stdUnits = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.std_work_units || 0), 0);
        const otUnits = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.ot_work_units || 0), 0);
        return {
            totalStdHours: formatUnitsToHours(stdUnits, workUnitMinutes),
            totalOtHours: formatUnitsToHours(otUnits, workUnitMinutes),
        };
    }, [reportData, workUnitMinutes]);

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
