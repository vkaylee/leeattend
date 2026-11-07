import { Link } from "@tanstack/react-router";
import { TrendingUp, Clock, DollarSign, Award, ArrowUpRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Skeleton } from "@/components/ui/skeleton";
import { useCurrency } from "@/hooks/use-currency";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { ROUTES } from "@/lib/routes";

export function ExecutiveKpiWidget() {
  const { t } = useTranslation();
  const { data: stats, isLoading, isError } = useDashboardStats();
  const { formatCurrency } = useCurrency();

  if (isLoading) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-14 w-full rounded-2xl" />
        <Skeleton className="h-14 w-full rounded-2xl" />
      </div>
    );
  }

  if (isError || !stats) {
    return (
      <div className="flex flex-col items-center justify-center p-6 text-center text-xs text-slate-400">
        <p>{t("desktop.kpiUnavailable", "Không thể tải số liệu KPI điều hành")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col justify-between h-full space-y-4">
      {/* Metric 1: Monthly Payroll Estimate */}
      <div className="p-3.5 rounded-2xl bg-slate-50/80 dark:bg-slate-900/60 border border-slate-100 dark:border-slate-800 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center">
            <DollarSign className="w-4 h-4" />
          </div>
          <div>
            <div className="text-[11px] text-slate-500 dark:text-slate-400 font-medium">
              {t("desktop.estimatedPayroll", "Dự toán quỹ lương tháng")}
            </div>
            <div className="text-base font-bold text-slate-800 dark:text-slate-100 font-mono">
              {stats.estimated_monthly_payroll
                ? formatCurrency(Number(stats.estimated_monthly_payroll) || 0)
                : "—"}
            </div>
          </div>
        </div>
        <Link
          to={ROUTES.PAYROLL}
          className="p-1.5 rounded-lg text-slate-400 hover:text-primary-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
          aria-label={t("desktop.viewPayroll", "Xem bảng tính lương")}
        >
          <ArrowUpRight className="w-4 h-4" />
        </Link>
      </div>

      {/* Metric 2: Attendance Rate & OT Status */}
      <div className="p-3.5 rounded-2xl bg-slate-50/80 dark:bg-slate-900/60 border border-slate-100 dark:border-slate-800 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-purple-500/10 text-purple-600 dark:text-purple-400 flex items-center justify-center">
            <Award className="w-4 h-4" />
          </div>
          <div>
            <div className="text-[11px] text-slate-500 dark:text-slate-400 font-medium">
              {t("desktop.punctualityRate", "Chỉ số hoàn thành công chuẩn")}
            </div>
            <div className="text-base font-bold text-slate-800 dark:text-slate-100 font-mono">
              {stats.total_employees > 0
                ? `${Math.round((stats.present_today / stats.total_employees) * 100)}%`
                : "100%"}
            </div>
          </div>
        </div>

        <Link
          to={ROUTES.REPORTS}
          className="p-1.5 rounded-lg text-slate-400 hover:text-primary-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
          aria-label={t("desktop.viewReports", "Xem báo cáo chuyên cần")}
        >
          <ArrowUpRight className="w-4 h-4" />
        </Link>
      </div>
    </div>
  );
}
