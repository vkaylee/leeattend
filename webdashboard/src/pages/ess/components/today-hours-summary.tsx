import { Clock, Coffee, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface TodayHoursSummaryProps {
  actualMinutes?: number;
  breakMinutes?: number;
  className?: string;
  otMinutes?: number;
  standardMinutes?: number;
}

function formatMins(mins: number): string {
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  if (h === 0) return `${m}p`;
  if (m === 0) return `${h}h`;
  return `${h}h ${m < 10 ? `0${m}` : m}p`;
}

export function TodayHoursSummary({
  actualMinutes = 0,
  standardMinutes = 480,
  breakMinutes = 60,
  otMinutes = 0,
  className,
}: TodayHoursSummaryProps) {
  const { t } = useTranslation();

  const progressPercent = Math.min(
    100,
    Math.round((actualMinutes / standardMinutes) * 100)
  );
  const remainingMinutes = Math.max(0, standardMinutes - actualMinutes);
  const isGoalReached = actualMinutes >= standardMinutes;

  return (
    <div
      aria-label={t("ess.clock.summary_label", "Tóm tắt giờ làm việc hôm nay")}
      className={cn(
        "p-5 rounded-3xl bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800 shadow-sm space-y-4",
        className
      )}
      data-testid="today-hours-summary"
    >
      <div className="flex items-center justify-between">
        <div className="space-y-0.5">
          <Typography className="text-sm font-bold text-slate-900 dark:text-slate-100 flex items-center gap-2" variant="h4">
            <Clock className="h-4 w-4 text-primary-600 dark:text-primary-400" />
            {t("ess.clock.today_progress", "Tiến độ giờ làm hôm nay")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">
            {isGoalReached
              ? t(
                  "ess.clock.goal_reached",
                  "Chúc mừng! Bạn đã hoàn thành đủ số giờ làm tiêu chuẩn"
                )
              : t("ess.clock.goal_remaining", "Còn {{remaining}} để đạt 8 tiếng", {
                  remaining: formatMins(remainingMinutes),
                })}
          </Typography>
        </div>

        <span
          className={cn(
            "text-xs font-black px-2.5 py-1 rounded-full border",
            isGoalReached
              ? "bg-emerald-50 text-emerald-800 border-emerald-200 dark:bg-emerald-950 dark:text-emerald-300"
              : "bg-primary-50 text-primary-800 border-primary-200 dark:bg-primary-950 dark:text-primary-300"
          )}
          data-testid="today-progress-percent"
        >
          {progressPercent}%
        </span>
      </div>

      {/* Progress Bar */}
      <div className="w-full h-3 rounded-full bg-slate-100 dark:bg-slate-800 overflow-hidden p-0.5 border border-slate-200/60 dark:border-slate-700/60">
        <div
          aria-valuemax={100}
          aria-valuemin={0}
          aria-valuenow={progressPercent}
          className={cn(
            "h-full rounded-full transition-all duration-700",
            isGoalReached
              ? "bg-gradient-to-r from-emerald-500 to-teal-500 shadow-sm"
              : "bg-gradient-to-r from-primary-500 to-indigo-500"
          )}
          role="progressbar"
          style={{ width: `${progressPercent}%` }}
        />
      </div>

      {/* Metric Cards Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 pt-1">
        {/* Actual */}
        <div className="p-3 rounded-2xl bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800">
          <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 block mb-1">
            {t("ess.clock.actual_time", "Thực tế")}
          </span>
          <span className="text-base font-black text-slate-800 dark:text-slate-100">
            {formatMins(actualMinutes)}
          </span>
        </div>

        {/* Target */}
        <div className="p-3 rounded-2xl bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800">
          <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 block mb-1">
            {t("ess.clock.standard_time", "Tiêu chuẩn")}
          </span>
          <span className="text-base font-black text-slate-800 dark:text-slate-100">
            {formatMins(standardMinutes)}
          </span>
        </div>

        {/* Break */}
        <div className="p-3 rounded-2xl bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800">
          <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 block mb-1 flex items-center gap-1">
            <Coffee className="h-3 w-3" />
            {t("ess.clock.break_time", "Nghỉ trưa")}
          </span>
          <span className="text-base font-black text-slate-800 dark:text-slate-100">
            {formatMins(breakMinutes)}
          </span>
        </div>

        {/* Overtime */}
        <div className="p-3 rounded-2xl bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800">
          <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 block mb-1 flex items-center gap-1">
            <Zap className="h-3 w-3 text-amber-500" />
            {t("ess.clock.ot_time", "Tăng ca")}
          </span>
          <span
            className={cn(
              "text-base font-black",
              otMinutes > 0
                ? "text-violet-600 dark:text-violet-400"
                : "text-slate-400 dark:text-slate-500"
            )}
          >
            {formatMins(otMinutes)}
          </span>
        </div>
      </div>
    </div>
  );
}
