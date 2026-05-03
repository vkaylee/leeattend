import { CheckCircle, RefreshCw } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { DailyAttendanceReport, AttendanceAdjustmentRequest } from "@/lib/api";

export interface ConfirmAttendanceModalProps {
    existingRequest?: AttendanceAdjustmentRequest;
    isOpen: boolean;
    isPending: boolean;
    onClose: () => void;
    onSubmit: (data: { punchIn: string; punchOut: string; reason: string }) => void;
    row: DailyAttendanceReport | null;
}

function PunchInputs({
    punchIn,
    setPunchIn,
    punchOut,
    setPunchOut,
    existingRequest,
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

export default function ConfirmAttendanceModal({
    isOpen,
    onClose,
    row,
    existingRequest,
    isPending,
    onSubmit,
}: ConfirmAttendanceModalProps) {
    const { t } = useTranslation();
    const { formatDate } = useDateTimeFormatter();

    const utcToLocalHHmm = (iso: string | null | undefined): string => {
        if (!iso) return "";
        const d = new Date(iso);
        if (isNaN(d.getTime())) {
            return iso.substring(0, 5);
        }
        return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
    };

    const [punchIn, setPunchIn] = useState(
        utcToLocalHHmm(existingRequest?.requested_punch_in_time) ||
        utcToLocalHHmm(row?.actual_punch_in) ||
        row?.scheduled_start?.substring(0, 5) ||
        ""
    );
    const [punchOut, setPunchOut] = useState(
        utcToLocalHHmm(existingRequest?.requested_punch_out_time) ||
        utcToLocalHHmm(row?.actual_punch_out) ||
        row?.scheduled_end?.substring(0, 5) ||
        ""
    );
    const [reason, setReason] = useState(existingRequest?.reason ?? "");

    if (!row) return null;

    return (
        <Modal
            icon={<CheckCircle size={24} />}
            isOpen={isOpen}
            maxWidth="md"
            onClose={() => !isPending && onClose()}
            subtitle={t("reports.confirm.modal_subtitle")}
            title={t("reports.confirm.modal_title")}
        >
            <div className="space-y-6 pt-2 animate-in fade-in duration-500">
                {/* Employee & Date info */}
                <div className="grid grid-cols-2 gap-4 p-5 bg-slate-50 dark:bg-slate-800 rounded-2xl border border-slate-100 dark:border-slate-800">
                    <div>
                        <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                            {t("reports.confirm.employee")}
                        </label>
                        <p className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1">
                            {row.employee_name}
                        </p>
                    </div>
                    <div>
                        <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                            {t("reports.confirm.date")}
                        </label>
                        <p className="text-sm font-bold text-slate-700 dark:text-slate-200 mt-1">
                            {formatDate(row.date)}
                        </p>
                    </div>
                </div>

                {/* Existing request banner */}
                {!!existingRequest && (
                    <div className="p-4 bg-orange-50 dark:bg-orange-950 rounded-2xl border border-orange-200 dark:border-orange-800">
                        <p className="text-xs font-bold text-orange-600 dark:text-orange-300 uppercase tracking-wider">
                            {t("reports.confirm.existing_request")}
                        </p>
                        {!!existingRequest.reason && (
                            <p className="text-sm text-orange-700 dark:text-orange-200 mt-2 italic">
                                &ldquo;{existingRequest.reason}&rdquo;
                            </p>
                          )}
                    </div>
                  )}

                {/* Punch In / Out */}
                <PunchInputs
                    existingRequest={existingRequest}
                    punchIn={punchIn}
                    punchOut={punchOut}
                    setPunchIn={setPunchIn}
                    setPunchOut={setPunchOut}
                    t={t}
                />

                {/* Reason */}
                {!existingRequest && (
                    <div>
                        <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                            {t("reports.confirm.reason")}
                        </label>
                        <input
                            className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                            onChange={(e) => { setReason(e.target.value); }}
                            placeholder={t("reports.confirm.reason_placeholder")}
                            type="text"
                            value={reason}
                        />
                    </div>
                )}

                {/* Buttons */}
                <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
                    <button
                        className="flex-1 px-6 py-3.5 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                        disabled={isPending}
                        onClick={onClose}
                    >
                        {t("common.cancel")}
                    </button>
                    <button
                        className="flex-[1.5] bg-emerald-600 hover:bg-emerald-700 text-white font-bold px-6 py-3.5 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-xl text-xs uppercase tracking-widest"
                        disabled={isPending || (!existingRequest && !punchIn)}
                        onClick={() => { onSubmit({ punchIn, punchOut, reason }); }}
                    >
                        {isPending ? (
                            <RefreshCw className="w-4 h-4 animate-spin" />
                        ) : (
                            <CheckCircle size={18} />
                        )}
                        {existingRequest ? t("reports.confirm.approve") : t("reports.confirm.submit")}
                    </button>
                </div>
            </div>
        </Modal>
    );
}
