import { Link } from "@tanstack/react-router";
import { UserCheck, CalendarOff, ClockAlert, ArrowRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { ROUTES } from "@/lib/routes";

export function TodayOverviewWidget() {
  const { t } = useTranslation();
  const { data: stats, isLoading, isError } = useDashboardStats();

  if (isLoading) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-16 w-full rounded-2xl" />
        <div className="grid grid-cols-2 gap-3">
          <Skeleton className="h-16 rounded-2xl" />
          <Skeleton className="h-16 rounded-2xl" />
        </div>
      </div>
    );
  }

  if (isError || !stats) {
    return (
      <div className="flex flex-col items-center justify-center p-6 text-center text-xs text-slate-400">
        <Typography variant="p">{t("desktop.statsUnavailable", "Không thể tải số liệu chuyên cần hôm nay")}</Typography>
      </div>
    );
  }

  const attendanceRate = typeof stats.attendance_rate.value === "string" && stats.attendance_rate.value
    ? Math.round(Number(stats.attendance_rate.value))
    : stats.total_employees > 0
      ? Math.round((stats.present_today / stats.total_employees) * 100)
      : 0;

  return (
    <div className="flex flex-col justify-between h-full space-y-4">
      {/* Attendance rate header bar */}
      <div className="p-3.5 rounded-2xl bg-gradient-to-r from-primary-500/10 via-primary-500/5 to-transparent border border-primary-500/20 flex items-center justify-between">
        <div>
          <div className="text-xs font-semibold text-slate-500 dark:text-slate-400">
            {t("desktop.attendanceRate", "Tỷ lệ có mặt hôm nay")}
          </div>
          <div className="text-2xl font-black text-primary-600 dark:text-primary-400 font-mono">
            {attendanceRate}%
          </div>
        </div>
        <div className="text-right">
          <div className="text-xs text-slate-500 dark:text-slate-400 font-medium">{t("desktop.totalHeadcount", "Tổng nhân sự")}</div>
          <div className="text-lg font-bold text-slate-700 dark:text-slate-200">
            {stats.total_employees}
          </div>
        </div>
      </div>

      {/* Grid counters */}
      <div className="grid grid-cols-3 gap-2.5">
        {/* Present */}
        <div className="p-3 rounded-2xl bg-slate-50/80 dark:bg-slate-900/60 border border-slate-100 dark:border-slate-800 text-center">
          <div className="w-7 h-7 mx-auto rounded-xl bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center mb-1">
            <UserCheck className="w-4 h-4" />
          </div>
          <div className="text-base font-bold text-slate-800 dark:text-slate-100">
            {stats.present_today}
          </div>
          <div className="text-[11px] text-slate-600 dark:text-slate-400 font-medium">{t("desktop.present", "Có mặt")}</div>
        </div>

        {/* On Leave */}
        <div className="p-3 rounded-2xl bg-slate-50/80 dark:bg-slate-900/60 border border-slate-100 dark:border-slate-800 text-center">
          <div className="w-7 h-7 mx-auto rounded-xl bg-sky-500/10 text-sky-600 dark:text-sky-400 flex items-center justify-center mb-1">
            <CalendarOff className="w-4 h-4" />
          </div>
          <div className="text-base font-bold text-slate-800 dark:text-slate-100">
            {stats.on_leave_today}
          </div>
          <div className="text-[11px] text-slate-600 dark:text-slate-400 font-medium">{t("desktop.onLeave", "Nghỉ phép")}</div>
        </div>

        {/* Late or Pending */}
        <div className="p-3 rounded-2xl bg-slate-50/80 dark:bg-slate-900/60 border border-slate-100 dark:border-slate-800 text-center">
          <div className="w-7 h-7 mx-auto rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 flex items-center justify-center mb-1">
            <ClockAlert className="w-4 h-4" />
          </div>
          <div className="text-base font-bold text-slate-800 dark:text-slate-100">
            {stats.pending_leave_requests}
          </div>
          <div className="text-[11px] text-slate-600 dark:text-slate-400 font-medium">{t("desktop.pending", "Chờ duyệt")}</div>
        </div>
      </div>

      {/* Footer navigation */}
      <div className="pt-2 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-xs">
        <Link
          className="text-primary-600 dark:text-primary-400 hover:underline font-semibold flex items-center gap-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary rounded-md px-1"
          search={{ tab: "recorded", mode: "this_month" }}
          to={ROUTES.ATTENDANCE_LOGS}
        >
          <span>{t("desktop.viewAttendanceLogs", "Xem bảng công đầy đủ")}</span>
          <ArrowRight className="w-3.5 h-3.5" />
        </Link>
      </div>
    </div>
  );
}
