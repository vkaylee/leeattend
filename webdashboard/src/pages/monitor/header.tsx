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
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
          <Activity
            size={24}
          />
        </div>
        <div>
          <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">
            {t("monitor.title")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
            {t("monitor.subtitle")}
          </Typography>
        </div>
      </div>

      <Button className="flex items-center gap-3 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-6 py-3 rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-700 transition-all font-bold text-xs uppercase tracking-widest shadow-lg dark:shadow-black/20 active:scale-95 disabled:opacity-50"
        disabled={isFetching}
        onClick={onRefresh}
        type="button"
        variant="ghost"
      >
        <RefreshCw className={cn(isFetching && "animate-spin")} size={16} />
        {t("common.refresh")}
      </Button>
    </div>
  );
}
