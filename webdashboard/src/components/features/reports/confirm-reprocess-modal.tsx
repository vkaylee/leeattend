import { Check, RefreshCw } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

export interface ConfirmReprocessModalProps {
    dialogData: {
        employeeId: string;
        employeeName: string;
        date: string;
    } | null;
    isOpen: boolean;
    isPending: boolean;
    onClose: () => void;
    onConfirm: () => void;
}

export default function ConfirmReprocessModal({
    isOpen,
    onClose,
    dialogData,
    isPending,
    onConfirm,
}: ConfirmReprocessModalProps) {
    const { t } = useTranslation();
    const { formatDate } = useDateTimeFormatter();

    return (
        <Modal
            icon={<RefreshCw size={24} />}
            isOpen={isOpen}
            maxWidth="md"
            onClose={() => !isPending && onClose()}
            title={t("reports.reprocess.confirm_title")}
        >
            <div className="space-y-10 pt-4 animate-in fade-in duration-500">
                <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
                    <p className="text-base font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight leading-relaxed text-center italic opacity-80">
                        "
                        {!!dialogData && t("reports.reprocess.confirm_message", {
                                employee: dialogData.employeeName,
                                date: formatDate(dialogData.date),
                            })}
                        "
                    </p>
                </div>
                <div className="flex gap-6 border-t border-slate-50 dark:border-slate-800 pt-2">
                    <button
                        className="flex-1 px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                        disabled={isPending}
                        onClick={onClose}
                    >
                        {t("common.cancel")}
                    </button>
                    <button
                        className="flex-[1.5] bg-slate-900 hover:bg-primary-700 text-white font-bold px-8 py-4 rounded-2xl flex items-center justify-center gap-4 transition-all active:scale-95 disabled:opacity-50 shadow-2xl shadow-slate-400 dark:shadow-black/40 text-xs uppercase tracking-widest"
                        disabled={isPending}
                        onClick={onConfirm}
                    >
                        {isPending ? (
                            <RefreshCw className="w-5 h-5 animate-spin" />
                        ) : (
                            <Check size={20} />
                        )}{" "}
                        {t("reports.reprocess.confirm").toUpperCase()}
                    </button>
                </div>
            </div>
        </Modal>
    );
}
