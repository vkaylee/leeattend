import { Clock, FileBarChart, Users2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

interface ReportsStatsProps {
  quickStats: {
    stdHours: string;
    otHours: string;
    totalEmployees: number;
  };
}

export function ReportsStats({ quickStats }: ReportsStatsProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-wrap items-center gap-2">
      <div className="flex items-center gap-2 bg-slate-900 dark:bg-slate-800 px-3.5 py-1.5 rounded-xl border border-slate-800 dark:border-slate-700 shadow-sm">
        <FileBarChart className="text-primary-400" size={14} />
        <span className="text-xs font-medium text-slate-300">
          {t("reports.quick_stats.std_hours", "Giờ trong ca")}
        </span>
        <span className="font-mono font-semibold text-white text-xs tracking-tight">{quickStats.stdHours}h</span>
      </div>
      <div className="flex items-center gap-2 bg-gradient-to-r from-orange-950 to-orange-900 px-3.5 py-1.5 rounded-xl border border-orange-800/80 shadow-sm">
        <Clock className="text-orange-400" size={14} />
        <span className="text-xs font-medium text-orange-200/90">
          {t("reports.quick_stats.ot_hours", "Tăng ca")}
        </span>
        <span className="font-mono font-semibold text-orange-100 text-xs tracking-tight">{quickStats.otHours}h</span>
      </div>
      <div className="flex items-center gap-2 bg-white dark:bg-slate-900 px-3.5 py-1.5 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm">
        <Users2 className="text-primary-600 dark:text-primary-400" size={14} />
        <span className="text-xs font-medium text-slate-500 dark:text-slate-400">{t("common.employees", "Nhân viên")}</span>
        <span className="font-mono font-semibold text-slate-700 dark:text-slate-200 text-xs tracking-tight">
          {quickStats.totalEmployees}
        </span>
      </div>
    </div>
  );
}
