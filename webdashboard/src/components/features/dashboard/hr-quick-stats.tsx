import { Timer, LogOut, Banknote } from "lucide-react";

import { type TFunction } from "i18next";

import StatCard from "@/components/ui/stat-card";
import { Typography } from "@/components/ui/typography";
import { type WorkspaceStats } from "@/hooks/use-dashboard";

interface HrQuickStatsProps {
  formatCurrency: (value: number) => string;
  hrStats: WorkspaceStats | undefined;
  isLoading: boolean;
  t: TFunction;
}

export function HrQuickStats({ hrStats, isLoading, formatCurrency, t }: HrQuickStatsProps) {
  const stats = [
    {
      label: "avg_work_hours",
      value: isLoading ? "..." : `${(hrStats?.avg_work_hours_today ?? 0).toFixed(1)}h`,
      icon: Timer,
      color: "bg-sky-50 dark:bg-sky-950 text-sky-600 dark:text-sky-400 border-sky-100 dark:border-sky-800",
      mono: true
    },
    {
      label: "early_departures",
      value: isLoading ? "..." : hrStats?.early_departure_today ?? 0,
      icon: LogOut,
      color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-800",
      mono: false
    },
    {
      label: "estimated_payroll",
      value: isLoading ? "..." : formatCurrency(Number(hrStats?.estimated_monthly_payroll ?? 0)),
      icon: Banknote,
      color: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-400 border-rose-100 dark:border-rose-800",
      mono: true
    },
  ];

  return (
    <div className="space-y-4 sm:space-y-6">
      <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] px-4" variant="h2">
        {t("dashboard.hr.quick_stats")}
      </Typography>
      {stats.map((item, index) => (
        <StatCard
          colorClasses={item.color}
          icon={item.icon}
          key={index}
          label={t(`dashboard.hr.${item.label}`)}
          mono={item.mono}
          size="lg"
          value={item.value}
        />
      ))}
    </div>
  );
}
