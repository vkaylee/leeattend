import { RefreshCw } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";

interface BulkReprocessSectionProps {
  reprocessEnd: string;
  reprocessStart: string;
  setReprocessEnd: (date: string) => void;
  setReprocessStart: (date: string) => void;
  setShowReprocessConfirm: (show: boolean) => void;
  t: TFunction;
}

export function BulkReprocessSection({
  reprocessEnd,
  reprocessStart,
  setReprocessEnd,
  setReprocessStart,
  setShowReprocessConfirm,
  t,
}: BulkReprocessSectionProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10 mt-8">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800 pb-6">
        <div className="flex items-center gap-3">
          <RefreshCw className="w-6 h-6 text-orange-500" />
          <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">
            {t("reports.reprocess.bulk_tooltip") || "Reprocess Attendance"}
          </Typography>
        </div>
      </div>
      <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
        {t("reports.reprocess.bulk_confirm_message") ||
          "Recalculate attendance for all employees in the selected date range."}
      </Typography>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
        <div className="space-y-3">
          <label className={LABEL_BASE}>{t("common.startDate") || "Start Date"}</label>
          <input
            className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm font-bold h-14"
            onChange={(e) => {
              setReprocessStart(e.target.value);
            }}
            type="date"
            value={reprocessStart}
          />
        </div>
        <div className="space-y-3">
          <label className={LABEL_BASE}>{t("common.endDate") || "End Date"}</label>
          <input
            className="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-sm font-bold h-14"
            onChange={(e) => {
              setReprocessEnd(e.target.value);
            }}
            type="date"
            value={reprocessEnd}
          />
        </div>
        <div className="space-y-3 md:col-span-2 flex justify-end">
          <Button
            className="text-orange-600 border-orange-200 hover:bg-orange-50 dark:border-orange-900/50 dark:hover:bg-orange-900/20"
            icon={<RefreshCw size={18} />}
            onClick={() => {
              setShowReprocessConfirm(true);
            }}
            variant="outline"
          >
            {(t("reports.reprocess.bulk_tooltip") || "REPROCESS DATA").toUpperCase()}
          </Button>
        </div>
      </div>
    </div>
  );
}
