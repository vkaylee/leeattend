import React from "react";

import { cn } from "@/lib/utils";

export interface ReportOtStatusBadgeProps {
  status?: string | null;
  t: (key: string) => string;
}

export function ReportOtStatusBadge({ status, t }: ReportOtStatusBadgeProps) {
  if (!status || status === "none") return null;

  const styles: Record<string, string> = {
    pending_employee: "bg-yellow-50 dark:bg-yellow-950 text-yellow-600 dark:text-yellow-300 border-yellow-200 dark:border-yellow-800",
    pending_manager: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-200 dark:border-orange-800",
    approved: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800",
    rejected: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-200 dark:border-rose-800",
  };

  const labels: Record<string, string> = {
    pending_employee: t("reports.ot_status.pending_employee"),
    pending_manager: t("reports.ot_status.pending_manager"),
    approved: t("reports.ot_status.approved"),
    rejected: t("reports.ot_status.rejected"),
  };

  return (
    <span className={cn("px-1.5 py-0.5 rounded border text-[9px] font-bold uppercase tracking-wide", styles[status] ?? "")}>
      {labels[status] ?? status}
    </span>
  );
}
