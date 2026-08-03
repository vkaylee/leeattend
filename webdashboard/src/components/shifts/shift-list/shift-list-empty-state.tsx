import { Search, Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

// ── ShiftListEmptyState ────────────────────────────────────────────────

export interface ShiftListEmptyStateProps {
  /** Whether the empty state is due to search/filter (vs. truly empty) */
  isFiltered: boolean;
}

/**
 * Renders the empty/no-results placeholder shown inside the desktop table
 * when the filtered shifts list is empty.
 */
export function ShiftListEmptyState({ isFiltered }: ShiftListEmptyStateProps) {
  const { t } = useTranslation();

  const Icon = isFiltered ? Search : Clock;
  const title = isFiltered
    ? t("shifts.picker.no_results_title")
    : t("shifts.messages.no_shifts");
  const description = isFiltered
    ? t("shifts.picker.no_results_desc")
    : `${t("shifts.directory_title")} is empty`;

  return (
    <div className="py-32 text-center bg-slate-50 dark:bg-slate-800/30">
      <div className="bg-white dark:bg-slate-900 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-6 shadow-sm border border-slate-100 dark:border-slate-800">
        <Icon className="text-slate-200 dark:text-slate-700" size={32} />
      </div>
      <Typography
        className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight"
        variant="h3"
      >
        {title}
      </Typography>
      <Typography
        className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-1 opacity-50"
        variant="p"
      >
        {description}
      </Typography>
    </div>
  );
}
