import { AlertCircle, CalendarRange, Search, User } from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface RosterErrorStateProps {
  onRetry: () => void;
}

export const RosterErrorState = memo(function RosterErrorState({
  onRetry,
}: RosterErrorStateProps) {
  const { t } = useTranslation();
  return (
    <div className="p-12 text-center flex flex-col items-center justify-center gap-4 animate-in fade-in zoom-in duration-300">
      <div className="w-12 h-12 rounded-full bg-rose-50 dark:bg-rose-950/20 flex items-center justify-center border border-dashed border-rose-200 dark:border-rose-800/50">
        <AlertCircle className="text-rose-500" size={20} />
      </div>
      <div className="text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-200">
        {t("common.error")}
      </div>
      <Typography
        className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto"
        variant="p"
      >
        {t("shifts.monthly.failed_to_load")}
      </Typography>
      <Button
        className="mt-2 px-4 py-2 text-xs font-semibold text-white bg-primary-600 hover:bg-primary-700 active:scale-95 rounded-xl transition-all shadow-sm shadow-primary-500/10 cursor-pointer"
        onClick={onRetry}
        variant="ghost"
      >
        {t("common.retry")}
      </Button>
    </div>
  );
});
RosterErrorState.displayName = "RosterErrorState";

// ── Skeleton loader ───────────────────────────────────────────────────

export const RosterSkeleton = memo(function RosterSkeleton() {
  return (
    <div className="p-4 w-full">
      <div className="h-10 bg-slate-100 dark:bg-slate-800 rounded-xl mb-4 animate-pulse" />
      <div className="space-y-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <div
            className="h-12 bg-slate-50 dark:bg-slate-800/50 rounded-xl animate-pulse"
            key={`skel-${i}`}
          />
        ))}
      </div>
    </div>
  );
});
RosterSkeleton.displayName = "RosterSkeleton";

// ── Empty states ──────────────────────────────────────────────────────

export const NoEmployeesState = memo(function NoEmployeesState() {
  const { t } = useTranslation();
  return (
    <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
      <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800">
        <CalendarRange className="text-slate-400" size={20} />
      </div>
      <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
        {t("shifts.monthly.all_employees")}
      </div>
      <Typography
        className="text-xs text-slate-400 dark:text-slate-500 max-w-xs mx-auto"
        variant="p"
      >
        {t("shifts.monthly.empty_state_desc")}
      </Typography>
    </div>
  );
});
NoEmployeesState.displayName = "NoEmployeesState";

export const NoSelectionState = memo(function NoSelectionState() {
  const { t } = useTranslation();
  return (
    <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
      <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800 animate-pulse text-slate-350 dark:text-slate-650">
        <User className="w-6 h-6 text-slate-450 dark:text-slate-500" />
      </div>
      <div className="text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-300">
        {t("shifts.empty_state.title")}
      </div>
      <Typography
        className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto"
        variant="p"
      >
        {t("shifts.empty_state.desc")}
      </Typography>
    </div>
  );
});
NoSelectionState.displayName = "NoSelectionState";

export const NoResultsState = memo(function NoResultsState() {
  const { t } = useTranslation();
  return (
    <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
      <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800">
        <Search className="text-slate-400" size={20} />
      </div>
      <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
        {t("common.combobox.no_results")}
      </div>
      <Typography
        className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto"
        variant="p"
      >
        {t("shifts.monthly.no_results_desc")}
      </Typography>
    </div>
  );
});
NoResultsState.displayName = "NoResultsState";
