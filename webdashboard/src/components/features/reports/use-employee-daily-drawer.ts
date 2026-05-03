import { useQuery, useMutation, useQueryClient, useInfiniteQuery } from "@tanstack/react-query";
import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
    fetchAttendanceReport,
    DailyAttendanceReport,
    reprocessAttendance,
    fetchWorkspaceAttendanceAdjustments,
    adminConfirmAttendance,
    AttendanceAdjustmentRequest,
    approveOt,
    fetchRawAttendanceLogs,
    fetchShifts,
    upsertRoster,
} from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";

interface UseEmployeeDailyDrawerProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    isOpen: boolean;
    workspaceId: string;
    workUnitMinutes: number;
}

export function useEmployeeDailyDrawer({
    isOpen,
    workspaceId,
    employeeId,
    dateRange,
    workUnitMinutes,
}: UseEmployeeDailyDrawerProps) {
    const { t } = useTranslation();
    const queryClient = useQueryClient();
    const toast = useToast();

    const [viewMode, setViewMode] = useState<"daily" | "raw">("daily");

    // Reset to daily tab whenever the drawer is (re-)opened
    // eslint-disable-next-line react-hooks/set-state-in-effect -- intentional
    useEffect(() => { if (isOpen) setViewMode("daily"); }, [isOpen]);

    const [expandedDebugRows, setExpandedDebugRows] = useState<Set<string>>(new Set());
    const [reprocessingRow, setReprocessingRow] = useState<string | null>(null);
    const [confirmingRow, setConfirmingRow] = useState<string | null>(null);
    const [confirmDialog, setConfirmDialog] = useState<{
        show: boolean;
        employeeId: string;
        employeeName: string;
        date: string;
    } | null>(null);
    const [confirmModalData, setConfirmModalData] = useState<{
        row: DailyAttendanceReport;
        existingRequest?: AttendanceAdjustmentRequest;
    } | null>(null);
    const [resolveModalData, setResolveModalData] = useState<{
        row: DailyAttendanceReport;
    } | null>(null);

    // Fetch daily data for this single employee
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

    // Fetch shift types for the unscheduled resolution modal
    const { data: shifts } = useQuery({
        queryKey: ["shifts", workspaceId],
        queryFn: () => fetchShifts(workspaceId),
        enabled: isOpen && !!workspaceId,
    });

    // Fetch raw logs for this single employee with infinite scroll
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

    // Extract flat array of logs
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

    // Compute total hours for summary bar
    const { totalStdHours, totalOtHours } = useMemo(() => {
        const stdUnits = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.std_work_units || 0), 0);
        const otUnits = reportData.reduce((sum: number, r: DailyAttendanceReport) => sum + (r.ot_work_units || 0), 0);
        return {
            totalStdHours: formatUnitsToHours(stdUnits, workUnitMinutes),
            totalOtHours: formatUnitsToHours(otUnits, workUnitMinutes),
        };
    }, [reportData, workUnitMinutes]);

    const toggleExpandRow = (rowKey: string) => {
        const next = new Set(expandedDebugRows);
        if (next.has(rowKey)) next.delete(rowKey);
        else next.add(rowKey);
        setExpandedDebugRows(next);
    };

    // Reprocess mutation
    const reprocessMutation = useMutation({
        mutationFn: ({ empId, date }: { empId: string; date: string }) =>
            reprocessAttendance(workspaceId, empId, date, date),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            setReprocessingRow(null);
            setConfirmDialog(null);
            toast.success(t("common.success"));
        },
    });

    // Confirm mutation
    const confirmMutation = useMutation({
        mutationFn: ({
            row,
            existingRequestId,
            reason,
        }: {
            row: DailyAttendanceReport;
            existingRequestId?: string;
            reason?: string;
        }) => adminConfirmAttendance(workspaceId, row, existingRequestId, reason),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            void queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
            setConfirmingRow(null);
            setConfirmModalData(null);
            toast.success(t("reports.confirm.success"));
        },
        onError: () => {
            setConfirmingRow(null);
            toast.error(t("reports.confirm.error"));
        },
    });

    // OT approve mutation
    const approveOtMutation = useMutation({
        mutationFn: ({ row, action }: { row: DailyAttendanceReport; action: "approved" | "rejected" }) =>
            approveOt(workspaceId, row.employee_id, {
                date: row.date,
                approved_minutes: row.ot_work_units * (workUnitMinutes || 60),
                status: action,
            }),
        onSuccess: (_data, variables) => {
            void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            const message =
                variables.action === "approved"
                    ? t("reports.ot_approve.success_approved")
                    : t("reports.ot_approve.success_rejected");
            toast.success(message);
        },
        onError: () => {
            toast.error(t("reports.ot_approve.error"));
        },
    });

    // Resolve unscheduled mutation: upsert roster + reprocess
    const resolveUnscheduledMutation = useMutation({
        mutationFn: async ({ row, shiftId }: { row: DailyAttendanceReport; shiftId: string }) => {
            // Step 1: Assign the shift to this day
            await upsertRoster(workspaceId, row.employee_id, {
                date: row.date,
                shift_id: shiftId,
            });
            // Step 2: Reprocess to recalculate attendance with the new shift
            await reprocessAttendance(workspaceId, row.employee_id, row.date, row.date);
        },
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
            setResolveModalData(null);
            toast.success(t("reports.unscheduled.success"));
        },
        onError: () => {
            toast.error(t("reports.unscheduled.error"));
        },
    });

    const handleApproveOt = (row: DailyAttendanceReport, action: "approved" | "rejected") => {
        approveOtMutation.mutate({ row, action });
    };

    const handleResolveUnscheduled = (row: DailyAttendanceReport) => {
        setResolveModalData({ row });
    };

    const handleResolveSubmit = (shiftId: string) => {
        if (!resolveModalData) return;
        resolveUnscheduledMutation.mutate({
            row: resolveModalData.row,
            shiftId,
        });
    };

    const handleReprocessClick = (row: DailyAttendanceReport) => {
        setConfirmDialog({
            show: true,
            employeeId: row.employee_id,
            employeeName: row.employee_name,
            date: row.date,
        });
    };

    const handleConfirmReprocess = () => {
        if (!confirmDialog) return;
        setReprocessingRow(`${confirmDialog.employeeId}-${confirmDialog.date}`);
        reprocessMutation.mutate({ empId: confirmDialog.employeeId, date: confirmDialog.date });
    };

    const handleConfirmClick = (row: DailyAttendanceReport, existing?: AttendanceAdjustmentRequest) => {
        setConfirmModalData({ row, existingRequest: existing });
    };

    const handleConfirmSubmit = ({ punchIn, punchOut, reason }: { punchIn: string; punchOut: string; reason: string }) => {
        if (!confirmModalData) return;
        const { row, existingRequest } = confirmModalData;
        const rowKey = `${row.employee_id}-${row.date}`;
        setConfirmingRow(rowKey);

        if (existingRequest) {
            confirmMutation.mutate({ row, existingRequestId: existingRequest.id });
        } else {
            const createIsoFromLocal = (dateString: string, hhmm: string): string | null => {
                if (!hhmm) return null;
                const [y, m, d] = dateString.split("-").map(Number);
                const [hours, mins] = hhmm.split(":").map(Number);
                return new Date(y, m - 1, d, hours, mins, 0).toISOString();
            };

            const modifiedRow = {
                ...row,
                actual_punch_in: createIsoFromLocal(row.date, punchIn),
                actual_punch_out: createIsoFromLocal(row.date, punchOut),
            };
            confirmMutation.mutate({ row: modifiedRow, reason });
        }
    };

    return {
        viewMode,
        setViewMode,
        expandedDebugRows,
        toggleExpandRow,
        reprocessingRow,
        confirmingRow,
        confirmDialog,
        setConfirmDialog,
        confirmModalData,
        setConfirmModalData,
        resolveModalData,
        setResolveModalData,
        reportData,
        isLoading,
        rawLogsData,
        rawLogsLoading,
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
        adjustmentMap,
        shifts,
        totalStdHours,
        totalOtHours,
        handleReprocessClick,
        handleConfirmReprocess,
        handleConfirmClick,
        handleConfirmSubmit,
        handleApproveOt,
        handleResolveUnscheduled,
        handleResolveSubmit,
        reprocessMutationPending: reprocessMutation.isPending,
        confirmMutationPending: confirmMutation.isPending,
        resolveUnscheduledMutationPending: resolveUnscheduledMutation.isPending,
    };
}
