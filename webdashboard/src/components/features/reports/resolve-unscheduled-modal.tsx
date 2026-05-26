import { AlertTriangle, CalendarCheck } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { DailyAttendanceReport, Shift } from "@/lib/api";

export interface ResolveUnscheduledModalProps {
    isOpen: boolean;
    isPending: boolean;
    onClose: () => void;
    onSubmit: (shiftId: string) => void;
    row: DailyAttendanceReport | null;
    shifts: Shift[] | undefined;
}

export default function ResolveUnscheduledModal({
    isOpen,
    onClose,
    row,
    shifts,
    isPending,
    onSubmit,
}: ResolveUnscheduledModalProps) {
    const { t } = useTranslation();
    const { formatDate, formatTime } = useDateTimeFormatter();

    const [selectedShiftId, setSelectedShiftId] = useState("");

    if (!row) return null;

    return (
        <Modal
            icon={<AlertTriangle size={24} />}
            isOpen={isOpen}
            onClose={() => { if (!isPending) onClose(); }}
            title={t("reports.unscheduled.resolve_title")}
        >
            <div className="space-y-6">
                {/* Employee + Date info */}
                <div className="flex items-center gap-4 p-4 bg-amber-50 dark:bg-amber-950 rounded-2xl border border-amber-200 dark:border-amber-800">
                    <div className="w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900 flex items-center justify-center text-amber-600 dark:text-amber-300 shadow-inner">
                        <AlertTriangle size={20} />
                    </div>
                    <div>
                        <Typography className="text-sm font-bold text-amber-800 dark:text-amber-200" variant="p">
                            {row.employee_name}
                        </Typography>
                        <Typography className="text-xs font-mono font-bold text-amber-600 dark:text-amber-400 uppercase" variant="p">
                            {formatDate(row.date)}
                        </Typography>
                    </div>
                </div>

                {/* Punch times display */}
                {!!row.actual_punch_in && (
                    <div className="grid grid-cols-2 gap-4">
                        <div className="p-3 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700">
                            <Typography className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-1" variant="p">{t("reports.confirm.punch_in")}</Typography>
                            <Typography className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200" variant="p">{formatTime(row.actual_punch_in)}</Typography>
                        </div>
                        <div className="p-3 bg-slate-50 dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700">
                            <Typography className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-1" variant="p">{t("reports.confirm.punch_out")}</Typography>
                            <Typography className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200" variant="p">{formatTime(row.actual_punch_out)}</Typography>
                        </div>
                    </div>
                  )}

                {/* Shift selector */}
                <div>
                    <label className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block">
                        {t("reports.unscheduled.select_shift")}
                    </label>
                    <select
                        className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                        onChange={(e) => { setSelectedShiftId(e.target.value); }}
                        value={selectedShiftId}
                    >
                        <option value="">-- {t("reports.unscheduled.select_shift")} --</option>
                        {(shifts ?? []).map((shift: Shift) => (
                            <option key={shift.id} value={shift.id}>
                                {shift.name} ({shift.start_time.substring(0, 5)} — {shift.end_time.substring(0, 5)})
                            </option>
                        ))}
                    </select>
                </div>

                {/* Explanation note */}
                <Typography className="text-[10px] text-slate-500 dark:text-slate-400 italic leading-relaxed" variant="p">
                    {t("reports.unscheduled.resolve_subtitle")}
                </Typography>

                {/* Buttons */}
                <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
                    <Button
                        className="flex-1 h-auto py-3.5 rounded-2xl text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800"
                        disabled={isPending}
                        onClick={onClose}
                        variant="ghost"
                    >
                        {t("common.cancel")}
                    </Button>
                    <Button
                        className="flex-[1.5] bg-amber-600 hover:bg-amber-700 shadow-xl h-auto py-3.5 rounded-2xl flex items-center justify-center gap-3"
                        disabled={isPending || !selectedShiftId}
                        icon={<CalendarCheck size={18} />}
                        loading={isPending}
                        onClick={() => { onSubmit(selectedShiftId); }}
                        variant="primary"
                    >
                        {t("reports.unscheduled.submit")}
                    </Button>
                </div>
            </div>
        </Modal>
    );
}
