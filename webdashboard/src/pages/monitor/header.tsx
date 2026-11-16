import { Activity, RefreshCw } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
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
    <PageHeader
      actions={
        <Button
          aria-label={t("monitor.refresh_label", { defaultValue: "Refresh monitoring snapshot" })}
          className="flex min-h-11 items-center gap-3 rounded-2xl border border-slate-200 bg-white px-5 py-3 text-xs font-bold uppercase tracking-widest text-slate-700 shadow-sm motion-safe:transition-colors hover:bg-slate-50 focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 active:scale-95 disabled:opacity-50 motion-reduce:transition-none dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
          disabled={isFetching}
          onClick={onRefresh}
          type="button"
          variant="ghost"
        >
          <RefreshCw aria-hidden="true" className={cn(isFetching && "motion-safe:animate-spin")} size={16} />
          <span>{t("common.refresh")}</span>
        </Button>
      }
      className="px-1 sm:px-2"
      icon={<Activity size={24} />}
      subtitle={t("monitor.subtitle")}
      title={t("monitor.title")}
    />
  );
}
