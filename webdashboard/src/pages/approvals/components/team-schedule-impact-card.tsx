import { AlertCircle, CheckCircle2, Users } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TeamImpactInfo } from "../types/unified-approvals";

interface TeamScheduleImpactCardProps {
  impact?: TeamImpactInfo;
  targetDate: string;
}

export function TeamScheduleImpactCard({
  impact,
  targetDate,
}: TeamScheduleImpactCardProps) {
  const { t } = useTranslation();

  if (!impact) return null;

  const {
    departmentName,
    colleaguesAwayCount,
    colleaguesAwayNames,
    isHighImpact,
  } = impact;

  if (colleaguesAwayCount === 0) {
    return (
      <div
        className="flex items-start gap-3 p-3.5 rounded-2xl bg-emerald-50/80 dark:bg-emerald-950/30 border border-emerald-200/60 dark:border-emerald-800/40 text-emerald-900 dark:text-emerald-200"
        data-testid="impact-safe-card"
      >
        <CheckCircle2 className="h-5 w-5 text-emerald-600 dark:text-emerald-400 shrink-0 mt-0.5" />
        <div className="text-xs space-y-0.5">
          <p className="font-semibold">
            {t(
              "approvals.impact.no_conflict_title",
              "Không có xung đột nhân sự"
            )}
          </p>
          <p className="text-emerald-700/90 dark:text-emerald-300/80">
            {departmentName
              ? t(
                  "approvals.impact.no_conflict_dept",
                  "Không có thành viên nào khác trong phòng {{dept}} nghỉ ngày {{date}}.",
                  { dept: departmentName, date: targetDate }
                )
              : t(
                  "approvals.impact.no_conflict_generic",
                  "Lịch làm việc của đội ngũ đảm bảo an toàn vận hành."
                )}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`flex items-start gap-3 p-3.5 rounded-2xl border ${
        isHighImpact
          ? "bg-amber-50/90 dark:bg-amber-950/40 border-amber-300/80 dark:border-amber-800/60 text-amber-950 dark:text-amber-200"
          : "bg-slate-50 dark:bg-slate-800/40 border-slate-200 dark:border-slate-700 text-slate-800 dark:text-slate-200"
      }`}
      data-testid="impact-warning-card"
    >
      {isHighImpact ? (
        <AlertCircle className="h-5 w-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
      ) : (
        <Users className="h-5 w-5 text-slate-500 shrink-0 mt-0.5" />
      )}
      <div className="text-xs space-y-1 w-full">
        <div className="flex items-center justify-between gap-2">
          <p className="font-bold text-amber-900 dark:text-amber-300">
            {isHighImpact
              ? t(
                  "approvals.impact.understaffing_alert",
                  "Cảnh báo thiếu hụt nhân sự"
                )
              : t("approvals.impact.colleagues_away", "Nhân sự vắng mặt cùng ngày")}
          </p>
          <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-200/80 dark:bg-amber-900/60 text-amber-900 dark:text-amber-200">
            {colleaguesAwayCount}{" "}
            {t("approvals.impact.people_count", "người vắng")}
          </span>
        </div>
        <p className="text-amber-800/90 dark:text-amber-300/80">
          {!!departmentName && <span className="font-medium">[{departmentName}] </span>}
          {t("approvals.impact.away_list", "Đã có người nghỉ phép:")}{" "}
          <span className="font-semibold underline">
            {colleaguesAwayNames.join(", ")}
          </span>
        </p>
      </div>
    </div>
  );
}
