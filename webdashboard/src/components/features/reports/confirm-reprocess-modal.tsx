import { Check, RefreshCw } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
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
            onClose={() => { if (!isPending) onClose(); }}
            title={t("reports.reprocess.confirm_title")}
        >
            <div className="space-y-10 pt-4 animate-in fade-in duration-500">
                <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
                    <Typography className="text-base font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight leading-relaxed text-center italic opacity-80" variant="p">
                        &quot;
                        {!!dialogData && t("reports.reprocess.confirm_message", {
                                employee: dialogData.employeeName,
                                date: formatDate(dialogData.date),
                            })}
                        &quot;
                    </Typography>
                </div>
                <div className="flex gap-6 border-t border-slate-50 dark:border-slate-800 pt-2">
                    <Button
                        className="flex-1 h-auto py-4 rounded-2xl text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800"
                        disabled={isPending}
                        onClick={onClose}
                        variant="ghost"
                    >
                        {t("common.cancel")}
                    </Button>
                    <Button
                        className="flex-[1.5] h-auto py-4 rounded-2xl flex items-center justify-center gap-4 shadow-2xl shadow-slate-400 dark:shadow-black/40"
                        disabled={isPending}
                        icon={<Check size={20} />}
                        loading={isPending}
                        onClick={onConfirm}
                        variant="secondary"
                    >
                        {t("reports.reprocess.confirm")}
                    </Button>
                </div>
            </div>
        </Modal>
    );
}
