import { UseMutateFunction } from "@tanstack/react-query";
import { useState, useEffect } from "react";

import {
    DailyAttendanceReport,
    AttendanceAdjustmentRequest,
} from "@/lib/api";

import { useDailyDrawerMutations } from "./use-daily-drawer-mutations";
import { useDailyDrawerQueries } from "./use-daily-drawer-queries";

interface UseEmployeeDailyDrawerProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    isOpen: boolean;
    workspaceId: string;
    workUnitMinutes: number;
}

function useEmployeeDailyDrawerState(isOpen: boolean) {
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

    return {
        viewMode, setViewMode,
        expandedDebugRows, setExpandedDebugRows,
        reprocessingRow, setReprocessingRow,
        confirmingRow, setConfirmingRow,
        confirmDialog, setConfirmDialog,
        confirmModalData, setConfirmModalData,
        resolveModalData, setResolveModalData,
    };
}



type ConfirmDialogType = {
    show: boolean;
    employeeId: string;
    employeeName: string;
    date: string;
} | null;

type ConfirmModalDataType = {
    row: DailyAttendanceReport;
    existingRequest?: AttendanceAdjustmentRequest;
} | null;

type ResolveModalDataType = {
    row: DailyAttendanceReport;
} | null;

interface DrawerHandlersProps {
    approveOtMutation: { mutate: UseMutateFunction<any, Error, { row: DailyAttendanceReport; action: "approved" | "rejected" }> };
    confirmDialog: ConfirmDialogType;
    confirmModalData: ConfirmModalDataType;
    confirmMutation: { mutate: UseMutateFunction<any, Error, { row: DailyAttendanceReport; reason?: string; existingRequestId?: string }> };
    reprocessMutation: { mutate: UseMutateFunction<any, Error, { empId: string; date: string }> };
    resolveModalData: ResolveModalDataType;
    resolveUnscheduledMutation: { mutate: UseMutateFunction<any, Error, { row: DailyAttendanceReport; shiftId: string }> };
    setConfirmDialog: (data: ConfirmDialogType) => void;
    setConfirmingRow: (data: string | null) => void;
    setConfirmModalData: (data: ConfirmModalDataType) => void;
    setReprocessingRow: (data: string | null) => void;
    setResolveModalData: (data: ResolveModalDataType) => void;
}

function useEmployeeDailyDrawerHandlers({
    approveOtMutation,
    resolveUnscheduledMutation,
    reprocessMutation,
    confirmMutation,
    setResolveModalData,
    setConfirmDialog,
    setReprocessingRow,
    setConfirmModalData,
    setConfirmingRow,
    confirmDialog,
    confirmModalData,
    resolveModalData
}: DrawerHandlersProps) {
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
        handleApproveOt,
        handleResolveUnscheduled,
        handleResolveSubmit,
        handleReprocessClick,
        handleConfirmReprocess,
        handleConfirmClick,
        handleConfirmSubmit,
    };
}

export function useEmployeeDailyDrawer({
    isOpen,
    workspaceId,
    employeeId,
    dateRange,
    workUnitMinutes,
}: UseEmployeeDailyDrawerProps) {
    const {
        viewMode, setViewMode,
        expandedDebugRows, setExpandedDebugRows,
        reprocessingRow, setReprocessingRow,
        confirmingRow, setConfirmingRow,
        confirmDialog, setConfirmDialog,
        confirmModalData, setConfirmModalData,
        resolveModalData, setResolveModalData,
    } = useEmployeeDailyDrawerState(isOpen);

    const {
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
    } = useDailyDrawerQueries({
        workspaceId,
        employeeId,
        dateRange,
        isOpen,
        viewMode,
        workUnitMinutes,
    });

    const toggleExpandRow = (rowKey: string) => {
        const next = new Set(expandedDebugRows);
        if (next.has(rowKey)) next.delete(rowKey);
        else next.add(rowKey);
        setExpandedDebugRows(next);
    };

    const mutations = useDailyDrawerMutations({
        workspaceId,
        workUnitMinutes,
        onReprocessSuccess: () => {
            setReprocessingRow(null);
            setConfirmDialog(null);
        },
        onConfirmSuccess: () => {
            setConfirmingRow(null);
            setConfirmModalData(null);
        },
        onResolveSuccess: () => {
            setResolveModalData(null);
        },
    });

    const handlers = useEmployeeDailyDrawerHandlers({
        ...mutations,
        setResolveModalData,
        setConfirmDialog,
        setReprocessingRow,
        setConfirmModalData,
        setConfirmingRow,
        confirmDialog,
        confirmModalData,
        resolveModalData
    });

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
        ...handlers,
        reprocessMutationPending: mutations.reprocessMutation.isPending,
        confirmMutationPending: mutations.confirmMutation.isPending,
        resolveUnscheduledMutationPending: mutations.resolveUnscheduledMutation.isPending,
    };
}
