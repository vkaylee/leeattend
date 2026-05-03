import React from "react";
import { useTranslation } from "react-i18next";

import { AttendanceDailySummary } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface OtStatusCellProps {
  setClaimOpen: (open: boolean) => void;
  setSelectedSummary: (summary: AttendanceDailySummary) => void;
  summary?: AttendanceDailySummary;
}

export function OtStatusCell({
  setClaimOpen,
  setSelectedSummary,
  summary,
}: OtStatusCellProps) {
  const { t } = useTranslation();
  if (!summary || summary.system_detected_ot_minutes <= 0) {
    return <span className="text-slate-300 dark:text-slate-600">-</span>;
  }

  const isPending =
    summary.ot_status === "pending_employee" ||
    (summary.ot_status === "none" && summary.system_detected_ot_minutes > 0);

  return (
    <div className="flex flex-col items-center gap-1">
      <span className="text-xs font-bold text-amber-600 dark:text-amber-300">
        {t("ess.attendance.system_ot_suffix", { minutes: summary.system_detected_ot_minutes })}
      </span>
      {isPending ? (
        <button
          className="text-[10px] font-bold uppercase tracking-wider bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 px-2 py-1 rounded-lg hover:bg-amber-200 transition-colors"
          onClick={() => {
            setSelectedSummary(summary);
            setClaimOpen(true);
          }}
          type="button"
        >
          {t("ess.attendance.claim_btn")}
        </button>
      ) : (
        <span
          className={cn(
            "text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full",
            summary.ot_status === "approved"
              ? "bg-emerald-100 dark:bg-emerald-900 text-emerald-700 dark:text-emerald-300"
              : summary.ot_status === "rejected"
              ? "bg-rose-100 dark:bg-rose-900 text-rose-700 dark:text-rose-300"
              : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400"
          )}
        >
          {summary.ot_status === "pending_manager"
            ? t("common.status_pending")
            : summary.ot_status}
        </span>
      )}
    </div>
  );
}
