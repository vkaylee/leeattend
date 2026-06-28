import { Users, UserCheck, Clock, FileText, DollarSign } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Skeleton } from "@/components/ui/skeleton";
import { useCurrency } from "@/hooks/use-currency";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { cn } from "@/lib/utils";

// ── Types ──────────────────────────────────────────────────────────

interface StatItem {
  accentClass: string;
  icon: React.ElementType;
  labelKey: string;
  value: string;
}

// ── Component ──────────────────────────────────────────────────────

export default function QuickStatsBar() {
  const { t } = useTranslation();
  const { data: stats, isLoading, isError } = useDashboardStats();
  const { formatCurrency } = useCurrency();

  // Skeleton state
  if (isLoading) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <div className="flex items-center gap-3 p-4 rounded-2xl bg-card border border-slate-100 dark:border-slate-800" key={i}>
            <Skeleton className="w-10 h-10 rounded-xl shrink-0" />
            <div className="space-y-1.5 flex-1">
              <Skeleton className="h-3 w-16" />
              <Skeleton className="h-5 w-12" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  // Silent failure — don't show stats on error
  if (isError || !stats) return null;

  const items: StatItem[] = [
    {
      icon: Users,
      labelKey: "dashboard.hr.total_employees",
      value: String(stats.total_employees ?? 0),
      accentClass: "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-950",
    },
    {
      icon: UserCheck,
      labelKey: "dashboard.hr.present_today",
      value: String(stats.present_today ?? 0),
      accentClass: "text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950",
    },
    {
      icon: Clock,
      labelKey: "dashboard.hr.late_today",
      value: String(stats.late_today ?? 0),
      accentClass: "text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950",
    },
    {
      icon: FileText,
      labelKey: "dashboard.hr.pending_requests",
      value: String(stats.pending_leave_requests ?? 0),
      accentClass: "text-rose-600 dark:text-rose-400 bg-rose-50 dark:bg-rose-950",
    },
    {
      icon: DollarSign,
      labelKey: "dashboard.hr.estimated_payroll",
      value: stats.estimated_monthly_payroll != null
        ? formatCurrency(stats.estimated_monthly_payroll)
        : "—",
      accentClass: "text-violet-600 dark:text-violet-400 bg-violet-50 dark:bg-violet-950",
    },
  ];

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
      {items.map((item) => (
        <div
          className="flex items-center gap-3 p-4 rounded-2xl bg-card border border-slate-100 dark:border-slate-800 hover:shadow-md transition-shadow"
          key={item.labelKey}
        >
          <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center shrink-0", item.accentClass)}>
            <item.icon className="h-4 w-4" />
          </div>
          <div className="flex flex-col min-w-0">
            <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider truncate">
              {t(item.labelKey)}
            </span>
            <span className="text-base font-bold text-slate-700 dark:text-slate-200 truncate">
              {item.value}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}
