import {
  ArrowUp,
  ArrowDown,
  Zap,
  Coffee,
  CheckCircle2,
  XCircle,
  Server,
  LucideIcon,
} from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { WorkerMonitorStats } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface MonitorStatCardsProps {
  isLoading: boolean;
  stats: WorkerMonitorStats | null | undefined;
  t: TFunction;
}

export function MonitorStatCards({ isLoading, stats, t }: MonitorStatCardsProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-7 gap-6">
      <StatCard
        bgIcon="text-orange-100"
        color="bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-100 dark:border-orange-800"
        icon={ArrowUp}
        isLoading={isLoading}
        label={t("monitor.pending_high")}
        value={stats?.pending_high}
      />
      <StatCard
        bgIcon="text-amber-100"
        color="bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800"
        icon={ArrowDown}
        isLoading={isLoading}
        label={t("monitor.pending_low")}
        value={stats?.pending_low}
      />
      <StatCard
        bgIcon="text-emerald-100"
        color="bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
        icon={Zap}
        isLoading={isLoading}
        label={t("monitor.active_workers")}
        value={stats?.active_workers}
      />
      <StatCard
        bgIcon="text-slate-100"
        color="bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border-slate-100 dark:border-slate-700"
        icon={Coffee}
        isLoading={isLoading}
        label={t("monitor.idle_workers")}
        value={stats?.idle_workers}
      />
      <StatCard
        bgIcon="text-primary-100"
        color="bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800"
        icon={CheckCircle2}
        isLoading={isLoading}
        label={t("monitor.completed_today")}
        value={stats?.completed_today}
      />
      <StatCard
        bgIcon="text-rose-100"
        color="bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800"
        icon={XCircle}
        isLoading={isLoading}
        label={t("monitor.failed_today")}
        value={stats?.failed_today}
      />
      <StatCard
        bgIcon="text-emerald-100 dark:text-emerald-800"
        color="bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
        icon={Server}
        isLoading={isLoading}
        label={t("monitor.total_workers")}
        value={stats?.total_workers}
      />
    </div>
  );
}

function StatCard({
  bgIcon,
  color,
  icon: Icon,
  isLoading,
  label,
  value,
}: {
  bgIcon: string;
  color: string;
  icon: LucideIcon;
  isLoading: boolean;
  label: string;
  value: number | null | undefined;
}) {
  return (
    <div className="bg-white dark:bg-slate-900 p-5 sm:p-6 rounded-2xl sm:rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm dark:shadow-black/10 hover:shadow-xl dark:hover:shadow-black/20 transition-all group relative overflow-hidden">
      <div className={cn("w-12 h-12 rounded-xl flex items-center justify-center mb-4 border", color)}>
        <Icon size={22} />
      </div>
      <Typography className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1 opacity-70" variant="p">
        {label}
      </Typography>
      <Typography className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter" variant="p">
        {isLoading ? "..." : value ?? "—"}
      </Typography>
      <Icon
        className={cn(
          "absolute -right-4 -bottom-4 w-24 h-24 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-500",
          bgIcon
        )}
      />
    </div>
  );
}
