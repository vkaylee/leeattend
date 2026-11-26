import { useRef, useState } from "react";

import { useOrganization } from "@/hooks/use-organization";
import {
    DailyAttendanceReport,
    AttendanceAdjustmentRequest,
} from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";
import { generateUUID } from "@/utils/uuid";

import { useDailyDrawerMutations } from "./use-daily-drawer-mutations";
import { useDailyDrawerQueries } from "./use-daily-drawer-queries";
import { useDailyDrawerModalState, createIsoFromLocal } from "./use-daily-drawer-utils";

interface UseEmployeeDailyDrawerProps {
    dateRange: { start: string; end: string };
    employeeId: string;
    employerId?: string;
    isOpen: boolean;
}

export function useEmployeeDailyDrawer(props: UseEmployeeDailyDrawerProps) {
    const { isOpen,employeeId, dateRange, employerId } = props;
    const [viewMode, setViewMode] = useState<"daily" | "raw">("daily");
    const [selectedEmployerId, setSelectedEmployerId] = useState<string | undefined>(employerId);
    const { organization } = useOrganization();
    const timezone = organization?.timezone ?? "UTC";

    const [previousSelection, setPreviousSelection] = useState<{
        employeeId: string;
        employerId?: string;
        isOpen: boolean;
    } | null>(null);

    const {
        confirmDialog, setConfirmDialog,
        confirmModalData, setConfirmModalData,
        resolveModalData, setResolveModalData
    } = useDailyDrawerModalState();

    if (
        employeeId !== maybeNull(previousSelection)?.employeeId
        || employerId !== maybeNull(previousSelection)?.employerId
        || isOpen !== maybeNull(previousSelection)?.isOpen
    ) {
        setPreviousSelection({ employeeId, employerId, isOpen });
        setResolveModalData(null);
        setConfirmModalData(null);
        setConfirmDialog(null);
        if (isOpen) {
            setViewMode("daily");
            setSelectedEmployerId(employerId);
        }
    }

    const [expandedDebugRows, setExpandedDebugRows] = useState<Set<string>>(new Set());
    const [reprocessingRow, setReprocessingRow] = useState<string | null>(null);
    const [confirmingRow, setConfirmingRow] = useState<string | null>(null);

    // In-flight guard: prevents double-clicking the modal's "Confirm Reprocess"
    // button from firing concurrent mutate calls before React flips isPending.
    // Without this the backend could receive overlapping reprocess requests for the
    // same (employee, day) and write 4 work sessions instead of 2.
    const reprocessInFlight = useRef(false);
    const resolveUnscheduledInFlight = useRef(false);
    const resolveAttemptKeyRef = useRef<{ fingerprint: string; key: string } | null>(null);

    const {
        isLoading, reportData, adjustmentMap, shifts, rawLogsData, rawLogsLoading,
        hasNextPage, isFetchingNextPage, fetchNextPage, totalStdHours, totalOtHours,
    } = useDailyDrawerQueries({employeeId, dateRange, isOpen, viewMode, employerId: selectedEmployerId,
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
    } = useDailyDrawerMutations({employeeId,
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

    const handleResolveSubmit = (shiftId: string, reason: string) => {
        if (!resolveModalData) return;
        if (resolveUnscheduledInFlight.current) return; // debounce lead-edge
        resolveUnscheduledInFlight.current = true;
        const fingerprint = `${resolveModalData.row.employment_id ?? employeeId}:${resolveModalData.row.date}:${shiftId}:${reason}`;
        if (resolveAttemptKeyRef.current?.fingerprint !== fingerprint) {
            resolveAttemptKeyRef.current = { fingerprint, key: generateUUID() };
        }
        const idempotencyKey = resolveAttemptKeyRef.current.key;
        resolveUnscheduledMutation.mutate(
            { idempotencyKey, reason, row: resolveModalData.row, shiftId },
            {
                onSuccess: () => {
                    resolveAttemptKeyRef.current = null;
                },
                onSettled: () => {
                    resolveUnscheduledInFlight.current = false;
                },
            },
        );
    };

    const handleReprocessClick = (row: DailyAttendanceReport) => {
        setConfirmDialog({
            show: true, employeeId, employeeName: row.employee_name, date: row.date,
        });
    };

    const handleConfirmReprocess = () => {
        if (!confirmDialog) return;
        if (reprocessInFlight.current) return; // debounce lead-edge
        reprocessInFlight.current = true;
        setReprocessingRow(`${confirmDialog.employeeId}-${confirmDialog.date}`);
        reprocessMutation.mutate(
            { empId: confirmDialog.employeeId, date: confirmDialog.date },
            {
                onSettled: () => {
                    reprocessInFlight.current = false;
                },
            },
        );
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
                actual_punch_in: createIsoFromLocal(row.date, punchIn, timezone),
                actual_punch_out: createIsoFromLocal(row.date, punchOut, timezone),
            };
            confirmMutation.mutate({ row: modifiedRow, reason });
        }
    };

    return {
        selectedEmployerId, setSelectedEmployerId,
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
