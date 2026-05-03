import { useState } from "react";
import type { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";

export function useDailyDrawerModalState() {
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
        confirmDialog,
        setConfirmDialog,
        confirmModalData,
        setConfirmModalData,
        resolveModalData,
        setResolveModalData,
    };
}

export const createIsoFromLocal = (dateString: string, hhmm: string): string | null => {
    if (!hhmm) return null;
    const [y, m, d] = dateString.split("-").map(Number);
    const [hours, mins] = hhmm.split(":").map(Number);
    return new Date(y, m - 1, d, hours, mins, 0).toISOString();
};
