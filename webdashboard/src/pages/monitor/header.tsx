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
      <div className="flex items-center gap-5">
        <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-800 flex items-center justify-center">
          <Activity
            className="text-primary-600 dark:text-primary-400"
            size={28}
          />
        </div>
        <div>
          <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" variant="h3">
            {t("monitor.title")}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-2 opacity-60" variant="p">
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
