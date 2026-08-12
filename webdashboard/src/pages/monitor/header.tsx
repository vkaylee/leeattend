import { Activity, RefreshCw } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface MonitorHeaderProps {
  isFetching: boolean;
  onRefresh: () => void;
  t: TFunction;
}

export function MonitorHeader({
  isFetching,
  onRefresh,
  t,
}: MonitorHeaderProps) {
  return (
    <header className="flex flex-col gap-5 px-1 sm:px-2 md:flex-row md:items-center md:justify-between">
      <div className="flex items-center gap-4">
        <div aria-hidden="true" className="flex h-12 w-12 items-center justify-center rounded-[1.25rem] border border-primary-100 bg-primary-50 text-primary-600 shadow-sm dark:border-primary-800/50 dark:bg-primary-950 dark:text-primary-400">
          <Activity
            size={24}
          />
        </div>
        <div>
          <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">
            <span id="monitor-page-title">{t("monitor.title")}</span>
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
            {t("monitor.subtitle")}
          </Typography>
        </div>
      </div>

      <Button aria-label={t("monitor.refresh_label", { defaultValue: "Refresh monitoring snapshot" })} className="flex min-h-11 items-center gap-3 rounded-2xl border border-slate-200 bg-white px-5 py-3 text-xs font-bold uppercase tracking-widest text-slate-700 shadow-sm motion-safe:transition-colors hover:bg-slate-50 focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 active:scale-95 disabled:opacity-50 motion-reduce:transition-none dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
        disabled={isFetching}
        onClick={onRefresh}
        type="button"
        variant="ghost"
      >
        <RefreshCw aria-hidden="true" className={cn(isFetching && "motion-safe:animate-spin")} size={16} />
        <span>{t("common.refresh")}</span>
      </Button>
    </header>
  );
}
