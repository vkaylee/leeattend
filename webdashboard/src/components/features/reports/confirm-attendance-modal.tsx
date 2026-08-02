import { CheckCircle } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useWorkspace } from "@/hooks/use-workspace";
import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";

import { ActionButtons, ExistingRequestBanner, ReasonInput } from "./confirm-attendance-modal-sections";

export interface ConfirmAttendanceModalProps {
    existingRequest?: AttendanceAdjustmentRequest;
    isOpen: boolean;
    isPending: boolean;
    onClose: () => void;
    onSubmit: (data: { punchIn: string; punchOut: string; reason: string }) => void;
    row: DailyAttendanceReport | null;
}

// Render a UTC punch timestamp in the workspace timezone (Excel parity), not
// the viewer's browser timezone. Falls back to UTC when the timezone is invalid.
const utcToLocalHHmm = (iso: string | null | undefined, timezone: string): string => {
    if (!iso) { return ""; }
    const d = new Date(iso);
    if (isNaN(d.getTime())) {
        return iso.substring(0, 5);
    }
    try {
        return new Intl.DateTimeFormat("en-GB", {
            timeZone: timezone,
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
        }).format(d);
    } catch {
        return new Intl.DateTimeFormat("en-GB", {
            timeZone: "UTC",
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
        }).format(d);
    }
};

function PunchInputs({
    existingRequest,
    punchIn,
    punchOut,
    setPunchIn,
    setPunchOut,
    t,
}: {
    existingRequest?: AttendanceAdjustmentRequest;
    punchIn: string;
    punchOut: string;
    setPunchIn: (val: string) => void;
    setPunchOut: (val: string) => void;
    t: (k: string) => string;
}) {
    return (
        <div className="grid grid-cols-2 gap-4">
            <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                    {t("reports.confirm.punch_in")}
                </label>
                <input
                    className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-mono font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled={!!existingRequest}
                    onChange={(e) => { setPunchIn(e.target.value); }}
                    type="time"
                    value={punchIn}
                />
            </div>
            <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                    {t("reports.confirm.punch_out")}
                </label>
                <input
                    className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-mono font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled={!!existingRequest}
                    onChange={(e) => { setPunchOut(e.target.value); }}
                    type="time"
                    value={punchOut}
                />
            </div>
        </div>
    );
}

function EmployeeInfo({ row, t, formatDate }: { row: DailyAttendanceReport; t: (k: string) => string; formatDate: (d: string) => string }) {
    return (
        <div className="grid grid-cols-2 gap-4 p-5 bg-slate-50 dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-800">
            <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("reports.confirm.employee")}
                </label>
                <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1" variant="p">
                    {row.employee_name}
                </Typography>
            </div>
            <div>
                <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("reports.confirm.date")}
                </label>
                <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1" variant="p">
                    {formatDate(row.date)}
                </Typography>
            </div>
        </div>
    );
}

function getInitialPunchIn(existingRequest: AttendanceAdjustmentRequest | undefined, row: DailyAttendanceReport | null | undefined, timezone: string) {
    if (existingRequest?.requested_punch_in_time) return utcToLocalHHmm(existingRequest.requested_punch_in_time, timezone);
    if (row?.actual_punch_in) return utcToLocalHHmm(row.actual_punch_in, timezone);
    if (row?.scheduled_start) return row.scheduled_start.substring(0, 5);
    return "";
}

function getInitialPunchOut(existingRequest: AttendanceAdjustmentRequest | undefined, row: DailyAttendanceReport | null | undefined, timezone: string) {
    if (existingRequest?.requested_punch_out_time) return utcToLocalHHmm(existingRequest.requested_punch_out_time, timezone);
    if (row?.actual_punch_out) return utcToLocalHHmm(row.actual_punch_out, timezone);
    if (row?.scheduled_end) return row.scheduled_end.substring(0, 5);
    return "";
}

export default function ConfirmAttendanceModal({
    existingRequest,
    isOpen,
    isPending,
    onClose,
    onSubmit,
    row,
}: ConfirmAttendanceModalProps) {
    const { t } = useTranslation();
    const { formatDate } = useDateTimeFormatter();
    const { workspace } = useWorkspace();
    const timezone = workspace?.timezone ?? "UTC";

    const [punchIn, setPunchIn] = useState(getInitialPunchIn(existingRequest, row, timezone));
    const [punchOut, setPunchOut] = useState(getInitialPunchOut(existingRequest, row, timezone));
    const [reason, setReason] = useState(existingRequest?.reason ?? "");

    if (!row) { return null; }

    const handleClose = () => { if (!isPending) { onClose(); } };

    return (
        <Modal
            icon={<CheckCircle size={24} />}
            isOpen={isOpen}
            maxWidth="md"
            onClose={handleClose}
            subtitle={t("reports.confirm.modal_subtitle")}
            title={t("reports.confirm.modal_title")}
        >
            <div className="space-y-6 pt-2 animate-in fade-in duration-500">
                <EmployeeInfo formatDate={formatDate} row={row} t={t} />

                {!!existingRequest && <ExistingRequestBanner existingRequest={existingRequest} t={t} />}

                <PunchInputs
                    existingRequest={existingRequest}
                    punchIn={punchIn}
                    punchOut={punchOut}
                    setPunchIn={setPunchIn}
                    setPunchOut={setPunchOut}
                    t={t}
                />

                {!existingRequest && <ReasonInput reason={reason} setReason={setReason} t={t} />}

                <ActionButtons 
                    canSubmit={!!existingRequest || !!punchIn}
                    isEdit={!!existingRequest}
                    isPending={isPending}
                    onClose={onClose}
                    onSubmit={() => { onSubmit({ punchIn, punchOut, reason }); }}
                    t={t}
                />
            </div>
        </Modal>
    );
}
