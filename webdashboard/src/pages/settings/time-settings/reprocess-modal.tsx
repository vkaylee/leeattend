import { AlertTriangle, RefreshCw } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";

interface ReprocessModalProps {
  isOpen: boolean;
  isPending: boolean;
  jobStatus: { progress: { percentage: number } } | undefined;
  onClose: () => void;
  onSubmit: () => void;
  pollingJobId: string | null;
  reprocessEnd: string;
  reprocessStart: string;
  t: TFunction;
}

export function ReprocessModal({
  isOpen,
  isPending,
  jobStatus,
  onClose,
  onSubmit,
  pollingJobId,
  reprocessEnd,
  reprocessStart,
  t,
}: ReprocessModalProps) {
  return (
    <Modal
      icon={<AlertTriangle size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      title={t("reports.reprocess.bulk_confirm_title") || "Confirm Reprocessing"}
    >
      <div className="space-y-6 pt-2 animate-in fade-in duration-500">
        <div className="p-4 bg-orange-50 dark:bg-orange-950 rounded-2xl border border-orange-200 dark:border-orange-800">
          <Typography className="text-sm font-bold text-orange-800 dark:text-orange-300" variant="p">
            Are you sure you want to reprocess attendance from {reprocessStart} to {reprocessEnd}?
          </Typography>
          <Typography className="text-xs text-orange-700 dark:text-orange-400 mt-2" variant="p">
            {t("reports.reprocess.bulk_confirm_message") ||
              "This action will recalculate work hours, overtime, and late/early statuses for ALL employees in the selected date range. This process cannot be undone."}
          </Typography>
        </div>

        {pollingJobId && jobStatus ? (
          <div className="space-y-3 pt-4 border-t border-slate-100 dark:border-slate-800">
            <div className="flex justify-between text-xs font-bold text-slate-500 uppercase tracking-widest">
              <span>{t("common.working")}</span>
              <span>{Math.round(jobStatus.progress.percentage).toString()}%</span>
            </div>
            <div className="h-3 w-full bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
              <div
                className="h-full bg-orange-500 transition-all duration-500 ease-out"
                style={{ width: `${Math.max(5, jobStatus.progress.percentage).toString()}%` }}
              />
            </div>
            <Typography className="text-center text-xs text-slate-400 mt-2 font-medium" variant="p">
              {t("common.workspace_settings.please_wait_minutes")}
            </Typography>
          </div>
        ) : (
          <div className="flex gap-4 border-t border-slate-100 dark:border-slate-800 pt-4">
            <button
              className="flex-1 px-6 py-3.5 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
              disabled={isPending}
              onClick={onClose}
              type="button"
            >
              {t("common.cancel")}
            </button>
            <button
              className="flex-[1.5] bg-orange-600 hover:bg-orange-700 text-white font-bold px-6 py-3.5 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-xl text-xs uppercase tracking-widest"
              disabled={isPending || !reprocessStart || !reprocessEnd}
              onClick={onSubmit}
              type="button"
            >
              {isPending ? <RefreshCw className="w-4 h-4 animate-spin" /> : <RefreshCw size={18} />}
              {(t("reports.reprocess.confirm") || "CONFIRM").toUpperCase()}
            </button>
          </div>
        )}
      </div>
    </Modal>
  );
}
