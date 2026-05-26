import { AlertTriangle } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { DailyAttendanceReport } from "@/lib/api";
import { cn } from "@/lib/utils";

export interface ReportStatusBadgeProps {
  onResolveUnscheduled?: (row: DailyAttendanceReport) => void;
  row: DailyAttendanceReport;
  status: string;
  t: (key: string) => string;
}

export function ReportStatusBadge({ status, row, t, onResolveUnscheduled }: ReportStatusBadgeProps) {
  const styles: Record<string, string> = {
    present: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
    late: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800",
    absent: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
    off: "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800",
    unscheduled: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-200 dark:border-amber-800",
    holiday: "bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800",
    holiday_paid: "bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800",
    holiday_leave: "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800",
    holiday_unpaid: "bg-rose-50 dark:bg-rose-950 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800",
    holiday_work: "bg-orange-50 dark:bg-orange-950 text-orange-700 dark:text-orange-300 border-orange-200 dark:border-orange-800",
  };

  const labels: Record<string, string> = {
    present: t("reports.status.present"),
    late: t("reports.status.late"),
    absent: t("reports.status.absent"),
    off: t("reports.status.off"),
    unscheduled: t("reports.status.unscheduled"),
    holiday: "🏖️ Holiday",
    holiday_paid: "🏖️ Holiday (Paid)",
    holiday_leave: "⚡ Holiday (Leave)",
    holiday_unpaid: "🔴 Holiday (Unpaid)",
    holiday_work: "🔶 Holiday Work",
  };

  if (status === "unscheduled") {
    return (
      <Button
        className={cn(
          "inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide border shadow-sm hover:bg-amber-100 dark:hover:bg-amber-900 hover:border-amber-300 dark:hover:border-amber-700 transition-colors h-auto",
          styles.unscheduled
        )}
        onClick={(e) => { e.stopPropagation(); onResolveUnscheduled?.(row); }}
        title={t("reports.status.unscheduled_tooltip")}
        variant="ghost"
      >
        <AlertTriangle size={10} /> {labels.unscheduled}
      </Button>
    );
  }

  return (
    <span
      className={cn(
        "px-1.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide border shadow-sm",
        styles[status] ?? styles.off
      )}
      title={row.holiday_name ?? undefined}
    >
      {labels[status] ?? status}
    </span>
  );
}
