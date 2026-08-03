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

// Convert a wall-clock time ("HH:mm" on `dateString`) to a UTC ISO timestamp,
// interpreting the wall-clock in the WORKSPACE timezone (Excel parity), not the
// viewer's browser timezone. Converges via offset iteration to handle DST.
export const createIsoFromLocal = (dateString: string, hhmm: string, timezone: string): string | null => {
    if (!hhmm) return null;
    const [y, m, d] = dateString.split("-").map(Number);
    const [hours, mins] = hhmm.split(":").map(Number);

    const dtf = ((): Intl.DateTimeFormat => {
        try {
            return new Intl.DateTimeFormat("en-US", {
                timeZone: timezone,
                hour12: false,
                year: "numeric",
                month: "2-digit",
                day: "2-digit",
                hour: "2-digit",
                minute: "2-digit",
                second: "2-digit",
            });
        } catch { /* invalid timezone — fall back to UTC */
            return new Intl.DateTimeFormat("en-US", {
                timeZone: "UTC",
                hour12: false,
                year: "numeric",
                month: "2-digit",
                day: "2-digit",
                hour: "2-digit",
                minute: "2-digit",
                second: "2-digit",
            });
        }
    })();

    let guess = new Date(y, m - 1, d, hours, mins, 0).getTime();
    for (let i = 0; i < 3; i++) {
        const parts = dtf.formatToParts(new Date(guess));
        const p = (t: string) => Number(parts.find((x) => x.type === t)?.value ?? 0);
        const got = new Date(p("year"), p("month") - 1, p("day"), p("hour"), p("minute"), p("second")).getTime();
        const want = new Date(y, m - 1, d, hours, mins, 0).getTime();
        const diff = want - got;
        if (diff === 0) break;
        guess += diff;
    }
    return new Date(guess).toISOString();
};
