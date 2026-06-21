import { useState } from "react";

import {
    DailyAttendanceReport,
    AttendanceAdjustmentRequest,
} from "@/lib/api";

import { useDailyDrawerMutations } from "./use-daily-drawer-mutations";
import { useDailyDrawerQueries } from "./use-daily-drawer-queries";
import { useDailyDrawerModalState, createIsoFromLocal } from "./use-daily-drawer-utils";

interface UseEmployeeDailyDrawerProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    employmentId?: string;
    isOpen: boolean;
    workspaceId: string;
}

export function useEmployeeDailyDrawer(props: UseEmployeeDailyDrawerProps) {
    const { isOpen, workspaceId, employeeId, dateRange, employmentId } = props;
    const [viewMode, setViewMode] = useState<"daily" | "raw">("daily");
    const [selectedEmploymentId, setSelectedEmploymentId] = useState<string | undefined>(employmentId);

    const [prevEmployeeId, setPrevEmployeeId] = useState<string | null>(null);
    const [prevIsOpen, setPrevIsOpen] = useState<boolean>(false);

    if (employeeId !== prevEmployeeId || isOpen !== prevIsOpen) {
        setPrevEmployeeId(employeeId);
        setPrevIsOpen(isOpen);
        if (isOpen) {
            setViewMode("daily");
            setSelectedEmploymentId(employmentId);
        }
    }

    const [expandedDebugRows, setExpandedDebugRows] = useState<Set<string>>(new Set());
    const [reprocessingRow, setReprocessingRow] = useState<string | null>(null);
    const [confirmingRow, setConfirmingRow] = useState<string | null>(null);
    
    const {
        confirmDialog, setConfirmDialog,
        confirmModalData, setConfirmModalData,
        resolveModalData, setResolveModalData
    } = useDailyDrawerModalState();

    const {
        isLoading, reportData, adjustmentMap, shifts, rawLogsData, rawLogsLoading,
        hasNextPage, isFetchingNextPage, fetchNextPage, totalStdHours, totalOtHours,
    } = useDailyDrawerQueries({
        workspaceId, employeeId, dateRange, isOpen, viewMode, employmentId: selectedEmploymentId,
    });

    const toggleExpandRow = (rowKey: string) => {
        const next = new Set(expandedDebugRows);
        if (next.has(rowKey)) next.delete(rowKey);
        else next.add(rowKey);
        setExpandedDebugRows(next);
    };

    // Mutations using the extracted hook
    const {
        reprocessMutation, confirmMutation, approveOtMutation, resolveUnscheduledMutation,
    } = useDailyDrawerMutations({
        workspaceId,
        employeeId,
        onReprocessSuccess: () => { setReprocessingRow(null); setConfirmDialog(null); },
        onConfirmSuccess: () => { setConfirmingRow(null); setConfirmModalData(null); },
        onResolveSuccess: () => { setResolveModalData(null); },
    });

    const handleApproveOt = (row: DailyAttendanceReport, action: "approved" | "rejected") => {
        approveOtMutation.mutate({ row, action });
    };

    const handleResolveUnscheduled = (row: DailyAttendanceReport) => {
        setResolveModalData({ row });
    };

    const handleResolveSubmit = (shiftId: string) => {
        if (!resolveModalData) return;
        resolveUnscheduledMutation.mutate({ row: resolveModalData.row, shiftId });
    };

    const handleReprocessClick = (row: DailyAttendanceReport) => {
        setConfirmDialog({
            show: true, employeeId, employeeName: row.employee_name, date: row.date,
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
        setConfirmingRow(`${employeeId}-${row.date}`);

        if (existingRequest) {
            confirmMutation.mutate({ row, existingRequestId: existingRequest.id });
        } else {
            const modifiedRow = {
                ...row,
                actual_punch_in: createIsoFromLocal(row.date, punchIn),
                actual_punch_out: createIsoFromLocal(row.date, punchOut),
            };
            confirmMutation.mutate({ row: modifiedRow, reason });
        }
    };

    return {
        selectedEmploymentId, setSelectedEmploymentId,
        viewMode, setViewMode, expandedDebugRows, toggleExpandRow, reprocessingRow, confirmingRow,
        confirmDialog, setConfirmDialog, confirmModalData, setConfirmModalData, resolveModalData, setResolveModalData,
        reportData, isLoading, rawLogsData, rawLogsLoading, hasNextPage, isFetchingNextPage, fetchNextPage,
        adjustmentMap, shifts, totalStdHours, totalOtHours, handleReprocessClick, handleConfirmReprocess,
        handleConfirmClick, handleConfirmSubmit, handleApproveOt, handleResolveUnscheduled, handleResolveSubmit,
        reprocessMutationPending: reprocessMutation.isPending,
        confirmMutationPending: confirmMutation.isPending,
        resolveUnscheduledMutationPending: resolveUnscheduledMutation.isPending,
    };
}
